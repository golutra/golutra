//! 监控模块聚合：统一暴露诊断记录与开关能力，避免散落在主流程中。

pub mod diagnostics;
pub mod gate;

pub use diagnostics::{diagnostics_log_backend_event, DiagnosticsState};
pub use gate::backend_passive_enabled;
