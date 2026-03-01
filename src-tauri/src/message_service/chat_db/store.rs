//! 聊天存储子域：负责表结构、序列化与数据库打开/维护。

use std::{
  collections::{HashMap, HashSet},
  fs,
  path::{Component, PathBuf},
  sync::{Arc, Mutex},
  time::{SystemTime, UNIX_EPOCH},
};

use redb::{Database, ReadableTable, TableDefinition};
use serde::{de::DeserializeOwned, Serialize};
use ulid::Ulid;

use super::types::{
  AttachmentIndexMeta, ChatClearResult, ChatMessage, ConversationKind, ConversationMeta, ConvId,
  MemberEntry, MessageAttachment, MessageAttachmentDb, MessageContent, MessageContentDb,
  MessageDto, MessageStatus, MsgId, TsRev, UserConversationSettings, UserId, WorkspaceUnreadSummary,
};

// 表结构与键：表名稳定，避免升级导致迁移困难。
pub(super) const USERS: TableDefinition<UserId, &[u8]> = TableDefinition::new("users");
pub(super) const CONVERSATIONS: TableDefinition<ConvId, &[u8]> = TableDefinition::new("conversations");
pub(super) const USER_CONVS: TableDefinition<(UserId, ConvId), &[u8]> = TableDefinition::new("user_convs");
pub(super) const TIMELINE_INDEX: TableDefinition<(UserId, TsRev, ConvId), ()> =
  TableDefinition::new("timeline_index");
pub(super) const MESSAGES: TableDefinition<(ConvId, MsgId), &[u8]> = TableDefinition::new("messages");
pub(super) const ATTACHMENTS_INDEX: TableDefinition<(ConvId, u8, TsRev, MsgId), &[u8]> =
  TableDefinition::new("attachments_index");
pub(super) const MEMBERS: TableDefinition<(ConvId, UserId), &[u8]> = TableDefinition::new("members");
pub(super) const TERMINAL_SESSION_MAP: TableDefinition<&str, &[u8]> =
  TableDefinition::new("terminal_session_map");
pub(super) const TERMINAL_SESSION_INDEX: TableDefinition<&str, &[u8]> =
  TableDefinition::new("terminal_session_index");
pub(super) const CHAT_OUTBOX_TASKS: TableDefinition<MsgId, &[u8]> =
  TableDefinition::new("chat_outbox_tasks");
pub(super) const CHAT_OUTBOX_SCHEDULE: TableDefinition<(u64, MsgId), ()> =
  TableDefinition::new("chat_outbox_schedule");
const CHAT_DB_FILE: &str = "chat.redb";

/// 聊天数据库管理器：按 workspace 缓存 DB，并确保修复仅执行一次。
pub struct ChatDbManager {
  dbs: Mutex<HashMap<String, Arc<Database>>>,
  repaired: Mutex<HashSet<String>>,
  base_dir: Mutex<Option<PathBuf>>,
}

impl Default for ChatDbManager {
  fn default() -> Self {
    Self {
      dbs: Mutex::new(HashMap::new()),
      repaired: Mutex::new(HashSet::new()),
      base_dir: Mutex::new(None),
    }
  }
}

impl ChatDbManager {
  /// 注入聊天数据库根目录，避免存储层依赖 AppHandle。
  pub(crate) fn set_base_dir(&self, base_dir: PathBuf) {
    if let Ok(mut guard) = self.base_dir.lock() {
      *guard = Some(base_dir);
    } else {
      log::warn!("chat db base dir lock poisoned");
    }
  }

  fn base_dir(&self) -> Result<PathBuf, String> {
    let guard = self
      .base_dir
      .lock()
      .map_err(|_| "chat db base dir lock poisoned".to_string())?;
    guard
      .clone()
      .ok_or_else(|| "chat db base dir not configured".to_string())
  }
}

pub(super) fn now_millis() -> Result<u64, String> {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|err| format!("failed to read system time: {err}"))
    .map(|value| value.as_millis() as u64)
}

