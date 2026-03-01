//! 归一化阶段：统一消息格式与提取元信息。

use crate::contracts::terminal_message::TerminalMessagePayload;

use super::types::MessageEnvelope;

pub(crate) fn normalize_terminal(payload: TerminalMessagePayload) -> Result<MessageEnvelope, String> {
  // [TODO/message-service, 2026-01-26] 补齐去重键、元信息抽取与格式标准化。
  Ok(MessageEnvelope { payload })
}
