//! 聊天数据库模块：按子域拆分读写与存储职责，避免单文件膨胀。

mod read;
mod store;
mod terminal_session_map;
mod types;
mod outbox;
mod write;

pub use store::ChatDbManager;
pub(crate) use store::list_workspace_ids;
pub use terminal_session_map::{
    terminal_session_delete_by_member_id, terminal_session_get_by_member_id,
    terminal_session_upsert,
};
pub use types::{
    ChatClearResult, ChatDeleteMemberConversationsResult, ChatHomeFeedDto, ChatRepairResult,
    ConversationSummaryDto, MessageAttachment, MessageContent, MessageDto,
};
pub(crate) use types::{ChatOutboxTask, MessageStatus};

pub(crate) use read::compute_workspace_unread_summary;
pub use read::{chat_get_conversation_member_ids, chat_get_messages, chat_list_conversations};
pub(crate) use outbox::{
    chat_outbox_claim_due, chat_outbox_enqueue, chat_outbox_mark_failed, chat_outbox_mark_sent,
};

pub(crate) use write::chat_append_terminal_message;
pub use write::{
    chat_clear_all_messages, chat_clear_conversation, chat_create_group, chat_delete_conversation,
    chat_delete_member_conversations, chat_ensure_direct, chat_mark_conversation_read_latest,
    chat_mark_workspace_read_latest, chat_rename_conversation, chat_repair_messages,
    chat_send_message, chat_set_conversation_members, chat_set_conversation_settings,
    chat_ulid_new,
};
pub(crate) use write::{chat_send_message_for_dispatch, chat_update_message_status};
