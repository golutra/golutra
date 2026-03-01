//! 轮询采集器：负责在锁内提取最小快照。

use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

use super::snapshot::SessionPollSnapshot;
use super::super::{lock_sessions, SessionRegistry};

pub(crate) fn collect_poll_snapshots_by_ids(
  sessions: &Arc<Mutex<SessionRegistry>>,
  terminal_ids: &[String],
) -> Vec<SessionPollSnapshot> {
  let guard = lock_sessions(sessions);
  terminal_ids
    .iter()
    .filter_map(|terminal_id| guard.sessions.get(terminal_id))
    .filter(|session| session.active)
    .map(build_snapshot)
    .collect()
}

pub(crate) fn collect_poll_snapshots_by_working_set(
  sessions: &Arc<Mutex<SessionRegistry>>,
) -> Vec<SessionPollSnapshot> {
  let guard = lock_sessions(sessions);
  let working_ids = match guard.working_sessions.lock() {
    Ok(value) => value.iter().cloned().collect::<Vec<_>>(),
    Err(err) => err.into_inner().iter().cloned().collect::<Vec<_>>(),
  };
  working_ids
    .into_iter()
    .filter_map(|terminal_id| guard.sessions.get(&terminal_id))
    .filter(|session| session.active)
    .map(build_snapshot)
    .collect()
}

fn build_snapshot(session: &super::super::TerminalSession) -> SessionPollSnapshot {
  SessionPollSnapshot {
    terminal_id: session.id.clone(),
    terminal_type: session.terminal_type.as_str().to_string(),
    status: session.status,
    status_locked: session.status_locked,
    ui_active: session.ui_active,
    flow_paused: session.flow_paused,
    shell_ready: session.shell_ready,
    post_ready_state: session.post_ready_state,
    post_ready_mode: session.post_ready_mode,
    post_ready_restart_pending: session.post_ready_restart_pending,
    last_activity_at: session.last_activity_at,
    last_output_at: session.last_output_at,
    idle_candidate_at: session.idle_candidate_at,
    chat_pending: session.chat_pending,
    chat_pending_since: session.chat_pending_since,
    chat_candidate_at: session.chat_candidate_at,
    pending_output_chunks: session.pending_output_chunks.load(Ordering::SeqCst),
    member_id: session.member_id.clone(),
    workspace_id: session.workspace_id.clone(),
  }
}
