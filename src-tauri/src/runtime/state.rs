use std::collections::HashMap;
use std::sync::Mutex;

/// 运行时全局状态：集中管理跨层共享实例。
pub(crate) struct AppState {
  pub(crate) workspace_registry_lock: Mutex<()>,
  pub(crate) workspace_windows: Mutex<HashMap<String, String>>,
  pub(crate) active_main_window: Mutex<Option<String>>,
}

impl Default for AppState {
  fn default() -> Self {
    Self {
      workspace_registry_lock: Mutex::new(()),
      workspace_windows: Mutex::new(HashMap::new()),
      active_main_window: Mutex::new(None),
    }
  }
}
