//! 终端语义子线程：用于异步处理快照、过滤与聊天写回。

use std::{
  sync::{mpsc, Arc, Mutex},
  thread,
  time::{Duration, Instant},
};

use serde_json::json;
use tauri::{AppHandle, Manager};
use ulid::Ulid;

use super::super::filters::{FilterContext, FilterDecision, FilterMode, FilterSource};
use super::super::semantic::{
  build_semantic_payload, extract_command_from_input, extract_input_lines, SemanticState,
  TerminalChatContext,
};
use super::snapshot_service;
use crate::platform::{diagnostics_log_backend_event, DiagnosticsState};
use crate::ports::terminal_dispatch_gate::TerminalDispatchGate;
use crate::ports::terminal_event::TerminalEventPort;
use crate::ports::terminal_message::TerminalMessagePipeline;
use crate::ports::settings::TerminalSettingsPort;
use crate::now_millis;

const STREAM_EMIT_INTERVAL_MS: u64 = 160; // 流式更新节流，避免高频事件影响 UI。
const STREAM_MESSAGE_TYPE: &str = "info"; // [TODO/terminal, 2026-01-26] 统一流式消息类型的业务口径。
const STREAM_SOURCE: &str = "pty"; // [TODO/terminal, 2026-01-26] 明确流式来源标识与前端约定。

