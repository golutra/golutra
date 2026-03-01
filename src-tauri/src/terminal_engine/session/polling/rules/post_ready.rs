//! post_ready 规则：根据稳定性与流程状态触发后置步骤。

use super::super::super::{PostReadyMode, PostReadyState};
use super::super::actions::PollAction;
use super::super::snapshot::SessionPollSnapshot;

pub(crate) fn collect_post_ready_actions(
    snapshot: &SessionPollSnapshot,
    _now: u64,
    actions: &mut Vec<PollAction>,
) {
    if snapshot.post_ready_restart_pending {
        actions.push(PollAction::PostReadyRestart {
            terminal_id: snapshot.terminal_id.clone(),
        });
        return;
    }
    if snapshot.post_ready_state == PostReadyState::Starting {
        actions.push(PollAction::PostReadyStep {
            terminal_id: snapshot.terminal_id.clone(),
        });
    }
    if snapshot.post_ready_mode == PostReadyMode::Invite
        && snapshot.post_ready_state == PostReadyState::Idle
        && snapshot.shell_ready
    {
        actions.push(PollAction::PostReadyStart {
            terminal_id: snapshot.terminal_id.clone(),
            terminal_type: snapshot.terminal_type.clone(),
        });
    }
}
