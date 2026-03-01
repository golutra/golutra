//! 终端派发门禁端口：语义 flush 完成后通知派发队列释放。

use std::sync::Arc;

use tauri::AppHandle;

pub(crate) trait TerminalDispatchGate: Send + Sync {
  fn on_semantic_flush_complete(&self, app: &AppHandle, terminal_id: &str);
}

struct NoopTerminalDispatchGate;

impl TerminalDispatchGate for NoopTerminalDispatchGate {
  fn on_semantic_flush_complete(&self, _app: &AppHandle, _terminal_id: &str) {}
}

pub(crate) fn default_terminal_dispatch_gate() -> Arc<dyn TerminalDispatchGate> {
  Arc::new(NoopTerminalDispatchGate)
}
