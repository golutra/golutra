//! 投递可靠性阶段：队列化、重试与失败补偿。

use tauri::{AppHandle, Emitter, Manager};

use crate::message_service::chat_db::{chat_append_terminal_message, ChatDbManager};

use super::types::{DispatchPlan, MessageEnvelope, PolicyDecision, ThrottleDecision};

pub(crate) fn deliver_terminal_stream(
  app: &AppHandle,
  envelope: &MessageEnvelope,
  plan: &DispatchPlan,
  policy: &PolicyDecision,
  throttle: &ThrottleDecision,
) -> Result<(), String> {
  if !plan.should_deliver || !policy.allowed || !throttle.allowed {
    return Ok(());
  }
  // [TODO/message-service, 2026-01-26] 引入可靠队列与重试策略后再切换到真实投递。
  let _ = app.emit("terminal-message-stream", envelope.payload.clone());
  Ok(())
}

pub(crate) fn deliver_terminal_final(
  app: &AppHandle,
  envelope: &MessageEnvelope,
  plan: &DispatchPlan,
  policy: &PolicyDecision,
  throttle: &ThrottleDecision,
) -> Result<(), String> {
  if !plan.should_deliver || !policy.allowed || !throttle.allowed {
    return Ok(());
  }
  let payload = &envelope.payload;
  let (Some(workspace_id), Some(conversation_id), Some(member_id), Some(viewer_id)) = (
    payload.workspace_id.as_ref(),
    payload.conversation_id.as_ref(),
    payload.member_id.as_ref(),
    payload.sender_id.as_ref(),
  ) else {
    return Ok(());
  };
  let chat_state = app.state::<ChatDbManager>();
  // [TODO/message-service, 2026-01-26] 接入重试/死信队列，避免持久化失败后丢消息。
  chat_append_terminal_message(
    app,
    chat_state.inner(),
    workspace_id,
    conversation_id,
    member_id,
    payload.content.clone(),
    viewer_id,
  )
}
