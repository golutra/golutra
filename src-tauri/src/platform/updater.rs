//! 软件更新模块：版本检查、下载与回滚的状态持有。

use std::sync::Mutex;

use serde::Serialize;

#[derive(Clone, Serialize)]
pub(crate) struct UpdaterStatusPayload {
  pub(crate) state: String,
  pub(crate) current_version: Option<String>,
  pub(crate) available_version: Option<String>,
  pub(crate) last_checked_at: Option<u64>,
  pub(crate) last_error: Option<String>,
}

pub(crate) struct UpdaterState {
  status: Mutex<UpdaterStatusPayload>,
}

impl UpdaterState {
  pub(crate) fn new() -> Self {
    Self {
      status: Mutex::new(UpdaterStatusPayload {
        state: "unknown".to_string(),
        current_version: None,
        available_version: None,
        last_checked_at: None,
        last_error: None,
      }),
    }
  }

  pub(crate) fn snapshot(&self) -> UpdaterStatusPayload {
    match self.status.lock() {
      Ok(guard) => guard.clone(),
      Err(err) => err.into_inner().clone(),
    }
  }

  // [TODO/platform, 2026-01-26] 补齐版本检查、下载与回滚流程，并更新状态字段。
}
