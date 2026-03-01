//! UI 诊断命令包装：集中承载监控相关的 IPC 暴露。

use serde_json::Value;
use tauri::{AppHandle, State};

use crate::platform::monitoring::diagnostics::{self, DiagnosticsState, FrontendLogEntry};

#[tauri::command]
pub(crate) fn diagnostics_start_run(
  app: AppHandle,
  state: State<'_, DiagnosticsState>,
  run_id: String,
  workspace_id: Option<String>,
  label: Option<String>,
) -> Result<(), String> {
  diagnostics::diagnostics_start_run(app, state, run_id, workspace_id, label)
}

#[tauri::command]
pub(crate) fn diagnostics_end_run(
  state: State<'_, DiagnosticsState>,
  run_id: String,
  status: Option<String>,
) -> Result<(), String> {
  diagnostics::diagnostics_end_run(state, run_id, status)
}

#[tauri::command]
pub(crate) fn diagnostics_register_member(
  state: State<'_, DiagnosticsState>,
  run_id: String,
  member_id: String,
  terminal_type: Option<String>,
  terminal_command: Option<String>,
  name: Option<String>,
) -> Result<(), String> {
  diagnostics::diagnostics_register_member(state, run_id, member_id, terminal_type, terminal_command, name)
}

#[tauri::command]
pub(crate) fn diagnostics_register_session(
  state: State<'_, DiagnosticsState>,
  run_id: String,
  terminal_id: String,
  member_id: Option<String>,
) -> Result<(), String> {
  diagnostics::diagnostics_register_session(state, run_id, terminal_id, member_id)
}

#[tauri::command]
pub(crate) fn diagnostics_register_conversation(
  state: State<'_, DiagnosticsState>,
  run_id: String,
  conversation_id: String,
  member_id: Option<String>,
) -> Result<(), String> {
  diagnostics::diagnostics_register_conversation(state, run_id, conversation_id, member_id)
}

#[tauri::command]
pub(crate) fn diagnostics_register_window(
  state: State<'_, DiagnosticsState>,
  run_id: String,
  window_label: String,
) -> Result<(), String> {
  diagnostics::diagnostics_register_window(state, run_id, window_label)
}

#[tauri::command]
pub(crate) fn diagnostics_log_frontend_event(
  state: State<'_, DiagnosticsState>,
  run_id: String,
  round: Option<u8>,
  member_id: Option<String>,
  step_id: String,
  payload: Value,
  client_ts: Option<u64>,
  seq: Option<u64>,
) -> Result<(), String> {
  diagnostics::diagnostics_log_frontend_event(
    state,
    run_id,
    round,
    member_id,
    step_id,
    payload,
    client_ts,
    seq,
  )
}

#[tauri::command]
pub(crate) fn diagnostics_log_frontend_batch(
  state: State<'_, DiagnosticsState>,
  entries: Vec<FrontendLogEntry>,
) -> Result<(), String> {
  diagnostics::diagnostics_log_frontend_batch(state, entries)
}

#[tauri::command]
pub(crate) fn diagnostics_log_snapshot_triplet(
  state: State<'_, DiagnosticsState>,
  run_id: String,
  round: u8,
  member_id: String,
  front_before: Vec<String>,
  backend_stored: Vec<String>,
  front_reopen: Vec<String>,
) -> Result<(), String> {
  diagnostics::diagnostics_log_snapshot_triplet(
    state,
    run_id,
    round,
    member_id,
    front_before,
    backend_stored,
    front_reopen,
  )
}

#[tauri::command]
pub(crate) fn diagnostics_log_chat_consistency(
  state: State<'_, DiagnosticsState>,
  run_id: String,
  round: u8,
  member_id: String,
  reply_text: Option<String>,
  reply_missing: Option<bool>,
  terminal_lines: Vec<String>,
) -> Result<(), String> {
  diagnostics::diagnostics_log_chat_consistency(
    state,
    run_id,
    round,
    member_id,
    reply_text,
    reply_missing,
    terminal_lines,
  )
}
