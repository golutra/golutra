//! 轮询快照：只读采集会话关键字段，避免在轮询线程内耦合业务逻辑。

use super::super::{PostReadyMode, PostReadyState, TerminalSessionStatus};

#[derive(Clone, Debug)]
pub(crate) struct SessionPollSnapshot {
  pub(crate) terminal_id: String,
  pub(crate) terminal_type: String,
  pub(crate) status: TerminalSessionStatus,
  pub(crate) status_locked: bool,
  pub(crate) ui_active: bool,
  pub(crate) flow_paused: bool,
  pub(crate) shell_ready: bool,
  pub(crate) post_ready_state: PostReadyState,
  pub(crate) post_ready_mode: PostReadyMode,
  pub(crate) post_ready_restart_pending: bool,
  pub(crate) last_activity_at: Option<u64>,
  pub(crate) last_output_at: Option<u64>,
  pub(crate) idle_candidate_at: Option<u64>,
  pub(crate) chat_pending: bool,
  pub(crate) chat_pending_since: Option<u64>,
  pub(crate) chat_candidate_at: Option<u64>,
  pub(crate) pending_output_chunks: usize,
  pub(crate) member_id: Option<String>,
  pub(crate) workspace_id: Option<String>,
}

impl SessionPollSnapshot {
}
