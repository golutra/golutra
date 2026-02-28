//! 消息流水线类型：承载规范化与投递决策数据。

use crate::terminal_engine::models::TerminalMessagePayload;

#[derive(Clone)]
pub(crate) struct MessageEnvelope {
  pub(crate) payload: TerminalMessagePayload,
}

#[derive(Clone)]
pub(crate) struct DispatchPlan {
  pub(crate) should_deliver: bool,
}

#[derive(Clone)]
pub(crate) struct PolicyDecision {
  pub(crate) allowed: bool,
}

#[derive(Clone)]
pub(crate) struct ThrottleDecision {
  pub(crate) allowed: bool,
}
