//! 终端命令入口：集中对外暴露会话控制能力。

use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

use serde_json::json;
use tauri::{AppHandle, Manager, State, WebviewWindow};

use crate::now_millis;
use crate::orchestration::terminal_friend_invite;
use crate::platform::{diagnostics_log_backend_event, DiagnosticsState};
use crate::runtime::{resize_pty, StorageManager};

use super::super::models::{
    TerminalSnapshotLinesPayload, TerminalSnapshotPayload, TerminalStatusPayload,
};
use super::launch::launch_terminal_with_fallback;
use super::snapshot_service;
use super::state::{PostReadyMode, PostReadyState, TerminalSessionStatus, TerminalType, TerminalSnapshot};
use super::{
    build_status_payload, ensure_session_active, flush_input_buffer, handle_buffered_write,
    lock_session_status_precreate, lock_sessions, mark_session_working_on_input, register_session,
    resolve_terminal_type, spawn_exit_watcher, spawn_pty_reader,
    subtract_unacked_bytes, terminal_trace_detail, unlock_session_status_precreate,
    update_session_status, InitialWriteState, SemanticEvent,
    REDRAW_SUPPRESSION_WINDOW_MS, SESSION_COUNTER,
};
use super::{TerminalDispatchContext, TerminalManager};
use crate::terminal_engine::default_members::{
    apply_resume_command, apply_unlimited_access_command,
};

