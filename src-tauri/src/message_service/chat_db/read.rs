//! 聊天读取子域：负责列表/消息读取与未读汇总。

use std::collections::{HashMap, HashSet};

use tauri::State;

use super::store::{
  build_conversation_summary, count_unread_messages, decode, ensure_default_channel, format_ulid,
  load_member_ids_from_table, open_db, parse_ulid, CONVERSATIONS, MEMBERS, MESSAGES,
  TIMELINE_INDEX, USER_CONVS,
};
use super::types::{ChatHomeFeedDto, ConversationMeta, ConvId, UserConversationSettings, WorkspaceUnreadSummary};
use super::{ChatDbManager};

/// 获取会话列表与时间线汇总。
/// 输入：`user_id` 为当前用户；`workspace_name` 用于默认频道命名。
/// 返回：首页汇总与默认频道 ID。
/// 错误：ID 解析失败或数据库不可用。
pub fn chat_list_conversations(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  workspace_name: Option<String>,
  member_ids: Vec<String>,
) -> Result<ChatHomeFeedDto, String> {
  let user_id = parse_ulid(&user_id)?;
  let mut member_ids_u128 = Vec::new();
  for id in member_ids {
    member_ids_u128.push(parse_ulid(&id)?);
  }
  if !member_ids_u128.contains(&user_id) {
    member_ids_u128.push(user_id);
  }

  let db = open_db(&state, &workspace_id)?;

  let (default_channel_id, default_meta) = {
    let mut txn = db
      .begin_write()
      .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
    let (conv_id, meta) = ensure_default_channel(&mut txn, workspace_name.as_deref(), &member_ids_u128)?;
    txn
      .commit()
      .map_err(|err| format!("failed to commit chat bootstrap: {err}"))?;
    (conv_id, meta)
  };

  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
  let message_table = read_txn
    .open_table(MESSAGES)
    .map_err(|err| format!("failed to open messages table: {err}"))?;

  let mut settings_map = HashMap::new();
  {
    let table = read_txn
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
      settings_map.insert(conv_id, settings);
    }
  }
  let mut unread_map = HashMap::new();
  let mut total_unread_count = 0usize;
  for (conv_id, settings) in settings_map.iter() {
    let unread_count = count_unread_messages(&message_table, *conv_id, settings.last_read_message_id)?;
    unread_map.insert(*conv_id, unread_count);
    total_unread_count = total_unread_count.saturating_add(unread_count);
  }

  let mut pinned_ids = HashSet::new();
  let mut pinned = Vec::new();
  for (conv_id, settings) in settings_map.iter().filter(|(_, settings)| settings.pinned) {
    let meta: ConversationMeta = {
      let table = read_txn
        .open_table(CONVERSATIONS)
        .map_err(|err| format!("failed to open conversations table: {err}"))?;
      let existing = table
        .get(*conv_id)
        .map_err(|err| format!("failed to read conversation: {err}"))?;
      let meta = match existing {
        Some(value) => decode(value.value())?,
        None => continue,
      };
      meta
    };
    let member_ids = {
      let table = read_txn
        .open_table(MEMBERS)
        .map_err(|err| format!("failed to open members table: {err}"))?;
      load_member_ids_from_table(&table, *conv_id)?
    };
    pinned_ids.insert(*conv_id);
    let unread_count = *unread_map.get(conv_id).unwrap_or(&0);
    pinned.push(build_conversation_summary(
      *conv_id,
      &meta,
      settings,
      member_ids,
      user_id,
      unread_count,
    ));
  }

  let mut timeline = Vec::new();
  let mut timeline_ids: HashSet<ConvId> = HashSet::new();
  {
    let table = read_txn
      .open_table(TIMELINE_INDEX)
      .map_err(|err| format!("failed to open timeline_index table: {err}"))?;
    let start = (user_id, 0, 0);
    let end = (user_id, u64::MAX, u128::MAX);
    for entry in table
      .range(start..=end)
      .map_err(|err| format!("failed to scan timeline_index: {err}"))?
    {
      let (key, _) = entry.map_err(|err| format!("failed to decode timeline entry: {err}"))?;
      let (_, _, conv_id) = key.value();
      if pinned_ids.contains(&conv_id) {
        continue;
      }
      let settings = match settings_map.get(&conv_id) {
        Some(value) => value,
        None => continue,
      };
      let meta: ConversationMeta = {
        let table = read_txn
          .open_table(CONVERSATIONS)
          .map_err(|err| format!("failed to open conversations table: {err}"))?;
        match table
          .get(conv_id)
          .map_err(|err| format!("failed to read conversation: {err}"))?
        {
          Some(value) => decode(value.value())?,
          None => continue,
        }
      };
      let member_ids = {
        let table = read_txn
          .open_table(MEMBERS)
          .map_err(|err| format!("failed to open members table: {err}"))?;
        load_member_ids_from_table(&table, conv_id)?
      };
      let unread_count = *unread_map.get(&conv_id).unwrap_or(&0);
      timeline.push(build_conversation_summary(
        conv_id,
        &meta,
        settings,
        member_ids,
        user_id,
        unread_count,
      ));
      timeline_ids.insert(conv_id);
    }
  }

  if timeline.is_empty() {
    let mut fallback: Vec<(u64, ConvId, super::ConversationSummaryDto)> = Vec::new();
    for (conv_id, settings) in settings_map.iter() {
      if pinned_ids.contains(conv_id) {
        continue;
      }
      let meta: ConversationMeta = {
        let table = read_txn
          .open_table(CONVERSATIONS)
          .map_err(|err| format!("failed to open conversations table: {err}"))?;
        match table
          .get(*conv_id)
          .map_err(|err| format!("failed to read conversation: {err}"))?
        {
          Some(value) => decode(value.value())?,
          None => continue,
        }
      };
      let member_ids = {
        let table = read_txn
          .open_table(MEMBERS)
          .map_err(|err| format!("failed to open members table: {err}"))?;
        load_member_ids_from_table(&table, *conv_id)?
      };
      let last_active = settings.last_active_at.unwrap_or(meta.created_at);
      let unread_count = *unread_map.get(conv_id).unwrap_or(&0);
      fallback.push((
        last_active,
        *conv_id,
        build_conversation_summary(*conv_id, &meta, settings, member_ids, user_id, unread_count),
      ));
    }
    fallback.sort_by(|(a, _, _), (b, _, _)| b.cmp(a));
    for (_, conv_id, summary) in fallback {
      timeline_ids.insert(conv_id);
      timeline.push(summary);
    }
  }

  if !timeline_ids.contains(&default_channel_id) && !pinned_ids.contains(&default_channel_id) {
    if let Some(settings) = settings_map.get(&default_channel_id) {
      let meta: ConversationMeta = {
        let table = read_txn
          .open_table(CONVERSATIONS)
          .map_err(|err| format!("failed to open conversations table: {err}"))?;
        match table
          .get(default_channel_id)
          .map_err(|err| format!("failed to read conversation: {err}"))?
        {
          Some(value) => decode(value.value())?,
          None => default_meta.clone(),
        }
      };
      let member_ids = {
        let table = read_txn
          .open_table(MEMBERS)
          .map_err(|err| format!("failed to open members table: {err}"))?;
        load_member_ids_from_table(&table, default_channel_id)?
      };
      let unread_count = *unread_map.get(&default_channel_id).unwrap_or(&0);
      timeline.push(build_conversation_summary(
        default_channel_id,
        &meta,
        settings,
        member_ids,
        user_id,
        unread_count,
      ));
    }
  }

  Ok(ChatHomeFeedDto {
    pinned,
    timeline,
    default_channel_id: Some(format_ulid(default_channel_id)),
    total_unread_count,
  })
}

