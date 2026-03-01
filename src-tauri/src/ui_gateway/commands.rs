//! UI 命令注册：统一导出 Tauri 命令入口。

use tauri::ipc::Invoke;

use super::app;
use super::message;
use super::monitoring;
use super::notification;
use super::platform;
use super::project_data;
use super::project_members;
use super::project_skills;
use super::skills;
use super::terminal;

pub(crate) fn export_commands() -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
  tauri::generate_handler![
    app::terminal_open_window,
    app::workspace_selection_open_window,
    terminal::terminal_create,
    terminal::terminal_list_environments,
    terminal::terminal_attach,
    terminal::terminal_write,
    terminal::terminal_ack,
    terminal::terminal_set_active,
    terminal::terminal_emit_status,
    terminal::terminal_set_member_status,
    terminal::terminal_dispatch,
    terminal::terminal_resize,
    terminal::terminal_close,
    terminal::terminal_list_statuses,
    terminal::terminal_snapshot_lines,
    terminal::terminal_snapshot_text,
    terminal::terminal_dump_snapshot_lines,
    notification::notification_update_state,
    notification::notification_get_state,
    notification::notification_preview_hover,
    notification::notification_preview_hide,
    notification::notification_request_ignore_all,
    notification::notification_set_active_window,
    app::notification_open_terminal,
    app::notification_open_all_unread,
    app::notification_open_unread_conversation,
    monitoring::diagnostics_start_run,
    monitoring::diagnostics_end_run,
    monitoring::diagnostics_register_member,
    monitoring::diagnostics_register_session,
    monitoring::diagnostics_register_conversation,
    monitoring::diagnostics_register_window,
    monitoring::diagnostics_log_frontend_event,
    monitoring::diagnostics_log_frontend_batch,
    monitoring::diagnostics_log_snapshot_triplet,
    monitoring::diagnostics_log_chat_consistency,
    platform::platform_get_updater_status,
    platform::platform_get_activation_status,
    message::chat_ulid_new,
    message::chat_repair_messages,
    message::chat_clear_all_messages,
    message::chat_list_conversations,
    message::chat_get_messages,
    message::chat_mark_conversation_read_latest,
    message::chat_send_message,
    message::chat_send_message_and_dispatch,
    message::chat_create_group,
    message::chat_ensure_direct,
    message::chat_set_conversation_settings,
    message::chat_rename_conversation,
    message::chat_clear_conversation,
    message::chat_delete_conversation,
    message::chat_set_conversation_members,
    app::workspace_recent_list,
    app::workspace_open_folder,
    app::workspace_open,
    app::workspace_clear_window,
    app::storage_read_app,
    app::storage_write_app,
    app::storage_read_cache,
    app::storage_write_cache,
    app::storage_write_cache_text,
    app::storage_read_workspace,
    app::storage_write_workspace,
    app::avatar_list,
    app::avatar_store,
    app::avatar_delete,
    app::avatar_resolve_path,
    app::avatar_read,
    project_data::project_data_read,
    project_data::project_data_write,
    project_members::project_members_invite,
    project_members::project_members_purge_terminal,
    project_skills::project_skills_list,
    project_skills::project_skills_link,
    project_skills::project_skills_unlink,
    skills::skills_import_folder,
    skills::skills_remove_folder,
    skills::skills_open_folder
  ]
}
