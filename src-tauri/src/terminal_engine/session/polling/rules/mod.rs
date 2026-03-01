//! 轮询规则集合：按会话快照生成动作列表。

use super::actions::PollAction;
use super::rule_mask::RuleMask;
use super::snapshot::SessionPollSnapshot;

mod post_ready;
mod semantic_flush;
mod status_fallback;

#[derive(Clone, Copy, Debug)]
enum RuleKind {
  StatusFallback,
  SemanticFlush,
  PostReady,
}

pub(crate) fn build_poll_actions(
  snapshots: &[SessionPollSnapshot],
  now: u64,
  mask: RuleMask,
) -> Vec<PollAction> {
  let mut actions = Vec::new();
  if mask.is_empty() {
    return actions;
  }
  let rules = resolve_rules(mask);
  for snapshot in snapshots {
    for rule in rules.iter() {
      match rule {
        RuleKind::StatusFallback => {
          status_fallback::collect_status_fallback_actions(snapshot, now, &mut actions);
        }
        RuleKind::SemanticFlush => {
          semantic_flush::collect_semantic_flush_actions(snapshot, now, &mut actions);
        }
        RuleKind::PostReady => {
          post_ready::collect_post_ready_actions(snapshot, now, &mut actions);
        }
      }
    }
  }
  actions
}

fn resolve_rules(mask: RuleMask) -> Vec<RuleKind> {
  let mut rules = Vec::new();
  if mask.contains(RuleMask::STATUS_FALLBACK) {
    rules.push(RuleKind::StatusFallback);
  }
  if mask.contains(RuleMask::SEMANTIC_FLUSH) {
    rules.push(RuleKind::SemanticFlush);
  }
  if mask.contains(RuleMask::POST_READY) {
    rules.push(RuleKind::PostReady);
  }
  rules
}
