use super::emulator::{create_emulator, EmulatorConfig, TerminalEmulator};
use super::models::{TerminalCursorPayload, TerminalMessageMeta, TerminalMessagePayload};
use crate::now_millis;

#[derive(Clone, Debug)]
pub(crate) struct TerminalChatContext {
  pub(crate) conversation_id: String,
  pub(crate) conversation_type: String,
  pub(crate) sender_id: String,
  pub(crate) sender_name: String,
}

pub(crate) fn extract_command_from_input(data: &str) -> Option<String> {
  let normalized = data.replace('\r', "\n");
  let mut last_non_empty = None;
  for part in normalized.split('\n') {
    let trimmed = part.trim();
    if !trimmed.is_empty() {
      last_non_empty = Some(trimmed.to_string());
    }
  }
  last_non_empty
}

pub(crate) struct SemanticState {
  pub(crate) session_id: String,
  pub(crate) member_id: Option<String>,
  pub(crate) workspace_id: Option<String>,
  pub(crate) emulator: Box<dyn TerminalEmulator>,
  pub(crate) chat_block_pending: bool,
  pub(crate) chat_context: Option<TerminalChatContext>,
  pub(crate) chat_seq: u64,
  pub(crate) chat_span_id: Option<String>,
  pub(crate) chat_last_command: Option<String>,
}

impl SemanticState {
  pub(crate) fn new(
    session_id: String,
    member_id: Option<String>,
    workspace_id: Option<String>,
    rows: u16,
    cols: u16,
  ) -> Self {
    let emulator = create_emulator(
      EmulatorConfig {
        rows,
        cols,
        scrollback_limit: 0,
      },
    );
    Self {
      session_id,
      member_id,
      workspace_id,
      emulator,
      chat_block_pending: false,
      chat_context: None,
      chat_seq: 0,
      chat_span_id: None,
      chat_last_command: None,
    }
  }
}

pub(crate) fn next_chat_seq(state: &mut SemanticState) -> u64 {
  state.chat_seq = state.chat_seq.saturating_add(1);
  state.chat_seq
}

pub(crate) fn build_snapshot_payload(
  state: &mut SemanticState,
  message_type: &str,
  source: &str,
) -> Option<TerminalMessagePayload> {
  let current_lines = state.emulator.snapshot_lines();
  let (start, end) = trim_empty_lines(&current_lines);
  if start >= end {
    return None;
  }
  let mut content = current_lines[start..end].join("\n");
  if content.trim().is_empty() {
    return None;
  }
  if let Some(context) = state.chat_context.as_ref() {
    if context.conversation_type == "channel" {
      let sender = context.sender_name.trim();
      if !sender.is_empty() {
        let mention = format!("@{sender}");
        let normalized = content.trim_start();
        if !normalized.starts_with(&mention) {
          content = format!("{mention} {content}");
        }
      }
    }
  }
  let cursor = state.emulator.cursor_position();
  let meta = TerminalMessageMeta {
    command: state.chat_last_command.clone(),
    line_count: Some((end - start) as u32),
    cursor: Some(TerminalCursorPayload {
      row: cursor.0 as u16,
      col: cursor.1 as u16,
    }),
    start_row: None,
    end_row: None,
  };
  Some(TerminalMessagePayload {
    session_id: state.session_id.clone(),
    member_id: state.member_id.clone(),
    workspace_id: state.workspace_id.clone(),
    conversation_id: state.chat_context.as_ref().map(|context| context.conversation_id.clone()),
    conversation_type: state
      .chat_context
      .as_ref()
      .map(|context| context.conversation_type.clone()),
    sender_id: state.chat_context.as_ref().map(|context| context.sender_id.clone()),
    sender_name: state.chat_context.as_ref().map(|context| context.sender_name.clone()),
    seq: next_chat_seq(state),
    timestamp: now_millis().unwrap_or(0),
    content,
    message_type: message_type.to_string(),
    source: if state.chat_context.is_some() {
      "chat".to_string()
    } else {
      source.to_string()
    },
    mode: "snapshot".to_string(),
    span_id: state.chat_span_id.clone(),
    meta: Some(meta),
  })
}

fn trim_empty_lines(lines: &[String]) -> (usize, usize) {
  let mut start = 0;
  let mut end = lines.len();
  while start < end && lines[start].trim().is_empty() {
    start += 1;
  }
  while end > start && lines[end - 1].trim().is_empty() {
    end -= 1;
  }
  (start, end)
}
