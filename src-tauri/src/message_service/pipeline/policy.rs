//! 权限与策略阶段：DND、优先级与例外规则。

use super::types::{DispatchPlan, MessageEnvelope, PolicyDecision};

pub(crate) fn evaluate_terminal(
  _envelope: &MessageEnvelope,
  _plan: &DispatchPlan,
) -> Result<PolicyDecision, String> {
  // [TODO/message-service, 2026-01-26] 权限校验、DND 与 @ 提及范围规则。
  Ok(PolicyDecision {
    allowed: true,
  })
}
