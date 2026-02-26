use std::{
  collections::{HashMap, HashSet},
  fs,
  path::PathBuf,
  sync::{Arc, Mutex},
  time::{SystemTime, UNIX_EPOCH},
};

use redb::TableDefinition;
use redb::{Database, ReadableTable};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use ulid::Ulid;

use crate::resolve_app_data_path;

type UserId = u128;
type ConvId = u128;
type MsgId = u128;
type TsRev = u64;

const USERS: TableDefinition<UserId, &[u8]> = TableDefinition::new("users");
const CONVERSATIONS: TableDefinition<ConvId, &[u8]> = TableDefinition::new("conversations");
const USER_CONVS: TableDefinition<(UserId, ConvId), &[u8]> = TableDefinition::new("user_convs");
const TIMELINE_INDEX: TableDefinition<(UserId, TsRev, ConvId), ()> = TableDefinition::new("timeline_index");
const MESSAGES: TableDefinition<(ConvId, MsgId), &[u8]> = TableDefinition::new("messages");
const ATTACHMENTS_INDEX: TableDefinition<(ConvId, u8, TsRev, MsgId), &[u8]> =
  TableDefinition::new("attachments_index");
const MEMBERS: TableDefinition<(ConvId, UserId), &[u8]> = TableDefinition::new("members");

pub struct ChatDbManager {
  dbs: Mutex<HashMap<String, Arc<Database>>>,
  repaired: Mutex<HashSet<String>>,
}

