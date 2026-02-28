//! 分发解析阶段：确定投递目标与渠道。

use super::types::{DispatchPlan, MessageEnvelope};

pub(crate) fn plan_terminal(_envelope: &MessageEnvelope) -> Result<DispatchPlan, String> {
  // [TODO/message-service, 2026-01-26] 生成投递计划（目标、渠道、去向）。
  Ok(DispatchPlan {
    should_deliver: true,
  })
}