// 逆序时间戳用于把“最新”排在范围扫描前面。
pub(super) fn ts_rev(timestamp: u64) -> u64 {
  u64::MAX.saturating_sub(timestamp)
}

pub(super) fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
  // 统一编码，便于集中处理序列化错误信息。
  bincode::serialize(value).map_err(|err| format!("failed to encode value: {err}"))
}

pub(super) fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
  // 统一解码，便于集中处理反序列化错误信息。
  bincode::deserialize(bytes).map_err(|err| format!("failed to decode value: {err}"))
}

pub(super) fn parse_ulid(value: &str) -> Result<u128, String> {
  // 前端与数据库之间的 ID 统一使用 ULID 字符串。
  let parsed = Ulid::from_string(value.trim()).map_err(|err| format!("invalid ULID: {err}"))?;
  Ok(parsed.0)
}

pub(super) fn format_ulid(value: u128) -> String {
  Ulid(value).to_string()
}

fn sanitize_workspace_id(workspace_id: &str) -> Result<&str, String> {
  // workspace_id 仅允许单段路径，避免路径穿越或绝对路径写入。
  let trimmed = workspace_id.trim();
  if trimmed.is_empty() {
    return Err("workspace_id is required".to_string());
  }
  let path = PathBuf::from(trimmed);
  let mut components = path.components();
  let Some(first) = components.next() else {
    return Err("workspace_id is required".to_string());
  };
  if components.next().is_some() || !matches!(first, Component::Normal(_)) {
    return Err("workspace_id contains invalid path segments".to_string());
  }
  Ok(trimmed)
}

fn db_path(state: &ChatDbManager, workspace_id: &str) -> Result<PathBuf, String> {
  let workspace_id = sanitize_workspace_id(workspace_id)?;
  let base_dir = state.base_dir()?;
  Ok(base_dir.join(workspace_id).join(CHAT_DB_FILE))
}

fn ensure_tables(db: &Database) -> Result<(), String> {
  // 在首次打开时创建所有表，避免运行时出现缺表错误。
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  {
    let _ = txn
      .open_table(USERS)
      .map_err(|err| format!("failed to open users table: {err}"))?;
    let _ = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let _ = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    let _ = txn
      .open_table(TIMELINE_INDEX)
      .map_err(|err| format!("failed to open timeline_index table: {err}"))?;
    let _ = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    let _ = txn
      .open_table(ATTACHMENTS_INDEX)
      .map_err(|err| format!("failed to open attachments_index table: {err}"))?;
    let _ = txn
      .open_table(MEMBERS)
      .map_err(|err| format!("failed to open members table: {err}"))?;
    let _ = txn
      .open_table(TERMINAL_SESSION_MAP)
      .map_err(|err| format!("failed to open terminal_session_map: {err}"))?;
    let _ = txn
      .open_table(TERMINAL_SESSION_INDEX)
      .map_err(|err| format!("failed to open terminal_session_index: {err}"))?;
    let _ = txn
      .open_table(CHAT_OUTBOX_TASKS)
      .map_err(|err| format!("failed to open chat_outbox_tasks: {err}"))?;
    let _ = txn
      .open_table(CHAT_OUTBOX_SCHEDULE)
      .map_err(|err| format!("failed to open chat_outbox_schedule: {err}"))?;
  }
  txn
    .commit()
    .map_err(|err| format!("failed to commit chat db init: {err}"))?;
  Ok(())
}

pub(super) fn open_db(
  state: &ChatDbManager,
  workspace_id: &str,
) -> Result<Arc<Database>, String> {
  // 按 workspace 缓存 DB，降低反复打开的 IO 成本。
  let mut guard = state
    .dbs
    .lock()
    .map_err(|_| "chat db registry lock poisoned".to_string())?;
  if let Some(db) = guard.get(workspace_id) {
    let db = db.clone();
    drop(guard);
    maybe_repair_messages(state, workspace_id, &db)?;
    return Ok(db);
  }
  let path = db_path(state, workspace_id)?;
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|err| format!("failed to create chat data dir: {err}"))?;
  }
  let db = Database::create(path).map_err(|err| format!("failed to open chat database: {err}"))?;
  ensure_tables(&db)?;
  maybe_repair_messages(state, workspace_id, &db)?;
  let db = Arc::new(db);
  guard.insert(workspace_id.to_string(), db.clone());
  Ok(db)
}