// 语义通道事件：用于从 IO 线程向语义线程传递输出与上下文。
pub(super) enum SemanticEvent {
  Output(Vec<u8>),
  SeedSnapshot { rows: u16, cols: u16, data: Vec<u8> },
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
  terminal_id: String,
  member_id: Option<String>,
  workspace_id: Option<String>,
  rows: u16,
  cols: u16,
  terminal_type: String,
  pipeline: Arc<dyn TerminalMessagePipeline>,
  settings_service: Option<Arc<dyn TerminalSettingsPort>>,
  sessions: Arc<Mutex<super::state::SessionRegistry>>,
  event_port: Arc<dyn TerminalEventPort>,
  dispatch_gate: Arc<dyn TerminalDispatchGate>,
) -> mpsc::Sender<SemanticEvent> {
  let (tx, rx) = mpsc::channel();
  let pipeline = Arc::clone(&pipeline);
  let sessions = Arc::clone(&sessions);
  let event_port = Arc::clone(&event_port);
  thread::spawn(move || {
    // 独立线程处理语义快照，避免阻塞 PTY 读写主路径。
    let mut state =
      SemanticState::new(terminal_id, member_id, workspace_id, rows, cols, terminal_type);
    let mut last_stream_emit_at = Instant::now();
    let mut last_stream_content: Option<String> = None;
    while let Ok(event) = rx.recv() {
      match event {
        SemanticEvent::Output(bytes) => {
          state.emulator.apply_output(&bytes);
          maybe_emit_stream(
            pipeline.as_ref(),
            &mut state,
            &mut last_stream_emit_at,
            &mut last_stream_content,
          );
        }
        SemanticEvent::SeedSnapshot { rows, cols, data } => {
          state.reset_emulator(rows, cols);
          state.emulator.apply_output(&data);
        }
        SemanticEvent::UserInput { data, context } => {
          if !state.chat_block_pending {
            state.chat_block_pending = true;
            state.chat_context = Some(context);
            state.chat_span_id = Some(Ulid::new().to_string());
            state.chat_last_command = extract_command_from_input(&data);
            state.chat_last_input_lines = extract_input_lines(&data);
            state.chat_stream_enabled =
              resolve_chat_stream_enabled(settings_service.as_deref());
            last_stream_content = None;
            last_stream_emit_at = Instant::now();
          }
        }
        SemanticEvent::Resize { rows, cols } => {
          state.set_size(rows, cols);
        }
        SemanticEvent::Flush { message_type, source } => {
          if state.chat_block_pending {
            let now_ms = now_millis().unwrap_or(0);
            let snapshot_lines = state.emulator.snapshot_lines();
            let logical_lines =
              snapshot_service::merge_semantic_lines(&snapshot_lines, state.terminal_cols);
            let snapshot_line_count = snapshot_lines.len();
            let filter_context = FilterContext {
              terminal_id: state.terminal_id.as_str(),
              terminal_type: state.terminal_type.as_str(),
              last_command: state.chat_last_command.as_deref(),
              last_input_lines: state.chat_last_input_lines.as_deref(),
              now_ms,
              source: FilterSource::Snapshot,
              mode: FilterMode::Final,
            };
            // 过滤仅影响聊天写回，不改变终端真实输出。
            let filter_result = state.filter.apply_snapshot(&filter_context, &logical_lines);
            let filtered_lines = filter_result
              .lines
              .as_ref()
              .map(|lines| lines.as_slice())
              .unwrap_or(logical_lines.as_slice());
            let filtered_line_count = filtered_lines.len();
            let (payload, should_clear_context) = match filter_result.decision {
              FilterDecision::Allow => (
                build_semantic_payload(&mut state, message_type, source, "final", filtered_lines),
                true,
              ),
              FilterDecision::Drop => (None, true),
              FilterDecision::Defer => (None, true),
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
              Some(state.terminal_id.clone()),
              conversation_id_for_log.clone(),
              None,
              workspace_id_for_log.clone(),
              "terminal_semantic_flush",
              json!({
                "terminalId": state.terminal_id,
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
            let mut dispatch_completed = false;
            if let Some(payload) = payload {
              let content_for_log = payload.content.clone();
              match pipeline.process_final(payload) {
                Ok(_) => {
                  dispatch_completed = true;
                }
                Err(err) => {
                log::warn!("terminal chat append failed terminal_id={} err={}", state.terminal_id, err);
                diagnostics_log_backend_event(
                  &app.state::<DiagnosticsState>(),
                  state.member_id.clone(),
                  Some(state.terminal_id.clone()),
                  state.chat_context.as_ref().map(|context| context.conversation_id.clone()),
                  None,
                  state.workspace_id.clone(),
                  "terminal_chat_append_error",
                  json!({
                    "terminalId": state.terminal_id,
                    "memberId": state.member_id,
                    "workspaceId": state.workspace_id,
                    "conversationId": state.chat_context.as_ref().map(|context| context.conversation_id.clone()),
                    "error": err,
                    "content": content_for_log
                  }),
                );
                // 语义回写失败也要释放派发队列，避免后续消息被阻塞。
                dispatch_completed = true;
                }
              }
            } else if should_clear_context {
              dispatch_completed = true;
            }
            if should_clear_context {
              if dispatch_completed {
                dispatch_gate.on_semantic_flush_complete(&app, &state.terminal_id);
                super::complete_dispatch_and_flush(
                  &app,
                  &sessions,
                  event_port.as_ref(),
                  &state.terminal_id,
                );
              }
              state.chat_block_pending = false;
              state.chat_context = None;
              state.chat_span_id = None;
              state.chat_last_command = None;
              state.chat_last_input_lines = None;
              last_stream_content = None;
              // 清空语义仿真器的屏幕与历史，避免后续 chat 使用旧上下文。
              state.reset_emulator(state.terminal_rows, state.terminal_cols);
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
  pipeline: &dyn crate::ports::terminal_message::TerminalMessagePipeline,
  state: &mut SemanticState,
  last_stream_emit_at: &mut Instant,
  last_stream_content: &mut Option<String>,
) {
  if !state.chat_block_pending {
    return;
  }
  if !state.chat_stream_enabled {
    return;
  }
  let now = Instant::now();
  if now.duration_since(*last_stream_emit_at) < Duration::from_millis(STREAM_EMIT_INTERVAL_MS) {
    return;
  }
  let now_ms = now_millis().unwrap_or(0);
  let snapshot_lines = state.emulator.snapshot_lines();
  let logical_lines = snapshot_service::merge_semantic_lines(&snapshot_lines, state.terminal_cols);
  let filter_context = FilterContext {
    terminal_id: state.terminal_id.as_str(),
    terminal_type: state.terminal_type.as_str(),
    last_command: state.chat_last_command.as_deref(),
    last_input_lines: state.chat_last_input_lines.as_deref(),
    now_ms,
    source: FilterSource::Snapshot,
    mode: FilterMode::Stream,
  };
  let filter_result = state.filter.apply_snapshot(&filter_context, &logical_lines);
  let filtered_lines = filter_result
    .lines
    .as_ref()
    .map(|lines| lines.as_slice())
    .unwrap_or(logical_lines.as_slice());
  if filter_result.decision != FilterDecision::Allow {
    return;
  }
  let Some(mut payload) = build_semantic_payload(
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
  let next_content = payload.content.clone();
  if let Some(previous) = last_stream_content.as_deref() {
    if let Some(delta) = next_content.strip_prefix(previous) {
      if !delta.is_empty() {
        payload.content = delta.to_string();
        payload.mode = "delta".to_string();
      }
    }
  }
  if let Err(err) = pipeline.process_stream(payload.clone()) {
    log::warn!("terminal stream dispatch failed terminal_id={} err={}", state.terminal_id, err);
  }
  *last_stream_content = Some(next_content);
  *last_stream_emit_at = now;
}

fn resolve_chat_stream_enabled(
  settings_service: Option<&dyn TerminalSettingsPort>,
) -> bool {
  settings_service
    .and_then(|service| service.get_chat_stream_enabled())
    .unwrap_or(true)
}
