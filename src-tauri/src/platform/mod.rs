//! 平台基础层入口：承载遥测/日志等横切能力。

pub(crate) mod monitoring;
pub(crate) mod paths;
pub(crate) mod updater;
pub(crate) mod activation;

pub(crate) use monitoring::{backend_passive_enabled, diagnostics_log_backend_event, DiagnosticsState};
pub(crate) use paths::resolve_log_dir;
pub(crate) use updater::{UpdaterState, UpdaterStatusPayload};
pub(crate) use activation::{ActivationState, ActivationStatusPayload};
