//! 终端子系统入口：组合 PTY、语义分析与会话管理，对外提供 Tauri 命令。
//! 边界：不负责 UI 渲染与前端状态，只输出事件与 IPC 接口。

pub(crate) mod models;
pub(crate) mod emulator;
pub(crate) mod filters;
pub(crate) mod semantic;
pub(crate) mod session;
pub(crate) mod errors;
pub(crate) mod default_members;

// 统一对外导出会话管理与命令接口，避免上层直接依赖内部模块细节。
pub(crate) use session::{
  cleanup_ephemeral_sessions_for_window, has_active_sessions, shutdown_sessions, spawn_snapshot_dumper,
  spawn_status_poller, TerminalManager,
};
