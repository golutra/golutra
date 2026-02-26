use serde::Serialize;

#[derive(Serialize, Clone)]
pub(crate) struct TerminalOutputPayload {
  #[serde(rename = "sessionId")]
  pub(crate) session_id: String,
  pub(crate) data: String,
  pub(crate) seq: u64,
}

#[derive(Serialize, Clone)]
pub(crate) struct TerminalExitPayload {
  #[serde(rename = "sessionId")]
  pub(crate) session_id: String,
  pub(crate) code: Option<i32>,
  pub(crate) signal: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct TerminalStatusPayload {
  #[serde(rename = "sessionId")]
  pub(crate) session_id: String,
  pub(crate) status: String,
  #[serde(rename = "memberId")]
  pub(crate) member_id: Option<String>,
  #[serde(rename = "workspaceId")]
  pub(crate) workspace_id: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct TerminalSnapshotPayload {
  #[serde(rename = "sessionId")]
  pub(crate) session_id: String,
  pub(crate) data: String,
  pub(crate) seq: u64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) history: Option<String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct TerminalErrorPayload {
  #[serde(rename = "sessionId")]
  pub(crate) session_id: String,
  pub(crate) error: String,
  pub(crate) fatal: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct TerminalCursorPayload {
  pub(crate) row: u16,
  pub(crate) col: u16,
}

#[derive(Serialize, Clone)]
pub(crate) struct TerminalMessageMeta {
  #[serde(rename = "command")]
  pub(crate) command: Option<String>,
  #[serde(rename = "lineCount")]
  pub(crate) line_count: Option<u32>,
  pub(crate) cursor: Option<TerminalCursorPayload>,
  #[serde(rename = "startRow")]
  pub(crate) start_row: Option<u16>,
  #[serde(rename = "endRow")]
  pub(crate) end_row: Option<u16>,
}

#[derive(Serialize, Clone)]
pub(crate) struct TerminalMessagePayload {
  #[serde(rename = "sessionId")]
  pub(crate) session_id: String,
  #[serde(rename = "memberId")]
  pub(crate) member_id: Option<String>,
  #[serde(rename = "workspaceId")]
  pub(crate) workspace_id: Option<String>,
  #[serde(rename = "conversationId")]
  pub(crate) conversation_id: Option<String>,
  #[serde(rename = "conversationType")]
  pub(crate) conversation_type: Option<String>,
  #[serde(rename = "senderId")]
  pub(crate) sender_id: Option<String>,
  #[serde(rename = "senderName")]
  pub(crate) sender_name: Option<String>,
  pub(crate) seq: u64,
  pub(crate) timestamp: u64,
  pub(crate) content: String,
  #[serde(rename = "type")]
  pub(crate) message_type: String,
  pub(crate) source: String,
  pub(crate) mode: String,
  #[serde(rename = "spanId")]
  pub(crate) span_id: Option<String>,
  pub(crate) meta: Option<TerminalMessageMeta>,
}
