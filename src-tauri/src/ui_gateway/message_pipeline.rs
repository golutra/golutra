//! 消息端口适配：将 Tauri 传输与存储绑定到消息流水线。

use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};

use crate::contracts::terminal_message::TerminalMessagePayload;
use crate::message_service::chat_db::{chat_append_terminal_message, ChatDbManager};
use crate::message_service::pipeline;
use crate::ports::message_service::{
  TerminalMessageAppendResult, TerminalMessageRepository, TerminalMessageTransport,
};
use crate::ports::terminal_message::TerminalMessagePipeline;

pub(crate) struct UiMessageTransport {
  app: AppHandle,
}

impl UiMessageTransport {
  pub(crate) fn new(app: AppHandle) -> Self {
    Self { app }
  }
}

impl TerminalMessageTransport for UiMessageTransport {
  fn emit_terminal_stream(&self, payload: TerminalMessagePayload) -> Result<(), String> {
    let _ = self.app.emit("terminal-message-stream", payload);
    Ok(())
  }
}

pub(crate) struct UiMessageRepository {
  app: AppHandle,
}

impl UiMessageRepository {
  pub(crate) fn new(app: AppHandle) -> Self {
    Self { app }
  }
}

impl TerminalMessageRepository for UiMessageRepository {
  fn append_terminal_message(
    &self,
    workspace_id: &str,
    conversation_id: &str,
    member_id: &str,
    content: String,
    viewer_id: &str,
    span_id: Option<&str>,
  ) -> Result<TerminalMessageAppendResult, String> {
    let state = self.app.state::<ChatDbManager>();
    let message_id = chat_append_terminal_message(
      &self.app,
      state.inner(),
      workspace_id,
      conversation_id,
      member_id,
      content,
      viewer_id,
      span_id,
    )?;
    Ok(TerminalMessageAppendResult::persisted(message_id))
  }
}

pub(crate) struct UiTerminalMessagePipeline {
  transport: Arc<dyn TerminalMessageTransport>,
  repository: Arc<dyn TerminalMessageRepository>,
}

impl UiTerminalMessagePipeline {
  pub(crate) fn new(
    transport: Arc<dyn TerminalMessageTransport>,
    repository: Arc<dyn TerminalMessageRepository>,
  ) -> Self {
    Self {
      transport,
      repository,
    }
  }
}

impl TerminalMessagePipeline for UiTerminalMessagePipeline {
  fn process_stream(&self, payload: TerminalMessagePayload) -> Result<(), String> {
    pipeline::process_terminal_stream(
      self.transport.as_ref(),
      self.repository.as_ref(),
      payload,
    )
  }

  fn process_final(
    &self,
    payload: TerminalMessagePayload,
  ) -> Result<TerminalMessageAppendResult, String> {
    pipeline::process_terminal_final(
      self.transport.as_ref(),
      self.repository.as_ref(),
      payload,
    )
  }
}