/// 枚举已创建聊天库的工作区，用于 Outbox 扫描。
pub(crate) fn list_workspace_ids(state: &ChatDbManager) -> Result<Vec<String>, String> {
  let base_dir = state.base_dir()?;
  let entries = fs::read_dir(&base_dir)
    .map_err(|err| format!("failed to read chat base dir: {err}"))?;
  let mut ids = Vec::new();
  for entry in entries {
    let entry = entry.map_err(|err| format!("failed to read chat base entry: {err}"))?;
    let file_type = entry
      .file_type()
      .map_err(|err| format!("failed to read chat base entry type: {err}"))?;
    if !file_type.is_dir() {
      continue;
    }
    let path = entry.path();
    if !path.join(CHAT_DB_FILE).exists() {
      continue;
    }
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
      continue;
    };
    if name.trim().is_empty() {
      continue;
    }
    ids.push(name.to_string());
  }
  ids.sort();
  ids.dedup();
  Ok(ids)
}

fn resolve_target_id(member_ids: &[UserId], user_id: UserId) -> Option<UserId> {
  for id in member_ids {
    if *id != user_id {
      return Some(*id);
    }
  }
  None
}

pub(super) fn load_member_ids_from_table<T>(
  table: &T,
  conv_id: ConvId,
) -> Result<Vec<UserId>, String>
where
  T: ReadableTable<(ConvId, UserId), &'static [u8]>,
{
  let start = (conv_id, 0);
  let end = (conv_id, u128::MAX);
  let mut ids = Vec::new();
  for entry in table
    .range(start..=end)
    .map_err(|err| format!("failed to scan members: {err}"))?
  {
    let (key, _) = entry.map_err(|err| format!("failed to decode members entry: {err}"))?;
    let (_, user_id) = key.value();
    ids.push(user_id);
  }
  Ok(ids)
}

pub(super) fn build_conversation_summary(
  conv_id: ConvId,
  meta: &ConversationMeta,
  settings: &UserConversationSettings,
  member_ids: Vec<UserId>,
  user_id: UserId,
  unread_count: usize,
) -> super::types::ConversationSummaryDto {
  let target_id = if meta.kind == ConversationKind::Dm {
    resolve_target_id(&member_ids, user_id).map(format_ulid)
  } else {
    None
  };
  super::types::ConversationSummaryDto {
    id: format_ulid(conv_id),
    conversation_type: meta.kind.as_str().to_string(),
    member_ids: member_ids.into_iter().map(format_ulid).collect(),
    target_id,
    custom_name: meta.custom_name.clone(),
    pinned: settings.pinned,
    muted: settings.muted,
    last_message_at: meta.last_message_at,
    last_message_preview: meta.last_message_preview.clone(),
    is_default: meta.is_default,
    unread_count,
  }
}

pub(super) fn count_unread_messages<T>(
  table: &T,
  conv_id: ConvId,
  last_read_message_id: Option<MsgId>,
) -> Result<usize, String>
where
  T: ReadableTable<(ConvId, MsgId), &'static [u8]>,
{
  let start_msg_id = last_read_message_id.map(|value| value.saturating_add(1)).unwrap_or(0);
  let start = (conv_id, start_msg_id);
  let end = (conv_id, u128::MAX);
  let mut count = 0usize;
  for entry in table
    .range(start..=end)
    .map_err(|err| format!("failed to scan messages: {err}"))?
  {
    if entry.is_ok() {
      count += 1;
    }
  }
  Ok(count)
}

