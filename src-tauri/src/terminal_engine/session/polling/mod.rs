//! 轮询分层：采集快照 -> 规则判定 -> 动作执行。

pub(crate) mod actions;
pub(crate) mod collector;
pub(crate) mod dispatcher;
pub(crate) mod rules;
pub(crate) mod rule_mask;
pub(crate) mod snapshot;

pub(crate) use collector::{collect_poll_snapshots_by_ids, collect_poll_snapshots_by_working_set};
pub(crate) use dispatcher::dispatch_poll_actions;
pub(crate) use rules::build_poll_actions;
pub(crate) use rule_mask::RuleMask;
