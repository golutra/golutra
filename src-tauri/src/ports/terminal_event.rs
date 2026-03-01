//! 终端事件端口：隔离终端引擎对 Tauri 事件通道的直接依赖。

use std::sync::Arc;

use crate::terminal_engine::models::{
  TerminalErrorPayload, TerminalExitPayload, TerminalOutputPayload, TerminalStatusPayload,
};

pub(crate) trait TerminalEventPort: Send + Sync {
  fn emit_output(
    &self,
    window_label: Option<&str>,
    payload: TerminalOutputPayload,
  ) -> Result<(), String>;
  fn emit_status(&self, payload: TerminalStatusPayload) -> Result<(), String>;
  fn emit_error(&self, payload: TerminalErrorPayload) -> Result<(), String>;
  fn emit_exit(&self, payload: TerminalExitPayload) -> Result<(), String>;
}

struct NoopTerminalEventPort;

impl TerminalEventPort for NoopTerminalEventPort {
  fn emit_output(
    &self,
    _window_label: Option<&str>,
    _payload: TerminalOutputPayload,
  ) -> Result<(), String> {
    Ok(())
  }

  fn emit_status(&self, _payload: TerminalStatusPayload) -> Result<(), String> {
    Ok(())
  }

  fn emit_error(&self, _payload: TerminalErrorPayload) -> Result<(), String> {
    Ok(())
  }

  fn emit_exit(&self, _payload: TerminalExitPayload) -> Result<(), String> {
    Ok(())
  }
}

pub(crate) fn default_terminal_event_port() -> Arc<dyn TerminalEventPort> {
  Arc::new(NoopTerminalEventPort)
}