pub(super) fn compute_total_unread_count(db: &Database, user_id: UserId) -> Result<usize, String> {
  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
  let message_table = read_txn
    .open_table(MESSAGES)
    .map_err(|err| format!("failed to open messages table: {err}"))?;
  let settings_table = read_txn
    .open_table(USER_CONVS)
    .map_err(|err| format!("failed to open user_convs table: {err}"))?;
  let start = (user_id, 0);
  let end = (user_id, u128::MAX);
  let mut total = 0usize;
  for entry in settings_table
    .range(start..=end)
    .map_err(|err| format!("failed to scan user_convs: {err}"))?
  {
    let (key, value) = entry.map_err(|err| format!("failed to decode user_convs entry: {err}"))?;
    let (_, conv_id) = key.value();
    let settings: UserConversationSettings = decode(value.value())?;
    let unread_count = count_unread_messages(&message_table, conv_id, settings.last_read_message_id)?;
    total = total.saturating_add(unread_count);
  }
  Ok(total)
}

pub(super) fn compute_conversation_unread_count(
  db: &Database,
  user_id: UserId,
  conv_id: ConvId,
) -> Result<usize, String> {
  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
  let message_table = read_txn
    .open_table(MESSAGES)
    .map_err(|err| format!("failed to open messages table: {err}"))?;
  let settings_table = read_txn
    .open_table(USER_CONVS)
    .map_err(|err| format!("failed to open user_convs table: {err}"))?;
  let settings = settings_table
    .get((user_id, conv_id))
    .map_err(|err| format!("failed to read user_convs: {err}"))?
    .and_then(|value| decode(value.value()).ok());
  let last_read = settings.and_then(|value: UserConversationSettings| value.last_read_message_id);
  count_unread_messages(&message_table, conv_id, last_read)
}

pub(super) fn build_message_preview(message: &ChatMessage) -> String {
  // 预览文本用于列表展示，应避免过长导致 UI 计算开销。
  let raw = match &message.content {
    MessageContentDb::Text { text } => text.clone(),
    MessageContentDb::System { key, .. } => format!("[{key}]"),
  };
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return String::new();
  }
  const MAX_PREVIEW: usize = 120; // UI 预览最大长度，避免列表渲染过重。
  if trimmed.len() <= MAX_PREVIEW {
    return trimmed.to_string();
  }
  let mut out = trimmed.to_string();
  let mut end = MAX_PREVIEW.min(out.len());
  while end > 0 && !out.is_char_boundary(end) {
    end -= 1;
  }
  out.truncate(end);
  out.push_str("...");
  out
}

fn maybe_repair_messages(
  state: &ChatDbManager,
  workspace_id: &str,
  db: &Database,
) -> Result<(), String> {
  let mut guard = state
    .repaired
    .lock()
    .map_err(|_| "chat repair lock poisoned".to_string())?;
  if guard.contains(workspace_id) {
    return Ok(());
  }
  let removed = repair_invalid_messages(db)?;
  if removed > 0 {
    log::warn!("chat repair removed {} messages for workspace {}", removed, workspace_id);
  }
  guard.insert(workspace_id.to_string());
  Ok(())
}

pub(super) fn repair_invalid_messages(db: &Database) -> Result<usize, String> {
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  let mut to_remove = Vec::new();
  {
    let table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    for entry in table.iter().map_err(|err| format!("failed to scan messages: {err}"))? {
      let (key, value) = entry.map_err(|err| format!("failed to decode message entry: {err}"))?;
      if decode::<ChatMessage>(value.value()).is_err() {
        to_remove.push(key.value());
      }
    }
  }
  if !to_remove.is_empty() {
    let mut table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    for key in to_remove.iter().copied() {
      let _ = table.remove(key);
    }
  }
  txn
    .commit()
    .map_err(|err| format!("failed to commit repair: {err}"))?;
  Ok(to_remove.len())
}