/// 获取会话消息列表。
/// 输入：`limit` 控制返回数量，`before_id` 用于分页向前拉取。
/// 返回：按时间倒序的消息列表。
/// 错误：ID 解析失败或数据库不可用。
pub fn chat_get_messages(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  limit: Option<u32>,
  before_id: Option<String>,
) -> Result<Vec<super::MessageDto>, String> {
  let conv_id = parse_ulid(&conversation_id)?;
  let before_id = match before_id {
    Some(value) => Some(parse_ulid(&value)?),
    None => None,
  };
  let db = open_db(&state, &workspace_id)?;

  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
  let table = read_txn
    .open_table(MESSAGES)
    .map_err(|err| format!("failed to open messages table: {err}"))?;
  let start = (conv_id, 0);
  let end = (conv_id, before_id.unwrap_or(u128::MAX));
  let mut messages = Vec::new();
  let mut count = 0u32;
  for entry in table
    .range(start..=end)
    .map_err(|err| format!("failed to scan messages: {err}"))?
    .rev()
  {
    let (key, value) = entry.map_err(|err| format!("failed to decode message entry: {err}"))?;
    let (_, msg_id) = key.value();
    let message: super::types::ChatMessage = decode(value.value())?;
    messages.push(super::MessageDto {
      id: format_ulid(msg_id),
      sender_id: message.sender_id.map(format_ulid),
      content: super::MessageContent::from(message.content),
      created_at: message.created_at,
      is_ai: message.is_ai,
      status: message.status,
      attachment: message.attachment.map(super::MessageAttachment::from),
    });
    count += 1;
    if let Some(limit) = limit {
      if count >= limit {
        break;
      }
    }
  }
  Ok(messages)
}

/// 获取会话成员 ID 列表。
/// 输入：workspaceId、conversationId。
/// 输出：成员 id 列表（字符串）。
pub fn chat_get_conversation_member_ids(
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
) -> Result<Vec<String>, String> {
  let conv_id = parse_ulid(&conversation_id)?;
  let db = open_db(&state, &workspace_id)?;
  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
  let table = read_txn
    .open_table(MEMBERS)
    .map_err(|err| format!("failed to open members table: {err}"))?;
  let member_ids = load_member_ids_from_table(&table, conv_id)?;
  Ok(member_ids.into_iter().map(format_ulid).collect())
}

/// 读取指定工作区未读概览，给窗口列表与托盘使用。
pub(crate) fn compute_workspace_unread_summary(
  state: &ChatDbManager,
  workspace_id: &str,
  user_id: &str,
) -> Result<WorkspaceUnreadSummary, String> {
  super::store::compute_workspace_unread_summary(state, workspace_id, user_id)
}
