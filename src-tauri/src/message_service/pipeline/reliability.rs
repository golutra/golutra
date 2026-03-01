//! 投递可靠性阶段：队列化、重试与失败补偿。

use crate::ports::message_service::{
  TerminalMessageAppendResult, TerminalMessageRepository, TerminalMessageTransport,
};

use super::types::{DispatchPlan, MessageEnvelope, PolicyDecision, ThrottleDecision};

pub(crate) fn deliver_terminal_stream(
  transport: &dyn TerminalMessageTransport,
  _repository: &dyn TerminalMessageRepository,
  envelope: &MessageEnvelope,
  plan: &DispatchPlan,
  policy: &PolicyDecision,
  throttle: &ThrottleDecision,
) -> Result<(), String> {
  if !plan.should_deliver || !policy.allowed || !throttle.allowed {
    return Ok(());
  }
  // [TODO/message-service, 2026-01-26] 引入可靠队列与重试策略后再切换到真实投递。
  transport.emit_terminal_stream(envelope.payload.clone())?;
  Ok(())
}

pub(crate) fn deliver_terminal_final(
  _transport: &dyn TerminalMessageTransport,
  repository: &dyn TerminalMessageRepository,
  envelope: &MessageEnvelope,
  plan: &DispatchPlan,
  policy: &PolicyDecision,
  throttle: &ThrottleDecision,
) -> Result<TerminalMessageAppendResult, String> {
  if !plan.should_deliver || !policy.allowed || !throttle.allowed {
    return Ok(TerminalMessageAppendResult::skipped());
  }
  let payload = &envelope.payload;
  let missing: Vec<&str> = [
    ("workspaceId", payload.workspace_id.as_ref()),
    ("conversationId", payload.conversation_id.as_ref()),
    ("memberId", payload.member_id.as_ref()),
    ("senderId", payload.sender_id.as_ref()),
  ]
  .into_iter()
  .filter_map(|(name, value)| if value.is_none() { Some(name) } else { None })
  .collect();
  if !missing.is_empty() {
    let missing_for_log: Vec<String> = missing.iter().map(|item| item.to_string()).collect();
    log::warn!(
      "terminal message drop terminal_id={} missing={:?}",
      payload.terminal_id,
      missing_for_log
    );
    return Ok(TerminalMessageAppendResult::skipped());
  }
  let (workspace_id, conversation_id, member_id, viewer_id) = (
    payload.workspace_id.as_ref().unwrap(),
    payload.conversation_id.as_ref().unwrap(),
    payload.member_id.as_ref().unwrap(),
    payload.sender_id.as_ref().unwrap(),
  );
  // [TODO/message-service, 2026-01-26] 接入重试/死信队列，避免持久化失败后丢消息。
  repository.append_terminal_message(
    workspace_id,
    conversation_id,
    member_id,
    payload.content.clone(),
    viewer_id,
    payload.span_id.as_deref(),
  )
}
