//! UI 消息命令包装：集中承载对消息服务的 IPC 暴露。

use tauri::{AppHandle, State};

use crate::application::chat as chat_app;
use crate::message_service::chat_db::{
  ChatClearResult, ChatDbManager, ChatHomeFeedDto, ChatRepairResult, ConversationSummaryDto,
  MessageAttachment, MessageContent, MessageDto,
};
use crate::contracts::chat_dispatch::ChatDispatchPayload;

#[tauri::command]
pub(crate) fn chat_ulid_new() -> Result<String, String> {
  chat_app::chat_ulid_new()
}

#[tauri::command]
pub(crate) fn chat_repair_messages(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
) -> Result<ChatRepairResult, String> {
  chat_app::chat_repair_messages(state, workspace_id)
}

#[tauri::command]
pub(crate) fn chat_clear_all_messages(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
) -> Result<ChatClearResult, String> {
  chat_app::chat_clear_all_messages(state, workspace_id)
}

#[tauri::command]
pub(crate) fn chat_list_conversations(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  workspace_name: Option<String>,
  member_ids: Vec<String>,
) -> Result<ChatHomeFeedDto, String> {
  chat_app::chat_list_conversations(state, workspace_id, user_id, workspace_name, member_ids)
}

#[tauri::command]
pub(crate) fn chat_get_messages(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  limit: Option<u32>,
  before_id: Option<String>,
) -> Result<Vec<MessageDto>, String> {
  chat_app::chat_get_messages(state, workspace_id, conversation_id, limit, before_id)
}

#[tauri::command]
pub(crate) fn chat_mark_conversation_read_latest(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  conversation_id: String,
) -> Result<(), String> {
  chat_app::chat_mark_conversation_read_latest(
    app,
    state,
    workspace_id,
    user_id,
    conversation_id,
  )
}

#[tauri::command]
pub(crate) fn chat_send_message(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  sender_id: Option<String>,
  viewer_id: Option<String>,
  content: MessageContent,
  is_ai: Option<bool>,
  attachment: Option<MessageAttachment>,
) -> Result<MessageDto, String> {
  chat_app::chat_send_message(
    app,
    state,
    workspace_id,
    conversation_id,
    sender_id,
    viewer_id,
    content,
    is_ai,
    attachment,
  )
}

#[tauri::command]
pub(crate) fn chat_send_message_and_dispatch(
  app: AppHandle,
  chat_state: State<'_, ChatDbManager>,
  payload: ChatDispatchPayload,
) -> Result<MessageDto, String> {
  chat_app::chat_send_message_and_enqueue(app, chat_state, payload)
}

#[tauri::command]
pub(crate) fn chat_create_group(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  member_ids: Vec<String>,
  custom_name: Option<String>,
) -> Result<ConversationSummaryDto, String> {
  chat_app::chat_create_group(state, workspace_id, user_id, member_ids, custom_name)
}

#[tauri::command]
pub(crate) fn chat_ensure_direct(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  target_id: String,
) -> Result<ConversationSummaryDto, String> {
  chat_app::chat_ensure_direct(app, state, workspace_id, user_id, target_id)
}

#[tauri::command]
pub(crate) fn chat_set_conversation_settings(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  conversation_id: String,
  pinned: Option<bool>,
  muted: Option<bool>,
) -> Result<(), String> {
  chat_app::chat_set_conversation_settings(
    state,
    workspace_id,
    user_id,
    conversation_id,
    pinned,
    muted,
  )
}

#[tauri::command]
pub(crate) fn chat_rename_conversation(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  custom_name: Option<String>,
) -> Result<(), String> {
  chat_app::chat_rename_conversation(state, workspace_id, conversation_id, custom_name)
}

#[tauri::command]
pub(crate) fn chat_clear_conversation(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
) -> Result<(), String> {
  chat_app::chat_clear_conversation(state, workspace_id, conversation_id)
}

#[tauri::command]
pub(crate) fn chat_delete_conversation(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
) -> Result<(), String> {
  chat_app::chat_delete_conversation(state, workspace_id, conversation_id)
}

#[tauri::command]
pub(crate) fn chat_set_conversation_members(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  member_ids: Vec<String>,
) -> Result<(), String> {
  chat_app::chat_set_conversation_members(app, state, workspace_id, conversation_id, member_ids)
}
