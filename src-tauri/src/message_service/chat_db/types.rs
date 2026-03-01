//! 聊天领域类型：覆盖会话/消息/附件与对外 DTO。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::contracts::chat_dispatch::ChatDispatchPayload;

// 统一使用 ULID 的 u128 表示，避免字符串重复解析。
pub(super) type UserId = u128;
pub(super) type ConvId = u128;
pub(super) type MsgId = u128;
// 逆序时间戳用于实现“最近优先”的范围扫描。
pub(super) type TsRev = u64;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
/// 会话类型：频道或私聊（DM）。
#[serde(rename_all = "lowercase")]
pub(super) enum ConversationKind {
  Channel,
  Dm,
}

impl ConversationKind {
  pub(super) fn as_str(&self) -> &'static str {
    match self {
      ConversationKind::Channel => "channel",
      ConversationKind::Dm => "dm",
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 会话元信息：用于列表展示与排序。
pub(super) struct ConversationMeta {
  pub(super) kind: ConversationKind,
  pub(super) created_at: u64,
  pub(super) custom_name: Option<String>,
  pub(super) is_default: bool,
  pub(super) last_message_at: Option<u64>,
  pub(super) last_message_preview: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 用户维度的会话设置：仅存储个性化状态。
pub(super) struct UserConversationSettings {
  pub(super) pinned: bool,
  pub(super) muted: bool,
  pub(super) last_read_message_id: Option<MsgId>,
  pub(super) last_active_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 终端映射：成员 member_id 与远端 session_id 的一一对应关系。
pub(crate) struct TerminalSessionMapEntry {
  pub(crate) member_id: String,
  pub(crate) session_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 终端映射反查索引：通过远端 session_id 找 member_id。
pub(crate) struct TerminalSessionIndexEntry {
  pub(crate) member_id: String,
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
pub(super) struct UserProfile {
  pub(super) name: String,
  pub(super) avatar: Option<String>,
  pub(super) role_type: Option<String>,
  pub(super) status: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 成员在会话中的关系与昵称。
pub(super) struct MemberEntry {
  pub(super) joined_at: u64,
  pub(super) nickname: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
/// 消息发送状态，用于 UI 展示与重试策略。
#[serde(rename_all = "lowercase")]
pub(crate) enum MessageStatus {
  Sent,
  Sending,
  Failed,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
/// 派发队列状态：用于终端派发的重试与诊断。
#[serde(rename_all = "lowercase")]
pub(crate) enum ChatOutboxStatus {
  Pending,
  Sending,
  Failed,
  Sent,
  Dead,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Outbox 任务：持久化终端派发计划，支持重试与恢复。
pub(crate) struct ChatOutboxTask {
  pub(crate) message_id: String,
  pub(crate) payload: ChatDispatchPayload,
  pub(crate) status: ChatOutboxStatus,
  pub(crate) attempts: u32,
  pub(crate) created_at: u64,
  pub(crate) updated_at: u64,
  pub(crate) next_attempt_at: u64,
  pub(crate) sending_since: Option<u64>,
  pub(crate) last_error: Option<String>,
}

#[derive(Serialize, Clone)]
/// 消息状态更新事件载荷（对外 API）。
#[serde(rename_all = "camelCase")]
pub(crate) struct ChatMessageStatusPayload {
  pub(crate) workspace_id: String,
  pub(crate) conversation_id: String,
  pub(crate) message_id: String,
  pub(crate) status: MessageStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatDeleteMemberConversationsResult {
  pub(crate) removed_conversations: u32,
  pub(crate) warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 消息内容类型（对外 API）。
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MessageContent {
  Text { text: String },
  System { key: String, args: Option<HashMap<String, String>> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 数据库存储版本的消息内容，便于向前兼容。
pub(super) enum MessageContentDb {
  Text { text: String },
  System { key: String, args: Option<HashMap<String, String>> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 附件类型（对外 API）。
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
/// 数据库存储版本的附件类型。
pub(super) enum MessageAttachmentDb {
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
      MessageContentDb::Text { text } => MessageContent::Text { text },
      MessageContentDb::System { key, args } => MessageContent::System { key, args },
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
      } => MessageAttachmentDb::Image {
        file_path,
        file_name,
        file_size,
        mime_type,
        width,
        height,
        thumbnail_path,
      },
      MessageAttachment::Roadmap { title } => MessageAttachmentDb::Roadmap { title },
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
      } => MessageAttachment::Image {
        file_path,
        file_name,
        file_size,
        mime_type,
        width,
        height,
        thumbnail_path,
      },
      MessageAttachmentDb::Roadmap { title } => MessageAttachment::Roadmap { title },
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 附件索引的元数据，用于快速检索与预览。
pub(super) struct AttachmentIndexMeta {
  pub(super) file_path: String,
  pub(super) file_name: String,
  pub(super) file_size: u64,
  pub(super) mime_type: String,
  pub(super) width: Option<u32>,
  pub(super) height: Option<u32>,
  pub(super) thumbnail_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
/// 内部消息存储结构。
pub(super) struct ChatMessage {
  pub(super) sender_id: Option<UserId>,
  pub(super) content: MessageContentDb,
  pub(super) created_at: u64,
  pub(super) is_ai: bool,
  pub(super) status: MessageStatus,
  pub(super) attachment: Option<MessageAttachmentDb>,
}

#[derive(Serialize)]
/// 会话摘要（对外 API）。
#[serde(rename_all = "camelCase")]
pub struct ConversationSummaryDto {
  pub(crate) id: String,
  #[serde(rename = "type")]
  pub(crate) conversation_type: String,
  pub(crate) member_ids: Vec<String>,
  pub(crate) target_id: Option<String>,
  pub(crate) custom_name: Option<String>,
  pub(crate) pinned: bool,
  pub(crate) muted: bool,
  pub(crate) last_message_at: Option<u64>,
  pub(crate) last_message_preview: Option<String>,
  pub(crate) is_default: bool,
  pub(crate) unread_count: usize,
}

#[derive(Serialize)]
/// 聊天首页汇总（对外 API）。
#[serde(rename_all = "camelCase")]
pub struct ChatHomeFeedDto {
  pub(crate) pinned: Vec<ConversationSummaryDto>,
  pub(crate) timeline: Vec<ConversationSummaryDto>,
  pub(crate) default_channel_id: Option<String>,
  pub(crate) total_unread_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceUnreadSummary {
  pub(crate) workspace_id: String,
  pub(crate) total_unread_count: usize,
  pub(crate) latest_conversation_id: Option<String>,
  pub(crate) latest_conversation_type: Option<String>,
  pub(crate) latest_message_at: Option<u64>,
  pub(crate) latest_unread_count: usize,
}

#[derive(Serialize, Clone)]
/// 未读同步事件载荷（对外 API）。
#[serde(rename_all = "camelCase")]
pub struct ChatUnreadSyncPayload {
  pub(crate) workspace_id: String,
  pub(crate) total_unread_count: usize,
  pub(crate) conversation_id: Option<String>,
  pub(crate) conversation_unread_count: Option<usize>,
  pub(crate) reset_all: bool,
}

#[derive(Serialize, Clone)]
/// 消息详情（对外 API）。
#[serde(rename_all = "camelCase")]
pub struct MessageDto {
  pub(crate) id: String,
  pub(crate) sender_id: Option<String>,
  pub(crate) content: MessageContent,
  pub(crate) created_at: u64,
  pub(crate) is_ai: bool,
  pub(crate) status: MessageStatus,
  pub(crate) attachment: Option<MessageAttachment>,
}

#[derive(Serialize, Clone)]
/// 新消息事件载荷（对外 API）。
#[serde(rename_all = "camelCase")]
pub(super) struct ChatMessageCreatedPayload {
  pub(super) workspace_id: String,
  pub(super) conversation_id: String,
  pub(super) message: MessageDto,
  pub(super) total_unread_count: usize,
  pub(super) span_id: Option<String>,
}

#[derive(Serialize)]
/// 修复任务结果统计（对外 API）。
#[serde(rename_all = "camelCase")]
pub struct ChatRepairResult {
  pub(crate) removed_messages: usize,
}

#[derive(Serialize)]
/// 清空任务结果统计（对外 API）。
#[serde(rename_all = "camelCase")]
pub struct ChatClearResult {
  pub(crate) removed_messages: usize,
  pub(crate) removed_attachments: usize,
  pub(crate) cleared_timeline: usize,
}
