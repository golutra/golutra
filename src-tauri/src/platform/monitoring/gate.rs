// 后端被动监控开关：统一判断后端诊断/追踪是否启用。
use std::env;

fn env_flag(name: &str) -> bool {
  match env::var(name) {
    Ok(value) => matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"),
    Err(_) => false,
  }
}

pub fn backend_passive_enabled() -> bool {
  env_flag("GOLUTRA_DEBUG")
}
