//! 终端消息端口：隔离终端引擎与消息服务的直接依赖。

use std::sync::Arc;

use crate::contracts::terminal_message::TerminalMessagePayload;
use crate::ports::message_service::TerminalMessageAppendResult;

pub(crate) trait TerminalMessagePipeline: Send + Sync {
  fn process_stream(&self, payload: TerminalMessagePayload) -> Result<(), String>;
  fn process_final(
    &self,
    payload: TerminalMessagePayload,
  ) -> Result<TerminalMessageAppendResult, String>;
}

pub(crate) struct NoopTerminalMessagePipeline;

impl TerminalMessagePipeline for NoopTerminalMessagePipeline {
  fn process_stream(&self, _payload: TerminalMessagePayload) -> Result<(), String> {
    Ok(())
  }

  fn process_final(
    &self,
    _payload: TerminalMessagePayload,
  ) -> Result<TerminalMessageAppendResult, String> {
    Ok(TerminalMessageAppendResult::skipped())
  }
}

pub(crate) fn default_terminal_message_pipeline() -> Arc<dyn TerminalMessagePipeline> {
  Arc::new(NoopTerminalMessagePipeline)
}