pub(super) fn clear_chat_storage(db: &Database) -> Result<ChatClearResult, String> {
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  let removed_messages = {
    let mut table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    let mut count = 0usize;
    let keys: Vec<(ConvId, MsgId)> = table
      .iter()
      .map_err(|err| format!("failed to scan messages: {err}"))?
      .filter_map(|entry| entry.ok().map(|(key, _)| key.value()))
      .collect();
    for key in keys {
      count += 1;
      let _ = table.remove(key);
    }
    count
  };
  let removed_attachments = {
    let mut table = txn
      .open_table(ATTACHMENTS_INDEX)
      .map_err(|err| format!("failed to open attachments_index table: {err}"))?;
    let mut count = 0usize;
    let keys: Vec<(ConvId, u8, TsRev, MsgId)> = table
      .iter()
      .map_err(|err| format!("failed to scan attachments index: {err}"))?
      .filter_map(|entry| entry.ok().map(|(key, _)| key.value()))
      .collect();
    for key in keys {
      count += 1;
      let _ = table.remove(key);
    }
    count
  };
  let cleared_timeline = {
    let mut table = txn
      .open_table(TIMELINE_INDEX)
      .map_err(|err| format!("failed to open timeline_index table: {err}"))?;
    let mut count = 0usize;
    let keys: Vec<(UserId, TsRev, ConvId)> = table
      .iter()
      .map_err(|err| format!("failed to scan timeline index: {err}"))?
      .filter_map(|entry| entry.ok().map(|(key, _)| key.value()))
      .collect();
    for key in keys {
      count += 1;
      let _ = table.remove(key);
    }
    count
  };
  {
    let mut table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    let keys: Vec<(UserId, ConvId)> = table
      .iter()
      .map_err(|err| format!("failed to scan user_convs: {err}"))?
      .filter_map(|entry| entry.ok().map(|(key, _)| key.value()))
      .collect();
    for key in keys {
      let _ = table.remove(key);
    }
  }
  {
    let mut table = txn
      .open_table(MEMBERS)
      .map_err(|err| format!("failed to open members table: {err}"))?;
    let keys: Vec<(ConvId, UserId)> = table
      .iter()
      .map_err(|err| format!("failed to scan members: {err}"))?
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
    let keys: Vec<ConvId> = table
      .iter()
      .map_err(|err| format!("failed to scan conversations: {err}"))?
      .filter_map(|entry| entry.ok().map(|(key, _)| key.value()))
      .collect();
    for key in keys {
      let _ = table.remove(key);
    }
  }
  txn
    .commit()
    .map_err(|err| format!("failed to commit clear storage: {err}"))?;
  Ok(ChatClearResult {
    removed_messages,
    removed_attachments,
    cleared_timeline,
  })
}

pub(super) fn attachment_index_entry(
  attachment: &MessageAttachmentDb,
) -> Option<(u8, AttachmentIndexMeta)> {
  match attachment {
    MessageAttachmentDb::Image {
      file_path,
      file_name,
      file_size,
      mime_type,
      width,
      height,
      thumbnail_path,
    } => Some((
      1,
      AttachmentIndexMeta {
        file_path: file_path.clone(),
        file_name: file_name.clone(),
        file_size: *file_size,
        mime_type: mime_type.clone(),
        width: *width,
        height: *height,
        thumbnail_path: thumbnail_path.clone(),
      },
    )),
    MessageAttachmentDb::Roadmap { .. } => None,
  }
}

pub(super) fn ensure_default_channel(
  txn: &mut redb::WriteTransaction,
  workspace_name: Option<&str>,
  member_ids: &[UserId],
) -> Result<(ConvId, ConversationMeta), String> {
  let conv_id = Ulid::new().0;
  let table = txn
    .open_table(CONVERSATIONS)
    .map_err(|err| format!("failed to open conversations table: {err}"))?;
  let existing_default: Option<(ConvId, ConversationMeta)> = table
    .iter()
    .map_err(|err| format!("failed to scan conversations: {err}"))?
    .filter_map(|entry| {
      entry.ok().and_then(|(key, value)| {
        let conv_id = key.value();
        decode::<ConversationMeta>(value.value())
          .ok()
          .filter(|meta| meta.is_default)
          .map(|meta| (conv_id, meta))
      })
    })
    .next();
  if let Some((conv_id, meta)) = existing_default {
    drop(table);
    sync_conversation_members(txn, conv_id, member_ids, now_millis()?)?;
    return Ok((conv_id, meta));
  }

  let trimmed_name = workspace_name
    .map(|name| name.trim())
    .filter(|name| !name.is_empty())
    .map(|name| name.to_string());
  let meta = ConversationMeta {
    kind: ConversationKind::Channel,
    created_at: now_millis()?,
    custom_name: trimmed_name,
    is_default: true,
    last_message_at: None,
    last_message_preview: None,
  };
  let payload = encode(&meta)?;
  drop(table);
  {
    let mut table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    table
      .insert(conv_id, payload.as_slice())
      .map_err(|err| format!("failed to store conversation: {err}"))?;
  }
  sync_conversation_members(txn, conv_id, member_ids, meta.created_at)?;
  Ok((conv_id, meta))
}

