//! 终端好友邀请编排：接收创建上下文并记录关键元信息。

use serde_json::json;
use tauri::{AppHandle, Manager};

use crate::platform::{diagnostics_log_backend_event, DiagnosticsState};
use crate::terminal_engine::session::TerminalManager;
use crate::terminal_engine::default_members::{
  resolve_default_command_for_invite, resolve_default_member,
};

#[derive(Clone)]
pub(crate) struct TerminalFriendInvitePayload {
  pub(crate) terminal_id: String,
  pub(crate) member_id: Option<String>,
  pub(crate) workspace_id: Option<String>,
  pub(crate) terminal_type: String,
  pub(crate) member_name: Option<String>,
  pub(crate) default_command: Option<String>,
  pub(crate) instance_count: Option<u32>,
  pub(crate) unlimited_access: Option<bool>,
  pub(crate) sandboxed: Option<bool>,
  pub(crate) default_terminal_name: Option<String>,
  pub(crate) default_terminal_path: Option<String>,
  pub(crate) post_ready_enabled: bool,
}

fn has_invite_meta(payload: &TerminalFriendInvitePayload) -> bool {
  payload.post_ready_enabled
}

/// 创建完成后的后续流程入口，失败时由调用方决定是否保持 Connecting。
pub(crate) fn run_post_create_flow(
  app: &AppHandle,
  payload: TerminalFriendInvitePayload,
) -> Result<(), String> {
  after_terminal_friend_create(app, payload);
  Ok(())
}

/// 启动终端好友创建后的编排流程，并在需要时接管状态门禁。
/// 返回：是否进入后续流程（true 表示已接管状态门禁）。
pub(crate) fn start_friend_flow(
  app: &AppHandle,
  _manager: &TerminalManager,
  payload: TerminalFriendInvitePayload,
) -> bool {
  // 步骤说明：
  // 1) 未开启 post_ready 或缺少 member_id 时直接跳过后置流程，不接管门禁。
  // 2) 开启 post_ready 时在创建前已预锁，本处不再加锁，仅执行后置流程。
  // 3) 异步执行创建后的后置流程（当前仅诊断记录）。
  // 4) 解锁交给 post_ready 完成时统一处理，确保连接态覆盖完整启动周期。
  if payload.member_id.is_none() || !has_invite_meta(&payload) {
    after_terminal_friend_create(app, payload);
    return false;
  }
  let member_id_for_log = payload.member_id.clone();
  let workspace_id_for_log = payload.workspace_id.clone();
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    if let Err(err) = run_post_create_flow(&app, payload) {
      log::warn!(
        "terminal friend post create flow failed member_id={:?} workspace_id={:?} err={}",
        member_id_for_log,
        workspace_id_for_log,
        err
      );
    }
  });
  true
}

pub(crate) fn after_terminal_friend_create(app: &AppHandle, payload: TerminalFriendInvitePayload) {
  if !has_invite_meta(&payload) {
    return;
  }
  let resolved_member = resolve_default_member(payload.terminal_type.as_str());
  let resolved_default_command = resolve_default_command_for_invite(
    payload.terminal_type.as_str(),
    payload.default_command.clone(),
    payload.unlimited_access.unwrap_or(false),
  );
  let resolved_member_id = resolved_member.map(|member| member.id.to_string());
  diagnostics_log_backend_event(
    &app.state::<DiagnosticsState>(),
    payload.member_id.clone(),
    Some(payload.terminal_id.clone()),
    None,
    None,
    payload.workspace_id.clone(),
    "terminal_friend_invite",
    json!({
      "terminalId": payload.terminal_id,
      "memberId": payload.member_id,
      "workspaceId": payload.workspace_id,
      "terminalType": payload.terminal_type,
      "memberName": payload.member_name,
      "defaultCommand": payload.default_command,
      "defaultTerminalName": payload.default_terminal_name,
      "defaultTerminalPath": payload.default_terminal_path,
      "resolvedDefaultMemberId": resolved_member_id,
      "resolvedDefaultCommand": resolved_default_command,
      "instanceCount": payload.instance_count,
      "unlimitedAccess": payload.unlimited_access,
      "sandboxed": payload.sandboxed
    }),
  );
}
