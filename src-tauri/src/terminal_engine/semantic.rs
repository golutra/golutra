//! 终端语义层：把终端内容整理为聊天侧可读的快照消息。
//! 边界：不执行命令，仅解析输出与上下文信息。

use super::emulator::{create_emulator, EmulatorConfig, TerminalEmulator};
use super::filters::FilterRuntime;
use crate::contracts::terminal_message::{
  TerminalCursorPayload, TerminalMessageMeta, TerminalMessagePayload,
};
use crate::now_millis;

// 语义层滚动历史上限：用于 chat 路线在一次会话内保留更长输出。
const SEMANTIC_SCROLLBACK_LINES: usize = 5000;

#[derive(Clone, Debug)]
/// 聊天上下文：用于将终端输出归因到对话与发送者。
pub(crate) struct TerminalChatContext {
  pub(crate) conversation_id: String,
  pub(crate) conversation_type: String,
  pub(crate) sender_id: String,
  pub(crate) sender_name: String,
}

/// 从输入中提取最后一个非空命令片段，避免噪音记录到聊天。
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

/// 从输入中提取多行内容，供提示符分段匹配。
/// 约束：忽略空行，避免因换行或尾部空白导致无法匹配。
pub(crate) fn extract_input_lines(data: &str) -> Option<Vec<String>> {
  let normalized = data.replace('\r', "\n");
  let mut lines = Vec::new();
  for part in normalized.split('\n') {
    let trimmed_end = part.trim_end();
    if trimmed_end.trim().is_empty() {
      continue;
    }
    lines.push(trimmed_end.to_string());
  }
  if lines.is_empty() {
    None
  } else {
    Some(lines)
  }
}

/// 语义处理状态，维持一段输出的上下文与序列号。
/// 约束：scrollback 设为 0，以避免语义层持有额外历史带来的内存压力。
pub(crate) struct SemanticState {
  pub(crate) terminal_id: String,
  pub(crate) member_id: Option<String>,
  pub(crate) workspace_id: Option<String>,
  pub(crate) terminal_type: String,
  pub(crate) terminal_rows: u16,
  pub(crate) terminal_cols: u16,
  pub(crate) emulator: Box<dyn TerminalEmulator>,
  pub(crate) filter: FilterRuntime,
  pub(crate) chat_block_pending: bool,
  pub(crate) chat_context: Option<TerminalChatContext>,
  pub(crate) chat_seq: u64,
  pub(crate) chat_span_id: Option<String>,
  pub(crate) chat_last_command: Option<String>,
  pub(crate) chat_last_input_lines: Option<Vec<String>>,
  pub(crate) chat_stream_enabled: bool,
}

impl SemanticState {
  /// 构造语义状态并初始化无回滚的终端仿真器。
  pub(crate) fn new(
    terminal_id: String,
    member_id: Option<String>,
    workspace_id: Option<String>,
    rows: u16,
    cols: u16,
    terminal_type: String,
  ) -> Self {
    let emulator = create_emulator(EmulatorConfig {
      rows,
      cols,
      scrollback_limit: SEMANTIC_SCROLLBACK_LINES,
    });
    Self {
      terminal_id,
      member_id,
      workspace_id,
      terminal_type: terminal_type.clone(),
      terminal_rows: rows,
      terminal_cols: cols,
      emulator,
      filter: FilterRuntime::new(&terminal_type),
      chat_block_pending: false,
      chat_context: None,
      chat_seq: 0,
      chat_span_id: None,
      chat_last_command: None,
      chat_last_input_lines: None,
      chat_stream_enabled: true,
    }
  }

  /// 重置语义仿真器，以最新尺寸重建干净视口。
  pub(crate) fn reset_emulator(&mut self, rows: u16, cols: u16) {
    self.emulator = create_emulator(EmulatorConfig {
      rows,
      cols,
      scrollback_limit: SEMANTIC_SCROLLBACK_LINES,
    });
    self.terminal_rows = rows;
    self.terminal_cols = cols;
  }

  pub(crate) fn set_size(&mut self, rows: u16, cols: u16) {
    self.emulator.set_size(rows, cols);
    self.terminal_rows = rows;
    self.terminal_cols = cols;
  }
}

/// 返回会话内递增序列，避免前端/聊天侧乱序拼接。
pub(crate) fn next_chat_seq(state: &mut SemanticState) -> u64 {
  state.chat_seq = state.chat_seq.saturating_add(1);
  state.chat_seq
}

/// 构建指定模式的语义消息负载。
/// 约束：模式由上层约定（stream/final/snapshot）。
pub(crate) fn build_semantic_payload(
  state: &mut SemanticState,
  message_type: &str,
  source: &str,
  mode: &str,
  lines: &[String],
) -> Option<TerminalMessagePayload> {
  let (start, end) = trim_empty_lines(lines);
  if start >= end {
    return None;
  }
  let mut content = lines[start..end].join("\n");
  if content.trim().is_empty() {
    return None;
  }
  if let Some(context) = state.chat_context.as_ref() {
    if context.conversation_type == "channel" {
      let sender = context.sender_name.trim();
      if !sender.is_empty() {
        // 频道消息必须带提及，避免消息被聊天侧忽略。
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
    terminal_id: state.terminal_id.clone(),
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
    mode: mode.to_string(),
    span_id: state.chat_span_id.clone(),
    meta: Some(meta),
  })
}

/// 去除首尾空行，避免消息包含大段无意义空白。
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