pub(super) fn sync_conversation_members(
  txn: &mut redb::WriteTransaction,
  conv_id: ConvId,
  member_ids: &[UserId],
  now: u64,
) -> Result<(), String> {
  let mut new_members = HashSet::new();
  for member_id in member_ids {
    new_members.insert(*member_id);
  }

  let existing_members: HashSet<UserId> = {
    let table = txn
      .open_table(MEMBERS)
      .map_err(|err| format!("failed to open members table: {err}"))?;
    load_member_ids_from_table(&table, conv_id)?
      .into_iter()
      .collect()
  };

  let mut to_add = Vec::new();
  for member_id in new_members.iter() {
    if !existing_members.contains(member_id) {
      to_add.push(*member_id);
    }
  }
  let mut to_remove = Vec::new();
  for member_id in existing_members.iter() {
    if !new_members.contains(member_id) {
      to_remove.push(*member_id);
    }
  }

  if !to_remove.is_empty() {
    let mut table = txn
      .open_table(MEMBERS)
      .map_err(|err| format!("failed to open members table: {err}"))?;
    for member_id in to_remove {
      let _ = table.remove((conv_id, member_id));
    }
  }

  if !to_add.is_empty() {
    let payload = encode(&MemberEntry {
      joined_at: now,
      nickname: None,
    })?;
    let mut members_table = txn
      .open_table(MEMBERS)
      .map_err(|err| format!("failed to open members table: {err}"))?;
    for member_id in to_add {
      members_table
        .insert((conv_id, member_id), payload.as_slice())
        .map_err(|err| format!("failed to store members: {err}"))?;
    }
  }

  let mut settings_table = txn
    .open_table(USER_CONVS)
    .map_err(|err| format!("failed to open user_convs table: {err}"))?;
  for member_id in member_ids {
    if settings_table
      .get((*member_id, conv_id))
      .map_err(|err| format!("failed to read user_convs: {err}"))?
      .is_none()
    {
      let settings = UserConversationSettings::default();
      let payload = encode(&settings)?;
      settings_table
        .insert((*member_id, conv_id), payload.as_slice())
        .map_err(|err| format!("failed to store user_convs: {err}"))?;
    }
  }

  Ok(())
}

pub(super) fn save_message_in_db(
  db: &Database,
  conv_id: ConvId,
  sender_id: Option<UserId>,
  content: MessageContent,
  is_ai: bool,
  status: MessageStatus,
  attachment: Option<MessageAttachment>,
) -> Result<MessageDto, String> {
  let msg_id = Ulid::new().0;
  let created_at = now_millis()?;
  let content_db = MessageContentDb::from(content.clone());
  let attachment_db = attachment.clone().map(MessageAttachmentDb::from);
  let message = ChatMessage {
    sender_id,
    content: content_db,
    created_at,
    is_ai,
    status,
    attachment: attachment_db.clone(),
  };

  let preview = build_message_preview(&message);

  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;

  {
    let mut table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    let payload = encode(&message)?;
    table
      .insert((conv_id, msg_id), payload.as_slice())
      .map_err(|err| format!("failed to store message: {err}"))?;
  }

  let meta: ConversationMeta = {
    let table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let existing = table
      .get(conv_id)
      .map_err(|err| format!("failed to read conversation: {err}"))?;
    match existing {
      Some(value) => decode(value.value())?,
      None => return Err("conversation not found".to_string()),
    }
  };
  {
    let mut table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let mut updated = meta;
    updated.last_message_at = Some(created_at);
    updated.last_message_preview = Some(preview);
    let payload = encode(&updated)?;
    table
      .insert(conv_id, payload.as_slice())
      .map_err(|err| format!("failed to update conversation: {err}"))?;
  }

  if let Some(attachment_db) = attachment_db {
    if let Some((kind, meta)) = attachment_index_entry(&attachment_db) {
      let payload = encode(&meta)?;
      let mut table = txn
        .open_table(ATTACHMENTS_INDEX)
        .map_err(|err| format!("failed to open attachments index: {err}"))?;
      table
        .insert((conv_id, kind, ts_rev(created_at), msg_id), payload.as_slice())
        .map_err(|err| format!("failed to index attachment: {err}"))?;
    }
  }

  txn
    .commit()
    .map_err(|err| format!("failed to commit message: {err}"))?;

  Ok(MessageDto {
    id: format_ulid(msg_id),
    sender_id: sender_id.map(format_ulid),
    content,
    created_at,
    is_ai,
    status,
    attachment,
  })
}

