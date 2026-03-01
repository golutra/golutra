//! 轮询动作与会话更新：用于拆分采集与业务执行。

use super::super::TerminalSessionStatus;

#[derive(Clone, Debug)]
pub(crate) struct ChatFlushMeta {
  pub(crate) terminal_id: String,
  pub(crate) member_id: Option<String>,
  pub(crate) workspace_id: Option<String>,
  pub(crate) terminal_type: String,
  pub(crate) last_output_at: Option<u64>,
  pub(crate) chat_pending_since: Option<u64>,
  pub(crate) silence_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub(crate) struct SessionUpdate {
  pub(crate) terminal_id: String,
  pub(crate) set_status: Option<TerminalSessionStatus>,
  pub(crate) set_idle_candidate_at: Option<Option<u64>>,
  pub(crate) set_chat_candidate_at: Option<Option<u64>>,
  pub(crate) set_chat_pending: Option<bool>,
  pub(crate) set_chat_pending_since: Option<Option<u64>>,
  pub(crate) set_semantic_active: Option<bool>,
  pub(crate) emit_status: bool,
}

impl SessionUpdate {
  pub(crate) fn new(terminal_id: &str) -> Self {
    Self {
      terminal_id: terminal_id.to_string(),
      set_status: None,
      set_idle_candidate_at: None,
      set_chat_candidate_at: None,
      set_chat_pending: None,
      set_chat_pending_since: None,
      set_semantic_active: None,
      emit_status: false,
    }
  }

  pub(crate) fn has_changes(&self) -> bool {
    self.set_status.is_some()
      || self.set_idle_candidate_at.is_some()
      || self.set_chat_candidate_at.is_some()
      || self.set_chat_pending.is_some()
      || self.set_chat_pending_since.is_some()
      || self.set_semantic_active.is_some()
}
}

#[derive(Clone, Debug)]
pub(crate) enum PollAction {
  SessionUpdate(SessionUpdate),
  SemanticFlush(ChatFlushMeta),
  PostReadyStart { terminal_id: String, terminal_type: String },
  PostReadyStep { terminal_id: String },
  PostReadyRestart { terminal_id: String },
}
