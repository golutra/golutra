//! UI 接口层入口：收拢对前端暴露的命令与状态类型。

pub(crate) mod app;
pub(crate) mod commands;
pub(crate) mod message;
pub(crate) mod message_pipeline;
pub(crate) mod monitoring;
pub(crate) mod notification;
pub(crate) mod platform;
pub(crate) mod project_skills;
pub(crate) mod project_data;
pub(crate) mod project_members;
pub(crate) mod skills;
pub(crate) mod terminal;
pub(crate) mod terminal_events;
pub(crate) mod terminal_session_repository;

pub(crate) use app::{
  apply_windows_rounding,
  apply_main_window_size,
  schedule_main_window_frame_refresh,
  show_main_window,
  setup_tray,
  MAIN_WINDOW_LABEL,
};
pub(crate) use commands::export_commands;
pub(crate) use notification::NotificationBadgeState;
pub(crate) use crate::message_service::chat_db::ChatDbManager;
pub(crate) use crate::platform::DiagnosticsState;
pub(crate) use crate::terminal_engine::{
  cleanup_ephemeral_sessions_for_window,
  has_active_sessions,
  shutdown_sessions,
  spawn_snapshot_dumper,
  spawn_status_poller,
};
