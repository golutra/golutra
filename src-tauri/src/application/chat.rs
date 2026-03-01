//! 聊天应用层：统一 UI 与 CLI 的业务入口，避免重复规则。

use tauri::{AppHandle, State};

use crate::contracts::chat_dispatch::ChatDispatchPayload;
use crate::message_service::chat_db::{
  self, chat_outbox_enqueue, chat_send_message_for_dispatch, ChatClearResult,
  ChatDeleteMemberConversationsResult, ChatDbManager, ChatHomeFeedDto, ChatRepairResult,
  ConversationSummaryDto, MessageAttachment, MessageContent, MessageDto,
};

pub(crate) fn chat_ulid_new() -> Result<String, String> {
  chat_db::chat_ulid_new()
}

pub(crate) fn chat_repair_messages(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
) -> Result<ChatRepairResult, String> {
  chat_db::chat_repair_messages(state, workspace_id)
}

pub(crate) fn chat_clear_all_messages(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
) -> Result<ChatClearResult, String> {
  chat_db::chat_clear_all_messages(state, workspace_id)
}

pub(crate) fn chat_list_conversations(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  workspace_name: Option<String>,
  member_ids: Vec<String>,
) -> Result<ChatHomeFeedDto, String> {
  chat_db::chat_list_conversations(state, workspace_id, user_id, workspace_name, member_ids)
}

pub(crate) fn chat_get_messages(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  limit: Option<u32>,
  before_id: Option<String>,
) -> Result<Vec<MessageDto>, String> {
  chat_db::chat_get_messages(state, workspace_id, conversation_id, limit, before_id)
}

pub(crate) fn chat_mark_conversation_read_latest(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  conversation_id: String,
) -> Result<(), String> {
  chat_db::chat_mark_conversation_read_latest(app, state, workspace_id, user_id, conversation_id)
}

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
  chat_db::chat_send_message(
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

pub(crate) fn chat_send_message_and_enqueue(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  mut payload: ChatDispatchPayload,
) -> Result<MessageDto, String> {
  let message = chat_send_message_for_dispatch(
    app,
    state.clone(),
    payload.workspace_id.clone(),
    payload.conversation_id.clone(),
    Some(payload.sender_id.clone()),
    Some(payload.sender_id.clone()),
    MessageContent::Text {
      text: payload.text.clone(),
    },
    Some(false),
    None,
  )?;
  let workspace_id = payload.workspace_id.clone();
  payload.message_id = Some(message.id.clone());
  chat_outbox_enqueue(state.inner(), workspace_id.as_str(), &message.id, payload)?;
  Ok(message)
}

pub(crate) fn chat_create_group(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  member_ids: Vec<String>,
  custom_name: Option<String>,
) -> Result<ConversationSummaryDto, String> {
  chat_db::chat_create_group(state, workspace_id, user_id, member_ids, custom_name)
}

pub(crate) fn chat_ensure_direct(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  target_id: String,
) -> Result<ConversationSummaryDto, String> {
  chat_db::chat_ensure_direct(app, state, workspace_id, user_id, target_id)
}

pub(crate) fn chat_set_conversation_settings(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  conversation_id: String,
  pinned: Option<bool>,
  muted: Option<bool>,
) -> Result<(), String> {
  chat_db::chat_set_conversation_settings(
    state,
    workspace_id,
    user_id,
    conversation_id,
    pinned,
    muted,
  )
}

pub(crate) fn chat_rename_conversation(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  custom_name: Option<String>,
) -> Result<(), String> {
  chat_db::chat_rename_conversation(state, workspace_id, conversation_id, custom_name)
}

pub(crate) fn chat_clear_conversation(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
) -> Result<(), String> {
  chat_db::chat_clear_conversation(state, workspace_id, conversation_id)
}

pub(crate) fn chat_delete_conversation(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
) -> Result<(), String> {
  chat_db::chat_delete_conversation(state, workspace_id, conversation_id)
}

pub(crate) fn chat_delete_member_conversations(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  member_ids: Vec<String>,
) -> Result<ChatDeleteMemberConversationsResult, String> {
  chat_db::chat_delete_member_conversations(state, workspace_id, member_ids)
}

pub(crate) fn chat_set_conversation_members(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  member_ids: Vec<String>,
) -> Result<(), String> {
  chat_db::chat_set_conversation_members(app, state, workspace_id, conversation_id, member_ids)
}
