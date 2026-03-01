//! 激活设置模块：授权验证、离线缓存与租期续期。

use std::sync::Mutex;

use serde::Serialize;

#[derive(Clone, Serialize)]
pub(crate) struct ActivationStatusPayload {
  pub(crate) state: String,
  pub(crate) expires_at: Option<u64>,
  pub(crate) last_checked_at: Option<u64>,
  pub(crate) last_error: Option<String>,
}

pub(crate) struct ActivationState {
  status: Mutex<ActivationStatusPayload>,
}

impl ActivationState {
  pub(crate) fn new() -> Self {
    Self {
      status: Mutex::new(ActivationStatusPayload {
        state: "unknown".to_string(),
        expires_at: None,
        last_checked_at: None,
        last_error: None,
      }),
    }
  }

  pub(crate) fn snapshot(&self) -> ActivationStatusPayload {
    match self.status.lock() {
      Ok(guard) => guard.clone(),
      Err(err) => err.into_inner().clone(),
    }
  }

  // [TODO/platform, 2026-01-26] 补齐授权验证、离线续期与设备绑定流程。
}
