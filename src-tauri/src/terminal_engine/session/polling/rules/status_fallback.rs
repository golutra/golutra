//! Working 状态回落规则：基于静默时长与门禁条件判断。

use super::super::actions::{PollAction, SessionUpdate};
use super::super::snapshot::SessionPollSnapshot;
use super::super::super::{
  TerminalSessionStatus, STATUS_IDLE_DEBOUNCE_MS, STATUS_WORKING_SILENCE_TIMEOUT_MS,
};

pub(crate) fn collect_status_fallback_actions(
  snapshot: &SessionPollSnapshot,
  now: u64,
  actions: &mut Vec<PollAction>,
) {
  if snapshot.status != TerminalSessionStatus::Working {
    return;
  }
  if snapshot.status_locked {
    if snapshot.idle_candidate_at.is_some() {
      let mut update = SessionUpdate::new(&snapshot.terminal_id);
      update.set_idle_candidate_at = Some(None);
      actions.push(PollAction::SessionUpdate(update));
    }
    return;
  }
  let mut update = SessionUpdate::new(&snapshot.terminal_id);
  let Some(last_activity) = snapshot.last_activity_at else {
    if snapshot.idle_candidate_at.is_some() {
      update.set_idle_candidate_at = Some(None);
    }
    if update.has_changes() {
      actions.push(PollAction::SessionUpdate(update));
    }
    return;
  };
  let silence_ms = now.saturating_sub(last_activity);
  if silence_ms < STATUS_WORKING_SILENCE_TIMEOUT_MS {
    if snapshot.idle_candidate_at.is_some() {
      update.set_idle_candidate_at = Some(None);
    }
    if update.has_changes() {
      actions.push(PollAction::SessionUpdate(update));
    }
    return;
  }
  let allow_fallback = snapshot.ui_active || !snapshot.flow_paused;
  if !allow_fallback {
    // 流控暂停或 UI 未就绪时，避免误判回落。
    if snapshot.idle_candidate_at.is_some() {
      update.set_idle_candidate_at = Some(None);
    }
    if update.has_changes() {
      actions.push(PollAction::SessionUpdate(update));
    }
    return;
  }
  match snapshot.idle_candidate_at {
    None => {
      // 首次命中静默门禁，先记录候选时间做防抖。
      update.set_idle_candidate_at = Some(Some(now));
    }
    Some(candidate_at) => {
      if now.saturating_sub(candidate_at) >= STATUS_IDLE_DEBOUNCE_MS {
        // 静默持续到达防抖门槛后才回落。
        update.set_idle_candidate_at = Some(None);
        update.set_status = Some(TerminalSessionStatus::Online);
        update.emit_status = true;
      }
    }
  }
  if update.has_changes() {
    actions.push(PollAction::SessionUpdate(update));
  }
}
