//! 轮询动作执行器：集中处理会话变更与外部副作用。

use std::sync::{Arc, Mutex};

use serde_json::json;
use tauri::{AppHandle, Manager};

use crate::platform::{backend_passive_enabled, diagnostics_log_backend_event, DiagnosticsState};
use crate::ports::terminal_event::TerminalEventPort;
use crate::ports::terminal_session::TerminalSessionRepository;

use super::super::post_ready::{maybe_start_post_ready, maybe_step_post_ready};
use super::super::{
    build_status_payload, flush_dispatch_queue_if_ready, lock_sessions, terminal_restart_post_ready,
    update_session_status, SemanticEvent, SessionRegistry,
};
use super::actions::{ChatFlushMeta, PollAction, SessionUpdate};

#[derive(Clone, Debug)]
struct ChatFlushLog {
    terminal_id: String,
    member_id: Option<String>,
    workspace_id: Option<String>,
    terminal_type: String,
    last_output_at: Option<u64>,
    chat_pending_since: Option<u64>,
    silence_ms: Option<u64>,
    snapshot_lines: Option<Vec<String>>,
    snapshot_line_count: Option<usize>,
    cursor_row: Option<u16>,
    cursor_col: Option<u16>,
    screen_rows: Option<u16>,
    screen_cols: Option<u16>,
}

pub(crate) fn dispatch_poll_actions(
    app: &AppHandle,
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &Arc<dyn TerminalEventPort>,
    session_repository: &Arc<dyn TerminalSessionRepository>,
    settings_service: Option<Arc<dyn crate::ports::settings::TerminalSettingsPort>>,
    actions: Vec<PollAction>,
) {
    if actions.is_empty() {
        return;
    }
    let mut status_payloads = Vec::new();
    let mut semantic_flushes: Vec<ChatFlushMeta> = Vec::new();
    let mut post_ready_starts = Vec::new();
    let mut post_ready_steps = Vec::new();
    let mut post_ready_restarts = Vec::new();
    let mut dispatch_flush_targets = Vec::new();
    {
        let mut guard = lock_sessions(sessions);
        let working_sessions = Arc::clone(&guard.working_sessions);
        for action in actions.iter() {
            match action {
                PollAction::SessionUpdate(update) => {
                    if let Some(session) = guard.sessions.get_mut(&update.terminal_id) {
                        let became_online = apply_session_update(
                            &working_sessions,
                            session,
                            update,
                            &mut status_payloads,
                        );
                        if became_online {
                            dispatch_flush_targets.push(update.terminal_id.clone());
                        }
                    }
                }
                PollAction::SemanticFlush(meta) => {
                    semantic_flushes.push(meta.clone());
                }
                PollAction::PostReadyStart {
                    terminal_id,
                    terminal_type,
                } => {
                    post_ready_starts.push((terminal_id.clone(), terminal_type.clone()));
                }
                PollAction::PostReadyStep { terminal_id } => {
                    post_ready_steps.push(terminal_id.clone());
                }
                PollAction::PostReadyRestart { terminal_id } => {
                    post_ready_restarts.push(terminal_id.clone());
                }
            }
        }
    }
    let mut chat_flush_logs = Vec::new();
    for meta in semantic_flushes {
        if let Some(log) = handle_semantic_flush(sessions, &meta) {
            chat_flush_logs.push(log);
        }
    }
    for payload in status_payloads {
        let _ = event_port.emit_status(payload);
    }
    for terminal_id in dispatch_flush_targets {
        flush_dispatch_queue_if_ready(app, sessions, event_port.as_ref(), &terminal_id);
    }
    for terminal_id in post_ready_restarts {
        if let Err(err) = terminal_restart_post_ready(
            app.clone(),
            app.state::<super::super::TerminalManager>(),
            &terminal_id,
            "post_ready_session_id_timeout",
        ) {
            log::warn!(
                "terminal post_ready restart failed terminal_id={} err={}",
                terminal_id,
                err
            );
        }
    }
    for (terminal_id, terminal_type) in post_ready_starts {
        if let Err(err) =
            maybe_start_post_ready(sessions, event_port.as_ref(), &terminal_id, &terminal_type)
        {
            log::warn!(
                "terminal post_ready start failed terminal_id={} err={}",
                terminal_id,
                err
            );
        }
    }
    for terminal_id in post_ready_steps {
        if let Err(err) = maybe_step_post_ready(
            sessions,
            event_port.as_ref(),
            session_repository.as_ref(),
            settings_service.as_deref(),
            &terminal_id,
            true,
        ) {
            log::warn!(
                "terminal post_ready step failed terminal_id={} err={}",
                terminal_id,
                err
            );
        }
        flush_dispatch_queue_if_ready(app, sessions, event_port.as_ref(), &terminal_id);
    }
    for log in chat_flush_logs {
        diagnostics_log_backend_event(
            &app.state::<DiagnosticsState>(),
            log.member_id.clone(),
            Some(log.terminal_id.clone()),
            None,
            None,
            log.workspace_id.clone(),
            "terminal_chat_flush_trigger",
            json!({
              "terminalId": log.terminal_id,
              "memberId": log.member_id,
              "workspaceId": log.workspace_id,
              "terminalType": log.terminal_type,
              "lastOutputAt": log.last_output_at,
              "chatPendingSince": log.chat_pending_since,
              "silenceMs": log.silence_ms,
              "snapshotLines": log.snapshot_lines,
              "snapshotLineCount": log.snapshot_line_count,
              "cursorRow": log.cursor_row,
              "cursorCol": log.cursor_col,
              "screenRows": log.screen_rows,
              "screenCols": log.screen_cols
            }),
        );
    }
}