pub(super) fn compute_workspace_unread_summary(
  state: &ChatDbManager,
  workspace_id: &str,
  user_id: &str,
) -> Result<WorkspaceUnreadSummary, String> {
  let user_id = parse_ulid(user_id)?;
  let path = db_path(state, workspace_id)?;
  if !path.exists() {
    return Ok(WorkspaceUnreadSummary {
      workspace_id: workspace_id.to_string(),
      total_unread_count: 0,
      latest_conversation_id: None,
      latest_conversation_type: None,
      latest_message_at: None,
      latest_unread_count: 0,
    });
  }

  let db = open_db(state, workspace_id)?;
  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
  let message_table = read_txn
    .open_table(MESSAGES)
    .map_err(|err| format!("failed to open messages table: {err}"))?;
  let settings_table = read_txn
    .open_table(USER_CONVS)
    .map_err(|err| format!("failed to open user_convs table: {err}"))?;
  let conv_table = read_txn
    .open_table(CONVERSATIONS)
    .map_err(|err| format!("failed to open conversations table: {err}"))?;

  let start = (user_id, 0);
  let end = (user_id, u128::MAX);
  let mut total = 0usize;
  let mut latest: Option<(ConvId, ConversationKind, Option<u64>, usize)> = None;

  for entry in settings_table
    .range(start..=end)
    .map_err(|err| format!("failed to scan user_convs: {err}"))?
  {
    let (key, value) = entry.map_err(|err| format!("failed to decode user_convs entry: {err}"))?;
    let (_, conv_id) = key.value();
    let settings: UserConversationSettings = decode(value.value())?;
    let unread_count = count_unread_messages(&message_table, conv_id, settings.last_read_message_id)?;
    if unread_count == 0 {
      continue;
    }
    total = total.saturating_add(unread_count);
    let meta: ConversationMeta = match conv_table
      .get(conv_id)
      .map_err(|err| format!("failed to read conversation: {err}"))?
    {
      Some(value) => decode(value.value())?,
      None => continue,
    };
    let last_at = meta.last_message_at;
    let current_ts = last_at.unwrap_or(0);
    let take = match latest {
      None => true,
      Some((_, _, prev_at, _)) => current_ts >= prev_at.unwrap_or(0),
    };
    if take {
      latest = Some((conv_id, meta.kind, last_at, unread_count));
    }
  }

  let (latest_conversation_id, latest_conversation_type, latest_message_at, latest_unread_count) =
    match latest {
      Some((conv_id, kind, last_at, count)) => (
        Some(format_ulid(conv_id)),
        Some(kind.as_str().to_string()),
        last_at,
        count,
      ),
      None => (None, None, None, 0),
    };

  Ok(WorkspaceUnreadSummary {
    workspace_id: workspace_id.to_string(),
    total_unread_count: total,
    latest_conversation_id,
    latest_conversation_type,
    latest_message_at,
    latest_unread_count,
  })
}
