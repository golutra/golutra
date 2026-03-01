//! 聊天写入子域：负责会话与消息的变更与事件广播。

use std::collections::HashSet;

use redb::ReadableTable;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, State};
use ulid::Ulid;

use crate::platform::{diagnostics_log_backend_event, DiagnosticsState};

use super::store::{
  build_conversation_summary, clear_chat_storage, compute_conversation_unread_count,
  compute_total_unread_count, count_unread_messages, decode, encode, format_ulid,
  load_member_ids_from_table, now_millis, open_db, parse_ulid, repair_invalid_messages,
  save_message_in_db, sync_conversation_members, ts_rev, ATTACHMENTS_INDEX, CONVERSATIONS, MEMBERS,
  MESSAGES, TIMELINE_INDEX, USER_CONVS,
};
use super::types::{
  ChatClearResult, ChatDeleteMemberConversationsResult, ChatMessage, ChatMessageCreatedPayload,
  ChatMessageStatusPayload, ChatRepairResult, ChatUnreadSyncPayload, ConversationKind,
  ConversationMeta, ConversationSummaryDto, MessageAttachment, MessageContent, MessageDto,
  MessageStatus, UserConversationSettings,
};
use super::ChatDbManager;

fn mark_conversation_read_with_message_id(
  db: &redb::Database,
  user_id: super::types::UserId,
  conv_id: super::types::ConvId,
  message_id: super::types::MsgId,
) -> Result<(), String> {
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  let mut settings = {
    let table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    let existing = table
      .get((user_id, conv_id))
      .map_err(|err| format!("failed to read user_convs: {err}"))?;
    match existing {
      Some(value) => decode(value.value())?,
      None => UserConversationSettings::default(),
    }
  };
  settings.last_read_message_id = Some(message_id);
  let payload = encode(&settings)?;
  {
    let mut table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    table
      .insert((user_id, conv_id), payload.as_slice())
      .map_err(|err| format!("failed to update user_convs: {err}"))?;
  }
  txn
    .commit()
    .map_err(|err| format!("failed to commit read marker: {err}"))?;
  Ok(())
}

fn emit_unread_sync(
  app: &AppHandle,
  workspace_id: &str,
  viewer_id: super::types::UserId,
  db: &redb::Database,
  conversation_id: Option<super::types::ConvId>,
  reset_all: bool,
) -> Result<(), String> {
  // 统一从数据库计算未读，确保多窗口共享同一真相来源。
  let total_unread_count = compute_total_unread_count(db, viewer_id)?;
  let conversation_unread_count = match conversation_id {
    Some(conv_id) => Some(compute_conversation_unread_count(db, viewer_id, conv_id)?),
    None => None,
  };
  let payload = ChatUnreadSyncPayload {
    workspace_id: workspace_id.to_string(),
    total_unread_count,
    conversation_id: conversation_id.map(format_ulid),
    conversation_unread_count,
    reset_all,
  };
  let _ = app.emit("chat-unread-sync", payload);
  Ok(())
}

/// 生成新的 ULID 字符串，用于前端预分配 ID。
/// 返回：ULID 字符串。
pub fn chat_ulid_new() -> Result<String, String> {
  Ok(Ulid::new().to_string())
}

/// 扫描并修复数据库中的无效消息记录。
/// 返回：修复统计信息。
/// 错误：数据库不可用或修复过程失败。
pub fn chat_repair_messages(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
) -> Result<ChatRepairResult, String> {
  let db = open_db(&state, &workspace_id)?;
  let removed = repair_invalid_messages(&db)?;
  Ok(ChatRepairResult {
    removed_messages: removed,
  })
}

/// 清空指定工作区的所有聊天消息与附件索引。
/// 返回：清理统计信息。
/// 错误：数据库不可用或清理失败。
pub fn chat_clear_all_messages(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
) -> Result<ChatClearResult, String> {
  let db = open_db(&state, &workspace_id)?;
  clear_chat_storage(&db)
}

