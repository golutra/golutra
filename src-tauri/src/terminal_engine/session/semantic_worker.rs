//! 终端语义子线程：用于异步处理快照、过滤与聊天写回。

use std::{
  sync::mpsc,
  thread,
  time::{Duration, Instant},
};

use serde_json::json;
use tauri::{AppHandle, Manager};
use ulid::Ulid;

use super::super::filters::{FilterContext, FilterDecision, FilterSource};
use super::super::semantic::{
  build_semantic_payload, extract_command_from_input, extract_input_lines, SemanticState,
  TerminalChatContext,
};
use super::snapshot_service;
use crate::message_service::pipeline;
use crate::platform::{diagnostics_log_backend_event, DiagnosticsState};
use crate::now_millis;

const STREAM_EMIT_INTERVAL_MS: u64 = 160; // 流式更新节流，避免高频事件影响 UI。
const STREAM_MESSAGE_TYPE: &str = "info"; // [TODO/terminal, 2026-01-26] 统一流式消息类型的业务口径。
const STREAM_SOURCE: &str = "pty"; // [TODO/terminal, 2026-01-26] 明确流式来源标识与前端约定。

// 语义通道事件：用于从 IO 线程向语义线程传递输出与上下文。
pub(super) enum SemanticEvent {
  Output(Vec<u8>),
  UserInput {
    data: String,
    context: TerminalChatContext,
  },
  Resize { rows: u16, cols: u16 },
  Flush { message_type: &'static str, source: &'static str },
  Shutdown,
}

pub(super) fn spawn_semantic_worker(
  app: AppHandle,
  session_id: String,
  member_id: Option<String>,
  workspace_id: Option<String>,
  rows: u16,
  cols: u16,
  terminal_type: String,
) -> mpsc::Sender<SemanticEvent> {
  let (tx, rx) = mpsc::channel();
  thread::spawn(move || {
    // 独立线程处理语义快照，避免阻塞 PTY 读写主路径。
    let mut state = SemanticState::new(session_id, member_id, workspace_id, rows, cols, terminal_type);
    let mut last_stream_emit_at = Instant::now();
    let mut last_stream_content: Option<String> = None;
    while let Ok(event) = rx.recv() {
      match event {
        SemanticEvent::Output(bytes) => {
          state.emulator.apply_output(&bytes);
          maybe_emit_stream(
            &app,
            &mut state,
            &mut last_stream_emit_at,
            &mut last_stream_content,
          );
        }
        SemanticEvent::UserInput { data, context } => {
          if !state.chat_block_pending {
            state.chat_block_pending = true;
            state.chat_context = Some(context);
            state.chat_span_id = Some(Ulid::new().to_string());
            state.chat_last_command = extract_command_from_input(&data);
            state.chat_last_input_lines = extract_input_lines(&data);
            last_stream_content = None;
            last_stream_emit_at = Instant::now();
          }
        }
        SemanticEvent::Resize { rows, cols } => {
          state.emulator.set_size(rows, cols);
        }
        SemanticEvent::Flush { message_type, source } => {
          if state.chat_block_pending {
            let snapshot_lines = snapshot_service::normalize_lines(state.emulator.snapshot_lines());
            let snapshot_line_count = snapshot_lines.len();
            let filter_context = FilterContext {
              session_id: state.session_id.as_str(),
              terminal_type: state.terminal_type.as_str(),
              last_command: state.chat_last_command.as_deref(),
              last_input_lines: state.chat_last_input_lines.as_deref(),
              now_ms: now_millis().unwrap_or(0),
              source: FilterSource::Snapshot,
            };
            // 过滤仅影响聊天写回，不改变终端真实输出。
            let filter_result = state.filter.apply_snapshot(&filter_context, &snapshot_lines);
            let filtered_lines = filter_result
              .lines
              .as_ref()
              .map(|lines| lines.as_slice())
              .unwrap_or(snapshot_lines.as_slice());
            let filtered_line_count = filtered_lines.len();
            let (payload, should_clear_context) = match filter_result.decision {
              FilterDecision::Allow => (
                build_semantic_payload(&mut state, message_type, source, "final", filtered_lines),
                true,
              ),
              FilterDecision::Drop => (None, true),
              FilterDecision::Defer => (None, false),
            };
            let member_id_for_log = state.member_id.clone();
            let workspace_id_for_log = state.workspace_id.clone();
            let conversation_id_for_log = state
              .chat_context
              .as_ref()
              .map(|context| context.conversation_id.clone());
            let span_id_for_log = state.chat_span_id.clone();
            let last_command_for_log = state.chat_last_command.clone();
            // 诊断链路需要完整记录语义 flush 的快照与上下文，便于定位聊天输出缺失。
            diagnostics_log_backend_event(
              &app.state::<DiagnosticsState>(),
              member_id_for_log.clone(),
              Some(state.session_id.clone()),
              conversation_id_for_log.clone(),
              None,
              workspace_id_for_log.clone(),
              "terminal_semantic_flush",
              json!({
                "sessionId": state.session_id,
                "memberId": member_id_for_log,
                "workspaceId": workspace_id_for_log,
                "conversationId": conversation_id_for_log,
                "messageType": message_type,
                "source": source,
                "hasPayload": payload.is_some(),
                "filterProfile": filter_result.profile.as_str(),
                "filterDecision": filter_result.decision.as_str(),
                "filterReason": filter_result.reason,
                "filteredLineCount": filtered_line_count,
                "snapshotLines": snapshot_lines,
                "lineCount": snapshot_line_count,
                "content": payload.as_ref().map(|item| item.content.clone()),
                "spanId": span_id_for_log,
                "lastCommand": last_command_for_log
              }),
            );
            if let Some(payload) = payload {
              let content_for_log = payload.content.clone();
              if let Err(err) = pipeline::process_terminal_final(&app, payload) {
                log::warn!("terminal chat append failed session_id={} err={}", state.session_id, err);
                diagnostics_log_backend_event(
                  &app.state::<DiagnosticsState>(),
                  state.member_id.clone(),
                  Some(state.session_id.clone()),
                  state.chat_context.as_ref().map(|context| context.conversation_id.clone()),
                  None,
                  state.workspace_id.clone(),
                  "terminal_chat_append_error",
                  json!({
                    "sessionId": state.session_id,
                    "memberId": state.member_id,
                    "workspaceId": state.workspace_id,
                    "conversationId": state.chat_context.as_ref().map(|context| context.conversation_id.clone()),
                    "error": err,
                    "content": content_for_log
                  }),
                );
              }
            }
            if should_clear_context {
              state.chat_block_pending = false;
              state.chat_context = None;
              state.chat_span_id = None;
              state.chat_last_command = None;
              state.chat_last_input_lines = None;
              last_stream_content = None;
            }
          }
        }
        SemanticEvent::Shutdown => break,
      }
    }
  });
  tx
}

