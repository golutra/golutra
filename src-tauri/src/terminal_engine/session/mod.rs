//! 终端会话管理与 IO 管线：处理 PTY 读写、流控、快照与语义事件。
//! 边界：不参与前端渲染，只通过事件与 Tauri 命令暴露状态与输出。

use std::{
    any::Any,
    collections::{HashSet, VecDeque},
    io::{Read, Write},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use portable_pty::Child;
use serde_json::json;
use tauri::{AppHandle, Manager};

use super::models::{
    TerminalErrorPayload, TerminalExitPayload, TerminalOutputPayload, TerminalStatusPayload,
};
use super::semantic::TerminalChatContext;
use crate::now_millis;
use crate::platform::backend_passive_enabled;
use crate::platform::{diagnostics_log_backend_event, DiagnosticsState};
use crate::ports::settings::TerminalSettingsPort;
use crate::ports::terminal_event::TerminalEventPort;
use crate::runtime::TerminalHandle;
mod commands;
mod keyboard_input;
mod stability;
use stability::record_output_sample;
mod launch;
mod poller;
mod polling;
mod post_ready;
mod semantic_worker;
mod snapshot_dump;
mod snapshot_service;
mod state;
mod trigger;

use state::{DispatchQueueItem, TerminalDispatchEnvelope};

pub(crate) use commands::*;
pub(crate) use poller::spawn_status_poller;
use semantic_worker::{spawn_semantic_worker, SemanticEvent};
pub(crate) use snapshot_dump::spawn_snapshot_dumper;
use state::{
    lock_sessions, PostReadyMode, PostReadyState, SessionRegistry, TerminalSession,
    TerminalSessionStatus, TerminalSnapshot, TerminalType,
};
pub(crate) use state::{TerminalDispatchContext, TerminalManager};
use trigger::FactEvent;

// 行为与性能参数说明：
// - 工作状态回落、chat 静默门禁、输出批次、语义批次与流控阈值均为体验/性能折中。
// - OSC 信号仅用于识别 shim 就绪，避免前端误判状态。
const STATUS_WORKING_SILENCE_TIMEOUT_MS: u64 = 4500; // Working 回落静默阈值，避免过早从 Working 回退到 Online。
const STATUS_IDLE_DEBOUNCE_MS: u64 = 1000; // Working 回落防抖，降低短暂停顿触发回落的概率。
const WORKING_INTENT_WINDOW_MS: u64 = 1500; // 命令提交后的 Working 意图窗口，避免布局输出误触发。
const CHAT_SILENCE_TIMEOUT_MS: u64 = 3000; // chat flush 静默门禁，避免输出未稳定时回写。
const CHAT_IDLE_DEBOUNCE_MS: u64 = 1000; // chat flush 防抖，避免短暂停顿触发回写。
const CHAT_PENDING_FORCE_FLUSH_MS: u64 = 30000; // chat flush 超时兜底，避免进度提示阻塞派发。
const DISPATCH_QUEUE_LIMIT: usize = 32; // 队列上限，避免 Working 状态无限堆积。
const DISPATCH_RECENT_LIMIT: usize = 128; // 去重窗口，防止重复派发。
// 积压合并分隔符：用空行保留原始消息边界，避免引入额外语义。
const DISPATCH_BATCH_SEPARATOR: &str = "\n\n";
const COMMAND_CONFIRM_DELAY_MS: u64 = 100; // 派发确认延迟，避免 CLI 输入模式误判。
const COMMAND_CONFIRM_SUFFIX: &str = "\r"; // 延迟补发回车用于提交指令。
const REDRAW_SUPPRESSION_WINDOW_MS: u64 = 400; // 布局期抑制 Working 触发，覆盖切换标签/attach/resize 场景。
const POST_READY_STABLE_MS: u64 = 1200; // 启动后流程门禁，避免刚启动就触发后置步骤。
const POST_READY_TICK_MS: u64 = 600; // post_ready 无输出时的轻量兜底触发间隔。
const STATUS_POLL_INTERVAL_MS: u64 = 500; // 状态轮询频率，避免频繁更新导致 UI 抖动。
const SESSION_SCROLLBACK_LINES: usize = 2000; // 快照保留行数，上限过大将显著增大内存与快照大小。
const SHIM_READY_SIGNAL: &str = "\x1b]633;A"; // shim 就绪信号（OSC 633;A）。
const SHIM_LAUNCH_ERROR_MARKER: &str = "SHIM_LAUNCH_ERROR"; // shim 启动失败的标记前缀。
const SHELL_READY_TIMEOUT_MS: u64 = 3000; // 超时仍未就绪时放行输入，避免用户阻塞。
const SHELL_READY_ACTIVITY_MS: u64 = 500; // 识别“有输出即就绪”的最小等待时间，避免过早放行。
const SHELL_READY_ACTIVITY_BYTES: usize = 1024; // 输出达到该字节量视为就绪，兼顾慢启动 CLI。
const OUTPUT_EMIT_INTERVAL_MS: u64 = 16; // 输出节流到 ~60fps，减少 IPC 压力。
const OUTPUT_EMIT_MAX_BYTES: usize = 64 * 1024; // 单批次输出上限，防止大块输出阻塞前端。
const OUTPUT_QUEUE_CAPACITY: usize = 256; // PTY 读队列容量，平衡背压与吞吐。
const STATS_LOG_INTERVAL_MS: u64 = 1000; // 统计日志节流，减少噪音。
const FLOW_CONTROL_HIGH_WATERMARK: usize = 200_000; // 未确认输出超过阈值时暂停读取，防止内存膨胀。
const FLOW_CONTROL_LOW_WATERMARK: usize = 20_000; // 低水位恢复读取，避免长时间停顿。
const FLOW_CONTROL_SLEEP_MS: u64 = 2; // 流控休眠步长，兼顾 CPU 与响应性。
const CSI_PARAM_BYTE_START: u8 = 0x30; // CSI 参数字节范围起点（ECMA-48）。
const CSI_PARAM_BYTE_END: u8 = 0x3f; // CSI 参数字节范围终点（ECMA-48）。
const CSI_INTERMEDIATE_BYTE_START: u8 = 0x20; // CSI 中间字节范围起点（ECMA-48）。
const CSI_INTERMEDIATE_BYTE_END: u8 = 0x2f; // CSI 中间字节范围终点（ECMA-48）。
const CSI_FINAL_BYTE_START: u8 = 0x40; // CSI 终止字节范围起点（ECMA-48）。
const CSI_FINAL_BYTE_END: u8 = 0x7e; // CSI 终止字节范围终点（ECMA-48）。

// 会话 ID 自增计数，避免并发创建时冲突。
static SESSION_COUNTER: AtomicUsize = AtomicUsize::new(1);

// 终端类型解析：优先显式类型，其次通过命令名推断。
fn resolve_terminal_type(raw: Option<&str>, command: Option<&str>) -> TerminalType {
    if let Some(value) = TerminalType::from_str(raw) {
        return value;
    }
    let command = command.unwrap_or("").trim();
    if command.is_empty() {
        return TerminalType::Shell;
    }
    let mut parts = command.split_whitespace();
    let binary = parts.next().unwrap_or("").to_lowercase();
    if parts.next().is_none() {
        match binary.as_str() {
            "codex" => return TerminalType::Codex,
            "gemini" => return TerminalType::Gemini,
            "claude" => return TerminalType::Claude,
            "opencode" => return TerminalType::Opencode,
            "qwen" => return TerminalType::Qwen,
            _ => {}
        }
    }
    TerminalType::Shell
}

// DND 时跳过终端派发，避免打扰用户的主动输入。
fn is_member_dnd(sessions: &Arc<Mutex<SessionRegistry>>, terminal_id: &str) -> bool {
    let guard = lock_sessions(sessions);
    let session = match guard.sessions.get(terminal_id) {
        Some(session) => session,
        None => return false,
    };
    let member_id = match session.member_id.as_ref() {
        Some(member_id) => member_id,
        None => return false,
    };
    matches!(
        guard
            .member_statuses
            .get(member_id)
            .map(|status| status.as_str()),
        Some("dnd")
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DispatchOutcome {
    Dispatched,
    Queued,
    Duplicate,
    SkippedDnd,
}

fn ensure_session_active_by_sessions(
    sessions: &Arc<Mutex<SessionRegistry>>,
    terminal_id: &str,
) -> Result<(), String> {
    let guard = lock_sessions(sessions);
    let session = guard
        .sessions
        .get(terminal_id)
        .ok_or_else(|| "terminal session not found".to_string())?;
    if session.broken {
        return Err("terminal session is broken".to_string());
    }
    if session.active && session.handle.is_some() {
        Ok(())
    } else {
        Err("terminal session is not active".to_string())
    }
}

fn can_dispatch_now(session: &TerminalSession) -> bool {
    session.status == TerminalSessionStatus::Online && !session.status_locked && !session.dispatch_inflight
}

fn is_duplicate_dispatch(session: &TerminalSession, message_id: &str) -> bool {
    if session
        .dispatch_inflight_message_ids
        .iter()
        .any(|id| id == message_id)
    {
        return true;
    }
    if session
        .dispatch_inflight_message_id
        .as_deref()
        .is_some_and(|id| id == message_id)
    {
        return true;
    }
    if session
        .dispatch_queue
        .iter()
        .any(|item| {
            item.envelope.context.message_id.as_deref() == Some(message_id)
                || item
                    .envelope
                    .batched_message_ids
                    .iter()
                    .any(|id| id == message_id)
        })
    {
        return true;
    }
    session
        .dispatch_recent_message_ids
        .iter()
        .any(|id| id == message_id)
}

fn record_dispatch_message_id(session: &mut TerminalSession, message_id: String) {
    session.dispatch_recent_message_ids.push_back(message_id);
    if session.dispatch_recent_message_ids.len() > DISPATCH_RECENT_LIMIT {
        session.dispatch_recent_message_ids.pop_front();
    }
}

fn record_dispatch_message_ids(session: &mut TerminalSession, message_ids: Vec<String>) {
    for message_id in message_ids {
        record_dispatch_message_id(session, message_id);
    }
}

fn ensure_envelope_message_ids(envelope: &mut TerminalDispatchEnvelope) {
    if let Some(message_id) = envelope.context.message_id.clone() {
        if !envelope
            .batched_message_ids
            .iter()
            .any(|id| id == &message_id)
        {
            envelope.batched_message_ids.insert(0, message_id);
        }
    }
}

fn can_merge_dispatch_envelope(
    base: &TerminalDispatchEnvelope,
    next: &TerminalDispatchEnvelope,
) -> bool {
    base.context.conversation_id == next.context.conversation_id
        && base.context.conversation_type == next.context.conversation_type
        && base.context.sender_id == next.context.sender_id
        && base.context.sender_name == next.context.sender_name
}

fn merge_dispatch_envelope(base: &mut TerminalDispatchEnvelope, mut next: TerminalDispatchEnvelope) {
    ensure_envelope_message_ids(base);
    ensure_envelope_message_ids(&mut next);
    if !base.text.is_empty() && !next.text.is_empty() {
        base.text.push_str(DISPATCH_BATCH_SEPARATOR);
    }
    base.text.push_str(&next.text);
    if base.context.message_id.is_none() {
        base.context.message_id = next.context.message_id.clone();
    }
    base.batched_message_ids.extend(next.batched_message_ids);
}

fn merge_or_enqueue_dispatch(
    session: &mut TerminalSession,
    mut envelope: TerminalDispatchEnvelope,
    enqueued_at: u64,
) -> Result<(), String> {
    ensure_envelope_message_ids(&mut envelope);
    if let Some(last) = session.dispatch_queue.back_mut() {
        if can_merge_dispatch_envelope(&last.envelope, &envelope) {
            // 积压合并仅发生在同一会话/发送者的连续消息，避免跨会话输出归属混乱。
            merge_dispatch_envelope(&mut last.envelope, envelope);
            return Ok(());
        }
    }
    if session.dispatch_queue.len() >= DISPATCH_QUEUE_LIMIT {
        return Err("terminal dispatch queue full".to_string());
    }
    session.dispatch_queue.push_back(DispatchQueueItem {
        envelope,
        enqueued_at,
    });
    Ok(())
}

fn pop_dispatch_batch(session: &mut TerminalSession) -> Option<TerminalDispatchEnvelope> {
    let mut item = session.dispatch_queue.pop_front()?;
    ensure_envelope_message_ids(&mut item.envelope);
    let mut envelope = item.envelope;
    loop {
        let should_merge = session
            .dispatch_queue
            .front()
            .map(|next| can_merge_dispatch_envelope(&envelope, &next.envelope))
            .unwrap_or(false);
        if !should_merge {
            break;
        }
        if let Some(next) = session.dispatch_queue.pop_front() {
            merge_dispatch_envelope(&mut envelope, next.envelope);
        } else {
            break;
        }
    }
    Some(envelope)
}

fn set_dispatch_inflight_message_ids(
    session: &mut TerminalSession,
    envelope: &TerminalDispatchEnvelope,
) {
    let mut inflight_message_ids = envelope.batched_message_ids.clone();
    if inflight_message_ids.is_empty() {
        if let Some(message_id) = envelope.context.message_id.clone() {
            inflight_message_ids.push(message_id);
        }
    }
    session.dispatch_inflight_message_id = inflight_message_ids.first().cloned();
    session.dispatch_inflight_message_ids = inflight_message_ids;
}

fn dispatch_input_with_context(
    app: &AppHandle,
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &dyn TerminalEventPort,
    terminal_id: &str,
    data: String,
    context: TerminalDispatchContext,
) -> Result<(), String> {
    let data_for_log = data.clone();
    if is_member_dnd(sessions, terminal_id) {
        if terminal_trace_detail() {
            log::info!("terminal_dispatch skipped dnd terminal_id={}", terminal_id);
        }
        return Ok(());
    }
    ensure_session_active_by_sessions(sessions, terminal_id)?;
    let chat_context = TerminalChatContext {
        conversation_id: context.conversation_id.clone(),
        conversation_type: context.conversation_type.clone(),
        sender_id: context.sender_id.clone(),
        sender_name: context.sender_name.clone(),
    };
    let settings_service = {
        let manager = app.state::<TerminalManager>();
        let guard = manager
            .settings_service
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        guard.clone()
    };
    mark_session_working_on_input(
        sessions,
        event_port,
        terminal_id,
        &data,
        Some(chat_context),
        Some(app),
        settings_service,
    );
    let now = now_millis()?;
    let (
        should_write,
        buffered,
        writer,
        shell_ready,
        data_len,
        member_id,
        workspace_id,
        terminal_type,
    ) = {
        let mut guard = lock_sessions(sessions);
        let session = guard
            .sessions
            .get_mut(terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        let shell_ready = session.shell_ready;
        let handle = session
            .handle
            .as_ref()
            .ok_or_else(|| "terminal session handle missing".to_string())?;
        let writer = Arc::clone(&handle.writer);
        let data_len = data.len();
        let (should_write, buffered) = handle_buffered_write(session, data, now);
        (
            should_write,
            buffered,
            writer,
            shell_ready,
            data_len,
            session.member_id.clone(),
            session.workspace_id.clone(),
            session.terminal_type.as_str().to_string(),
        )
    };
    if terminal_trace_detail() {
        let buffered_bytes: usize = buffered.iter().map(|entry| entry.len()).sum();
        log::info!(
            "terminal_dispatch terminal_id={} data_len={} buffered_bytes={} write_now={} shell_ready={}",
            terminal_id,
            data_len,
            buffered_bytes,
            should_write,
            shell_ready
        );
    }
    diagnostics_log_backend_event(
        &app.state::<DiagnosticsState>(),
        member_id.clone(),
        Some(terminal_id.to_string()),
        Some(context.conversation_id.clone()),
        None,
        workspace_id.clone(),
        "terminal_dispatch",
        json!({
          "terminalId": terminal_id,
          "memberId": member_id,
          "workspaceId": workspace_id,
          "terminalType": terminal_type,
          "conversationId": context.conversation_id,
          "conversationType": context.conversation_type,
          "senderId": context.sender_id,
          "senderName": context.sender_name,
          "messageId": context.message_id,
          "clientTraceId": context.client_trace_id,
          "clientTimestamp": context.client_timestamp,
          "data": data_for_log,
          "dataLen": data_len,
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

fn dispatch_chat_sequence(
    app: &AppHandle,
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &dyn TerminalEventPort,
    terminal_id: &str,
    envelope: TerminalDispatchEnvelope,
) -> Result<(), String> {
    dispatch_input_with_context(
        app,
        sessions,
        event_port,
        terminal_id,
        envelope.text.clone(),
        envelope.context.clone(),
    )?;
    thread::sleep(Duration::from_millis(COMMAND_CONFIRM_DELAY_MS));
    dispatch_input_with_context(
        app,
        sessions,
        event_port,
        terminal_id,
        COMMAND_CONFIRM_SUFFIX.to_string(),
        envelope.context,
    )?;
    Ok(())
}

pub(super) fn dispatch_chat_or_queue(
    app: &AppHandle,
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &dyn TerminalEventPort,
    terminal_id: &str,
    text: String,
    context: TerminalDispatchContext,
) -> Result<DispatchOutcome, String> {
    if is_member_dnd(sessions, terminal_id) {
        return Ok(DispatchOutcome::SkippedDnd);
    }
    ensure_session_active_by_sessions(sessions, terminal_id)?;
    let mut envelope = TerminalDispatchEnvelope {
        text,
        context,
        batched_message_ids: Vec::new(),
    };
    ensure_envelope_message_ids(&mut envelope);
    let mut envelope_slot = Some(envelope);
    let now = now_millis().unwrap_or(0);
    let mut kick_flush = false;
    let should_dispatch = {
        let mut guard = lock_sessions(sessions);
        let session = guard
            .sessions
            .get_mut(terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        {
            let envelope_ref = envelope_slot
                .as_ref()
                .ok_or_else(|| "dispatch envelope missing".to_string())?;
            for message_id in envelope_ref.batched_message_ids.iter() {
                if is_duplicate_dispatch(session, message_id) {
                    return Ok(DispatchOutcome::Duplicate);
                }
            }
        }
        let can_dispatch_immediately = can_dispatch_now(session) && session.dispatch_queue.is_empty();
        if can_dispatch_immediately {
            let envelope_ref = envelope_slot
                .as_ref()
                .ok_or_else(|| "dispatch envelope missing".to_string())?;
            session.dispatch_inflight = true;
            set_dispatch_inflight_message_ids(session, envelope_ref);
            true
        } else {
            let queued_envelope = envelope_slot
                .take()
                .ok_or_else(|| "dispatch envelope missing".to_string())?;
            merge_or_enqueue_dispatch(session, queued_envelope, now)?;
            if can_dispatch_now(session) {
                kick_flush = true;
            }
            false
        }
    };
    if !should_dispatch {
        if kick_flush {
            flush_dispatch_queue_if_ready(app, sessions, event_port, terminal_id);
        }
        return Ok(DispatchOutcome::Queued);
    }
    let envelope = envelope_slot
        .ok_or_else(|| "dispatch envelope missing".to_string())?;
    if let Err(err) = dispatch_chat_sequence(app, sessions, event_port, terminal_id, envelope.clone())
    {
        let mut guard = lock_sessions(sessions);
        if let Some(session) = guard.sessions.get_mut(terminal_id) {
            session.dispatch_inflight = false;
            session.dispatch_inflight_message_id = None;
            session.dispatch_inflight_message_ids.clear();
            if session.dispatch_queue.len() < DISPATCH_QUEUE_LIMIT {
                session.dispatch_queue.push_front(DispatchQueueItem {
                    envelope,
                    enqueued_at: now,
                });
            }
        }
        return Err(err);
    }
    Ok(DispatchOutcome::Dispatched)
}

pub(super) fn complete_dispatch_and_flush(
    app: &AppHandle,
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &dyn TerminalEventPort,
    terminal_id: &str,
) {
    let next = {
        let mut guard = lock_sessions(sessions);
        let session = match guard.sessions.get_mut(terminal_id) {
            Some(session) => session,
            None => return,
        };
        session.dispatch_inflight = false;
        let mut inflight_message_ids = std::mem::take(&mut session.dispatch_inflight_message_ids);
        if inflight_message_ids.is_empty() {
            if let Some(message_id) = session.dispatch_inflight_message_id.take() {
                inflight_message_ids.push(message_id);
            }
        } else {
            session.dispatch_inflight_message_id = None;
        }
        if !inflight_message_ids.is_empty() {
            record_dispatch_message_ids(session, inflight_message_ids);
        }
        if !can_dispatch_now(session) {
            return;
        }
        pop_dispatch_batch(session).map(|envelope| {
            session.dispatch_inflight = true;
            set_dispatch_inflight_message_ids(session, &envelope);
            envelope
        })
    };
    let Some(envelope) = next else {
        return;
    };
    if let Err(err) = dispatch_chat_sequence(app, sessions, event_port, terminal_id, envelope.clone())
    {
        let mut guard = lock_sessions(sessions);
        if let Some(session) = guard.sessions.get_mut(terminal_id) {
            session.dispatch_inflight = false;
            session.dispatch_inflight_message_id = None;
            session.dispatch_inflight_message_ids.clear();
            if session.dispatch_queue.len() < DISPATCH_QUEUE_LIMIT {
                session.dispatch_queue.push_front(DispatchQueueItem {
                    envelope,
                    enqueued_at: now_millis().unwrap_or(0),
                });
            }
        }
        log::warn!(
            "terminal dispatch queue flush failed terminal_id={} err={}",
            terminal_id,
            err
        );
    }
}

pub(super) fn flush_dispatch_queue_if_ready(
    app: &AppHandle,
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &dyn TerminalEventPort,
    terminal_id: &str,
) {
    let next = {
        let mut guard = lock_sessions(sessions);
        let session = match guard.sessions.get_mut(terminal_id) {
            Some(session) => session,
            None => return,
        };
        if !can_dispatch_now(session) {
            return;
        }
        pop_dispatch_batch(session).map(|envelope| {
            session.dispatch_inflight = true;
            set_dispatch_inflight_message_ids(session, &envelope);
            envelope
        })
    };
    let Some(envelope) = next else {
        return;
    };
    if let Err(err) = dispatch_chat_sequence(app, sessions, event_port, terminal_id, envelope.clone())
    {
        let mut guard = lock_sessions(sessions);
        if let Some(session) = guard.sessions.get_mut(terminal_id) {
            session.dispatch_inflight = false;
            session.dispatch_inflight_message_id = None;
            session.dispatch_inflight_message_ids.clear();
            if session.dispatch_queue.len() < DISPATCH_QUEUE_LIMIT {
                session.dispatch_queue.push_front(DispatchQueueItem {
                    envelope,
                    enqueued_at: now_millis().unwrap_or(0),
                });
            }
        }
        log::warn!(
            "terminal dispatch queue flush failed terminal_id={} err={}",
            terminal_id,
            err
        );
    }
}

fn get_flow_control_state(
    sessions: &Arc<Mutex<SessionRegistry>>,
    terminal_id: &str,
) -> Option<(usize, bool, bool)> {
    let guard = lock_sessions(sessions);
    guard.sessions.get(terminal_id).map(|session| {
        (
            session.unacked_bytes,
            session.ui_active,
            session.chat_pending,
        )
    })
}

fn set_flow_paused(sessions: &Arc<Mutex<SessionRegistry>>, terminal_id: &str, paused: bool) {
    let mut guard = lock_sessions(sessions);
    if let Some(session) = guard.sessions.get_mut(terminal_id) {
        session.flow_paused = paused;
        if !paused {
            session.idle_candidate_at = None;
            session.chat_candidate_at = None;
        }
    }
}

// 基于 ACK 的流控：累加/扣减未确认字节用于背压。
fn add_unacked_bytes(sessions: &Arc<Mutex<SessionRegistry>>, terminal_id: &str, count: usize) {
    if count == 0 {
        return;
    }
    let mut guard = lock_sessions(sessions);
    if let Some(session) = guard.sessions.get_mut(terminal_id) {
        session.unacked_bytes = session.unacked_bytes.saturating_add(count);
    }
}

fn subtract_unacked_bytes(sessions: &Arc<Mutex<SessionRegistry>>, terminal_id: &str, count: usize) {
    if count == 0 {
        return;
    }
    let mut guard = lock_sessions(sessions);
    if let Some(session) = guard.sessions.get_mut(terminal_id) {
        session.unacked_bytes = session.unacked_bytes.saturating_sub(count);
    }
}

fn mark_session_working_on_input(
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &dyn TerminalEventPort,
    terminal_id: &str,
    data: &str,
    chat_context: Option<TerminalChatContext>,
    app: Option<&AppHandle>,
    settings_service: Option<Arc<dyn TerminalSettingsPort>>,
) {
    let is_command = data.contains('\n') || data.contains('\r');
    let mut status_payload = None;
    let now = now_millis().ok();
    let mut chat_context = chat_context;
    let mut chat_pending_at = None;
    {
        let mut guard = lock_sessions(sessions);
        let working_sessions = Arc::clone(&guard.working_sessions);
        let session = match guard.sessions.get_mut(terminal_id) {
            Some(session) => session,
            None => return,
        };
        if !is_command && session.status != TerminalSessionStatus::Working {
            return;
        }
        if is_command {
            if let Some(context) = chat_context.take() {
                session.chat_pending = true;
                session.chat_pending_since = now;
                session.chat_candidate_at = None;
                session.semantic_active = false;
                session.chat_stream_enabled =
                    resolve_chat_stream_enabled(settings_service.as_deref());
                if let Some(now) = now {
                    chat_pending_at = Some(now);
                }
                let semantic_tx = if let Some(tx) = session.semantic_tx.clone() {
                    Some(tx)
                } else if let Some(app) = app {
                    let manager = app.state::<TerminalManager>();
                    let pipeline = manager.message_pipeline();
                    let dispatch_gate = manager.dispatch_gate();
                    let event_port = manager.event_port();
                    let settings_service = {
                        let guard = manager
                            .settings_service
                            .lock()
                            .unwrap_or_else(|err| err.into_inner());
                        guard.clone()
                    };
                    let tx = spawn_semantic_worker(
                        app.clone(),
                        session.id.clone(),
                        session.member_id.clone(),
                        session.workspace_id.clone(),
                        session.screen_rows,
                        session.screen_cols,
                        session.terminal_type.as_str().to_string(),
                        pipeline,
                        settings_service,
                        Arc::clone(sessions),
                        event_port,
                        dispatch_gate,
                    );
                    session.semantic_tx = Some(tx.clone());
                    Some(tx)
                } else {
                    None
                };
                if let Some(tx) = semantic_tx {
                    let mut seed_sent = true;
                    if session.chat_stream_enabled {
                        let snapshot = session.snapshot.snapshot_segments().data;
                        let rows = session.screen_rows;
                        let cols = session.screen_cols;
                        seed_sent = tx
                            .send(SemanticEvent::SeedSnapshot {
                                rows,
                                cols,
                                data: snapshot,
                            })
                            .is_ok();
                    }
                    let input_sent = tx
                        .send(SemanticEvent::UserInput {
                            data: data.to_string(),
                            context,
                        })
                        .is_ok();
                    if seed_sent && input_sent {
                        session.semantic_active = true;
                    }
                }
            }
            if let Some(now) = now {
                session.working_intent_until =
                    Some(now.saturating_add(WORKING_INTENT_WINDOW_MS));
            }
            if !session.status_locked && session.status != TerminalSessionStatus::Working {
                if update_session_status(&working_sessions, session, TerminalSessionStatus::Working)
                {
                    status_payload = Some(build_status_payload(session));
                }
            }
        }
        // 输入在 Working 中也续命，避免无输出的交互型场景被误判回落。
        if let Some(now) = now {
            session.last_activity_at = Some(now);
        }
        session.idle_candidate_at = None;
        session.chat_candidate_at = None;
    }
    if let Some(payload) = status_payload {
        let _ = event_port.emit_status(payload);
    }
    if let Some(observed_at) = chat_pending_at {
        emit_trigger_signal(
            sessions,
            FactEvent::ChatPending {
                terminal_id: terminal_id.to_string(),
                observed_at,
            },
        );
    }
}

fn resolve_chat_stream_enabled(
    settings_service: Option<&dyn TerminalSettingsPort>,
) -> bool {
    settings_service
        .and_then(|service| service.get_chat_stream_enabled())
        .unwrap_or(true)
}

/// 关闭所有会话并终止子进程。
/// 返回：始终尽力清理，除非内部出现不可恢复的同步错误（目前无显式错误路径）。
pub(crate) fn shutdown_sessions(state: &TerminalManager) -> Result<(), String> {
    // 进程退出或最后窗口关闭时集中清理，避免孤儿子进程。
    let mut killers = Vec::new();
    let mut semantic_shutdowns = Vec::new();
    let mut semantic_flushes = Vec::new();
    {
        let mut guard = lock_sessions(&state.sessions);
        let working_sessions = Arc::clone(&guard.working_sessions);
        for session in guard.sessions.values_mut() {
            if let Some(handle) = session.handle.take() {
                killers.push(handle.killer);
            }
            if let Some(tx) = session.semantic_tx.take() {
                // 如果有 pending 的聊天块，先发送 Flush 来持久化，避免关闭应用丢失消息
                if session.chat_pending {
                    semantic_flushes.push(tx.clone());
                }
                semantic_shutdowns.push(tx);
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
            session.redraw_suppression_until = None;
            session.flow_paused = false;
            session.dispatch_inflight = false;
            session.dispatch_inflight_message_id = None;
            session.dispatch_inflight_message_ids.clear();
            if session.status != TerminalSessionStatus::Offline {
                update_session_status(&working_sessions, session, TerminalSessionStatus::Offline);
            }
            session.status_locked = false;
        }
    }
    for mut killer in killers {
        let _ = killer.kill();
    }
    // 先发送 Flush 来持久化 pending 的聊天消息
    for tx in semantic_flushes {
        let _ = tx.send(SemanticEvent::Flush {
            message_type: "info",
            source: "shutdown",
        });
    }
    for tx in semantic_shutdowns {
        let _ = tx.send(SemanticEvent::Shutdown);
    }
    Ok(())
}

fn build_status_payload(session: &TerminalSession) -> TerminalStatusPayload {
    TerminalStatusPayload {
        terminal_id: session.id.clone(),
        status: session.status.as_str().to_string(),
        member_id: session.member_id.clone(),
        workspace_id: session.workspace_id.clone(),
    }
}

fn update_session_status(
    working_sessions: &Arc<Mutex<HashSet<String>>>,
    session: &mut TerminalSession,
    next: TerminalSessionStatus,
) -> bool {
    if session.status == next {
        return false;
    }
    let prev = session.status;
    session.status = next;
    if next == TerminalSessionStatus::Offline {
        session.dispatch_inflight = false;
        session.dispatch_inflight_message_id = None;
        session.dispatch_inflight_message_ids.clear();
    }
    if prev == TerminalSessionStatus::Working || next == TerminalSessionStatus::Working {
        if let Ok(mut guard) = working_sessions.lock() {
            if prev == TerminalSessionStatus::Working {
                guard.remove(&session.id);
            }
            if next == TerminalSessionStatus::Working {
                guard.insert(session.id.clone());
            }
        }
    }
    true
}

fn emit_trigger_signal(sessions: &Arc<Mutex<SessionRegistry>>, signal: FactEvent) {
    let trigger_bus = {
        let guard = lock_sessions(sessions);
        guard.trigger_bus.clone()
    };
    if let Some(bus) = trigger_bus {
        bus.emit_fact(signal);
    }
}

struct OutputCleanupSignals {
    has_erase_line: bool,
    has_erase_screen: bool,
    has_lone_carriage_return: bool,
    has_cursor_up: bool,
    has_cursor_down: bool,
    has_insert_line: bool,
    has_delete_line: bool,
    has_delete_char: bool,
    has_erase_char: bool,
}

fn detect_output_cleanup(data: &str) -> Option<OutputCleanupSignals> {
    let has_erase_line = data.contains("\x1b[K")
        || data.contains("\x1b[0K")
        || data.contains("\x1b[1K")
        || data.contains("\x1b[2K");
    let has_erase_screen = data.contains("\x1b[J")
        || data.contains("\x1b[0J")
        || data.contains("\x1b[1J")
        || data.contains("\x1b[2J");
    let has_lone_carriage_return = contains_lone_carriage_return(data.as_bytes());
    let csi_actions = detect_csi_actions(data.as_bytes());
    if has_erase_line
        || has_erase_screen
        || has_lone_carriage_return
        || csi_actions.has_cursor_up
        || csi_actions.has_cursor_down
        || csi_actions.has_insert_line
        || csi_actions.has_delete_line
        || csi_actions.has_delete_char
        || csi_actions.has_erase_char
    {
        Some(OutputCleanupSignals {
            has_erase_line,
            has_erase_screen,
            has_lone_carriage_return,
            has_cursor_up: csi_actions.has_cursor_up,
            has_cursor_down: csi_actions.has_cursor_down,
            has_insert_line: csi_actions.has_insert_line,
            has_delete_line: csi_actions.has_delete_line,
            has_delete_char: csi_actions.has_delete_char,
            has_erase_char: csi_actions.has_erase_char,
        })
    } else {
        None
    }
}

#[derive(Default)]
struct CsiActionSignals {
    has_cursor_up: bool,
    has_cursor_down: bool,
    has_insert_line: bool,
    has_delete_line: bool,
    has_delete_char: bool,
    has_erase_char: bool,
}

fn detect_csi_actions(bytes: &[u8]) -> CsiActionSignals {
    let mut signals = CsiActionSignals::default();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b'\x1b' && bytes[index + 1] == b'[' {
            index += 2;
            // 按 ECMA-48 扫描 CSI 参数/中间/终止字节，识别关键动作。
            while index < bytes.len() {
                let value = bytes[index];
                if (CSI_PARAM_BYTE_START..=CSI_PARAM_BYTE_END).contains(&value)
                    || (CSI_INTERMEDIATE_BYTE_START..=CSI_INTERMEDIATE_BYTE_END).contains(&value)
                {
                    index += 1;
                    continue;
                }
                if (CSI_FINAL_BYTE_START..=CSI_FINAL_BYTE_END).contains(&value) {
                    match value {
                        b'A' => signals.has_cursor_up = true,
                        b'B' => signals.has_cursor_down = true,
                        b'L' => signals.has_insert_line = true,
                        b'M' => signals.has_delete_line = true,
                        b'P' => signals.has_delete_char = true,
                        b'X' => signals.has_erase_char = true,
                        _ => {}
                    }
                    index += 1;
                    break;
                }
                index += 1;
                break;
            }
            continue;
        }
        index += 1;
    }
    signals
}

fn contains_lone_carriage_return(bytes: &[u8]) -> bool {
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\r' {
            let next = index + 1;
            if next >= bytes.len() || bytes[next] != b'\n' {
                return true;
            }
        }
        index += 1;
    }
    false
}

fn detect_shell_ready(session: &mut TerminalSession, output: &str, now: u64) -> Option<String> {
    // 通过 OSC 就绪信号或启动期输出量判断 shell 可交互，避免过早放行输入。
    let mut became_ready = false;
    let mut reason = "unknown";
    if output.contains(SHIM_READY_SIGNAL) {
        became_ready = true;
        reason = "osc";
    } else {
        session.ready_probe_bytes = session.ready_probe_bytes.saturating_add(output.len());
        if now.saturating_sub(session.created_at) >= SHELL_READY_ACTIVITY_MS {
            if session.ready_probe_bytes >= SHELL_READY_ACTIVITY_BYTES || output.contains('\n') {
                became_ready = true;
                reason = "activity";
            }
        }
    }
    if became_ready {
        session.shell_ready = true;
        log::info!(
            "terminal shell ready terminal_id={} reason={}",
            session.id,
            reason
        );
    }
    if output.contains(SHIM_LAUNCH_ERROR_MARKER) {
        return Some(output.to_string());
    }
    None
}

fn flush_input_buffer(
    writer: &Arc<Mutex<Box<dyn Write + Send>>>,
    buffer: Vec<String>,
) -> Result<(), String> {
    if buffer.is_empty() {
        return Ok(());
    }
    let mut writer = writer
        .lock()
        .map_err(|_| "terminal writer lock poisoned".to_string())?;
    for entry in buffer {
        writer
            .write_all(entry.as_bytes())
            .map_err(|err| format!("failed to write to pty: {err}"))?;
    }
    writer
        .flush()
        .map_err(|err| format!("failed to flush pty: {err}"))?;
    Ok(())
}

fn handle_buffered_write(
    session: &mut TerminalSession,
    data: String,
    now: u64,
) -> (bool, Vec<String>) {
    if session.shell_ready {
        return (true, vec![data]);
    }
    // shell 未就绪时先缓冲输入，避免输入在启动期丢失或打断探测。
    if now.saturating_sub(session.created_at) >= SHELL_READY_TIMEOUT_MS {
        log::warn!(
            "terminal session ready timeout terminal_id={} forcing ready",
            session.id
        );
        session.shell_ready = true;
        let mut buffered: Vec<String> = session.input_buffer.drain(..).collect();
        buffered.push(data);
        return (true, buffered);
    }
    let was_empty = session.input_buffer.is_empty();
    session.input_buffer.push_back(data);
    if was_empty {
        log::info!("terminal input buffered terminal_id={}", session.id);
    }
    (false, Vec::new())
}

fn register_session(
    state: &TerminalManager,
    terminal_id: &str,
    member_id: Option<String>,
    member_name: Option<String>,
    workspace_id: Option<String>,
    launch_cwd: Option<String>,
    launch_command: Option<String>,
    launch_path: Option<String>,
    launch_strict_shell: bool,
    post_ready_mode: PostReadyMode,
    rows: u16,
    cols: u16,
    terminal_type: TerminalType,
    keep_alive: bool,
    owner_window_label: Option<String>,
    output_window_label: Option<String>,
    semantic_tx: Option<mpsc::Sender<SemanticEvent>>,
    handle: TerminalHandle,
) -> Result<TerminalStatusPayload, String> {
    // 注册时构建快照与语义管线，确保 attach/语义事件一致。
    let created_at = now_millis().unwrap_or(0);
    let mut guard = state
        .sessions
        .lock()
        .map_err(|_| "terminal session lock poisoned".to_string())?;
    if guard.sessions.contains_key(terminal_id) {
        return Err("terminal session already exists".to_string());
    }
    // 消费创建前预锁：注册时将 pending 标记转为真实锁定状态。
    let pending_locked = guard.pending_status_locks.remove(terminal_id);
    let response_writer = Arc::clone(&handle.writer);
    let pending_output_chunks = Arc::new(AtomicUsize::new(0));
    let status = if pending_locked {
        TerminalSessionStatus::Connecting
    } else {
        TerminalSessionStatus::Online
    };
    let post_ready_state = if post_ready_mode.should_run() {
        PostReadyState::Idle
    } else {
        PostReadyState::Done
    };
    let session = TerminalSession {
        id: terminal_id.to_string(),
        terminal_type,
        status,
        output_bytes_total: 0,
        output_seq: 0,
        unacked_bytes: 0,
        screen_rows: rows,
        screen_cols: cols,
        member_id: member_id.clone(),
        member_name: member_name,
        workspace_id: workspace_id.clone(),
        status_locked: pending_locked,
        active: true,
        last_activity_at: None,
        last_output_at: None,
        last_read_at: None,
        last_applied_at: None,
        idle_candidate_at: None,
        working_intent_until: None,
        chat_pending_since: None,
        chat_candidate_at: None,
        semantic_active: false,
        chat_stream_enabled: true,
        redraw_suppression_until: None,
        flow_paused: false,
        broken: false,
        chat_pending: false,
        post_ready_state,
        post_ready_mode,
        post_ready_queue: VecDeque::new(),
        remote_session_id: None,
        post_ready_session_id_started_at: None,
        post_ready_session_id_restart_count: 0,
        post_ready_restart_pending: false,
        spawn_epoch: 1,
        ui_active: false,
        handle: Some(handle),
        snapshot: TerminalSnapshot::new(rows, cols, Some(response_writer)),
        semantic_tx,
        keep_alive,
        launch_cwd,
        launch_command,
        launch_path,
        launch_strict_shell,
        owner_window_label,
        output_window_label,
        shell_ready: false,
        created_at,
        input_buffer: VecDeque::new(),
        ready_probe_bytes: 0,
        pending_output_chunks,
        output_rate_samples: VecDeque::new(),
        dispatch_queue: VecDeque::new(),
        dispatch_inflight: false,
        dispatch_inflight_message_id: None,
        dispatch_inflight_message_ids: Vec::new(),
        dispatch_recent_message_ids: VecDeque::new(),
    };
    let payload = build_status_payload(&session);
    guard.sessions.insert(terminal_id.to_string(), session);
    if let Some(member_id) = member_id {
        guard
            .member_sessions
            .insert(member_id, terminal_id.to_string());
    }
    Ok(payload)
}

/// 创建前预锁会话状态，用于在注册前就保持 Connecting。
pub(crate) fn lock_session_status_precreate(
    sessions: &Arc<Mutex<SessionRegistry>>,
    terminal_id: &str,
) {
    let mut guard = lock_sessions(sessions);
    guard.pending_status_locks.insert(terminal_id.to_string());
}

/// 创建前预锁清理，避免失败后残留锁。
pub(crate) fn unlock_session_status_precreate(
    sessions: &Arc<Mutex<SessionRegistry>>,
    terminal_id: &str,
) {
    let mut guard = lock_sessions(sessions);
    guard.pending_status_locks.remove(terminal_id);
}

/// 按 member_id 解锁会话状态并回到 Online（若未离线）。
pub(crate) fn unlock_session_status_by_member(
    sessions: &Arc<Mutex<SessionRegistry>>,
    member_id: &str,
) -> Result<Option<TerminalStatusPayload>, String> {
    let status_payload = {
        let mut guard = lock_sessions(sessions);
        let working_sessions = Arc::clone(&guard.working_sessions);
        let terminal_id = guard
            .member_sessions
            .get(member_id)
            .cloned()
            .ok_or_else(|| "terminal session not found for member".to_string())?;
        let session = guard
            .sessions
            .get_mut(&terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        if session.status == TerminalSessionStatus::Offline {
            session.status_locked = false;
            return Ok(None);
        }
        if !session.status_locked && session.status == TerminalSessionStatus::Online {
            return Ok(None);
        }
        session.status_locked = false;
        if session.status != TerminalSessionStatus::Online {
            if update_session_status(&working_sessions, session, TerminalSessionStatus::Online) {
                Some(build_status_payload(session))
            } else {
                None
            }
        } else {
            None
        }
    };
    Ok(status_payload)
}

fn ensure_session_active(state: &TerminalManager, terminal_id: &str) -> Result<(), String> {
    let guard = lock_sessions(&state.sessions);
    let session = guard
        .sessions
        .get(terminal_id)
        .ok_or_else(|| "terminal session not found".to_string())?;
    // 统一检查：避免对已崩溃或已关闭的会话进行写入。
    if session.broken {
        return Err("terminal session is broken".to_string());
    }
    if session.active && session.handle.is_some() {
        Ok(())
    } else {
        Err("terminal session is not active".to_string())
    }
}

/// 判断是否仍有活跃会话，用于决定是否可以安全关闭进程。
pub(crate) fn has_active_sessions(state: &TerminalManager) -> bool {
    let guard = lock_sessions(&state.sessions);
    guard
        .sessions
        .values()
        .any(|session| session.active && session.handle.is_some())
}

/// 清理指定窗口对应的临时会话。
/// 约束：仅作用于 `keep_alive=false` 的会话，避免误杀共享会话。
pub(crate) fn cleanup_ephemeral_sessions_for_window(
    state: &TerminalManager,
    window_label: &str,
) -> Result<(), String> {
    // 窗口级临时会话在关闭时清理，避免后台进程残留。
    let mut killers = Vec::new();
    let mut semantic_shutdowns = Vec::new();
    {
        let mut guard = lock_sessions(&state.sessions);
        let targets: Vec<String> = guard
            .sessions
            .iter()
            .filter(|(_, session)| {
                !session.keep_alive && session.owner_window_label.as_deref() == Some(window_label)
            })
            .map(|(terminal_id, _)| terminal_id.clone())
            .collect();
        for terminal_id in targets {
            if let Some(removed) = guard.sessions.remove(&terminal_id) {
                if let Some(handle) = removed.handle {
                    killers.push(handle.killer);
                }
                if let Some(tx) = removed.semantic_tx {
                    semantic_shutdowns.push(tx);
                }
            }
        }
    }
    for mut killer in killers {
        let _ = killer.kill();
    }
    for tx in semantic_shutdowns {
        let _ = tx.send(SemanticEvent::Shutdown);
    }
    Ok(())
}

struct InitialWriteState {
    terminal_id: String,
    payload: String,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    sessions: Arc<Mutex<SessionRegistry>>,
    event_port: Arc<dyn TerminalEventPort>,
    sent: AtomicBool,
}

impl InitialWriteState {
    fn send_if_needed(&self, reason: &str) {
        if self.sent.swap(true, Ordering::SeqCst) {
            return;
        }
        // 初始命令仅发送一次，避免重复执行导致副作用。
        mark_session_working_on_input(
            &self.sessions,
            self.event_port.as_ref(),
            &self.terminal_id,
            &self.payload,
            None,
            None,
            None,
        );
        let now = now_millis().unwrap_or(0);
        let (should_write, buffered) = {
            let mut guard = lock_sessions(&self.sessions);
            let session = match guard.sessions.get_mut(&self.terminal_id) {
                Some(session) => session,
                None => return,
            };
            handle_buffered_write(session, self.payload.clone(), now)
        };
        if should_write {
            if let Err(err) = flush_input_buffer(&self.writer, buffered) {
                log::warn!(
                    "terminal initial write failed terminal_id={} err={}",
                    self.terminal_id,
                    err
                );
            }
        }
        log::info!(
            "terminal initial write terminal_id={} reason={}",
            self.terminal_id,
            reason
        );
    }

    fn schedule(self: &Arc<Self>, delay_ms: u64, reason: &'static str) {
        let state = Arc::clone(self);
        thread::spawn(move || {
            if delay_ms > 0 {
                thread::sleep(Duration::from_millis(delay_ms));
            }
            // 延迟发送用于等待首帧输出或启动超时。
            state.send_if_needed(reason);
        });
    }
}

fn panic_message(err: Box<dyn Any + Send>) -> String {
    if let Some(message) = err.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = err.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic".to_string()
    }
}

fn mark_session_broken(
    event_port: &dyn TerminalEventPort,
    sessions: &Arc<Mutex<SessionRegistry>>,
    terminal_id: &str,
    reason: String,
) {
    // 读线程崩溃时切断会话，避免继续向前端推送不一致状态。
    log::error!(
        "terminal reader crashed terminal_id={} reason={}",
        terminal_id,
        reason
    );
    let (status_payload, killer, semantic_tx) = {
        let mut guard = lock_sessions(sessions);
        let working_sessions = Arc::clone(&guard.working_sessions);
        let session = match guard.sessions.get_mut(terminal_id) {
            Some(session) => session,
            None => return,
        };
        if session.broken {
            return;
        }
        session.broken = true;
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
        session.redraw_suppression_until = None;
        session.flow_paused = false;
        let status_payload = if session.status != TerminalSessionStatus::Offline {
            if update_session_status(&working_sessions, session, TerminalSessionStatus::Offline) {
                session.status_locked = false;
                Some(build_status_payload(session))
            } else {
                None
            }
        } else {
            None
        };
        let killer = session.handle.take().map(|handle| handle.killer);
        let semantic_tx = session.semantic_tx.take();
        (status_payload, killer, semantic_tx)
    };
    let _ = event_port.emit_error(TerminalErrorPayload {
        terminal_id: terminal_id.to_string(),
        error: format!("terminal reader crashed: {reason}"),
        fatal: true,
    });
    if let Some(payload) = status_payload {
        let _ = event_port.emit_status(payload);
    }
    if let Some(mut killer) = killer {
        let _ = killer.kill();
    }
    if let Some(tx) = semantic_tx {
        let _ = tx.send(SemanticEvent::Shutdown);
    }
}

fn spawn_pty_reader(
    mut reader: Box<dyn Read + Send>,
    app: AppHandle,
    event_port: Arc<dyn TerminalEventPort>,
    sessions: Arc<Mutex<SessionRegistry>>,
    terminal_id: String,
    initial_write: Option<Arc<InitialWriteState>>,
    initial_write_delay_ms: u64,
    session_repository: Arc<dyn crate::ports::terminal_session::TerminalSessionRepository>,
    settings_service: Option<Arc<dyn crate::ports::settings::TerminalSettingsPort>>,
) {
    let (tx, rx) = mpsc::sync_channel(OUTPUT_QUEUE_CAPACITY);
    let pending_output_chunks = {
        let guard = lock_sessions(&sessions);
        guard
            .sessions
            .get(&terminal_id)
            .map(|session| Arc::clone(&session.pending_output_chunks))
    };
    let Some(pending_output_chunks) = pending_output_chunks else {
        return;
    };
    spawn_pty_processor(
        app.clone(),
        Arc::clone(&event_port),
        sessions.clone(),
        terminal_id.clone(),
        rx,
        Arc::clone(&pending_output_chunks),
        session_repository,
        settings_service,
    );
    thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // 8KB 读缓冲是吞吐与延迟的折中，避免频繁系统调用。
            let mut buffer = [0u8; 8192];
            let mut initial_scheduled = false;
            loop {
                let size = match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(size) => size,
                    Err(_) => break,
                };
                if let Some(state) = initial_write.as_ref() {
                    if !initial_scheduled {
                        initial_scheduled = true;
                        state.schedule(initial_write_delay_ms, "first-data");
                    }
                }
                if let Ok(now) = now_millis() {
                    let mut guard = lock_sessions(&sessions);
                    if let Some(session) = guard.sessions.get_mut(&terminal_id) {
                        session.last_read_at = Some(now);
                    }
                }
                let chunk = buffer[..size].to_vec();
                pending_output_chunks.fetch_add(1, Ordering::SeqCst);
                if tx.send(chunk).is_err() {
                    pending_output_chunks.fetch_sub(1, Ordering::SeqCst);
                    break;
                }
            }
        }));
        if let Err(err) = result {
            let reason = panic_message(err);
            mark_session_broken(event_port.as_ref(), &sessions, &terminal_id, reason);
        }
    });
}

fn spawn_pty_processor(
    app: AppHandle,
    event_port: Arc<dyn TerminalEventPort>,
    sessions: Arc<Mutex<SessionRegistry>>,
    terminal_id: String,
    rx: mpsc::Receiver<Vec<u8>>,
    pending_output_chunks: Arc<AtomicUsize>,
    session_repository: Arc<dyn crate::ports::terminal_session::TerminalSessionRepository>,
    settings_service: Option<Arc<dyn crate::ports::settings::TerminalSettingsPort>>,
) {
    thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut pending_output = String::new();
            let mut pending_output_seq = 0u64;
            let mut pending_semantic: Vec<u8> = Vec::new();
            let mut last_output_flush = Instant::now();
            let mut stats_last_log = Instant::now();
            let mut stats_bytes_read: u64 = 0;
            let mut stats_bytes_emitted: u64 = 0;
            let mut flow_paused = false;
            loop {
                // UI 不活跃且未确认输出过多时暂停处理，避免内存与 IPC 压力。
                if let Some((unacked, ui_active, chat_pending)) =
                    get_flow_control_state(&sessions, &terminal_id)
                {
                    let mut next_flow_paused = flow_paused;
                    // chat 任务期间不暂停处理，避免窗口未激活导致语义回写卡住。
                    if chat_pending {
                        next_flow_paused = false;
                    } else if ui_active {
                        next_flow_paused = false;
                    } else if flow_paused {
                        if unacked <= FLOW_CONTROL_LOW_WATERMARK {
                            next_flow_paused = false;
                        }
                    } else if unacked >= FLOW_CONTROL_HIGH_WATERMARK {
                        next_flow_paused = true;
                    }
                    if next_flow_paused != flow_paused {
                        flow_paused = next_flow_paused;
                        set_flow_paused(&sessions, &terminal_id, flow_paused);
                    }
                    if flow_paused {
                        thread::sleep(Duration::from_millis(FLOW_CONTROL_SLEEP_MS));
                        continue;
                    }
                }
                let chunk = match rx.recv() {
                    Ok(chunk) => chunk,
                    Err(_) => break,
                };
                let _ = pending_output_chunks.fetch_update(
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                    |value| Some(value.saturating_sub(1)),
                );
                stats_bytes_read = stats_bytes_read.saturating_add(chunk.len() as u64);
                let data = String::from_utf8_lossy(&chunk).to_string();
                let data_len = data.len();
                let cleanup_signals = if data.is_empty() {
                    None
                } else {
                    detect_output_cleanup(&data)
                };
                let mut shell_ready_observed_at = None;
                let output_updated_at: Option<u64>;
                let (
                    status_payload,
                    first_output,
                    output_bytes_total,
                    semantic_tx,
                    flush_needed,
                    shim_error,
                    output_seq,
                    output_at,
                    output_window_label,
                    ui_active,
                    semantic_active,
                    chat_stream_enabled,
                    unacked_bytes,
                    member_id,
                    workspace_id,
                    terminal_type,
                ) = {
                    let mut guard = lock_sessions(&sessions);
                        let (
                            status_payload,
                            first_output,
                            output_bytes_total,
                            semantic_tx,
                            flush_needed,
                            shim_error,
                            output_seq,
                            output_at,
                    output_window_label,
                    ui_active,
                    semantic_active,
                    chat_stream_enabled,
                    unacked_bytes,
                    member_id,
                    workspace_id,
                    terminal_type,
                        ) = {
                        let working_sessions = Arc::clone(&guard.working_sessions);
                        let session = match guard.sessions.get_mut(&terminal_id) {
                            Some(session) => session,
                            None => break,
                        };
                        let first_output = session.output_seq == 0;
                        if !chunk.is_empty() {
                            session.snapshot.apply_output(&chunk);
                        }
                        session.output_seq = session.output_seq.saturating_add(1);
                        session.output_bytes_total = session
                            .output_bytes_total
                            .saturating_add(chunk.len() as u64);
                        record_output_sample(session, now_millis().unwrap_or(0));
                        let output_seq = session.output_seq;
                        let output_bytes_total = session.output_bytes_total;
                        let semantic_tx = session.semantic_tx.clone();
                        let output_window_label = session.output_window_label.clone();
                        let ui_active = session.ui_active;
                        let semantic_active = session.semantic_active;
                        let chat_stream_enabled = session.chat_stream_enabled;
                        let unacked_bytes = session.unacked_bytes;
                        let member_id = session.member_id.clone();
                        let workspace_id = session.workspace_id.clone();
                        let terminal_type = session.terminal_type.as_str().to_string();
                        let mut flush_needed = false;
                        let now = now_millis().unwrap_or(0);
                        let shell_ready_before = session.shell_ready;
                        let shim_error = if session.shell_ready {
                            None
                        } else {
                            let error = detect_shell_ready(session, &data, now);
                            if session.shell_ready {
                                flush_needed = true;
                            }
                            error
                        };
                        if !shell_ready_before && session.shell_ready {
                            shell_ready_observed_at = Some(now);
                        }
                        if let Some(until) = session.redraw_suppression_until {
                            if now > until {
                                session.redraw_suppression_until = None;
                            }
                        }
                        // Working 触发门禁（输出路径）：
                        // 1) 布局期（切换标签/attach/resize）内无条件抑制；
                        // 2) 仅在命令意图窗口或派发/聊天进行中才允许输出触发；
                        // 3) 其余输出视为背景噪声，保持 Online。
                        // 备注：命令提交会立即切 Working，这里只负责“输出触发”分支。
                        let suppress_working = session
                            .redraw_suppression_until
                            .map(|until| now <= until)
                            .unwrap_or(false);
                        let mut working_intent_active =
                            session.chat_pending || session.dispatch_inflight;
                        if let Some(until) = session.working_intent_until {
                            if now <= until {
                                working_intent_active = true;
                            } else {
                                session.working_intent_until = None;
                            }
                        }
                        let status_payload = if !session.status_locked
                            && !suppress_working
                            && working_intent_active
                            && session.status != TerminalSessionStatus::Working
                        {
                            if update_session_status(
                                &working_sessions,
                                session,
                                TerminalSessionStatus::Working,
                            ) {
                                Some(build_status_payload(session))
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        session.last_activity_at = Some(now);
                        session.last_output_at = Some(now);
                        session.last_applied_at = Some(now);
                        session.idle_candidate_at = None;
                        session.chat_candidate_at = None;
                        let output_at = session.last_applied_at;
                        output_updated_at = output_at;
                        (
                            status_payload,
                            first_output,
                            output_bytes_total,
                            semantic_tx,
                            flush_needed,
                            shim_error,
                            output_seq,
                            output_at,
                            output_window_label,
                            ui_active,
                            semantic_active,
                            chat_stream_enabled,
                            unacked_bytes,
                            member_id,
                            workspace_id,
                            terminal_type,
                        )
                    };
                    (
                        status_payload,
                        first_output,
                        output_bytes_total,
                        semantic_tx,
                        flush_needed,
                        shim_error,
                        output_seq,
                        output_at,
                        output_window_label,
                        ui_active,
                        semantic_active,
                        chat_stream_enabled,
                        unacked_bytes,
                        member_id,
                        workspace_id,
                        terminal_type,
                    )
                };
                if let Some(observed_at) = output_updated_at {
                    emit_trigger_signal(
                        &sessions,
                        FactEvent::OutputUpdated {
                            terminal_id: terminal_id.clone(),
                            observed_at,
                        },
                    );
                }
                if shell_ready_observed_at.is_some() {
                    emit_trigger_signal(
                        &sessions,
                        FactEvent::ShellReady {
                            terminal_id: terminal_id.clone(),
                        },
                    );
                }
                if let Some(signals) = cleanup_signals {
                    diagnostics_log_backend_event(
                        &app.state::<DiagnosticsState>(),
                        member_id.clone(),
                        Some(terminal_id.clone()),
                        None,
                        output_window_label.clone(),
                        workspace_id.clone(),
                        "terminal_output_cleanup",
                        json!({
                          "terminalId": terminal_id,
                          "memberId": member_id,
                          "workspaceId": workspace_id,
                          "terminalType": terminal_type,
                          "seq": output_seq,
                          "outputAt": output_at,
                          "dataLen": data_len,
                          "hasEraseLine": signals.has_erase_line,
                          "hasEraseScreen": signals.has_erase_screen,
                          "hasLoneCarriageReturn": signals.has_lone_carriage_return,
                          "hasCursorUp": signals.has_cursor_up,
                          "hasCursorDown": signals.has_cursor_down,
                          "hasInsertLine": signals.has_insert_line,
                          "hasDeleteLine": signals.has_delete_line,
                          "hasDeleteChar": signals.has_delete_char,
                          "hasEraseChar": signals.has_erase_char
                        }),
                    );
                }
                if !data.is_empty() {
                    pending_output.push_str(&data);
                    pending_output_seq = output_seq;
                }
                if !semantic_active && !pending_semantic.is_empty() {
                    pending_semantic.clear();
                }
                if semantic_active && chat_stream_enabled && !chunk.is_empty() {
                    pending_semantic.extend_from_slice(&chunk);
                }
                let now = Instant::now();
                // UI 活跃时尽量低延迟 flush，其余情况按尺寸/时间节流。
                let should_flush_output = !pending_output.is_empty()
                    && (ui_active
                        || pending_output.len() >= OUTPUT_EMIT_MAX_BYTES
                        || now.duration_since(last_output_flush)
                            >= Duration::from_millis(OUTPUT_EMIT_INTERVAL_MS));
                if should_flush_output {
                    let payload = TerminalOutputPayload {
                        terminal_id: terminal_id.clone(),
                        data: pending_output.clone(),
                        seq: pending_output_seq,
                    };
                    let payload_len = payload.data.len();
                    if terminal_trace_detail() {
                        let target = output_window_label.as_deref().unwrap_or("<broadcast>");
                        let unacked_after = unacked_bytes.saturating_add(payload_len);
                        log::info!(
              "terminal_output_emit terminal_id={} seq={} data_len={} unacked={} ui_active={} flow_paused={} target={}",
              terminal_id,
              pending_output_seq,
              payload_len,
              unacked_after,
              ui_active,
              flow_paused,
              target
            );
                    }
                    stats_bytes_emitted =
                        stats_bytes_emitted.saturating_add(pending_output.len() as u64);
                    pending_output.clear();
                    last_output_flush = now;
                    // 输出事件按窗口标签定向，减少多窗口广播成本。
                    let _ = event_port.emit_output(output_window_label.as_deref(), payload);
                    add_unacked_bytes(&sessions, &terminal_id, payload_len);
                }
                if let Some(payload) = status_payload {
                    let _ = event_port.emit_status(payload);
                }
                if let Some(tx) = semantic_tx.as_ref() {
                    // 语义输出仅在 chat 任务期间发送，并确保最后一段输出不被节流滞留。
                    let should_flush_semantic =
                        semantic_active && chat_stream_enabled && !pending_semantic.is_empty();
                    if should_flush_semantic {
                        let payload = std::mem::take(&mut pending_semantic);
                        let _ = tx.send(SemanticEvent::Output(payload));
                    }
                }
                if let Some(error) = shim_error {
                    let _ = event_port.emit_error(TerminalErrorPayload {
                        terminal_id: terminal_id.clone(),
                        error,
                        fatal: true,
                    });
                }
                if flush_needed {
                    let (buffered, writer) = {
                        let mut guard = lock_sessions(&sessions);
                        let session = match guard.sessions.get_mut(&terminal_id) {
                            Some(session) => session,
                            None => break,
                        };
                        // shell 就绪后释放此前缓冲的输入。
                        let buffered: Vec<String> = session.input_buffer.drain(..).collect();
                        let writer = session
                            .handle
                            .as_ref()
                            .map(|handle| Arc::clone(&handle.writer));
                        (buffered, writer)
                    };
                    if let Some(writer) = writer {
                        if let Err(err) = flush_input_buffer(&writer, buffered) {
                            log::warn!(
                                "terminal buffer flush failed terminal_id={} err={}",
                                terminal_id,
                                err
                            );
                        }
                    }
                    if let Err(err) = post_ready::maybe_step_post_ready(
                        &sessions,
                        event_port.as_ref(),
                        session_repository.as_ref(),
                        settings_service.as_deref(),
                        &terminal_id,
                        false,
                    ) {
                        log::warn!(
                            "terminal post_ready step failed terminal_id={} err={}",
                            terminal_id,
                            err
                        );
                    }
                }
                if first_output {
                    log::info!(
                        "terminal first output terminal_id={} data_len={} total_bytes={}",
                        terminal_id,
                        data_len,
                        output_bytes_total
                    );
                }
                if terminal_trace_enabled()
                    && now.duration_since(stats_last_log)
                        >= Duration::from_millis(STATS_LOG_INTERVAL_MS)
                {
                    log::info!(
                        "terminal io stats terminal_id={} read_bytes={} emit_bytes={}",
                        terminal_id,
                        stats_bytes_read,
                        stats_bytes_emitted
                    );
                    stats_last_log = now;
                    stats_bytes_read = 0;
                    stats_bytes_emitted = 0;
                }
            }
            if !pending_output.is_empty() {
                let payload = TerminalOutputPayload {
                    terminal_id: terminal_id.clone(),
                    data: pending_output,
                    seq: pending_output_seq,
                };
                let payload_len = payload.data.len();
                let output_window_label = {
                    let guard = lock_sessions(&sessions);
                    guard
                        .sessions
                        .get(&terminal_id)
                        .and_then(|session| session.output_window_label.clone())
                };
                if terminal_trace_detail() {
                    let target = output_window_label.as_deref().unwrap_or("<broadcast>");
                    log::info!(
                        "terminal_output_emit terminal_id={} seq={} data_len={} target={}",
                        terminal_id,
                        pending_output_seq,
                        payload_len,
                        target
                    );
                }
                let _ = event_port.emit_output(output_window_label.as_deref(), payload);
                add_unacked_bytes(&sessions, &terminal_id, payload_len);
            }
            if !pending_semantic.is_empty() {
                let guard = lock_sessions(&sessions);
                if let Some(session) = guard.sessions.get(&terminal_id) {
                    if session.semantic_active && session.chat_stream_enabled {
                        if let Some(tx) = session.semantic_tx.clone() {
                            let _ = tx.send(SemanticEvent::Output(pending_semantic));
                        }
                    }
                }
            }
        }));
        if let Err(err) = result {
            let reason = panic_message(err);
            mark_session_broken(event_port.as_ref(), &sessions, &terminal_id, reason);
        }
    });
}