impl Default for ChatDbManager {
  fn default() -> Self {
    Self {
      dbs: Mutex::new(HashMap::new()),
      repaired: Mutex::new(HashSet::new()),
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ConversationKind {
  Channel,
  Dm,
}

impl ConversationKind {
  fn as_str(&self) -> &'static str {
    match self {
      ConversationKind::Channel => "channel",
      ConversationKind::Dm => "dm",
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ConversationMeta {
  kind: ConversationKind,
  created_at: u64,
  custom_name: Option<String>,
  is_default: bool,
  last_message_at: Option<u64>,
  last_message_preview: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserConversationSettings {
  pinned: bool,
  muted: bool,
  last_read_message_id: Option<MsgId>,
  last_active_at: Option<u64>,
}

impl Default for UserConversationSettings {
  fn default() -> Self {
    Self {
      pinned: false,
      muted: false,
      last_read_message_id: None,
      last_active_at: None,
    }
  }
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserProfile {
  name: String,
  avatar: Option<String>,
  role_type: Option<String>,
  status: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MemberEntry {
  joined_at: u64,
  nickname: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
enum MessageStatus {
  Sent,
  Sending,
  Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MessageContent {
  Text { text: String },
  System { key: String, args: Option<HashMap<String, String>> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum MessageContentDb {
  Text { text: String },
  System { key: String, args: Option<HashMap<String, String>> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MessageAttachment {
  Image {
    file_path: String,
    file_name: String,
    file_size: u64,
    mime_type: String,
    width: Option<u32>,
    height: Option<u32>,
    thumbnail_path: Option<String>,
  },
  Roadmap { title: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum MessageAttachmentDb {
  Image {
    file_path: String,
    file_name: String,
    file_size: u64,
    mime_type: String,
    width: Option<u32>,
    height: Option<u32>,
    thumbnail_path: Option<String>,
  },
  Roadmap { title: String },
}

impl From<MessageContent> for MessageContentDb {
  fn from(value: MessageContent) -> Self {
    match value {
      MessageContent::Text { text } => Self::Text { text },
      MessageContent::System { key, args } => Self::System { key, args },
    }
  }
}

impl From<MessageContentDb> for MessageContent {
  fn from(value: MessageContentDb) -> Self {
    match value {
      MessageContentDb::Text { text } => Self::Text { text },
      MessageContentDb::System { key, args } => Self::System { key, args },
    }
  }
}

impl From<MessageAttachment> for MessageAttachmentDb {
  fn from(value: MessageAttachment) -> Self {
    match value {
      MessageAttachment::Image {
        file_path,
        file_name,
        file_size,
        mime_type,
        width,
        height,
        thumbnail_path,
      } => Self::Image {
        file_path,
        file_name,
        file_size,
        mime_type,
        width,
        height,
        thumbnail_path,
      },
      MessageAttachment::Roadmap { title } => Self::Roadmap { title },
    }
  }
}

impl From<MessageAttachmentDb> for MessageAttachment {
  fn from(value: MessageAttachmentDb) -> Self {
    match value {
      MessageAttachmentDb::Image {
        file_path,
        file_name,
        file_size,
        mime_type,
        width,
        height,
        thumbnail_path,
      } => Self::Image {
        file_path,
        file_name,
        file_size,
        mime_type,
        width,
        height,
        thumbnail_path,
      },
      MessageAttachmentDb::Roadmap { title } => Self::Roadmap { title },
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AttachmentIndexMeta {
  file_path: String,
  file_name: String,
  file_size: u64,
  mime_type: String,
  width: Option<u32>,
  height: Option<u32>,
  thumbnail_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChatMessage {
  sender_id: Option<UserId>,
  content: MessageContentDb,
  created_at: u64,
  is_ai: bool,
  status: MessageStatus,
  attachment: Option<MessageAttachmentDb>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummaryDto {
  id: String,
  #[serde(rename = "type")]
  conversation_type: String,
  member_ids: Vec<String>,
  target_id: Option<String>,
  custom_name: Option<String>,
  pinned: bool,
  muted: bool,
  last_message_at: Option<u64>,
  last_message_preview: Option<String>,
  is_default: bool,
  unread_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatHomeFeedDto {
  pinned: Vec<ConversationSummaryDto>,
  timeline: Vec<ConversationSummaryDto>,
  default_channel_id: Option<String>,
  total_unread_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageDto {
  id: String,
  sender_id: Option<String>,
  content: MessageContent,
  created_at: u64,
  is_ai: bool,
  status: MessageStatus,
  attachment: Option<MessageAttachment>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageCreatedPayload {
  workspace_id: String,
  conversation_id: String,
  message: MessageDto,
  total_unread_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRepairResult {
  removed_messages: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatClearResult {
  removed_messages: usize,
  removed_attachments: usize,
  cleared_timeline: usize,
}

fn now_millis() -> Result<u64, String> {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|err| format!("failed to read system time: {err}"))
    .map(|value| value.as_millis() as u64)
}

fn ts_rev(timestamp: u64) -> u64 {
  u64::MAX.saturating_sub(timestamp)
}

fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
  bincode::serialize(value).map_err(|err| format!("failed to encode value: {err}"))
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
  bincode::deserialize(bytes).map_err(|err| format!("failed to decode value: {err}"))
}

fn parse_ulid(value: &str) -> Result<u128, String> {
  let parsed = Ulid::from_string(value.trim()).map_err(|err| format!("invalid ULID: {err}"))?;
  Ok(parsed.0)
}

fn format_ulid(value: u128) -> String {
  Ulid(value).to_string()
}

fn db_path(app: &AppHandle, workspace_id: &str) -> Result<PathBuf, String> {
  let trimmed = workspace_id.trim();
  if trimmed.is_empty() {
    return Err("workspace_id is required".to_string());
  }
  resolve_app_data_path(app, &format!("{trimmed}/chat.redb"))
}

fn ensure_tables(db: &Database) -> Result<(), String> {
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
  }
  txn
    .commit()
    .map_err(|err| format!("failed to commit chat db init: {err}"))?;
  Ok(())
}

fn open_db(app: &AppHandle, state: &ChatDbManager, workspace_id: &str) -> Result<Arc<Database>, String> {
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
  let path = db_path(app, workspace_id)?;
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

fn resolve_target_id(member_ids: &[UserId], user_id: UserId) -> Option<UserId> {
  for id in member_ids {
    if *id != user_id {
      return Some(*id);
    }
  }
  None
}

fn load_member_ids_from_table<T>(
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

fn build_conversation_summary(
  conv_id: ConvId,
  meta: &ConversationMeta,
  settings: &UserConversationSettings,
  member_ids: Vec<UserId>,
  user_id: UserId,
  unread_count: usize,
) -> ConversationSummaryDto {
  let target_id = if meta.kind == ConversationKind::Dm {
    resolve_target_id(&member_ids, user_id).map(format_ulid)
  } else {
    None
  };
  ConversationSummaryDto {
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

fn count_unread_messages<T>(
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

fn compute_total_unread_count(db: &Database, user_id: UserId) -> Result<usize, String> {
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

fn build_message_preview(message: &ChatMessage) -> String {
  let raw = match &message.content {
    MessageContentDb::Text { text } => text.clone(),
    MessageContentDb::System { key, .. } => format!("[{key}]"),
  };
  let trimmed = raw.trim();
  if trimmed.is_empty() {
    return String::new();
  }
  const MAX_PREVIEW: usize = 120;
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
    .map_err(|_| "chat repair registry lock poisoned".to_string())?;
  if guard.contains(workspace_id) {
    return Ok(());
  }
  guard.insert(workspace_id.to_string());
  drop(guard);

  log::info!("chat repair scan workspace_id={}", workspace_id);
  let removed = repair_invalid_messages(db)?;
  if removed > 0 {
    log::warn!(
      "chat repair removed {} invalid messages workspace_id={}",
      removed,
      workspace_id
    );
  }
  Ok(())
}

fn repair_invalid_messages(db: &Database) -> Result<usize, String> {
  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
  let table = read_txn
    .open_table(MESSAGES)
    .map_err(|err| format!("failed to open messages table: {err}"))?;

  let mut invalid_keys: Vec<(ConvId, MsgId)> = Vec::new();
  let mut latest_by_conv: HashMap<ConvId, (u64, String)> = HashMap::new();
  let mut touched_convs: HashSet<ConvId> = HashSet::new();

  for entry in table
    .iter()
    .map_err(|err| format!("failed to scan messages: {err}"))?
  {
    let (key, value) = entry.map_err(|err| format!("failed to decode message entry: {err}"))?;
    let (conv_id, msg_id) = key.value();
    match decode::<ChatMessage>(value.value()) {
      Ok(message) => {
        let preview = build_message_preview(&message);
        let entry = latest_by_conv
          .entry(conv_id)
          .or_insert_with(|| (message.created_at, preview.clone()));
        if message.created_at >= entry.0 {
          *entry = (message.created_at, preview);
        }
      }
      Err(_) => {
        invalid_keys.push((conv_id, msg_id));
        touched_convs.insert(conv_id);
      }
    }
  }

  if invalid_keys.is_empty() {
    return Ok(0);
  }
  drop(table);
  drop(read_txn);

  let write_txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  {
    let mut table = write_txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    for key in invalid_keys.iter().copied() {
      let _ = table.remove(key);
    }
  }

  {
    let mut table = write_txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    for conv_id in touched_convs {
      let mut meta: ConversationMeta = {
        let existing = table
          .get(conv_id)
          .map_err(|err| format!("failed to read conversation: {err}"))?;
        match existing {
          Some(value) => decode(value.value())?,
          None => continue,
        }
      };
      match latest_by_conv.get(&conv_id) {
        Some((created_at, preview)) => {
          meta.last_message_at = Some(*created_at);
          meta.last_message_preview = if preview.is_empty() { None } else { Some(preview.clone()) };
        }
        None => {
          meta.last_message_at = None;
          meta.last_message_preview = None;
        }
      }
      let payload = encode(&meta)?;
      table
        .insert(conv_id, payload.as_slice())
        .map_err(|err| format!("failed to update conversation meta: {err}"))?;
    }
  }

  write_txn
    .commit()
    .map_err(|err| format!("failed to commit chat repair: {err}"))?;

  Ok(invalid_keys.len())
}

fn clear_chat_storage(db: &Database) -> Result<ChatClearResult, String> {
  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;

  let mut message_keys: Vec<(ConvId, MsgId)> = Vec::new();
  let mut attachment_keys: Vec<(ConvId, u8, TsRev, MsgId)> = Vec::new();
  let mut timeline_keys: Vec<(UserId, TsRev, ConvId)> = Vec::new();
  let mut settings_updates: Vec<((UserId, ConvId), UserConversationSettings)> = Vec::new();
  let mut conversation_updates: Vec<(ConvId, ConversationMeta)> = Vec::new();

  {
    let table = read_txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    for entry in table
      .iter()
      .map_err(|err| format!("failed to scan messages: {err}"))?
    {
      let (key, _) = entry.map_err(|err| format!("failed to decode message entry: {err}"))?;
      message_keys.push(key.value());
    }
  }

  {
    let table = read_txn
      .open_table(ATTACHMENTS_INDEX)
      .map_err(|err| format!("failed to open attachments_index table: {err}"))?;
    for entry in table
      .iter()
      .map_err(|err| format!("failed to scan attachments: {err}"))?
    {
      let (key, _) = entry.map_err(|err| format!("failed to decode attachments entry: {err}"))?;
      attachment_keys.push(key.value());
    }
  }

  {
    let table = read_txn
      .open_table(TIMELINE_INDEX)
      .map_err(|err| format!("failed to open timeline_index table: {err}"))?;
    for entry in table
      .iter()
      .map_err(|err| format!("failed to scan timeline_index: {err}"))?
    {
      let (key, _) = entry.map_err(|err| format!("failed to decode timeline entry: {err}"))?;
      timeline_keys.push(key.value());
    }
  }

  {
    let table = read_txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    for entry in table
      .iter()
      .map_err(|err| format!("failed to scan user_convs: {err}"))?
    {
      let (key, value) = entry.map_err(|err| format!("failed to decode user_convs entry: {err}"))?;
      let mut settings: UserConversationSettings = decode(value.value())?;
      settings.last_active_at = None;
      settings.last_read_message_id = None;
      settings_updates.push((key.value(), settings));
    }
  }

  {
    let table = read_txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    for entry in table
      .iter()
      .map_err(|err| format!("failed to scan conversations: {err}"))?
    {
      let (key, value) = entry.map_err(|err| format!("failed to decode conversation entry: {err}"))?;
      let mut meta: ConversationMeta = decode(value.value())?;
      meta.last_message_at = None;
      meta.last_message_preview = None;
      conversation_updates.push((key.value(), meta));
    }
  }

  drop(read_txn);

  let write_txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;

  {
    let mut table = write_txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    for key in message_keys.iter().copied() {
      let _ = table.remove(key);
    }
  }

  {
    let mut table = write_txn
      .open_table(ATTACHMENTS_INDEX)
      .map_err(|err| format!("failed to open attachments_index table: {err}"))?;
    for key in attachment_keys.iter().copied() {
      let _ = table.remove(key);
    }
  }

  {
    let mut table = write_txn
      .open_table(TIMELINE_INDEX)
      .map_err(|err| format!("failed to open timeline_index table: {err}"))?;
    for key in timeline_keys.iter().copied() {
      let _ = table.remove(key);
    }
  }

  {
    let mut table = write_txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    for (key, settings) in settings_updates {
      let payload = encode(&settings)?;
      table
        .insert(key, payload.as_slice())
        .map_err(|err| format!("failed to update user_convs: {err}"))?;
    }
  }

  {
    let mut table = write_txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    for (key, meta) in conversation_updates {
      let payload = encode(&meta)?;
      table
        .insert(key, payload.as_slice())
        .map_err(|err| format!("failed to update conversations: {err}"))?;
    }
  }

  write_txn
    .commit()
    .map_err(|err| format!("failed to commit chat clear: {err}"))?;

  Ok(ChatClearResult {
    removed_messages: message_keys.len(),
    removed_attachments: attachment_keys.len(),
    cleared_timeline: timeline_keys.len(),
  })
}

fn attachment_index_entry(attachment: &MessageAttachmentDb) -> Option<(u8, AttachmentIndexMeta)> {
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
      0,
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

fn ensure_default_channel(
  txn: &mut redb::WriteTransaction,
  workspace_name: Option<&str>,
  member_ids: &[UserId],
) -> Result<(ConvId, ConversationMeta), String> {
  let mut existing: Option<(ConvId, ConversationMeta)> = None;
  {
    let table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    for entry in table
      .iter()
      .map_err(|err| format!("failed to scan conversations: {err}"))?
    {
      let (key, value) = entry.map_err(|err| format!("failed to decode conversation entry: {err}"))?;
      let meta: ConversationMeta = decode(value.value())?;
      if meta.is_default {
        existing = Some((key.value(), meta));
        break;
      }
    }
  }

  let now = now_millis()?;
  let mut conv_id = existing.as_ref().map(|(id, _)| *id);
  let mut meta = existing
    .map(|(_, meta)| meta)
    .unwrap_or_else(|| ConversationMeta {
      kind: ConversationKind::Channel,
      created_at: now,
      custom_name: None,
      is_default: true,
      last_message_at: None,
      last_message_preview: None,
    });

  if conv_id.is_none() {
    conv_id = Some(Ulid::new().0);
  }

  if let Some(name) = workspace_name {
    let trimmed = name.trim();
    if !trimmed.is_empty() && meta.custom_name.as_deref() != Some(trimmed) {
      meta.custom_name = Some(trimmed.to_string());
    }
  }

  let conv_id = conv_id.ok_or_else(|| "failed to resolve default channel id".to_string())?;

  {
    let mut table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let payload = encode(&meta)?;
    table
      .insert(conv_id, payload.as_slice())
      .map_err(|err| format!("failed to store default conversation: {err}"))?;
  }

  sync_conversation_members(txn, conv_id, member_ids, meta.created_at)?;
  Ok((conv_id, meta))
}

fn sync_conversation_members(
  txn: &mut redb::WriteTransaction,
  conv_id: ConvId,
  member_ids: &[UserId],
  created_at: u64,
) -> Result<(), String> {
  let mut desired = HashSet::new();
  for id in member_ids {
    desired.insert(*id);
  }

  let existing: Vec<UserId> = {
    let table = txn
      .open_table(MEMBERS)
      .map_err(|err| format!("failed to open members table: {err}"))?;
    load_member_ids_from_table(&table, conv_id)?
  };
  let existing_set: HashSet<UserId> = existing.iter().copied().collect();

  for user_id in existing.iter().copied().filter(|id| !desired.contains(id)) {
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

  for user_id in member_ids.iter().copied() {
    if !existing_set.contains(&user_id) {
      let entry = MemberEntry {
        joined_at: created_at,
        nickname: None,
      };
      let payload = encode(&entry)?;
      let mut table = txn
        .open_table(MEMBERS)
        .map_err(|err| format!("failed to open members table: {err}"))?;
      table
        .insert((conv_id, user_id), payload.as_slice())
        .map_err(|err| format!("failed to add member: {err}"))?;
    }

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

    if settings.last_active_at.is_none() {
      settings.last_active_at = Some(created_at);
      let payload = encode(&settings)?;
      let mut table = txn
        .open_table(USER_CONVS)
        .map_err(|err| format!("failed to open user_convs table: {err}"))?;
      table
        .insert((user_id, conv_id), payload.as_slice())
        .map_err(|err| format!("failed to update user_convs: {err}"))?;

      let mut timeline = txn
        .open_table(TIMELINE_INDEX)
        .map_err(|err| format!("failed to open timeline_index table: {err}"))?;
      let _ = timeline.insert((user_id, ts_rev(created_at), conv_id), ());
    }
  }

  Ok(())
}

#[tauri::command]
pub fn chat_ulid_new() -> Result<String, String> {
  Ok(Ulid::new().to_string())
}

#[tauri::command]
pub fn chat_repair_messages(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
) -> Result<ChatRepairResult, String> {
  let db = open_db(&app, &state, &workspace_id)?;
  let removed = repair_invalid_messages(&db)?;
  Ok(ChatRepairResult {
    removed_messages: removed,
  })
}

#[tauri::command]
pub fn chat_clear_all_messages(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
) -> Result<ChatClearResult, String> {
  let db = open_db(&app, &state, &workspace_id)?;
  clear_chat_storage(&db)
}

#[tauri::command]
pub fn chat_list_conversations(
  app: AppHandle,
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

  let db = open_db(&app, &state, &workspace_id)?;

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
    let mut fallback: Vec<(u64, ConvId, ConversationSummaryDto)> = Vec::new();
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
      timeline_ids.insert(default_channel_id);
    }
  }

  let default_channel_id = if default_meta.is_default {
    Some(format_ulid(default_channel_id))
  } else {
    None
  };

  Ok(ChatHomeFeedDto {
    pinned,
    timeline,
    default_channel_id,
    total_unread_count,
  })
}

#[tauri::command]
pub fn chat_get_messages(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  limit: Option<u32>,
  before_id: Option<String>,
) -> Result<Vec<MessageDto>, String> {
  let conv_id = parse_ulid(&conversation_id)?;
  let before_id = match before_id {
    Some(value) => Some(parse_ulid(&value)?),
    None => None,
  };
  let db = open_db(&app, &state, &workspace_id)?;

  let read_txn = db
    .begin_read()
    .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
  let table = read_txn
    .open_table(MESSAGES)
    .map_err(|err| format!("failed to open messages table: {err}"))?;

  let mut results = Vec::new();
  let iter = if let Some(before) = before_id {
    table
      .range((conv_id, 0)..(conv_id, before))
      .map_err(|err| format!("failed to scan messages: {err}"))?
  } else {
    table
      .range((conv_id, 0)..=(conv_id, u128::MAX))
      .map_err(|err| format!("failed to scan messages: {err}"))?
  };

  let mut push_message = |msg_id: u128, value: &[u8]| {
    match decode::<ChatMessage>(value) {
      Ok(message) => {
        let content = MessageContent::from(message.content);
        let attachment = message.attachment.map(MessageAttachment::from);
        results.push(MessageDto {
          id: format_ulid(msg_id),
          sender_id: message.sender_id.map(format_ulid),
          content,
          created_at: message.created_at,
          is_ai: message.is_ai,
          status: message.status,
          attachment,
        });
      }
      Err(err) => {
        log::warn!("failed to decode chat message conversation_id={} msg_id={} err={}", conversation_id, msg_id, err);
      }
    }
  };

  if let Some(limit) = limit {
    for entry in iter.rev().take(limit as usize) {
      let (key, value) = entry.map_err(|err| format!("failed to decode message entry: {err}"))?;
      let (_, msg_id) = key.value();
      push_message(msg_id, value.value());
    }
    results.reverse();
  } else {
    for entry in iter {
      let (key, value) = entry.map_err(|err| format!("failed to decode message entry: {err}"))?;
      let (_, msg_id) = key.value();
      push_message(msg_id, value.value());
    }
  }

  Ok(results)
}

#[tauri::command]
pub fn chat_mark_conversation_read_latest(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  conversation_id: String,
) -> Result<(), String> {
  let user_id = parse_ulid(&user_id)?;
  let conv_id = parse_ulid(&conversation_id)?;
  let db = open_db(&app, &state, &workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;

  let latest_msg_id = {
    let table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    let start = (conv_id, 0);
    let end = (conv_id, u128::MAX);
    let mut latest = None;
    for entry in table
      .range(start..=end)
      .map_err(|err| format!("failed to scan messages: {err}"))?
    {
      let (key, _) = entry.map_err(|err| format!("failed to decode message entry: {err}"))?;
      let (_, msg_id) = key.value();
      latest = Some(msg_id);
    }
    latest
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
  Ok(())
}

fn save_message_in_db(
  db: &Database,
  conv_id: ConvId,
  sender_id: Option<UserId>,
  content: MessageContent,
  is_ai: bool,
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
    status: MessageStatus::Sent,
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
    let meta = match existing {
      Some(value) => decode(value.value())?,
      None => {
        return Err("conversation not found".to_string());
      }
    };
    meta
  };

  {
    let mut table = txn
      .open_table(CONVERSATIONS)
      .map_err(|err| format!("failed to open conversations table: {err}"))?;
    let mut updated = meta;
    updated.last_message_at = Some(created_at);
    updated.last_message_preview = if preview.is_empty() { None } else { Some(preview) };
    let payload = encode(&updated)?;
    table
      .insert(conv_id, payload.as_slice())
      .map_err(|err| format!("failed to update conversation: {err}"))?;
  }

  let member_ids = {
    let table = txn
      .open_table(MEMBERS)
      .map_err(|err| format!("failed to open members table: {err}"))?;
    load_member_ids_from_table(&table, conv_id)?
  };

  let mut settings_map = HashMap::new();
  {
    let table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    for user_id in member_ids.iter().copied() {
      let settings = match table
        .get((user_id, conv_id))
        .map_err(|err| format!("failed to read user_convs: {err}"))?
      {
        Some(value) => decode(value.value())?,
        None => UserConversationSettings::default(),
      };
      settings_map.insert(user_id, settings);
    }
  }

  {
    let mut table = txn
      .open_table(TIMELINE_INDEX)
      .map_err(|err| format!("failed to open timeline_index table: {err}"))?;
    for (user_id, settings) in settings_map.iter() {
      if let Some(last_active_at) = settings.last_active_at {
        let _ = table.remove((*user_id, ts_rev(last_active_at), conv_id));
      }
    }
    for user_id in member_ids.iter().copied() {
      let _ = table.insert((user_id, ts_rev(created_at), conv_id), ());
    }
  }

  {
    let mut table = txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    for (user_id, mut settings) in settings_map {
      settings.last_active_at = Some(created_at);
      if let Some(sender_id) = sender_id {
        if user_id == sender_id {
          settings.last_read_message_id = Some(msg_id);
        }
      }
      let payload = encode(&settings)?;
      table
        .insert((user_id, conv_id), payload.as_slice())
        .map_err(|err| format!("failed to update user_convs: {err}"))?;
    }
  }

  if let Some(ref attachment) = attachment_db {
    if let Some((type_key, meta)) = attachment_index_entry(attachment) {
      let payload = encode(&meta)?;
      let mut table = txn
        .open_table(ATTACHMENTS_INDEX)
        .map_err(|err| format!("failed to open attachments_index table: {err}"))?;
      let _ = table.insert((conv_id, type_key, ts_rev(created_at), msg_id), payload.as_slice());
    }
  }

  txn
    .commit()
    .map_err(|err| format!("failed to commit message: {err}"))?;

  Ok(MessageDto {
    id: format_ulid(msg_id),
    sender_id: sender_id.map(format_ulid),
    content,
    created_at: message.created_at,
    is_ai: message.is_ai,
    status: message.status,
    attachment,
  })
}

#[tauri::command]
pub fn chat_send_message(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  sender_id: Option<String>,
  content: MessageContent,
  is_ai: Option<bool>,
  attachment: Option<MessageAttachment>,
) -> Result<MessageDto, String> {
  let conv_id = parse_ulid(&conversation_id)?;
  let sender_id = match sender_id {
    Some(value) => Some(parse_ulid(&value)?),
    None => None,
  };
  let db = open_db(&app, &state, &workspace_id)?;
  save_message_in_db(
    &db,
    conv_id,
    sender_id,
    content,
    is_ai.unwrap_or(false),
    attachment,
  )
}

pub(crate) fn chat_append_terminal_message(
  app: &AppHandle,
  state: &ChatDbManager,
  workspace_id: &str,
  conversation_id: &str,
  sender_id: &str,
  content: String,
  viewer_id: &str,
) -> Result<(), String> {
  let conv_id = parse_ulid(conversation_id)?;
  let sender_id = parse_ulid(sender_id)?;
  let viewer_id = parse_ulid(viewer_id)?;
  let db = open_db(app, state, workspace_id)?;
  let message = save_message_in_db(
    &db,
    conv_id,
    Some(sender_id),
    MessageContent::Text { text: content },
    false,
    None,
  )?;
  let total_unread_count = compute_total_unread_count(&db, viewer_id)?;
  let payload = ChatMessageCreatedPayload {
    workspace_id: workspace_id.to_string(),
    conversation_id: conversation_id.to_string(),
    message,
    total_unread_count,
  };
  let _ = app.emit("chat-message-created", payload);
  Ok(())
}

#[tauri::command]
pub fn chat_create_group(
  app: AppHandle,
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

  let db = open_db(&app, &state, &workspace_id)?;
  let created_at = now_millis()?;
  let conv_id = Ulid::new().0;
  let trimmed_name = custom_name.and_then(|value| {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() { None } else { Some(trimmed) }
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

#[tauri::command]
pub fn chat_ensure_direct(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  target_id: String,
) -> Result<ConversationSummaryDto, String> {
  let user_id = parse_ulid(&user_id)?;
  let target_id = parse_ulid(&target_id)?;

  let db = open_db(&app, &state, &workspace_id)?;
  let mut existing: Option<ConvId> = None;

  {
    let read_txn = db
      .begin_read()
      .map_err(|err| format!("failed to open chat read transaction: {err}"))?;
    let table = read_txn
      .open_table(USER_CONVS)
      .map_err(|err| format!("failed to open user_convs table: {err}"))?;
    let start = (user_id, 0);
    let end = (user_id, u128::MAX);
    let conv_ids: Vec<ConvId> = table
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

  Ok(build_conversation_summary(
    conv_id,
    &meta,
    &settings,
    member_ids,
    user_id,
    unread_count,
  ))
}

#[tauri::command]
pub fn chat_set_conversation_settings(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  user_id: String,
  conversation_id: String,
  pinned: Option<bool>,
  muted: Option<bool>,
) -> Result<(), String> {
  let user_id = parse_ulid(&user_id)?;
  let conv_id = parse_ulid(&conversation_id)?;
  let db = open_db(&app, &state, &workspace_id)?;

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

#[tauri::command]
pub fn chat_rename_conversation(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  custom_name: Option<String>,
) -> Result<(), String> {
  let conv_id = parse_ulid(&conversation_id)?;
  let db = open_db(&app, &state, &workspace_id)?;
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

#[tauri::command]
pub fn chat_clear_conversation(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
) -> Result<(), String> {
  let conv_id = parse_ulid(&conversation_id)?;
  let db = open_db(&app, &state, &workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;

  {
    let mut table = txn
      .open_table(MESSAGES)
      .map_err(|err| format!("failed to open messages table: {err}"))?;
    let start = (conv_id, 0);
    let end = (conv_id, u128::MAX);
    let mut keys: Vec<(ConvId, MsgId)> = Vec::new();
    let mut latest_msg_id: Option<MsgId> = None;
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
    let keys: Vec<(ConvId, u8, TsRev, MsgId)> = table
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

#[tauri::command]
pub fn chat_delete_conversation(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
) -> Result<(), String> {
  let conv_id = parse_ulid(&conversation_id)?;
  let db = open_db(&app, &state, &workspace_id)?;
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
    let keys: Vec<(ConvId, MsgId)> = table
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
    let keys: Vec<(ConvId, u8, TsRev, MsgId)> = table
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

#[tauri::command]
pub fn chat_set_conversation_members(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  workspace_id: String,
  conversation_id: String,
  member_ids: Vec<String>,
) -> Result<(), String> {
  let conv_id = parse_ulid(&conversation_id)?;
  let mut member_ids_u128 = Vec::new();
  for id in member_ids {
    member_ids_u128.push(parse_ulid(&id)?);
  }
  if member_ids_u128.is_empty() {
    return Ok(());
  }
  let db = open_db(&app, &state, &workspace_id)?;
  let created_at = now_millis()?;
  let mut txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat write transaction: {err}"))?;
  sync_conversation_members(&mut txn, conv_id, &member_ids_u128, created_at)?;
  txn
    .commit()
    .map_err(|err| format!("failed to commit member sync: {err}"))?;
  Ok(())
}
