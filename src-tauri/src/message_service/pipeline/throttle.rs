//! 节流与配额阶段：统一限速与削峰逻辑。

use super::types::{DispatchPlan, MessageEnvelope, PolicyDecision, ThrottleDecision};

pub(crate) fn apply_terminal(
  _envelope: &MessageEnvelope,
  _plan: &DispatchPlan,
  _policy: &PolicyDecision,
) -> Result<ThrottleDecision, String> {
  // [TODO/message-service, 2026-01-26] 会话/成员/频道维度限流与配额。
  Ok(ThrottleDecision {
    allowed: true,
  })
}