fn terminal_trace_enabled() -> bool {
    backend_passive_enabled()
        || match std::env::var("GOLUTRA_TERMINAL_TRACE") {
            Ok(value) => matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"),
            Err(_) => false,
        }
}

fn terminal_trace_detail() -> bool {
    backend_passive_enabled()
        || match std::env::var("GOLUTRA_TERMINAL_TRACE_DETAIL") {
            Ok(value) => matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"),
            Err(_) => false,
        }
}

fn spawn_exit_watcher(
    mut child: Box<dyn Child + Send + Sync>,
    event_port: Arc<dyn TerminalEventPort>,
    sessions: Arc<Mutex<SessionRegistry>>,
    terminal_id: String,
    spawn_epoch: u64,
) {
    thread::spawn(move || {
        // 等待子进程退出后同步状态与语义输出，保证 UI 与聊天一致。
        let (code, signal, reason) = match child.wait() {
            Ok(status) => {
                if let Some(signal) = status.signal() {
                    (None, Some(signal.to_string()), format!("signal {signal}"))
                } else {
                    let code = i32::try_from(status.exit_code()).ok();
                    let reason = code
                        .map(|value| format!("code {value}"))
                        .unwrap_or_else(|| "unknown".to_string());
                    (code, None, reason)
                }
            }
            Err(err) => {
                log::warn!(
                    "terminal child wait failed terminal_id={} err={}",
                    terminal_id,
                    err
                );
                (None, Some("error".to_string()), "error".to_string())
            }
        };
        let (exit_payload, status_payload, semantic_tx, notice_bytes, semantic_active, chat_stream_enabled, snapshot_seed, seed_rows, seed_cols) = {
            let mut guard = lock_sessions(&sessions);
            let working_sessions = Arc::clone(&guard.working_sessions);
            let (exit_payload, status_payload, semantic_tx, notice_bytes, semantic_active, chat_stream_enabled, snapshot_seed, seed_rows, seed_cols) = {
                let session = match guard.sessions.get_mut(&terminal_id) {
                    Some(session) => session,
                    None => return,
                };
                if session.spawn_epoch != spawn_epoch {
                    log::warn!(
                        "terminal exit ignored stale epoch terminal_id={} expected={} got={}",
                        terminal_id,
                        session.spawn_epoch,
                        spawn_epoch
                    );
                    return;
                }
                let semantic_active = session.semantic_active;
                let chat_stream_enabled = session.chat_stream_enabled;
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
                session.redraw_suppression_until = None;
                session.flow_paused = false;
                session.handle = None;
                let status_payload = if session.status != TerminalSessionStatus::Offline {
                    if update_session_status(
                        &working_sessions,
                        session,
                        TerminalSessionStatus::Offline,
                    ) {
                        session.status_locked = false;
                        Some(build_status_payload(session))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let notice_bytes =
                    format!("\r\n\x1b[31m[Process exited with {reason}]\x1b[0m").into_bytes();
                session.snapshot.apply_output(&notice_bytes);
                let applied_at = now_millis().ok();
                session.last_output_at = applied_at;
                session.last_applied_at = applied_at;
                session.output_bytes_total = session
                    .output_bytes_total
                    .saturating_add(notice_bytes.len() as u64);
                let semantic_tx = session.semantic_tx.clone();
                let snapshot_seed = session.snapshot.snapshot_segments().data;
                let seed_rows = session.screen_rows;
                let seed_cols = session.screen_cols;
                (
                    TerminalExitPayload {
                        terminal_id: terminal_id.clone(),
                        code,
                        signal,
                    },
                    status_payload,
                    semantic_tx,
                    notice_bytes,
                    semantic_active,
                    chat_stream_enabled,
                    snapshot_seed,
                    seed_rows,
                    seed_cols,
                )
            };
            (
                exit_payload,
                status_payload,
                semantic_tx,
                notice_bytes,
                semantic_active,
                chat_stream_enabled,
                snapshot_seed,
                seed_rows,
                seed_cols,
            )
        };
        let _ = event_port.emit_exit(exit_payload);
        if let Some(payload) = status_payload {
            let _ = event_port.emit_status(payload);
        }
        if semantic_active {
            if let Some(tx) = semantic_tx {
                if chat_stream_enabled {
                    let _ = tx.send(SemanticEvent::Output(notice_bytes));
                } else {
                    let _ = tx.send(SemanticEvent::SeedSnapshot {
                        rows: seed_rows,
                        cols: seed_cols,
                        data: snapshot_seed,
                    });
                }
                let _ = tx.send(SemanticEvent::Flush {
                    message_type: "system",
                    source: "system",
                });
            }
        }
    });
}
