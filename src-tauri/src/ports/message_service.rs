//! 消息服务端口：隔离业务流水线对传输与存储的直接依赖。

use crate::contracts::terminal_message::TerminalMessagePayload;

#[derive(Clone, Debug)]
/// 终端消息落库结果：用于队列推进与诊断链路。
pub(crate) struct TerminalMessageAppendResult {
  pub(crate) message_id: Option<String>,
  pub(crate) persisted: bool,
}

impl TerminalMessageAppendResult {
  pub(crate) fn persisted(message_id: String) -> Self {
    Self {
      message_id: Some(message_id),
      persisted: true,
    }
  }

  pub(crate) fn skipped() -> Self {
    Self {
      message_id: None,
      persisted: false,
    }
  }
}

pub(crate) trait TerminalMessageTransport: Send + Sync {
  fn emit_terminal_stream(&self, payload: TerminalMessagePayload) -> Result<(), String>;
}

pub(crate) trait TerminalMessageRepository: Send + Sync {
  fn append_terminal_message(
    &self,
    workspace_id: &str,
    conversation_id: &str,
    member_id: &str,
    content: String,
    viewer_id: &str,
    span_id: Option<&str>,
  ) -> Result<TerminalMessageAppendResult, String>;
}