fn maybe_emit_stream(
  app: &AppHandle,
  state: &mut SemanticState,
  last_stream_emit_at: &mut Instant,
  last_stream_content: &mut Option<String>,
) {
  if !state.chat_block_pending {
    return;
  }
  let now = Instant::now();
  if now.duration_since(*last_stream_emit_at) < Duration::from_millis(STREAM_EMIT_INTERVAL_MS) {
    return;
  }
  let snapshot_lines = snapshot_service::normalize_lines(state.emulator.snapshot_lines());
  let filter_context = FilterContext {
    session_id: state.session_id.as_str(),
    terminal_type: state.terminal_type.as_str(),
    last_command: state.chat_last_command.as_deref(),
    last_input_lines: state.chat_last_input_lines.as_deref(),
    now_ms: now_millis().unwrap_or(0),
    source: FilterSource::Snapshot,
  };
  let filter_result = state.filter.apply_snapshot(&filter_context, &snapshot_lines);
  let filtered_lines = filter_result
    .lines
    .as_ref()
    .map(|lines| lines.as_slice())
    .unwrap_or(snapshot_lines.as_slice());
  if filter_result.decision != FilterDecision::Allow {
    return;
  }
  let Some(payload) = build_semantic_payload(
    state,
    STREAM_MESSAGE_TYPE,
    STREAM_SOURCE,
    "stream",
    filtered_lines,
  ) else {
    return;
  };
  if last_stream_content.as_deref() == Some(payload.content.as_str()) {
    return;
  }
  if let Err(err) = pipeline::process_terminal_stream(app, payload.clone()) {
    log::warn!("terminal stream dispatch failed session_id={} err={}", state.session_id, err);
  }
  *last_stream_content = Some(payload.content);
  *last_stream_emit_at = now;
}
