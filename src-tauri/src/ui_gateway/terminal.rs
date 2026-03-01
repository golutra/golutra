//! UI 终端命令包装：集中承载对终端引擎的 IPC 暴露。

use std::fs::{self, File};
use std::io::Write;

use tauri::{AppHandle, State, WebviewWindow};

use crate::now_millis;
use crate::platform::resolve_log_dir;
use crate::runtime::{list_terminal_environments, TerminalEnvironmentOption};
use crate::terminal_engine::models::{TerminalSnapshotLinesPayload, TerminalSnapshotPayload, TerminalStatusPayload};
use crate::terminal_engine::session::{self, TerminalDispatchContext};
use crate::terminal_engine::TerminalManager;

#[tauri::command]
pub(crate) fn terminal_create(
  app: AppHandle,
  window: WebviewWindow,
  state: State<'_, TerminalManager>,
  cols: Option<u16>,
  rows: Option<u16>,
  cwd: Option<String>,
  member_id: Option<String>,
  workspace_id: Option<String>,
  keep_alive: Option<bool>,
  terminal_id: Option<String>,
  terminal_type: Option<String>,
  terminal_command: Option<String>,
  terminal_path: Option<String>,
  strict_shell: Option<bool>,
  post_ready_mode: Option<String>,
  member_name: Option<String>,
  default_command: Option<String>,
  invite_instance_count: Option<u32>,
  invite_unlimited_access: Option<bool>,
  invite_sandboxed: Option<bool>,
) -> Result<String, String> {
  session::terminal_create(
    app,
    window,
    state,
    cols,
    rows,
    cwd,
    member_id,
    workspace_id,
    keep_alive,
    terminal_id,
    terminal_type,
    terminal_command,
    terminal_path,
    strict_shell,
    post_ready_mode,
    member_name,
    default_command,
    invite_instance_count,
    invite_unlimited_access,
    invite_sandboxed,
  )
}

#[tauri::command]
pub(crate) fn terminal_list_environments() -> Vec<TerminalEnvironmentOption> {
  list_terminal_environments()
}

#[tauri::command]
pub(crate) fn terminal_attach(
  window: WebviewWindow,
  state: State<'_, TerminalManager>,
  terminal_id: String,
) -> Result<TerminalSnapshotPayload, String> {
  session::terminal_attach(window, state, terminal_id)
}

#[tauri::command]
pub(crate) fn terminal_snapshot_lines(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  terminal_id: String,
) -> Result<TerminalSnapshotLinesPayload, String> {
  session::terminal_snapshot_lines(app, state, terminal_id)
}

#[tauri::command]
pub(crate) fn terminal_snapshot_text(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  terminal_id: String,
) -> Result<TerminalSnapshotPayload, String> {
  session::terminal_snapshot_text(app, state, terminal_id)
}

#[tauri::command]
pub(crate) fn terminal_dump_snapshot_lines(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  terminal_id: String,
) -> Result<String, String> {
  if !snapshot_dump_enabled() {
    return Err("snapshot dump requires GOLUTRA_Backend_DEBUG > 0".to_string());
  }
  let payload = session::terminal_snapshot_lines(app, state, terminal_id.clone())?;
  let log_dir = resolve_log_dir();
  fs::create_dir_all(&log_dir)
    .map_err(|err| format!("failed to create log dir: {err}"))?;
  let timestamp = now_millis().unwrap_or(0);
  let file_name = format!("terminal-snapshot-{}-{}.log", terminal_id, timestamp);
  let log_path = log_dir.join(file_name);
  let mut file = File::create(&log_path)
    .map_err(|err| format!("failed to create snapshot log: {err}"))?;
  // 只写可读快照，避免 UI 侧再做二次格式化。
  writeln!(
    file,
    "# terminal_id={} seq={} line_count={}",
    payload.terminal_id,
    payload.seq,
    payload.lines.len()
  )
  .map_err(|err| format!("failed to write snapshot header: {err}"))?;
  for (index, line) in payload.lines.iter().enumerate() {
    let sanitized = line.replace('\t', "\\t").replace('\r', "\\r");
    writeln!(file, "{:04} | {}", index.saturating_add(1), sanitized)
      .map_err(|err| format!("failed to write snapshot line: {err}"))?;
  }
  Ok(log_path.to_string_lossy().to_string())
}

fn snapshot_dump_enabled() -> bool {
  std::env::var("GOLUTRA_Backend_DEBUG")
    .ok()
    .and_then(|value| value.trim().parse::<f64>().ok())
    .map(|value| value > 0.0)
    .unwrap_or(false)
}

#[tauri::command]
pub(crate) fn terminal_write(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  terminal_id: String,
  data: String,
) -> Result<(), String> {
  session::terminal_write(app, state, terminal_id, data)
}

#[tauri::command]
pub(crate) fn terminal_set_active(
  state: State<'_, TerminalManager>,
  terminal_id: String,
  active: bool,
) -> Result<(), String> {
  session::terminal_set_active(state, terminal_id, active)
}

#[tauri::command]
pub(crate) fn terminal_emit_status(
  state: State<'_, TerminalManager>,
  terminal_id: String,
) -> Result<(), String> {
  session::terminal_emit_status(state, terminal_id)
}

#[tauri::command]
pub(crate) fn terminal_set_member_status(
  state: State<'_, TerminalManager>,
  member_id: String,
  status: String,
) -> Result<(), String> {
  session::terminal_set_member_status(state, member_id, status)
}

#[tauri::command]
pub(crate) fn terminal_list_statuses(
  state: State<'_, TerminalManager>,
  workspace_id: Option<String>,
) -> Vec<TerminalStatusPayload> {
  session::terminal_list_statuses(state, workspace_id)
}

#[tauri::command]
pub(crate) fn terminal_ack(
  state: State<'_, TerminalManager>,
  terminal_id: String,
  count: usize,
) -> Result<(), String> {
  session::terminal_ack(state, terminal_id, count)
}

#[tauri::command]
pub(crate) fn terminal_dispatch(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  terminal_id: String,
  data: String,
  context: TerminalDispatchContext,
) -> Result<(), String> {
  session::terminal_dispatch(app, state, terminal_id, data, context)
}

#[tauri::command]
pub(crate) fn terminal_resize(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  terminal_id: String,
  cols: u16,
  rows: u16,
) -> Result<(), String> {
  session::terminal_resize(app, state, terminal_id, cols, rows)
}

#[tauri::command]
pub(crate) fn terminal_close(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  terminal_id: String,
  preserve: Option<bool>,
  delete_session_map: Option<bool>,
) -> Result<(), String> {
  session::terminal_close(app, state, terminal_id, preserve, delete_session_map)
}
