//! 终端事件端口适配：将终端引擎事件映射到 Tauri IPC。

use tauri::{AppHandle, Emitter, Manager};

use crate::ports::terminal_event::TerminalEventPort;
use crate::terminal_engine::models::{
  TerminalErrorPayload, TerminalExitPayload, TerminalOutputPayload, TerminalStatusPayload,
};

pub(crate) struct UiTerminalEventPort {
  app: AppHandle,
}

impl UiTerminalEventPort {
  pub(crate) fn new(app: AppHandle) -> Self {
    Self { app }
  }
}

impl TerminalEventPort for UiTerminalEventPort {
  fn emit_output(
    &self,
    window_label: Option<&str>,
    payload: TerminalOutputPayload,
  ) -> Result<(), String> {
    if let Some(label) = window_label {
      if let Some(window) = self.app.get_webview_window(label) {
        let _ = window.emit("terminal-output", payload);
        return Ok(());
      }
    }
    let _ = self.app.emit("terminal-output", payload);
    Ok(())
  }

  fn emit_status(&self, payload: TerminalStatusPayload) -> Result<(), String> {
    let _ = self.app.emit("terminal-status-change", payload);
    Ok(())
  }

  fn emit_error(&self, payload: TerminalErrorPayload) -> Result<(), String> {
    let _ = self.app.emit("terminal-error", payload);
    Ok(())
  }

  fn emit_exit(&self, payload: TerminalExitPayload) -> Result<(), String> {
    let _ = self.app.emit("terminal-exit", payload);
    Ok(())
  }
}
