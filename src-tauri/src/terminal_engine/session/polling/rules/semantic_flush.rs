//! 语义 flush 规则：在静默窗口内触发语义缓冲落盘。

use super::super::actions::{ChatFlushMeta, PollAction, SessionUpdate};
use super::super::snapshot::SessionPollSnapshot;
use super::super::super::{
  CHAT_IDLE_DEBOUNCE_MS, CHAT_PENDING_FORCE_FLUSH_MS, CHAT_SILENCE_TIMEOUT_MS,
};

pub(crate) fn collect_semantic_flush_actions(
  snapshot: &SessionPollSnapshot,
  now: u64,
  actions: &mut Vec<PollAction>,
) {
  if !snapshot.chat_pending {
    return;
  }
  let force_flush = snapshot
    .chat_pending_since
    .map(|since| now.saturating_sub(since) >= CHAT_PENDING_FORCE_FLUSH_MS)
    .unwrap_or(false);
  let mut update = SessionUpdate::new(&snapshot.terminal_id);
  if !force_flush && (snapshot.flow_paused || snapshot.pending_output_chunks != 0) {
    if snapshot.chat_candidate_at.is_some() {
      update.set_chat_candidate_at = Some(None);
    }
    if update.has_changes() {
      actions.push(PollAction::SessionUpdate(update));
    }
    return;
  }
  let last_output_at = snapshot.last_output_at;
  let silence_ms = last_output_at
    .map(|value| now.saturating_sub(value))
    .or_else(|| snapshot.chat_pending_since.map(|value| now.saturating_sub(value)));
  if !force_flush {
    let Some(last_output_at) = last_output_at else {
      return;
    };
    let silence_ms = now.saturating_sub(last_output_at);
    if silence_ms < CHAT_SILENCE_TIMEOUT_MS {
      if snapshot.chat_candidate_at.is_some() {
        update.set_chat_candidate_at = Some(None);
      }
      if update.has_changes() {
        actions.push(PollAction::SessionUpdate(update));
      }
      return;
    }
    match snapshot.chat_candidate_at {
      None => {
        // 首次达到静默门槛，记录候选时间做防抖。
        update.set_chat_candidate_at = Some(Some(now));
        if update.has_changes() {
          actions.push(PollAction::SessionUpdate(update));
        }
        return;
      }
      Some(candidate_at) => {
        if now.saturating_sub(candidate_at) < CHAT_IDLE_DEBOUNCE_MS {
          return;
        }
      }
    }
  }
  // 静默持续满足防抖要求后触发 flush。
  update.set_chat_candidate_at = Some(None);
  update.set_chat_pending = Some(false);
  update.set_chat_pending_since = Some(None);
  update.set_semantic_active = Some(false);
  if update.has_changes() {
    actions.push(PollAction::SessionUpdate(update));
  }
  actions.push(PollAction::SemanticFlush(ChatFlushMeta {
    terminal_id: snapshot.terminal_id.clone(),
    member_id: snapshot.member_id.clone(),
    workspace_id: snapshot.workspace_id.clone(),
    terminal_type: snapshot.terminal_type.clone(),
    last_output_at,
    chat_pending_since: snapshot.chat_pending_since,
    silence_ms,
  }));
}
