//! 聊天派发契约：跨层共享的派发载荷定义。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChatDispatchMentions {
  pub(crate) mention_ids: Vec<String>,
  pub(crate) mention_all: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChatDispatchPayload {
  pub(crate) workspace_id: String,
  pub(crate) workspace_path: String,
  pub(crate) conversation_id: String,
  pub(crate) conversation_type: String,
  pub(crate) text: String,
  pub(crate) sender_id: String,
  pub(crate) sender_name: String,
  pub(crate) mentions: Option<ChatDispatchMentions>,
  pub(crate) message_id: Option<String>,
  pub(crate) client_trace_id: Option<String>,
  pub(crate) timestamp: Option<u64>,
}