fn apply_session_update(
    working_sessions: &Arc<Mutex<std::collections::HashSet<String>>>,
    session: &mut super::super::TerminalSession,
    update: &SessionUpdate,
    status_payloads: &mut Vec<crate::terminal_engine::models::TerminalStatusPayload>,
) -> bool {
    let mut status_changed = false;
    let mut became_online = false;
    if let Some(status) = update.set_status {
        if session.status != status {
            if update_session_status(working_sessions, session, status) {
                status_changed = true;
                became_online = status == super::super::state::TerminalSessionStatus::Online;
            }
        }
    }
    if let Some(value) = update.set_idle_candidate_at {
        session.idle_candidate_at = value;
    }
    if let Some(value) = update.set_chat_candidate_at {
        session.chat_candidate_at = value;
    }
    if let Some(value) = update.set_chat_pending {
        session.chat_pending = value;
    }
    if let Some(value) = update.set_chat_pending_since {
        session.chat_pending_since = value;
    }
    if let Some(value) = update.set_semantic_active {
        session.semantic_active = value;
    }
    if update.emit_status && status_changed {
        status_payloads.push(build_status_payload(session));
    }
    became_online
}

fn handle_semantic_flush(
    sessions: &Arc<Mutex<SessionRegistry>>,
    meta: &ChatFlushMeta,
) -> Option<ChatFlushLog> {
    let (
        semantic_tx,
        snapshot_seed,
        seed_rows,
        seed_cols,
        snapshot_lines,
        snapshot_line_count,
        cursor_row,
        cursor_col,
        screen_rows,
        screen_cols,
    ) = {
        let mut guard = lock_sessions(sessions);
        let session = guard.sessions.get_mut(meta.terminal_id.as_str())?;
        // 强制用最新快照重建语义仿真器，避免语义线程滞后导致过期行混入。
        let snapshot_seed = session.snapshot.snapshot_segments().data;
        let seed_rows = session.screen_rows;
        let seed_cols = session.screen_cols;
        let (snapshot_lines, snapshot_line_count, cursor_row, cursor_col, screen_rows, screen_cols) =
            if backend_passive_enabled() {
                // 仅在被动监控开启时采集快照细节，避免无意义复制。
                let lines = session.snapshot.snapshot_lines();
                let line_count = lines.len();
                let (cursor_row_0, cursor_col_0) = session.snapshot.cursor_position();
                (
                    Some(lines),
                    Some(line_count),
                    Some(cursor_row_0.saturating_add(1)),
                    Some(cursor_col_0.saturating_add(1)),
                    Some(session.screen_rows),
                    Some(session.screen_cols),
                )
            } else {
                (None, None, None, None, None, None)
            };
        (
            session.semantic_tx.clone(),
            snapshot_seed,
            seed_rows,
            seed_cols,
            snapshot_lines,
            snapshot_line_count,
            cursor_row,
            cursor_col,
            screen_rows,
            screen_cols,
        )
    };
    if let Some(tx) = semantic_tx {
        let _ = tx.send(SemanticEvent::SeedSnapshot {
            rows: seed_rows,
            cols: seed_cols,
            data: snapshot_seed,
        });
        let _ = tx.send(SemanticEvent::Flush {
            message_type: "info",
            source: "pty",
        });
    }
    Some(ChatFlushLog {
        terminal_id: meta.terminal_id.clone(),
        member_id: meta.member_id.clone(),
        workspace_id: meta.workspace_id.clone(),
        terminal_type: meta.terminal_type.clone(),
        last_output_at: meta.last_output_at,
        chat_pending_since: meta.chat_pending_since,
        silence_ms: meta.silence_ms,
        snapshot_lines,
        snapshot_line_count,
        cursor_row,
        cursor_col,
        screen_rows,
        screen_cols,
    })
}