/// 创建终端会话并启动进程。
/// 输入：`cols/rows` 为可选尺寸；`cwd` 为工作目录；`terminal_id` 可选用于指定会话标识；
/// `terminal_type/terminal_command/terminal_path` 决定启动方式与二进制来源；shell 类型时 terminal_path 可覆盖默认 shell。
/// `strict_shell` 为 true 时仅按指定路径/系统默认启动，不再启用兜底候选。
/// 邀请元信息：`member_name/default_command/instance_count/unlimited_access/sandboxed` 仅用于编排记录；
/// `post_ready_mode` 控制是否执行启动后流程。
/// 约束：非 shell 类型会使用 `terminal_command` 的参数（如包含旗标）。
/// 返回：新会话 ID。
/// 错误：会话 ID 冲突、二进制不可用、PTY 启动失败或命令解析失败。
pub(crate) fn terminal_create(
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, TerminalManager>,
    cols: Option<u16>,
    rows: Option<u16>,
    cwd: Option<String>,
    member_id: Option<String>,
    workspace_id: Option<String>,
    keep_alive: Option<bool>,
    terminal_id: Option<String>,
    terminal_type: Option<String>,
    terminal_command: Option<String>,
    terminal_path: Option<String>,
    strict_shell: Option<bool>,
    post_ready_mode: Option<String>,
    member_name: Option<String>,
    default_command: Option<String>,
    invite_instance_count: Option<u32>,
    invite_unlimited_access: Option<bool>,
    invite_sandboxed: Option<bool>,
) -> Result<String, String> {
    // 好友创建相关流程（后端侧）：
    // 1) 若带邀请元信息，创建前先预锁状态，避免 UI 误判为已连接。
    // 2) 启动 PTY 并注册会话，注册时消费预锁并进入 Connecting。
    // 3) 启动后置流程（诊断记录），post_ready 完成后解锁并回到 Online。
    // 4) 注册完成后若状态为 Connecting，立即广播一次状态事件。
    let requested_id = terminal_id
        .as_deref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let derived_member_id = workspace_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .zip(
            member_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .map(|(workspace_id, member_id)| format!("member-{workspace_id}-{member_id}"));
    let command = terminal_command
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    let terminal_type = resolve_terminal_type(terminal_type.as_deref(), command.as_deref());
    let requested_terminal_type = terminal_type;
    let post_ready_mode = PostReadyMode::from_str(post_ready_mode.as_deref());
    let command = apply_unlimited_access_command(
        terminal_type.as_str(),
        command,
        invite_unlimited_access.unwrap_or(false),
    );
    // 查询数据库中的 session_id，仅在非邀请打开时追加 resume。
    let command = if post_ready_mode.should_run() {
        command
    } else {
        let session_id = workspace_id
            .as_deref()
            .zip(member_id.as_deref())
            .and_then(|(ws, m)| {
                state
                    .session_repository()
                    .get_terminal_session(ws, m)
                    .ok()
                    .flatten()
            });
        if session_id.is_some() {
            log::info!(
                "terminal resume session found terminal_type={} session_id={}",
                terminal_type.as_str(),
                session_id.as_deref().unwrap_or("")
            );
        }
        apply_resume_command(terminal_type.as_str(), command, session_id.as_deref())
    };
    let terminal_path = terminal_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    let strict_shell = strict_shell.unwrap_or(false);
    let invite_member_name = member_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    let invite_default_command = default_command
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    let member_id_for_log = member_id.clone();
    let workspace_id_for_log = workspace_id.clone();
    let command_for_log = command.clone();
    let terminal_path_for_log = terminal_path.clone();
    let sanitized_command = command
        .as_deref()
        .map(|value| value.replace('\r', "\\r").replace('\n', "\\n"))
        .unwrap_or_else(|| "<none>".to_string());
    let sanitized_path = terminal_path.as_deref().unwrap_or("<none>");
    log::info!(
        "terminal_create type={} command={} path={}",
        terminal_type.as_str(),
        sanitized_command,
        sanitized_path
    );
    let terminal_id = requested_id
        .or(derived_member_id)
        .unwrap_or_else(|| format!("term-{}", SESSION_COUNTER.fetch_add(1, Ordering::Relaxed)));
    let keep_alive = keep_alive.unwrap_or(false);
    let owner_window_label = if keep_alive {
        None
    } else {
        Some(window.label().to_string())
    };
    let output_window_label = Some(window.label().to_string());
    let owner_window_label_for_log = owner_window_label.clone();
    let output_window_label_for_log = output_window_label.clone();
    {
        let guard = state
            .sessions
            .lock()
            .map_err(|_| "terminal session lock poisoned".to_string())?;
        if guard.sessions.contains_key(&terminal_id) {
            return Err("terminal session already exists".to_string());
        }
    }

    // 创建前预锁：只要有邀请元信息就先锁状态，确保创建期间保持 Connecting。
    let should_prelock = member_id_for_log.is_some() && post_ready_mode.should_run();
    if should_prelock {
        lock_session_status_precreate(&state.sessions, &terminal_id);
    }

    // 默认 80x24 兼容传统终端程序，同时避免 0 行列导致 PTY 失败。
    let cols = cols.unwrap_or(80).max(1);
    let rows = rows.unwrap_or(24).max(1);
    let storage = app.state::<StorageManager>();
    let launch_cwd = cwd.clone();
    let launch_result = match launch_terminal_with_fallback(
        storage.inner(),
        cols,
        rows,
        cwd,
        terminal_type,
        terminal_path.as_deref(),
        command.as_deref(),
        strict_shell,
    ) {
        Ok(result) => result,
        Err(err) => {
            if should_prelock {
                unlock_session_status_precreate(&state.sessions, &terminal_id);
            }
            return Err(err.to_invoke_error());
        }
    };
    let spawned = launch_result.spawned;
    let terminal_type = launch_result.terminal_type;
    let fallback_used = launch_result.fallback_used;
    let default_terminal = launch_result.default_terminal;
    let terminal_type_for_log = terminal_type.as_str().to_string();
    if fallback_used && terminal_type != requested_terminal_type {
        log::warn!(
            "terminal type fallback from {} to {}",
            requested_terminal_type.as_str(),
            terminal_type.as_str()
        );
    }
    let resolved_default_terminal_name = default_terminal.name;
    let resolved_default_terminal_path = default_terminal.path;
    let writer = Arc::clone(&spawned.handle.writer);
    let mut cleanup_killer = spawned.child.clone_killer();
    let event_port = state.event_port();
    let semantic_tx = None;
    let status_payload = match register_session(
        &state,
        &terminal_id,
        member_id,
        invite_member_name.clone(),
        workspace_id,
        launch_cwd.clone(),
        command.clone(),
        terminal_path.clone(),
        strict_shell,
        post_ready_mode,
        rows,
        cols,
        terminal_type,
        keep_alive,
        owner_window_label,
        output_window_label,
        semantic_tx.clone(),
        spawned.handle,
    ) {
        Ok(payload) => payload,
        Err(err) => {
            let _ = cleanup_killer.kill();
            if should_prelock {
                unlock_session_status_precreate(&state.sessions, &terminal_id);
            }
            return Err(err);
        }
    };
    let invite_payload = terminal_friend_invite::TerminalFriendInvitePayload {
        terminal_id: terminal_id.clone(),
        member_id: member_id_for_log.clone(),
        workspace_id: workspace_id_for_log.clone(),
        terminal_type: terminal_type_for_log.clone(),
        member_name: invite_member_name,
        default_command: invite_default_command,
        instance_count: invite_instance_count,
        unlimited_access: invite_unlimited_access,
        sandboxed: invite_sandboxed,
        default_terminal_name: resolved_default_terminal_name,
        default_terminal_path: resolved_default_terminal_path,
        post_ready_enabled: post_ready_mode.should_run(),
    };

    let initial_payload = if matches!(terminal_type, TerminalType::Shell) {
        command.as_deref().map(|value| {
            if value.ends_with('\n') || value.ends_with('\r') {
                value.to_string()
            } else {
                format!("{value}\r")
            }
        })
    } else {
        None
    };
    // Windows 启动与首帧输出通常更慢，适当延迟避免误判就绪。
    let (initial_timeout_ms, initial_delay_ms) = if cfg!(windows) { (1200, 300) } else { (500, 0) };
    let initial_write = initial_payload.map(|payload| {
        Arc::new(InitialWriteState {
            terminal_id: terminal_id.clone(),
            payload,
            writer,
            sessions: state.sessions.clone(),
            event_port: Arc::clone(&event_port),
            sent: AtomicBool::new(false),
        })
    });
    if let Some(state) = initial_write.as_ref() {
        state.schedule(initial_timeout_ms, "timeout");
    }

    // 启动好友后置流程（诊断记录 + post_ready 完成后解锁）：若接管门禁，则本地不再重复发状态。
    let should_emit_status =
        !terminal_friend_invite::start_friend_flow(&app, &state, invite_payload);

    let session_repository = {
        let guard = state
            .session_repository
            .lock()
            .map_err(|_| "session repository lock poisoned".to_string())?;
        Arc::clone(&guard)
    };

    let settings_service = {
        let guard = state
            .settings_service
            .lock()
            .map_err(|_| "settings service lock poisoned".to_string())?;
        guard.clone()
    };

    spawn_pty_reader(
        spawned.reader,
        app.clone(),
        Arc::clone(&event_port),
        state.sessions.clone(),
        terminal_id.clone(),
        initial_write,
        initial_delay_ms,
        session_repository,
        settings_service,
    );
    let spawn_epoch = {
        let guard = lock_sessions(&state.sessions);
        guard
            .sessions
            .get(&terminal_id)
            .map(|session| session.spawn_epoch)
            .unwrap_or(0)
    };
    spawn_exit_watcher(
        spawned.child,
        Arc::clone(&event_port),
        state.sessions.clone(),
        terminal_id.clone(),
        spawn_epoch,
    );

    // 若处于 Connecting，立刻广播一次状态，确保跨窗口/订阅方同步。
    if status_payload.status == "connecting" {
        let _ = event_port.emit_status(status_payload.clone());
    } else if should_emit_status {
        let _ = event_port.emit_status(status_payload);
    }
    diagnostics_log_backend_event(
        &app.state::<DiagnosticsState>(),
        member_id_for_log.clone(),
        Some(terminal_id.clone()),
        None,
        output_window_label_for_log.clone(),
        workspace_id_for_log.clone(),
        "terminal_create",
        json!({
          "terminalId": terminal_id,
          "memberId": member_id_for_log,
          "workspaceId": workspace_id_for_log,
          "terminalType": terminal_type_for_log,
          "terminalCommand": command_for_log,
          "terminalPath": terminal_path_for_log,
          "cols": cols,
          "rows": rows,
          "keepAlive": keep_alive,
          "ownerWindowLabel": owner_window_label_for_log,
          "outputWindowLabel": output_window_label_for_log,
          "postReadyMode": post_ready_mode.as_str()
        }),
    );
    Ok(terminal_id)
}

/// post_ready 解析会话 ID 超时后的重启：关闭当前进程并在原会话上重启。
pub(crate) fn terminal_restart_post_ready(
    app: AppHandle,
    state: State<'_, TerminalManager>,
    terminal_id: &str,
    reason: &str,
) -> Result<(), String> {
    let (
        rows,
        cols,
        terminal_type,
        command,
        terminal_path,
        cwd,
        strict_shell,
        member_id,
        workspace_id,
        keep_alive,
        status_locked,
        owner_window_label,
        output_window_label,
        handle,
        semantic_tx,
    ) = {
        let mut guard = state
            .sessions
            .lock()
            .map_err(|_| "terminal session lock poisoned".to_string())?;
        let session = guard
            .sessions
            .get_mut(terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        if !session.post_ready_restart_pending {
            return Ok(());
        }
        session.post_ready_restart_pending = false;
        session.flow_paused = true;
        (
            session.screen_rows,
            session.screen_cols,
            session.terminal_type,
            session.launch_command.clone(),
            session.launch_path.clone(),
            session.launch_cwd.clone(),
            session.launch_strict_shell,
            session.member_id.clone(),
            session.workspace_id.clone(),
            session.keep_alive,
            session.status_locked,
            session.owner_window_label.clone(),
            session.output_window_label.clone(),
            session.handle.take(),
            session.semantic_tx.take(),
        )
    };

    if let Some(mut killer) = handle.map(|handle| handle.killer) {
        thread::spawn(move || {
            let _ = killer.kill();
        });
    }
    if let Some(tx) = semantic_tx {
        let _ = tx.send(SemanticEvent::Shutdown);
    }

    let event_port = state.event_port();
    let cols = cols.max(1);
    let rows = rows.max(1);
    let storage = app.state::<StorageManager>();
    let launch_result = match launch_terminal_with_fallback(
        storage.inner(),
        cols,
        rows,
        cwd.clone(),
        terminal_type,
        terminal_path.as_deref(),
        command.as_deref(),
        strict_shell,
    ) {
        Ok(result) => result,
        Err(err) => {
            let status_payload = {
                let mut guard = state
                    .sessions
                    .lock()
                    .map_err(|_| "terminal session lock poisoned".to_string())?;
                guard.sessions.get_mut(terminal_id).map(|session| {
                    session.post_ready_state = PostReadyState::Done;
                    session.flow_paused = false;
                    session.active = false;
                    session.status_locked = false;
                    session.status = TerminalSessionStatus::Offline;
                    build_status_payload(session)
                })
            };
            if let Some(payload) = status_payload {
                let _ = event_port.emit_status(payload);
            }
            return Err(err.to_invoke_error());
        }
    };

    let spawned = launch_result.spawned;
    let terminal_type = launch_result.terminal_type;
    let fallback_used = launch_result.fallback_used;
    if fallback_used {
        log::warn!(
            "terminal post_ready restart fallback terminal_id={} type={}",
            terminal_id,
            terminal_type.as_str()
        );
    }

    let writer = Arc::clone(&spawned.handle.writer);
    let event_port = state.event_port();
    let semantic_tx = None;

    let (status_payload, spawn_epoch) = {
        let mut guard = state
            .sessions
            .lock()
            .map_err(|_| "terminal session lock poisoned".to_string())?;
        let (payload, member_id, spawn_epoch) = {
            let session = guard
                .sessions
                .get_mut(terminal_id)
                .ok_or_else(|| "terminal session not found".to_string())?;
            let spawn_epoch = session.spawn_epoch.saturating_add(1);
            session.spawn_epoch = spawn_epoch;
            session.active = true;
            session.terminal_type = terminal_type;
            session.screen_rows = rows;
            session.screen_cols = cols;
            session.output_bytes_total = 0;
            session.unacked_bytes = 0;
            session.last_activity_at = None;
            session.last_output_at = None;
            session.last_read_at = None;
            session.last_applied_at = None;
            session.idle_candidate_at = None;
            session.working_intent_until = None;
            session.chat_pending_since = None;
            session.chat_candidate_at = None;
            session.chat_pending = false;
            session.semantic_active = false;
            session.chat_stream_enabled = true;
            session.redraw_suppression_until = None;
            session.flow_paused = false;
            session.broken = false;
            session.post_ready_state = if session.post_ready_mode.should_run() {
                PostReadyState::Idle
            } else {
                PostReadyState::Done
            };
            session.post_ready_queue.clear();
            session.remote_session_id = None;
            session.post_ready_session_id_started_at = None;
            session.post_ready_restart_pending = false;
            session.shell_ready = false;
            session.ready_probe_bytes = 0;
            session.input_buffer.clear();
            session.pending_output_chunks = Arc::new(AtomicUsize::new(0));
            session.output_rate_samples.clear();
            session.handle = Some(spawned.handle);
            session.semantic_tx = semantic_tx.clone();
            session.snapshot = TerminalSnapshot::new(rows, cols, Some(Arc::clone(&writer)));
            session.status = if status_locked {
                TerminalSessionStatus::Connecting
            } else {
                TerminalSessionStatus::Online
            };
            (build_status_payload(session), session.member_id.clone(), spawn_epoch)
        };
        if let Some(member_id) = member_id {
            guard
                .member_sessions
                .entry(member_id)
                .or_insert_with(|| terminal_id.to_string());
        }
        (payload, spawn_epoch)
    };

    let initial_payload = if matches!(terminal_type, TerminalType::Shell) {
        command.as_deref().map(|value| {
            if value.ends_with('\n') || value.ends_with('\r') {
                value.to_string()
            } else {
                format!("{value}\r")
            }
        })
    } else {
        None
    };
    let (initial_timeout_ms, initial_delay_ms) = if cfg!(windows) { (1200, 300) } else { (500, 0) };
    let initial_write = initial_payload.map(|payload| {
        Arc::new(InitialWriteState {
            terminal_id: terminal_id.to_string(),
            payload,
            writer,
            sessions: state.sessions.clone(),
            event_port: Arc::clone(&event_port),
            sent: AtomicBool::new(false),
        })
    });
    if let Some(state) = initial_write.as_ref() {
        state.schedule(initial_timeout_ms, "timeout");
    }

    let session_repository = {
        let guard = state
            .session_repository
            .lock()
            .map_err(|_| "session repository lock poisoned".to_string())?;
        Arc::clone(&guard)
    };
    let settings_service = {
        let guard = state
            .settings_service
            .lock()
            .map_err(|_| "settings service lock poisoned".to_string())?;
        guard.clone()
    };

    spawn_pty_reader(
        spawned.reader,
        app.clone(),
        Arc::clone(&event_port),
        state.sessions.clone(),
        terminal_id.to_string(),
        initial_write,
        initial_delay_ms,
        session_repository,
        settings_service,
    );
    spawn_exit_watcher(
        spawned.child,
        Arc::clone(&event_port),
        state.sessions.clone(),
        terminal_id.to_string(),
        spawn_epoch,
    );

    let _ = event_port.emit_status(status_payload);
    diagnostics_log_backend_event(
        &app.state::<DiagnosticsState>(),
        member_id.clone(),
        Some(terminal_id.to_string()),
        None,
        output_window_label.clone(),
        workspace_id.clone(),
        "terminal_restart_post_ready",
        json!({
          "terminalId": terminal_id,
          "memberId": member_id,
          "workspaceId": workspace_id,
          "terminalType": terminal_type.as_str(),
          "terminalCommand": command,
          "terminalPath": terminal_path,
          "cols": cols,
          "rows": rows,
          "keepAlive": keep_alive,
          "ownerWindowLabel": owner_window_label,
          "outputWindowLabel": output_window_label,
          "reason": reason
        }),
    );
    Ok(())
}

/// 获取会话快照，用于前端 attach 时恢复可视内容。
/// 返回：ANSI 快照与当前序列号。
/// 错误：会话不存在或内部快照生成失败。
pub(crate) fn terminal_attach(
    window: WebviewWindow,
    state: State<'_, TerminalManager>,
    terminal_id: String,
) -> Result<TerminalSnapshotPayload, String> {
    let output_window_label = window.label().to_string();
    let (
        data,
        history,
        snapshot_metrics,
        data_len,
        seq,
        member_id,
        workspace_id,
        terminal_type,
        screen_rows,
        screen_cols,
        cursor_row,
        cursor_col,
    ) = {
        let mut guard = lock_sessions(&state.sessions);
        let session = guard
            .sessions
            .get_mut(&terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        session.output_window_label = Some(output_window_label.clone());
        // 为了解决切换标签/attach 误触发 Working，布局期短暂抑制状态触发。
        let now = now_millis().unwrap_or(0);
        session.redraw_suppression_until = Some(now.saturating_add(REDRAW_SUPPRESSION_WINDOW_MS));
        let snapshot = snapshot_service::snapshot_text_with_history(&session.snapshot);
        let data_len = snapshot.data.as_bytes().len();
        let history_len = snapshot
            .history
            .as_ref()
            .map(|value| value.as_bytes().len())
            .unwrap_or(0);
        let cursor = session.snapshot.cursor_position();
        session.unacked_bytes = session
            .unacked_bytes
            .saturating_add(data_len.saturating_add(history_len));
        (
            snapshot.data,
            snapshot.history,
            snapshot.metrics,
            data_len,
            session.output_seq,
            session.member_id.clone(),
            session.workspace_id.clone(),
            session.terminal_type.as_str().to_string(),
            session.screen_rows,
            session.screen_cols,
            cursor.0.saturating_add(1),
            cursor.1.saturating_add(1),
        )
    };
    let data_line_count = if data.is_empty() {
        0
    } else {
        data.split("\r\n").count()
    };
    log::info!(
        "terminal_attach terminal_id={} seq={} data_len={}",
        terminal_id,
        seq,
        data_len
    );
    if terminal_trace_detail() {
        log::info!(
            "terminal_attach_window terminal_id={} window={}",
            terminal_id,
            output_window_label
        );
    }
    diagnostics_log_backend_event(
        &window.app_handle().state::<DiagnosticsState>(),
        member_id.clone(),
        Some(terminal_id.clone()),
        None,
        Some(output_window_label.clone()),
        workspace_id.clone(),
        "terminal_attach",
        json!({
          "terminalId": terminal_id,
          "memberId": member_id,
          "workspaceId": workspace_id,
          "terminalType": terminal_type,
          "windowLabel": output_window_label,
          "seq": seq,
          "data": data,
          "dataLen": data_len,
          "dataLineCount": data_line_count,
          "screenRows": screen_rows,
          "screenCols": screen_cols,
          "cursorRow": cursor_row,
          "cursorCol": cursor_col,
          "history": history.clone(),
          "scrollbackRows": snapshot_metrics.scrollback_rows,
          "visibleRows": snapshot_metrics.visible_rows,
          "historyRows": snapshot_metrics.history_rows,
          "dataLastContentRow": snapshot_metrics.data_last_content_row
        }),
    );
    Ok(TerminalSnapshotPayload {
        terminal_id: terminal_id.clone(),
        data,
        seq,
        rows: screen_rows,
        cols: screen_cols,
        cursor_row,
        cursor_col,
        history,
    })
}

/// 获取会话快照文本行，便于前端对比当前视口与后端状态。
/// 返回：文本行列表与当前序列号。
/// 错误：会话不存在。
pub(crate) fn terminal_snapshot_lines(
    app: AppHandle,
    state: State<'_, TerminalManager>,
    terminal_id: String,
) -> Result<TerminalSnapshotLinesPayload, String> {
    let (
        lines,
        seq,
        member_id,
        workspace_id,
        terminal_type,
        screen_rows,
        screen_cols,
        cursor_row,
        cursor_col,
    ) = {
        let mut guard = lock_sessions(&state.sessions);
        let session = guard
            .sessions
            .get_mut(&terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        let lines = snapshot_service::snapshot_lines(&session.snapshot);
        let cursor = session.snapshot.cursor_position();
        (
            lines,
            session.output_seq,
            session.member_id.clone(),
            session.workspace_id.clone(),
            session.terminal_type.as_str().to_string(),
            session.screen_rows,
            session.screen_cols,
            cursor.0,
            cursor.1,
        )
    };
    let line_count = lines.len();
    let mut non_empty_count = 0usize;
    let mut first_non_empty: Option<usize> = None;
    let mut last_non_empty: Option<usize> = None;
    let mut max_line_length = 0usize;
    for (index, line) in lines.iter().enumerate() {
        let line_length = line.chars().count();
        if line_length > max_line_length {
            max_line_length = line_length;
        }
        if !line.trim().is_empty() {
            non_empty_count = non_empty_count.saturating_add(1);
            if first_non_empty.is_none() {
                first_non_empty = Some(index);
            }
            last_non_empty = Some(index);
        }
    }
    diagnostics_log_backend_event(
        &app.state::<DiagnosticsState>(),
        member_id.clone(),
        Some(terminal_id.clone()),
        None,
        None,
        workspace_id.clone(),
        "terminal_snapshot_lines",
        json!({
          "terminalId": terminal_id,
          "memberId": member_id,
          "workspaceId": workspace_id,
          "terminalType": terminal_type,
          "seq": seq,
          "lines": lines.clone(),
          "lineCount": line_count,
          "nonEmptyCount": non_empty_count,
          "firstNonEmpty": first_non_empty,
          "lastNonEmpty": last_non_empty,
          "maxLineLength": max_line_length,
          "screenRows": screen_rows,
          "screenCols": screen_cols,
          "cursorRow": cursor_row,
          "cursorCol": cursor_col
        }),
    );
    Ok(TerminalSnapshotLinesPayload {
        terminal_id,
        seq,
        lines,
    })
}

/// 只读获取会话快照文本，不改变输出窗口路由。
/// 返回：ANSI 快照与当前序列号。
/// 错误：会话不存在或快照生成失败。
pub(crate) fn terminal_snapshot_text(
    app: AppHandle,
    state: State<'_, TerminalManager>,
    terminal_id: String,
) -> Result<TerminalSnapshotPayload, String> {
    let (
        data,
        history,
        snapshot_metrics,
        seq,
        member_id,
        workspace_id,
        terminal_type,
        screen_rows,
        screen_cols,
        cursor_row,
        cursor_col,
    ) = {
        let mut guard = lock_sessions(&state.sessions);
        let session = guard
            .sessions
            .get_mut(&terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        let snapshot = snapshot_service::snapshot_text_with_history(&session.snapshot);
        let cursor = session.snapshot.cursor_position();
        (
            snapshot.data,
            snapshot.history,
            snapshot.metrics,
            session.output_seq,
            session.member_id.clone(),
            session.workspace_id.clone(),
            session.terminal_type.as_str().to_string(),
            session.screen_rows,
            session.screen_cols,
            cursor.0.saturating_add(1),
            cursor.1.saturating_add(1),
        )
    };
    diagnostics_log_backend_event(
        &app.state::<DiagnosticsState>(),
        member_id.clone(),
        Some(terminal_id.clone()),
        None,
        None,
        workspace_id.clone(),
        "terminal_snapshot_text",
        json!({
          "terminalId": terminal_id,
          "memberId": member_id,
          "workspaceId": workspace_id,
          "terminalType": terminal_type,
          "seq": seq,
          "data": data.clone(),
          "history": history.clone(),
          "screenRows": screen_rows,
          "screenCols": screen_cols,
          "scrollbackRows": snapshot_metrics.scrollback_rows,
          "visibleRows": snapshot_metrics.visible_rows,
          "historyRows": snapshot_metrics.history_rows,
          "dataLastContentRow": snapshot_metrics.data_last_content_row
        }),
    );
    Ok(TerminalSnapshotPayload {
        terminal_id,
        data,
        seq,
        rows: screen_rows,
        cols: screen_cols,
        cursor_row,
        cursor_col,
        history,
    })
}

/// 写入用户输入到终端会话。
/// 输入：`data` 为原始按键流（可能包含控制字符）。
/// 错误：会话不存在、已崩溃或已关闭。
pub(crate) fn terminal_write(
    app: AppHandle,
    state: State<'_, TerminalManager>,
    terminal_id: String,
    data: String,
) -> Result<(), String> {
    ensure_session_active(&state, &terminal_id)?;
    let event_port = state.event_port();
    mark_session_working_on_input(
        &state.sessions,
        event_port.as_ref(),
        &terminal_id,
        &data,
        None,
        None,
        None,
    );
    let now = now_millis()?;
    let (should_write, buffered, writer, shell_ready, member_id, workspace_id, terminal_type) = {
        let mut guard = lock_sessions(&state.sessions);
        let session = guard
            .sessions
            .get_mut(&terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        let shell_ready = session.shell_ready;
        let handle = session
            .handle
            .as_ref()
            .ok_or_else(|| "terminal session handle missing".to_string())?;
        let writer = Arc::clone(&handle.writer);
        let (should_write, buffered) = handle_buffered_write(session, data.clone(), now);
        (
            should_write,
            buffered,
            writer,
            shell_ready,
            session.member_id.clone(),
            session.workspace_id.clone(),
            session.terminal_type.as_str().to_string(),
        )
    };
    if terminal_trace_detail() {
        let buffered_bytes: usize = buffered.iter().map(|entry| entry.len()).sum();
        log::info!(
      "terminal_write terminal_id={} data_len={} buffered_bytes={} write_now={} shell_ready={}",
      terminal_id,
      data.len(),
      buffered_bytes,
      should_write,
      shell_ready
    );
    }
    diagnostics_log_backend_event(
        &app.state::<DiagnosticsState>(),
        member_id.clone(),
        Some(terminal_id.clone()),
        None,
        None,
        workspace_id.clone(),
        "terminal_write",
        json!({
          "terminalId": terminal_id,
          "memberId": member_id,
          "workspaceId": workspace_id,
          "terminalType": terminal_type,
          "data": data,
          "shellReady": shell_ready,
          "writeNow": should_write,
          "bufferedCount": buffered.len()
        }),
    );
    if !should_write {
        return Ok(());
    }
    flush_input_buffer(&writer, buffered)?;
    Ok(())
}

/// 设置会话在 UI 中是否活跃，用于流控与输出节流策略。
/// 错误：会话不存在。
pub(crate) fn terminal_set_active(
    state: State<'_, TerminalManager>,
    terminal_id: String,
    active: bool,
) -> Result<(), String> {
    let mut guard = lock_sessions(&state.sessions);
    let session = guard
        .sessions
        .get_mut(&terminal_id)
        .ok_or_else(|| "terminal session not found".to_string())?;
    session.ui_active = active;
    if active {
        session.idle_candidate_at = None;
        let now = now_millis()?;
        // 切换为活跃时短暂抑制 Working，避免布局重绘导致误触发。
        session.redraw_suppression_until = Some(now.saturating_add(REDRAW_SUPPRESSION_WINDOW_MS));
    }
    if terminal_trace_detail() {
        log::info!(
            "terminal_set_active terminal_id={} active={}",
            terminal_id,
            active
        );
    }
    Ok(())
}

/// 手动广播指定会话的当前状态，用于前端复用会话时同步状态。
pub(crate) fn terminal_emit_status(
    state: State<'_, TerminalManager>,
    terminal_id: String,
) -> Result<(), String> {
    let payload = {
        let guard = lock_sessions(&state.sessions);
        let session = guard
            .sessions
            .get(&terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        build_status_payload(session)
    };
    let event_port = state.event_port();
    event_port.emit_status(payload)?;
    Ok(())
}

/// 同步成员状态到终端侧（online/working/dnd/offline）。
/// 约束：不在白名单中的状态会被清空以回退到默认行为。
pub(crate) fn terminal_set_member_status(
    state: State<'_, TerminalManager>,
    member_id: String,
    status: String,
) -> Result<(), String> {
    let normalized = status.trim().to_lowercase();
    let mut guard = lock_sessions(&state.sessions);
    if normalized.is_empty() {
        guard.member_statuses.remove(&member_id);
        return Ok(());
    }
    if matches!(
        normalized.as_str(),
        "online" | "working" | "dnd" | "offline"
    ) {
        guard.member_statuses.insert(member_id, normalized);
        return Ok(());
    }
    guard.member_statuses.remove(&member_id);
    Ok(())
}

/// 获取终端会话状态快照，供前端同步展示。
/// 输入：可选 workspaceId；为空则返回全部工作区会话。
/// 输出：终端会话状态列表（仅包含具备成员信息的会话）。
pub(crate) fn terminal_list_statuses(
    state: State<'_, TerminalManager>,
    workspace_id: Option<String>,
) -> Vec<TerminalStatusPayload> {
    let workspace_filter = workspace_id.as_deref();
    let guard = lock_sessions(&state.sessions);
    let result: Vec<TerminalStatusPayload> = guard
        .sessions
        .values()
        .filter(|session| session.member_id.is_some())
        .filter(|session| match workspace_filter {
            Some(workspace_id) => session.workspace_id.as_deref() == Some(workspace_id),
            None => true,
        })
        .map(build_status_payload)
        .collect();
    result
}

/// 确认已消费的输出字节数，用于后端流控。
/// 约束：若会话不存在则无副作用。
pub(crate) fn terminal_ack(
    state: State<'_, TerminalManager>,
    terminal_id: String,
    count: usize,
) -> Result<(), String> {
    subtract_unacked_bytes(&state.sessions, &terminal_id, count);
    if terminal_trace_detail() {
        let current = {
            let guard = lock_sessions(&state.sessions);
            guard
                .sessions
                .get(&terminal_id)
                .map(|session| session.unacked_bytes)
                .unwrap_or(0)
        };
        log::info!(
            "terminal_ack terminal_id={} count={} unacked={}",
            terminal_id,
            count,
            current
        );
    }
    Ok(())
}

/// 派发聊天上下文相关的输入到终端会话。
/// 约束：DND 时会直接跳过。
/// 错误：会话不存在、已崩溃或已关闭。
pub(crate) fn terminal_dispatch(
    app: AppHandle,
    state: State<'_, TerminalManager>,
    terminal_id: String,
    data: String,
    context: TerminalDispatchContext,
) -> Result<(), String> {
    let event_port = state.event_port();
    super::dispatch_input_with_context(
        &app,
        &state.sessions,
        event_port.as_ref(),
        &terminal_id,
        data,
        context,
    )
}

/// 编排聊天指令派发，必要时进入队列等待 Online。
pub(crate) fn terminal_dispatch_chat(
    app: AppHandle,
    state: State<'_, TerminalManager>,
    terminal_id: String,
    text: String,
    context: TerminalDispatchContext,
) -> Result<(), String> {
    let event_port = state.event_port();
    let outcome = super::dispatch_chat_or_queue(
        &app,
        &state.sessions,
        event_port.as_ref(),
        &terminal_id,
        text,
        context,
    )?;
    if terminal_trace_detail() {
        log::info!(
            "terminal_dispatch_chat terminal_id={} outcome={:?}",
            terminal_id,
            outcome
        );
    }
    Ok(())
}

/// 调整终端尺寸并同步到模拟器与 PTY。
/// 错误：会话不存在或 PTY 调整失败。
pub(crate) fn terminal_resize(
    app: AppHandle,
    state: State<'_, TerminalManager>,
    terminal_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    ensure_session_active(&state, &terminal_id)?;
    let rows = rows.max(1);
    let cols = cols.max(1);
    let mut guard = state
        .sessions
        .lock()
        .map_err(|_| "terminal session lock poisoned".to_string())?;
    let session = guard
        .sessions
        .get_mut(&terminal_id)
        .ok_or_else(|| "terminal session not found".to_string())?;
    let prev_rows = session.screen_rows;
    let prev_cols = session.screen_cols;
    if prev_rows == rows && prev_cols == cols {
        return Ok(());
    }
    session.screen_rows = rows;
    session.screen_cols = cols;
    session.snapshot.set_size(rows, cols);
    // 为了解决切换标签/resize 误触发 Working，布局期短暂抑制状态触发。
    let now = now_millis().unwrap_or(0);
    session.redraw_suppression_until = Some(now.saturating_add(REDRAW_SUPPRESSION_WINDOW_MS));
    let handle = session
        .handle
        .as_ref()
        .ok_or_else(|| "terminal session handle missing".to_string())?;
    resize_pty(handle, session.screen_rows, session.screen_cols)?;
    let semantic_tx = session.semantic_tx.clone();
    let member_id = session.member_id.clone();
    let workspace_id = session.workspace_id.clone();
    let terminal_type = session.terminal_type.as_str().to_string();
    let output_seq = session.output_seq;
    drop(guard);
    diagnostics_log_backend_event(
        &app.state::<DiagnosticsState>(),
        member_id.clone(),
        Some(terminal_id.clone()),
        None,
        None,
        workspace_id.clone(),
        "terminal_resize",
        json!({
          "terminalId": terminal_id,
          "memberId": member_id,
          "workspaceId": workspace_id,
          "terminalType": terminal_type,
          "prevRows": prev_rows,
          "prevCols": prev_cols,
          "rows": rows,
          "cols": cols,
          "seq": output_seq
        }),
    );
    if let Some(tx) = semantic_tx {
        let _ = tx.send(SemanticEvent::Resize { rows, cols });
    }
    Ok(())
}

/// 关闭会话；`preserve=true` 仅关闭进程但保留会话条目。
/// 错误：会话不存在。
pub(crate) fn terminal_close(
    app: AppHandle,
    state: State<'_, TerminalManager>,
    terminal_id: String,
    preserve: Option<bool>,
    delete_session_map: Option<bool>,
) -> Result<(), String> {
    let preserve = preserve.unwrap_or(false);
    let delete_session_map = delete_session_map.unwrap_or(false);
    let (
        status_payload,
        killer,
        semantic_tx,
        member_id,
        workspace_id,
        terminal_type,
        keep_alive,
        flush_pending,
        snapshot_seed,
        seed_rows,
        seed_cols,
    ) = {
        let mut guard = state
            .sessions
            .lock()
            .map_err(|_| "terminal session lock poisoned".to_string())?;
        let working_sessions = Arc::clone(&guard.working_sessions);
        let session = guard
            .sessions
            .get_mut(&terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        let mut status_payload = None;
        let mut killer = None;
        let mut semantic_tx = None;
        let mut member_id = None;
        let mut workspace_id = None;
        let mut terminal_type = None;
        let mut keep_alive = None;
        let mut flush_pending = false;
        let mut snapshot_seed = None;
        let mut seed_rows = 0;
        let mut seed_cols = 0;
        if preserve {
            member_id = session.member_id.clone();
            workspace_id = session.workspace_id.clone();
            terminal_type = Some(session.terminal_type.as_str().to_string());
            keep_alive = Some(session.keep_alive);
            flush_pending = session.chat_pending || session.semantic_active;
            if flush_pending {
                snapshot_seed = Some(session.snapshot.snapshot_segments().data);
                seed_rows = session.screen_rows;
                seed_cols = session.screen_cols;
            }
            session.active = false;
            session.last_activity_at = None;
            session.last_output_at = None;
            session.last_read_at = None;
            session.last_applied_at = None;
            session.idle_candidate_at = None;
            session.working_intent_until = None;
            session.chat_pending = false;
            session.chat_pending_since = None;
            session.chat_candidate_at = None;
            session.semantic_active = false;
            session.chat_stream_enabled = true;
            session.redraw_suppression_until = None;
            session.post_ready_state = if session.post_ready_mode.should_run() {
                PostReadyState::Idle
            } else {
                PostReadyState::Done
            };
            session.post_ready_queue.clear();
            session.remote_session_id = None;
            session.post_ready_session_id_started_at = None;
            session.post_ready_session_id_restart_count = 0;
            session.post_ready_restart_pending = false;
            session.flow_paused = false;
            if session.status != TerminalSessionStatus::Offline {
                if update_session_status(&working_sessions, session, TerminalSessionStatus::Offline)
                {
                    session.status_locked = false;
                    status_payload = Some(build_status_payload(session));
                }
            }
            if let Some(handle) = session.handle.take() {
                killer = Some(handle.killer);
            }
            semantic_tx = session.semantic_tx.take();
        } else {
            let removed = guard.sessions.remove(&terminal_id);
            if let Ok(mut working) = guard.working_sessions.lock() {
                working.remove(&terminal_id);
            }
            if let Some(removed) = removed {
                flush_pending = removed.chat_pending || removed.semantic_active;
                if flush_pending {
                    snapshot_seed = Some(removed.snapshot.snapshot_segments().data);
                    seed_rows = removed.screen_rows;
                    seed_cols = removed.screen_cols;
                }
                if let Some(member_id) = removed.member_id.as_ref() {
                    guard.member_sessions.remove(member_id);
                }
                if removed.status != TerminalSessionStatus::Offline {
                    status_payload = Some(TerminalStatusPayload {
                        terminal_id: removed.id.clone(),
                        status: TerminalSessionStatus::Offline.as_str().to_string(),
                        member_id: removed.member_id.clone(),
                        workspace_id: removed.workspace_id.clone(),
                    });
                }
                member_id = removed.member_id;
                workspace_id = removed.workspace_id;
                terminal_type = Some(removed.terminal_type.as_str().to_string());
                keep_alive = Some(removed.keep_alive);
                if let Some(handle) = removed.handle {
                    killer = Some(handle.killer);
                }
                semantic_tx = removed.semantic_tx;
            }
        }
        (
            status_payload,
            killer,
            semantic_tx,
            member_id,
            workspace_id,
            terminal_type,
            keep_alive,
            flush_pending,
            snapshot_seed,
            seed_rows,
            seed_cols,
        )
    };
    if let Some(mut killer) = killer {
        thread::spawn(move || {
            let _ = killer.kill();
        });
    }
    if let Some(tx) = semantic_tx {
        if flush_pending {
            if let Some(snapshot_seed) = snapshot_seed {
                let _ = tx.send(SemanticEvent::SeedSnapshot {
                    rows: seed_rows,
                    cols: seed_cols,
                    data: snapshot_seed,
                });
            }
            let _ = tx.send(SemanticEvent::Flush {
                message_type: "info",
                source: "close",
            });
        }
        let _ = tx.send(SemanticEvent::Shutdown);
    }
    if let Some(payload) = status_payload {
        let event_port = state.event_port();
        let _ = event_port.emit_status(payload);
    }
    diagnostics_log_backend_event(
        &app.state::<DiagnosticsState>(),
        member_id.clone(),
        Some(terminal_id.clone()),
        None,
        None,
        workspace_id.clone(),
        "terminal_close",
        json!({
          "terminalId": terminal_id,
          "memberId": member_id,
          "workspaceId": workspace_id,
          "terminalType": terminal_type,
          "preserve": preserve,
          "keepAlive": keep_alive
        }),
    );
    if !preserve && delete_session_map {
        if let (Some(member_id), Some(workspace_id)) =
            (member_id.as_deref(), workspace_id.as_deref())
        {
            let repository = state.session_repository();
            if let Err(err) = repository.delete_terminal_session(workspace_id, member_id) {
                log::warn!(
                    "terminal session map delete failed member_id={} workspace_id={} err={}",
                    member_id,
                    workspace_id,
                    err
                );
            }
        }
    }
    Ok(())
}

/// 按成员 ID 批量关闭会话并清理映射。
/// 约束：尽量清理，单个会话失败不会中断整体流程。
pub(crate) fn terminal_close_by_member_ids(
    app: AppHandle,
    state: State<'_, TerminalManager>,
    workspace_id: &str,
    member_ids: &[String],
    delete_session_map: bool,
) -> Vec<String> {
    if member_ids.is_empty() {
        return Vec::new();
    }
    let target_ids: HashSet<String> = member_ids
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();
    if target_ids.is_empty() {
        return Vec::new();
    }
    let mut terminal_ids = HashSet::new();
    {
        let mut guard = lock_sessions(&state.sessions);
        for member_id in &target_ids {
            if let Some(terminal_id) = guard.member_sessions.get(member_id) {
                terminal_ids.insert(terminal_id.clone());
            }
            // 成员删除后同步清理状态缓存，避免残留状态影响其他视图。
            guard.member_statuses.remove(member_id);
            guard.member_sessions.remove(member_id);
        }
        for session in guard.sessions.values() {
            let Some(member_id) = session.member_id.as_ref() else {
                continue;
            };
            if target_ids.contains(member_id) {
                terminal_ids.insert(session.id.clone());
            }
        }
    }
    let mut warnings = Vec::new();
    for terminal_id in terminal_ids {
        if let Err(err) = terminal_close(
            app.clone(),
            state.clone(),
            terminal_id.clone(),
            Some(false),
            Some(delete_session_map),
        ) {
            warnings.push(format!(
                "terminal close failed terminal_id={} err={}",
                terminal_id, err
            ));
        }
    }
    if delete_session_map {
        let repository = state.session_repository();
        let workspace_id = workspace_id.trim();
        if !workspace_id.is_empty() {
            for member_id in &target_ids {
                if let Err(err) = repository.delete_terminal_session(workspace_id, member_id) {
                    warnings.push(format!(
                        "terminal session map delete failed member_id={} workspace_id={} err={}",
                        member_id, workspace_id, err
                    ));
                }
            }
        }
    }
    warnings
}
