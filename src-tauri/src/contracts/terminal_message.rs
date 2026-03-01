//! 终端消息契约：用于引擎与消息服务之间的数据交换。

use serde::Serialize;

#[derive(Serialize, Clone)]
/// 光标位置载荷，默认使用 0 基坐标以贴合内部模拟器状态。
pub(crate) struct TerminalCursorPayload {
  pub(crate) row: u16,
  pub(crate) col: u16,
}

#[derive(Serialize, Clone)]
/// 消息元数据：用于聊天侧渲染补充信息（命令、行数、光标等）。
/// 约束：字段可选以避免在低成本路径中强制计算。
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
/// 终端内容快照消息，用于写回聊天记录或侧边输出流。
/// 约束：`seq` 为会话内递增序列；`mode` 与 `source` 需与前端约定一致。
pub(crate) struct TerminalMessagePayload {
  #[serde(rename = "terminalId")]
  pub(crate) terminal_id: String,
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