/// 将会话标记为已读（最新消息）。
/// 错误：ID 解析失败或数据库写入失败。
pub fn chat_mark_conversation_read_latest(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  conversation_id: String,
) -> Result<(), String> {
  let user_id = parse_ulid(&user_id)?;
  let conv_id = parse_ulid(&conversation_id)?;
  let db = open_db(&state, &workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;

  let latest_msg_id = {
    let table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    let start = (conv_id, 0);
    let end = (conv_id, u128::MAX);
    table
      .range(start..=end)
      .map_err(|err| format!("failed to scan messages: {err}"))?
      .last()
      .and_then(|entry| entry.ok().map(|(key, _)| key.value().1))
  };
  let mut settings = {
    let table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    let existing = table
      .get((user_id, conv_id))
      .map_err(|err| format!("failed to read user_convs: {err}"))?;
    match existing {
      Some(value) => decode(value.value())?,
      None => UserConversationSettings::default(),
    }
  };
  settings.last_read_message_id = latest_msg_id;
  let payload = encode(&settings)?;
  {
    let mut table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    table
      .insert((user_id, conv_id), payload.as_slice())
      .map_err(|err| format!("failed to update user_convs: {err}"))?;
  }
  txn
    .commit()
    .map_err(|err| format!("failed to commit read marker: {err}"))?;
  emit_unread_sync(&app, &workspace_id, user_id, &db, Some(conv_id), false)?;
  Ok(())
}

/// 将工作区内所有会话标记为已读（最新消息）。
/// 错误：ID 解析失败或数据库写入失败。
pub fn chat_mark_workspace_read_latest(
  app: &AppHandle,
  state: &ChatDbManager,
  workspace_id: String,
  user_id: String,
) -> Result<(), String> {
  let user_id = parse_ulid(&user_id)?;
  let db = open_db(state, &workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;

  let mut targets: Vec<(u128, UserConversationSettings)> = Vec::new();
  {
    let table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    let start = (user_id, 0);
    let end = (user_id, u128::MAX);
    for entry in table
      .range(start..=end)
      .map_err(|err| format!("failed to scan user_convs: {err}"))?
    {
      let (key, value) = entry.map_err(|err| format!("failed to decode user_convs entry: {err}"))?;
      let (_, conv_id) = key.value();
      let settings: UserConversationSettings = decode(value.value())?;
      targets.push((conv_id, settings));
    }
  }

  if targets.is_empty() {
    txn
      .commit()
      .map_err(|err| format!("failed to commit read marker: {err}"))?;
    emit_unread_sync(app, &workspace_id, user_id, &db, None, true)?;
    return Ok(());
  }

  {
    let message_table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    let mut settings_table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    for (conv_id, mut settings) in targets {
      let latest_msg_id = message_table
        .range((conv_id, 0)..=(conv_id, u128::MAX))
        .map_err(|err| format!("failed to scan messages: {err}"))?
        .last()
        .and_then(|entry| entry.ok().map(|(key, _)| key.value().1));
      let Some(latest_msg_id) = latest_msg_id else {
        continue;
      };
      if settings.last_read_message_id == Some(latest_msg_id) {
        continue;
      }
      settings.last_read_message_id = Some(latest_msg_id);
      let payload = encode(&settings)?;
      settings_table
        .insert((user_id, conv_id), payload.as_slice())
        .map_err(|err| format!("failed to update user_convs: {err}"))?;
    }
  }

  txn
    .commit()
    .map_err(|err| format!("failed to commit read marker: {err}"))?;
  emit_unread_sync(app, &workspace_id, user_id, &db, None, true)?;
  Ok(())
}

fn chat_send_message_with_status(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  sender_id: Option<String>,
  viewer_id: Option<String>,
  content: MessageContent,
  is_ai: Option<bool>,
  attachment: Option<MessageAttachment>,
  status: MessageStatus,
) -> Result<MessageDto, String> {
  let workspace_id_for_log = workspace_id.clone();
  let conversation_id_for_log = conversation_id.clone();
  let sender_id_for_log = sender_id.clone();
  let content_for_log = content.clone();
  let attachment_for_log = attachment.clone();
  let conv_id = parse_ulid(&conversation_id)?;
  let sender_id = match sender_id {
    Some(value) => Some(parse_ulid(&value)?),
    None => None,
  };
  let viewer_id = match viewer_id {
    Some(value) => Some(parse_ulid(&value)?),
    None => None,
  };
  let db = open_db(&state, &workspace_id)?;
  let message = save_message_in_db(
    &db,
    conv_id,
    sender_id,
    content,
    is_ai.unwrap_or(false),
    status,
    attachment,
  )?;
  if let (Some(sender_id), Some(viewer_id)) = (sender_id, viewer_id) {
    if sender_id == viewer_id {
      match parse_ulid(&message.id)
        .ok()
        .and_then(|msg_id| mark_conversation_read_with_message_id(&db, viewer_id, conv_id, msg_id).ok())
      {
        Some(_) => {}
        None => {
          log::warn!(
            "chat_send_message self-read update failed workspace_id={} conversation_id={}",
            workspace_id,
            conversation_id
          );
        }
      }
    }
  }
  diagnostics_log_backend_event(
    &app.state::<DiagnosticsState>(),
    sender_id_for_log.clone(),
    None,
    Some(conversation_id_for_log.clone()),
    None,
    Some(workspace_id_for_log.clone()),
    "chat_send_message",
    json!({
      "workspaceId": workspace_id_for_log,
      "conversationId": conversation_id_for_log,
      "senderId": sender_id_for_log,
      "viewerId": viewer_id.as_ref().map(|value| value.to_string()),
      "content": content_for_log,
      "attachment": attachment_for_log
    }),
  );
  if let Some(viewer_id) = viewer_id {
    emit_unread_sync(&app, &workspace_id, viewer_id, &db, Some(conv_id), false)?;
  }
  Ok(message)
}

/// 追加一条消息到会话。
/// 输入：`content` 与 `attachment` 为消息主体；`is_ai` 标记 AI 消息。
/// 返回：标准化后的消息 DTO。
/// 错误：ID 解析失败或数据库写入失败。
pub fn chat_send_message(
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
  chat_send_message_with_status(
    app,
    state,
    workspace_id,
    conversation_id,
    sender_id,
    viewer_id,
    content,
    is_ai,
    attachment,
    MessageStatus::Sent,
  )
}

/// 追加一条待派发消息：状态初始为 Sending，交由 Outbox 处理。
pub(crate) fn chat_send_message_for_dispatch(
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
  chat_send_message_with_status(
    app,
    state,
    workspace_id,
    conversation_id,
    sender_id,
    viewer_id,
    content,
    is_ai,
    attachment,
    MessageStatus::Sending,
  )
}

/// 更新消息状态并广播给前端。
/// 约束：消息不存在时直接返回 Ok。
pub(crate) fn chat_update_message_status(
  app: &AppHandle,
  state: &ChatDbManager,
  workspace_id: &str,
  conversation_id: &str,
  message_id: &str,
  status: MessageStatus,
) -> Result<(), String> {
  let conv_id = parse_ulid(conversation_id)?;
  let msg_id = parse_ulid(message_id)?;
  let db = open_db(state, workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  {
    let mut table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    let mut message: ChatMessage = {
      let value = table
        .get((conv_id, msg_id))
        .map_err(|err| format!("failed to read message: {err}"))?;
      let Some(value) = value else {
        return Ok(());
      };
      decode(value.value())?
    };
    if message.status == status {
      return Ok(());
    }
    message.status = status;
    let payload = encode(&message)?;
    table
      .insert((conv_id, msg_id), payload.as_slice())
      .map_err(|err| format!("failed to update message: {err}"))?;
  }

  txn
    .commit()
    .map_err(|err| format!("failed to commit message status: {err}"))?;

  let payload = ChatMessageStatusPayload {
    workspace_id: workspace_id.to_string(),
    conversation_id: conversation_id.to_string(),
    message_id: message_id.to_string(),
    status,
  };
  let _ = app.emit("chat-message-status", payload);
  Ok(())
}

/// 终端语义回写的消息追加逻辑。
/// 返回：持久化后的消息 id。
pub(crate) fn chat_append_terminal_message(
  app: &AppHandle,
  state: &ChatDbManager,
  workspace_id: &str,
  conversation_id: &str,
  member_id: &str,
  content: String,
  viewer_id: &str,
  span_id: Option<&str>,
) -> Result<String, String> {
  let content_for_log = content.clone();
  let conv_id = parse_ulid(conversation_id)?;
  let sender_id = parse_ulid(member_id)?;
  let viewer_id = parse_ulid(viewer_id)?;
  let db = open_db(state, workspace_id)?;
  let message = save_message_in_db(
    &db,
    conv_id,
    Some(sender_id),
    MessageContent::Text { text: content },
    false,
    MessageStatus::Sent,
    None,
  )?;
  let message_id = message.id.clone();
  diagnostics_log_backend_event(
    &app.state::<DiagnosticsState>(),
    Some(sender_id.to_string()),
    None,
    Some(conversation_id.to_string()),
    None,
    Some(workspace_id.to_string()),
    "chat_append_terminal_message",
    json!({
      "workspaceId": workspace_id,
      "conversationId": conversation_id,
      "senderId": sender_id.to_string(),
      "viewerId": viewer_id.to_string(),
      "messageId": message.id.clone(),
      "content": content_for_log
    }),
  );
  let total_unread_count = compute_total_unread_count(&db, viewer_id)?;
  let payload = ChatMessageCreatedPayload {
    workspace_id: workspace_id.to_string(),
    conversation_id: conversation_id.to_string(),
    message,
    total_unread_count,
    span_id: span_id.map(|value| value.to_string()),
  };
  let _ = app.emit("chat-message-created", payload);
  emit_unread_sync(app, workspace_id, viewer_id, &db, Some(conv_id), false)?;
  diagnostics_log_backend_event(
    &app.state::<DiagnosticsState>(),
    Some(sender_id.to_string()),
    None,
    Some(conversation_id.to_string()),
    None,
    Some(workspace_id.to_string()),
    "chat_message_created",
    json!({
      "workspaceId": workspace_id,
      "conversationId": conversation_id,
      "senderId": sender_id.to_string(),
      "viewerId": viewer_id.to_string(),
      "totalUnreadCount": total_unread_count
    }),
  );
  Ok(message_id)
}

/// 创建群聊并返回摘要信息。
/// 输入：`member_ids` 至少包含 2 人；`custom_name` 可选。
/// 返回：新会话摘要。
/// 错误：ID 解析失败、成员不足或数据库写入失败。
pub fn chat_create_group(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  member_ids: Vec<String>,
  custom_name: Option<String>,
) -> Result<ConversationSummaryDto, String> {
  let user_id = parse_ulid(&user_id)?;
  let mut members = Vec::new();
  for id in member_ids {
    members.push(parse_ulid(&id)?);
  }
  if !members.contains(&user_id) {
    members.push(user_id);
  }
  members.sort_unstable();
  members.dedup();
  if members.len() < 2 {
    return Err("group requires at least 2 members".to_string());
  }

  let db = open_db(&state, &workspace_id)?;
  let created_at = now_millis()?;
  let conv_id = Ulid::new().0;
  let trimmed_name = custom_name.and_then(|value| {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
      None
    } else {
      Some(trimmed)
    }
  });
  let meta = ConversationMeta {
    kind: ConversationKind::Channel,
    created_at,
    custom_name: trimmed_name,
    is_default: false,
    last_message_at: None,
    last_message_preview: None,
  };

  let mut txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  {
    let mut table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let payload = encode(&meta)?;
    table
      .insert(conv_id, payload.as_slice())
      .map_err(|err| format!("failed to store conversation: {err}"))?;
  }
  sync_conversation_members(&mut txn, conv_id, &members, created_at)?;
  txn
    .commit()
    .map_err(|err| format!("failed to commit group creation: {err}"))?;

  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
  let settings = {
    let table = read_txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    let existing = table
      .get((user_id, conv_id))
      .map_err(|err| format!("failed to read user_convs: {err}"))?;
    match existing {
      Some(value) => decode(value.value())?,
      None => UserConversationSettings::default(),
    }
  };
  let member_ids = {
    let table = read_txn
      .open_table(MEMBERS)
      .map_err(|err| format!("failed to open members table: {err}"))?;
    load_member_ids_from_table(&table, conv_id)?
  };

  Ok(build_conversation_summary(
    conv_id,
    &meta,
    &settings,
    member_ids,
    user_id,
    0,
  ))
}

/// 确保与目标用户的私聊会话存在（不存在则创建）。
/// 返回：会话摘要。
/// 错误：ID 解析失败或数据库写入失败。
pub fn chat_ensure_direct(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  target_id: String,
) -> Result<ConversationSummaryDto, String> {
  let workspace_id_for_log = workspace_id.clone();
  let user_id_for_log = user_id.clone();
  let target_id_for_log = target_id.clone();
  let user_id = parse_ulid(&user_id)?;
  let target_id = parse_ulid(&target_id)?;

  let db = open_db(&state, &workspace_id)?;
  let mut existing: Option<u128> = None;

  {
    let read_txn = db
      .begin_read()
      .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
    let table = read_txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    let start = (user_id, 0);
    let end = (user_id, u128::MAX);
    let conv_ids: Vec<u128> = table
      .range(start..=end)
      .map_err(|err| format!("failed to scan user_convs: {err}"))?
      .filter_map(|entry| {
        entry.ok().map(|(key, _)| {
          let (_, conv_id) = key.value();
          conv_id
        })
      })
      .collect();

    for conv_id in conv_ids {
      let meta: ConversationMeta = {
        let table = read_txn
          .open_table(CONVERSATIONS)
          .map_err(|err| format!("failed to open conversations table: {err}"))?;
        let existing = table
          .get(conv_id)
          .map_err(|err| format!("failed to read conversation: {err}"))?;
        let meta = match existing {
          Some(value) => decode(value.value())?,
          None => continue,
        };
        meta
      };
      if meta.kind != ConversationKind::Dm {
        continue;
      }
      let member_ids = {
        let table = read_txn
          .open_table(MEMBERS)
          .map_err(|err| format!("failed to open members table: {err}"))?;
        load_member_ids_from_table(&table, conv_id)?
      };
      if member_ids.contains(&user_id) && member_ids.contains(&target_id) {
        existing = Some(conv_id);
        break;
      }
    }
  }

  let conv_id = if let Some(conv_id) = existing {
    conv_id
  } else {
    let now = now_millis()?;
    let conv_id = Ulid::new().0;
    let meta = ConversationMeta {
      kind: ConversationKind::Dm,
      created_at: now,
      custom_name: None,
      is_default: false,
      last_message_at: None,
      last_message_preview: None,
    };
    let mut txn = db
      .begin_write()
      .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
    {
      let mut table = txn
        .open_table(CONVERSATIONS)
        .map_err(|err| format!("failed to open conversations table: {err}"))?;
      let payload = encode(&meta)?;
      table
        .insert(conv_id, payload.as_slice())
        .map_err(|err| format!("failed to store conversation: {err}"))?;
    }
    let members = vec![user_id, target_id];
    sync_conversation_members(&mut txn, conv_id, &members, now)?;
    txn
      .commit()
      .map_err(|err| format!("failed to commit dm creation: {err}"))?;
    conv_id
  };

  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
  let meta: ConversationMeta = {
    let table = read_txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let existing = table
      .get(conv_id)
      .map_err(|err| format!("failed to read conversation: {err}"))?;
    let meta = match existing {
      Some(value) => decode(value.value())?,
      None => return Err("conversation not found".to_string()),
    };
    meta
  };
  let member_ids = {
    let table = read_txn
      .open_table(MEMBERS)
      .map_err(|err| format!("failed to open members table: {err}"))?;
    load_member_ids_from_table(&table, conv_id)?
  };
  let settings = {
    let table = read_txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    match table
      .get((user_id, conv_id))
      .map_err(|err| format!("failed to read user_convs: {err}"))?
    {
      Some(value) => decode(value.value())?,
      None => UserConversationSettings::default(),
    }
  };

  let message_table = read_txn
    .open_table(MESSAGES)
    .map_err(|err| format!("failed to open messages table: {err}"))?;
  let unread_count = count_unread_messages(&message_table, conv_id, settings.last_read_message_id)?;

  let summary = build_conversation_summary(
    conv_id,
    &meta,
    &settings,
    member_ids,
    user_id,
    unread_count,
  );
  diagnostics_log_backend_event(
    &app.state::<DiagnosticsState>(),
    Some(user_id_for_log.clone()),
    None,
    Some(summary.id.clone()),
    None,
    Some(workspace_id_for_log),
    "chat_ensure_direct",
    json!({
      "conversationId": summary.id.clone(),
      "userId": user_id_for_log,
      "targetId": target_id_for_log,
      "unreadCount": unread_count
    }),
  );
  Ok(summary)
}

/// 更新用户的会话设置（置顶/静音）。
/// 错误：ID 解析失败或数据库写入失败。
pub fn chat_set_conversation_settings(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  conversation_id: String,
  pinned: Option<bool>,
  muted: Option<bool>,
) -> Result<(), String> {
  let user_id = parse_ulid(&user_id)?;
  let conv_id = parse_ulid(&conversation_id)?;
  let db = open_db(&state, &workspace_id)?;

  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  let mut settings = {
    let table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    let existing = table
      .get((user_id, conv_id))
      .map_err(|err| format!("failed to read user_convs: {err}"))?;
    let settings = match existing {
      Some(value) => decode(value.value())?,
      None => UserConversationSettings::default(),
    };
    settings
  };
  if let Some(pinned) = pinned {
    settings.pinned = pinned;
  }
  if let Some(muted) = muted {
    settings.muted = muted;
  }
  let payload = encode(&settings)?;
  {
    let mut table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    table
      .insert((user_id, conv_id), payload.as_slice())
      .map_err(|err| format!("failed to update user_convs: {err}"))?;
  }
  txn
    .commit()
    .map_err(|err| format!("failed to commit user settings: {err}"))?;
  Ok(())
}

/// 重命名会话。
/// 约束：空白名称会被视为清空自定义名称。
/// 错误：ID 解析失败或数据库写入失败。
pub fn chat_rename_conversation(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  custom_name: Option<String>,
) -> Result<(), String> {
  let conv_id = parse_ulid(&conversation_id)?;
  let db = open_db(&state, &workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  let meta: ConversationMeta = {
    let table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let existing = table
      .get(conv_id)
      .map_err(|err| format!("failed to read conversation: {err}"))?;
    let meta = match existing {
      Some(value) => decode(value.value())?,
      None => return Err("conversation not found".to_string()),
    };
    meta
  };

  let trimmed = custom_name.and_then(|value| {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
      None
    } else {
      Some(trimmed)
    }
  });

  {
    let mut table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let mut updated = meta;
    updated.custom_name = trimmed;
    let payload = encode(&updated)?;
    table
      .insert(conv_id, payload.as_slice())
      .map_err(|err| format!("failed to update conversation: {err}"))?;
  }
  txn
    .commit()
    .map_err(|err| format!("failed to commit rename: {err}"))?;
  Ok(())
}

/// 清空会话消息但保留会话本身。
/// 错误：ID 解析失败或数据库写入失败。
pub fn chat_clear_conversation(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
) -> Result<(), String> {
  let conv_id = parse_ulid(&conversation_id)?;
  let db = open_db(&state, &workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;

  {
    let mut table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    let start = (conv_id, 0);
    let end = (conv_id, u128::MAX);
    let mut keys: Vec<(u128, u128)> = Vec::new();
    let mut latest_msg_id: Option<u128> = None;
    for entry in table
      .range(start..=end)
      .map_err(|err| format!("failed to scan messages: {err}"))?
    {
      let (key, _) = entry.map_err(|err| format!("failed to decode message entry: {err}"))?;
      let (conv_id, msg_id) = key.value();
      latest_msg_id = Some(msg_id);
      keys.push((conv_id, msg_id));
    }

    let member_ids = {
      let table = txn
        .open_table(MEMBERS)
        .map_err(|err| format!("failed to open members table: {err}"))?;
      load_member_ids_from_table(&table, conv_id)?
    };

    {
      let mut table = txn
        .open_table(USER_CONVS)
        .map_err(|err| format!("failed to open user_convs table: {err}"))?;
      for user_id in member_ids {
        let mut settings = {
          let existing = table
            .get((user_id, conv_id))
            .map_err(|err| format!("failed to read user_convs: {err}"))?;
          match existing {
            Some(value) => decode(value.value())?,
            None => UserConversationSettings::default(),
          }
        };
        settings.last_read_message_id = latest_msg_id;
        let payload = encode(&settings)?;
        table
          .insert((user_id, conv_id), payload.as_slice())
          .map_err(|err| format!("failed to update user_convs: {err}"))?;
      }
    }

    for key in keys {
      let _ = table.remove(key);
    }
  }

  {
    let mut table = txn
      .open_table(ATTACHMENTS_INDEX)
      .map_err(|err| format!("failed to open attachments_index table: {err}"))?;
    let start = (conv_id, 0, 0, 0);
    let end = (conv_id, u8::MAX, u64::MAX, u128::MAX);
    let keys: Vec<(u128, u8, u64, u128)> = table
      .range(start..=end)
      .map_err(|err| format!("failed to scan attachments: {err}"))?
      .filter_map(|entry| entry.ok().map(|(key, _)| key.value()))
      .collect();
    for key in keys {
      let _ = table.remove(key);
    }
  }

  let meta: ConversationMeta = {
    let table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let existing = table
      .get(conv_id)
      .map_err(|err| format!("failed to read conversation: {err}"))?;
    let meta = match existing {
      Some(value) => decode(value.value())?,
      None => return Ok(()),
    };
    meta
  };

  {
    let mut table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let mut updated = meta;
    updated.last_message_at = None;
    updated.last_message_preview = None;
    let payload = encode(&updated)?;
    table
      .insert(conv_id, payload.as_slice())
      .map_err(|err| format!("failed to update conversation: {err}"))?;
  }

  txn
    .commit()
    .map_err(|err| format!("failed to commit clear: {err}"))?;
  Ok(())
}

/// 删除会话及其所有相关数据（消息、附件、成员、时间线）。
/// 错误：ID 解析失败或数据库写入失败。
pub fn chat_delete_conversation(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
) -> Result<(), String> {
  let conv_id = parse_ulid(&conversation_id)?;
  delete_conversation_by_id(state.inner(), &workspace_id, conv_id)
}

pub fn chat_delete_member_conversations(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  member_ids: Vec<String>,
) -> Result<ChatDeleteMemberConversationsResult, String> {
  let mut warnings = Vec::new();
  if member_ids.is_empty() {
    return Ok(ChatDeleteMemberConversationsResult {
      removed_conversations: 0,
      warnings,
    });
  }
  let db = open_db(state.inner(), &workspace_id)?;
  let mut targets = HashSet::new();
  {
    let read_txn = db
      .begin_read()
      .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
    let user_table = read_txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    let conv_table = read_txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    for member_id in &member_ids {
      let member_id = member_id.trim();
      if member_id.is_empty() {
        continue;
      }
      let user_id = match parse_ulid(member_id) {
        Ok(user_id) => user_id,
        Err(err) => {
          warnings.push(format!("invalid member id {member_id}: {err}"));
          continue;
        }
      };
      let start = (user_id, 0);
      let end = (user_id, u128::MAX);
      for entry in user_table
        .range(start..=end)
        .map_err(|err| format!("failed to scan user_convs: {err}"))?
      {
        let (key, _) = entry.map_err(|err| format!("failed to decode user_convs entry: {err}"))?;
        let (_, conv_id) = key.value();
        if targets.contains(&conv_id) {
          continue;
        }
        let meta: ConversationMeta = match conv_table
          .get(conv_id)
          .map_err(|err| format!("failed to read conversation: {err}"))?
        {
          Some(value) => decode(value.value())?,
          None => continue,
        };
        if meta.kind == ConversationKind::Dm {
          targets.insert(conv_id);
        }
      }
    }
  }

  let mut removed = 0u32;
  for conv_id in targets {
    if let Err(err) = delete_conversation_by_id(state.inner(), &workspace_id, conv_id) {
      warnings.push(format!(
        "conversation delete failed id={} err={}",
        format_ulid(conv_id),
        err
      ));
    } else {
      removed = removed.saturating_add(1);
    }
  }

  Ok(ChatDeleteMemberConversationsResult {
    removed_conversations: removed,
    warnings,
  })
}

fn delete_conversation_by_id(
  state: &ChatDbManager,
  workspace_id: &str,
  conv_id: u128,
) -> Result<(), String> {
  let db = open_db(state, workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;

  let member_ids = {
    let table = txn
      .open_table(MEMBERS)
      .map_err(|err| format!("failed to open members table: {err}"))?;
    load_member_ids_from_table(&table, conv_id)?
  };

  for user_id in member_ids.iter().copied() {
    let last_active = {
      let table = txn
        .open_table(USER_CONVS)
        .map_err(|err| format!("failed to open user_convs table: {err}"))?;
      let existing = table
        .get((user_id, conv_id))
        .map_err(|err| format!("failed to read user_convs: {err}"))?;
      let last_active = match existing {
        Some(value) => {
          let settings: UserConversationSettings = decode(value.value())?;
          settings.last_active_at
        }
        None => None,
      };
      last_active
    };

    if let Some(last_active_at) = last_active {
      let mut table = txn
        .open_table(TIMELINE_INDEX)
        .map_err(|err| format!("failed to open timeline_index table: {err}"))?;
      let _ = table.remove((user_id, ts_rev(last_active_at), conv_id));
    }

    {
      let mut table = txn
        .open_table(USER_CONVS)
        .map_err(|err| format!("failed to open user_convs table: {err}"))?;
      let _ = table.remove((user_id, conv_id));
    }

    {
      let mut table = txn
        .open_table(MEMBERS)
        .map_err(|err| format!("failed to open members table: {err}"))?;
      let _ = table.remove((conv_id, user_id));
    }
  }

  {
    let mut table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    let start = (conv_id, 0);
    let end = (conv_id, u128::MAX);
    let keys: Vec<(u128, u128)> = table
      .range(start..=end)
      .map_err(|err| format!("failed to scan messages: {err}"))?
      .filter_map(|entry| entry.ok().map(|(key, _)| key.value()))
      .collect();
    for key in keys {
      let _ = table.remove(key);
    }
  }

  {
    let mut table = txn
      .open_table(ATTACHMENTS_INDEX)
      .map_err(|err| format!("failed to open attachments_index table: {err}"))?;
    let start = (conv_id, 0, 0, 0);
    let end = (conv_id, u8::MAX, u64::MAX, u128::MAX);
    let keys: Vec<(u128, u8, u64, u128)> = table
      .range(start..=end)
      .map_err(|err| format!("failed to scan attachments: {err}"))?
      .filter_map(|entry| entry.ok().map(|(key, _)| key.value()))
      .collect();
    for key in keys {
      let _ = table.remove(key);
    }
  }

  {
    let mut table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let _ = table.remove(conv_id);
  }

  txn
    .commit()
    .map_err(|err| format!("failed to commit delete: {err}"))?;
  Ok(())
}

/// 同步会话成员列表（以输入为准）。
/// 约束：空列表直接返回，不做任何修改。
/// 错误：ID 解析失败或数据库写入失败。
pub fn chat_set_conversation_members(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  member_ids: Vec<String>,
) -> Result<(), String> {
  let workspace_id_for_log = workspace_id.clone();
  let conversation_id_for_log = conversation_id.clone();
  let member_ids_for_log = member_ids.clone();
  let conv_id = parse_ulid(&conversation_id)?;
  let mut member_ids_u128 = Vec::new();
  for id in member_ids {
    member_ids_u128.push(parse_ulid(&id)?);
  }
  if member_ids_u128.is_empty() {
    return Ok(());
  }
  let db = open_db(&state, &workspace_id)?;
  let created_at = now_millis()?;
  let mut txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  sync_conversation_members(&mut txn, conv_id, &member_ids_u128, created_at)?;
  txn
    .commit()
    .map_err(|err| format!("failed to commit member sync: {err}"))?;
  diagnostics_log_backend_event(
    &app.state::<DiagnosticsState>(),
    None,
    None,
    Some(conversation_id_for_log.clone()),
    None,
    Some(workspace_id_for_log),
    "chat_set_conversation_members",
    json!({
      "conversationId": conversation_id_for_log,
      "memberIds": member_ids_for_log
    }),
  );
  Ok(())
}
