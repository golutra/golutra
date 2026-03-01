//! 消息服务流水线：按分层顺序执行业务投递链路。

mod dispatch;
mod normalize;
mod policy;
mod reliability;
mod throttle;
mod types;

use crate::contracts::terminal_message::TerminalMessagePayload;
use crate::ports::message_service::{
  TerminalMessageAppendResult, TerminalMessageRepository, TerminalMessageTransport,
};

use dispatch::plan_terminal;
use normalize::normalize_terminal;
use policy::evaluate_terminal;
use reliability::{deliver_terminal_final, deliver_terminal_stream};
use throttle::apply_terminal;

pub(crate) fn process_terminal_stream(
  transport: &dyn TerminalMessageTransport,
  repository: &dyn TerminalMessageRepository,
  payload: TerminalMessagePayload,
) -> Result<(), String> {
  let envelope = normalize_terminal(payload)?;
  let plan = plan_terminal(&envelope)?;
  let policy = evaluate_terminal(&envelope, &plan)?;
  let throttle = apply_terminal(&envelope, &plan, &policy)?;
  deliver_terminal_stream(transport, repository, &envelope, &plan, &policy, &throttle)
}

pub(crate) fn process_terminal_final(
  transport: &dyn TerminalMessageTransport,
  repository: &dyn TerminalMessageRepository,
  payload: TerminalMessagePayload,
) -> Result<TerminalMessageAppendResult, String> {
  let envelope = normalize_terminal(payload)?;
  let plan = plan_terminal(&envelope)?;
  let policy = evaluate_terminal(&envelope, &plan)?;
  let throttle = apply_terminal(&envelope, &plan, &policy)?;
  deliver_terminal_final(transport, repository, &envelope, &plan, &policy, &throttle)
}
