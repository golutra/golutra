use std::{
  any::Any,
  collections::{HashMap, VecDeque},
  io::{Read, Write},
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc, Mutex,
  },
  thread,
  time::{Duration, Instant},
};

use portable_pty::Child;
use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager, State, Window};
use ulid::Ulid;

use super::emulator::{create_emulator_with_writer, EmulatorConfig, TerminalEmulator};
use super::models::{
  TerminalErrorPayload, TerminalExitPayload, TerminalOutputPayload, TerminalSnapshotPayload,
  TerminalStatusPayload,
};
use super::pty::{lookup_binary, resize_pty, spawn_command, spawn_shell, TerminalHandle};
use super::semantic::{
  build_snapshot_payload, extract_command_from_input, SemanticState, TerminalChatContext,
};
use crate::chat_db::{chat_append_terminal_message, ChatDbManager};
use crate::now_millis;

const WORKING_SILENCE_TIMEOUT_MS: u64 = 2000;
const STATUS_POLL_INTERVAL_MS: u64 = 500;
const SESSION_SCROLLBACK_LINES: usize = 2000;
const SHIM_READY_SIGNAL: &str = "\x1b]633;A";
const SHIM_LAUNCH_ERROR_MARKER: &str = "SHIM_LAUNCH_ERROR";
const OSC_COMMAND_FINISHED_PREFIX: &str = "\x1b]633;D;";
const SHELL_READY_TIMEOUT_MS: u64 = 3000;
const SHELL_READY_ACTIVITY_MS: u64 = 500;
const SHELL_READY_ACTIVITY_BYTES: usize = 1024;
const OUTPUT_EMIT_INTERVAL_MS: u64 = 16;
const OUTPUT_EMIT_MAX_BYTES: usize = 64 * 1024;
const SEMANTIC_EMIT_INTERVAL_MS: u64 = 50;
const SEMANTIC_EMIT_MAX_BYTES: usize = 128 * 1024;
const STATS_LOG_INTERVAL_MS: u64 = 1000;
const FLOW_CONTROL_HIGH_WATERMARK: usize = 200_000;
const FLOW_CONTROL_LOW_WATERMARK: usize = 20_000;
const FLOW_CONTROL_SLEEP_MS: u64 = 2;

static SESSION_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerminalSessionStatus {
  Online,
  Working,
  Offline,
}

impl TerminalSessionStatus {
  fn as_str(&self) -> &'static str {
    match self {
      TerminalSessionStatus::Online => "online",
      TerminalSessionStatus::Working => "working",
      TerminalSessionStatus::Offline => "offline",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerminalType {
  Shell,
  Codex,
  Gemini,
  Claude,
}

impl TerminalType {
  fn from_str(value: Option<&str>) -> Option<Self> {
    match value?.trim().to_lowercase().as_str() {
      "shell" => Some(TerminalType::Shell),
      "codex" => Some(TerminalType::Codex),
      "gemini" => Some(TerminalType::Gemini),
      "claude" => Some(TerminalType::Claude),
      _ => None,
    }
  }

  fn as_str(&self) -> &'static str {
    match self {
      TerminalType::Shell => "shell",
      TerminalType::Codex => "codex",
      TerminalType::Gemini => "gemini",
      TerminalType::Claude => "claude",
    }
  }

  fn default_binary(&self) -> Option<&'static str> {
    match self {
      TerminalType::Shell => None,
      TerminalType::Codex => Some("codex"),
      TerminalType::Gemini => Some("gemini"),
      TerminalType::Claude => Some("claude"),
    }
  }
}

struct TerminalSnapshot {
  emulator: Box<dyn TerminalEmulator>,
}

impl TerminalSnapshot {
  fn new(rows: u16, cols: u16, writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>) -> Self {
    let response_writer = writer.map(|writer| Box::new(PtyResponseWriter { writer }) as Box<dyn Write + Send>);
    let emulator = create_emulator_with_writer(
      EmulatorConfig {
        rows,
        cols,
        scrollback_limit: SESSION_SCROLLBACK_LINES,
      },
      response_writer,
    );
    Self { emulator }
  }

  fn apply_output(&mut self, bytes: &[u8]) {
    self.emulator.apply_output(bytes);
  }

  fn set_size(&mut self, rows: u16, cols: u16) {
    self.emulator.set_size(rows, cols);
  }

  fn snapshot(&self) -> Vec<u8> {
    self.emulator.snapshot_ansi()
  }

}

struct PtyResponseWriter {
  writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl Write for PtyResponseWriter {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let mut writer = self
      .writer
      .lock()
      .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "terminal writer lock poisoned"))?;
    writer.write(buf)
  }

  fn flush(&mut self) -> std::io::Result<()> {
    let mut writer = self
      .writer
      .lock()
      .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "terminal writer lock poisoned"))?;
    writer.flush()
  }
}

fn resolve_terminal_type(raw: Option<&str>, command: Option<&str>) -> TerminalType {
  if let Some(value) = TerminalType::from_str(raw) {
    return value;
  }
  let command = command.unwrap_or("").trim();
  if command.is_empty() {
    return TerminalType::Shell;
  }
  let mut parts = command.split_whitespace();
  let binary = parts.next().unwrap_or("").to_lowercase();
  if parts.next().is_none() {
    match binary.as_str() {
      "codex" => return TerminalType::Codex,
      "gemini" => return TerminalType::Gemini,
      "claude" => return TerminalType::Claude,
      _ => {}
    }
  }
  TerminalType::Shell
}

pub(crate) struct TerminalSession {
  pub(crate) id: String,
  terminal_type: TerminalType,
  status: TerminalSessionStatus,
  pub(crate) output_bytes_total: u64,
  pub(crate) output_seq: u64,
  pub(crate) unacked_bytes: usize,
  pub(crate) screen_rows: u16,
  pub(crate) screen_cols: u16,
  pub(crate) member_id: Option<String>,
  pub(crate) workspace_id: Option<String>,
  pub(crate) active: bool,
  pub(crate) last_activity_at: Option<u64>,
  pub(crate) broken: bool,
  pub(crate) chat_pending: bool,
  pub(crate) ui_active: bool,
  pub(crate) handle: Option<TerminalHandle>,
  snapshot: TerminalSnapshot,
  semantic_tx: Option<mpsc::Sender<SemanticEvent>>,
  pub(crate) keep_alive: bool,
  pub(crate) owner_window_label: Option<String>,
  pub(crate) output_window_label: Option<String>,
  pub(crate) shell_ready: bool,
  pub(crate) created_at: u64,
  pub(crate) input_buffer: VecDeque<String>,
  pub(crate) ready_probe_bytes: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TerminalDispatchContext {
  pub(crate) conversation_id: String,
  pub(crate) conversation_type: String,
  pub(crate) sender_id: String,
  pub(crate) sender_name: String,
}

struct SessionRegistry {
  sessions: HashMap<String, TerminalSession>,
  member_statuses: HashMap<String, String>,
}

pub(crate) struct TerminalManager {
  sessions: Arc<Mutex<SessionRegistry>>,
}

impl Default for TerminalManager {
  fn default() -> Self {
    Self {
      sessions: Arc::new(Mutex::new(SessionRegistry {
        sessions: HashMap::new(),
        member_statuses: HashMap::new(),
      })),
    }
  }
}

fn lock_sessions<'a>(
  sessions: &'a Arc<Mutex<SessionRegistry>>,
) -> std::sync::MutexGuard<'a, SessionRegistry> {
  match sessions.lock() {
    Ok(guard) => guard,
    Err(err) => {
      log::warn!("terminal sessions lock poisoned; recovering");
      err.into_inner()
    }
  }
}

fn is_member_dnd(sessions: &Arc<Mutex<SessionRegistry>>, session_id: &str) -> bool {
  let guard = lock_sessions(sessions);
  let session = match guard.sessions.get(session_id) {
    Some(session) => session,
    None => return false,
  };
  let member_id = match session.member_id.as_ref() {
    Some(member_id) => member_id,
    None => return false,
  };
  matches!(guard.member_statuses.get(member_id).map(|status| status.as_str()), Some("dnd"))
}

fn get_flow_control_state(
  sessions: &Arc<Mutex<SessionRegistry>>,
  session_id: &str,
) -> Option<(usize, bool)> {
  let guard = lock_sessions(sessions);
  guard
    .sessions
    .get(session_id)
    .map(|session| (session.unacked_bytes, session.ui_active))
}

fn add_unacked_bytes(sessions: &Arc<Mutex<SessionRegistry>>, session_id: &str, count: usize) {
  if count == 0 {
    return;
  }
  let mut guard = lock_sessions(sessions);
  if let Some(session) = guard.sessions.get_mut(session_id) {
    session.unacked_bytes = session.unacked_bytes.saturating_add(count);
  }
}

fn subtract_unacked_bytes(sessions: &Arc<Mutex<SessionRegistry>>, session_id: &str, count: usize) {
  if count == 0 {
    return;
  }
  let mut guard = lock_sessions(sessions);
  if let Some(session) = guard.sessions.get_mut(session_id) {
    session.unacked_bytes = session.unacked_bytes.saturating_sub(count);
  }
}

enum SemanticEvent {
  Output(Vec<u8>),
  UserInput {
    data: String,
    context: TerminalChatContext,
  },
  Resize { rows: u16, cols: u16 },
  Flush { message_type: &'static str, source: &'static str },
  Shutdown,
}

fn spawn_semantic_worker(
  app: AppHandle,
  session_id: String,
  member_id: Option<String>,
  workspace_id: Option<String>,
  rows: u16,
  cols: u16,
) -> mpsc::Sender<SemanticEvent> {
  let (tx, rx) = mpsc::channel();
  thread::spawn(move || {
    let mut state = SemanticState::new(session_id, member_id, workspace_id, rows, cols);
    while let Ok(event) = rx.recv() {
      match event {
        SemanticEvent::Output(bytes) => {
          state.emulator.apply_output(&bytes);
        }
        SemanticEvent::UserInput { data, context } => {
          if !state.chat_block_pending {
            state.chat_block_pending = true;
            state.chat_context = Some(context);
            state.chat_span_id = Some(Ulid::new().to_string());
            state.chat_last_command = extract_command_from_input(&data);
          }
        }
        SemanticEvent::Resize { rows, cols } => {
          state.emulator.set_size(rows, cols);
        }
        SemanticEvent::Flush { message_type, source } => {
          if state.chat_block_pending {
            if let Some(payload) = build_snapshot_payload(&mut state, message_type, source) {
              if let (Some(workspace_id), Some(member_id), Some(context)) = (
                state.workspace_id.clone(),
                state.member_id.clone(),
                state.chat_context.clone(),
              ) {
                let chat_state = app.state::<ChatDbManager>();
                if let Err(err) = chat_append_terminal_message(
                  &app,
                  chat_state.inner(),
                  &workspace_id,
                  &context.conversation_id,
                  &member_id,
                  payload.content,
                  &context.sender_id,
                ) {
                  log::warn!("terminal chat append failed session_id={} err={}", state.session_id, err);
                }
              }
            }
            state.chat_block_pending = false;
            state.chat_context = None;
            state.chat_span_id = None;
            state.chat_last_command = None;
          }
        }
        SemanticEvent::Shutdown => break,
      }
    }
  });
  tx
}

fn mark_session_working_on_input(
  sessions: &Arc<Mutex<SessionRegistry>>,
  app: &AppHandle,
  session_id: &str,
  data: &str,
  chat_context: Option<TerminalChatContext>,
) {
  if !(data.contains('\n') || data.contains('\r')) {
    return;
  }
  let mut status_payload = None;
  let mut semantic_tx = None;
  {
    let mut guard = lock_sessions(sessions);
    if let Some(session) = guard.sessions.get_mut(session_id) {
      semantic_tx = session.semantic_tx.clone();
      if chat_context.is_some() {
        session.chat_pending = true;
      }
      if session.status != TerminalSessionStatus::Working {
        session.status = TerminalSessionStatus::Working;
        status_payload = Some(build_status_payload(session));
      }
      if let Ok(now) = now_millis() {
        session.last_activity_at = Some(now);
      }
    }
  }
  if let Some(payload) = status_payload {
    let _ = app.emit("terminal-status-change", payload);
  }
  if let (Some(tx), Some(context)) = (semantic_tx, chat_context) {
    let _ = tx.send(SemanticEvent::UserInput {
      data: data.to_string(),
      context,
    });
  }
}

pub(crate) fn shutdown_sessions(state: &TerminalManager) -> Result<(), String> {
  let mut killers = Vec::new();
  let mut semantic_shutdowns = Vec::new();
  {
    let mut guard = lock_sessions(&state.sessions);
    for session in guard.sessions.values_mut() {
      if let Some(handle) = session.handle.take() {
        killers.push(handle.killer);
      }
      if let Some(tx) = session.semantic_tx.take() {
        semantic_shutdowns.push(tx);
      }
      session.active = false;
      session.last_activity_at = None;
      if session.status != TerminalSessionStatus::Offline {
        session.status = TerminalSessionStatus::Offline;
      }
    }
  }
  for mut killer in killers {
    let _ = killer.kill();
  }
  for tx in semantic_shutdowns {
    let _ = tx.send(SemanticEvent::Shutdown);
  }
  Ok(())
}

fn build_status_payload(session: &TerminalSession) -> TerminalStatusPayload {
  TerminalStatusPayload {
    session_id: session.id.clone(),
    status: session.status.as_str().to_string(),
    member_id: session.member_id.clone(),
    workspace_id: session.workspace_id.clone(),
  }
}

fn detect_shell_ready(session: &mut TerminalSession, output: &str, now: u64) -> Option<String> {
  let mut became_ready = false;
  let mut reason = "unknown";
  if output.contains(SHIM_READY_SIGNAL) {
    became_ready = true;
    reason = "osc";
  } else {
    session.ready_probe_bytes = session.ready_probe_bytes.saturating_add(output.len());
    if now.saturating_sub(session.created_at) >= SHELL_READY_ACTIVITY_MS {
      if session.ready_probe_bytes >= SHELL_READY_ACTIVITY_BYTES || output.contains('\n') {
        became_ready = true;
        reason = "activity";
      }
    }
  }
  if became_ready {
    session.shell_ready = true;
    log::info!(
      "terminal shell ready session_id={} reason={}",
      session.id,
      reason
    );
  }
  if output.contains(SHIM_LAUNCH_ERROR_MARKER) {
    return Some(output.to_string());
  }
  None
}

fn append_command_finished_marker(data: &str, terminal_type: TerminalType) -> String {
  if terminal_type != TerminalType::Shell {
    return data.to_string();
  }
  if data.contains(OSC_COMMAND_FINISHED_PREFIX) {
    return data.to_string();
  }
  let trimmed = data.trim_end_matches(&['\r', '\n'][..]);
  if trimmed.is_empty() {
    return data.to_string();
  }
  let line_end = if data.ends_with("\r\n") {
    "\r\n"
  } else if data.ends_with('\n') {
    "\n"
  } else if data.ends_with('\r') {
    "\r"
  } else {
    ""
  };
  if line_end.is_empty() {
    return data.to_string();
  }
  let marker = if cfg!(windows) {
    " ; [Console]::Write(\"`e]633;D;0`a\")"
  } else {
    " ; printf '\\e]633;D;0\\a'"
  };
  format!("{trimmed}{marker}{line_end}")
}

fn flush_input_buffer(
  writer: &Arc<Mutex<Box<dyn Write + Send>>>,
  buffer: Vec<String>,
) -> Result<(), String> {
  if buffer.is_empty() {
    return Ok(());
  }
  let mut writer = writer
    .lock()
    .map_err(|_| "terminal writer lock poisoned".to_string())?;
  for entry in buffer {
    writer
      .write_all(entry.as_bytes())
      .map_err(|err| format!("failed to write to pty: {err}"))?;
  }
  writer
    .flush()
    .map_err(|err| format!("failed to flush pty: {err}"))?;
  Ok(())
}

fn handle_buffered_write(
  session: &mut TerminalSession,
  data: String,
  now: u64,
) -> (bool, Vec<String>) {
  if session.shell_ready {
    return (true, vec![data]);
  }
  if now.saturating_sub(session.created_at) >= SHELL_READY_TIMEOUT_MS {
    log::warn!(
      "terminal session ready timeout session_id={} forcing ready",
      session.id
    );
    session.shell_ready = true;
    let mut buffered: Vec<String> = session.input_buffer.drain(..).collect();
    buffered.push(data);
    return (true, buffered);
  }
  let was_empty = session.input_buffer.is_empty();
  session.input_buffer.push_back(data);
  if was_empty {
    log::info!("terminal input buffered session_id={}", session.id);
  }
  (false, Vec::new())
}

fn register_session(
  state: &TerminalManager,
  session_id: &str,
  member_id: Option<String>,
  workspace_id: Option<String>,
  rows: u16,
  cols: u16,
  terminal_type: TerminalType,
  keep_alive: bool,
  owner_window_label: Option<String>,
  output_window_label: Option<String>,
  semantic_tx: mpsc::Sender<SemanticEvent>,
  handle: TerminalHandle,
) -> Result<TerminalStatusPayload, String> {
  let created_at = now_millis().unwrap_or(0);
  let mut guard = state
    .sessions
    .lock()
    .map_err(|_| "terminal session lock poisoned".to_string())?;
  if guard.sessions.contains_key(session_id) {
    return Err("terminal session already exists".to_string());
  }
  let response_writer = Arc::clone(&handle.writer);
  let session = TerminalSession {
    id: session_id.to_string(),
    terminal_type,
    status: TerminalSessionStatus::Online,
    output_bytes_total: 0,
    output_seq: 0,
    unacked_bytes: 0,
    screen_rows: rows,
    screen_cols: cols,
    member_id: member_id.clone(),
    workspace_id: workspace_id.clone(),
    active: true,
    last_activity_at: None,
    broken: false,
    chat_pending: false,
    ui_active: false,
    handle: Some(handle),
    snapshot: TerminalSnapshot::new(rows, cols, Some(response_writer)),
    semantic_tx: Some(semantic_tx),
    keep_alive,
    owner_window_label,
    output_window_label,
    shell_ready: false,
    created_at,
    input_buffer: VecDeque::new(),
    ready_probe_bytes: 0,
  };
  let payload = build_status_payload(&session);
  guard.sessions.insert(session_id.to_string(), session);
  Ok(payload)
}

fn ensure_session_active(state: &TerminalManager, session_id: &str) -> Result<(), String> {
  let guard = lock_sessions(&state.sessions);
  let session = guard
    .sessions
    .get(session_id)
    .ok_or_else(|| "terminal session not found".to_string())?;
  if session.broken {
    return Err("terminal session is broken".to_string());
  }
  if session.active && session.handle.is_some() {
    Ok(())
  } else {
    Err("terminal session is not active".to_string())
  }
}

pub(crate) fn has_active_sessions(state: &TerminalManager) -> bool {
  let guard = lock_sessions(&state.sessions);
  guard
    .sessions
    .values()
    .any(|session| session.active && session.handle.is_some())
}

pub(crate) fn cleanup_ephemeral_sessions_for_window(
  state: &TerminalManager,
  window_label: &str,
) -> Result<(), String> {
  let mut killers = Vec::new();
  let mut semantic_shutdowns = Vec::new();
  {
    let mut guard = lock_sessions(&state.sessions);
    let targets: Vec<String> = guard
      .sessions
      .iter()
      .filter(|(_, session)| !session.keep_alive && session.owner_window_label.as_deref() == Some(window_label))
      .map(|(session_id, _)| session_id.clone())
      .collect();
    for session_id in targets {
      if let Some(removed) = guard.sessions.remove(&session_id) {
        if let Some(handle) = removed.handle {
          killers.push(handle.killer);
        }
        if let Some(tx) = removed.semantic_tx {
          semantic_shutdowns.push(tx);
        }
      }
    }
  }
  for mut killer in killers {
    let _ = killer.kill();
  }
  for tx in semantic_shutdowns {
    let _ = tx.send(SemanticEvent::Shutdown);
  }
  Ok(())
}

pub(crate) fn spawn_status_poller(app: AppHandle, manager: &TerminalManager) {
  let sessions = Arc::clone(&manager.sessions);
  thread::spawn(move || loop {
    thread::sleep(Duration::from_millis(STATUS_POLL_INTERVAL_MS));
    let now = match now_millis() {
      Ok(value) => value,
      Err(_) => continue,
    };
    let mut updates = Vec::new();
    let mut semantic_flushes = Vec::new();
    {
      let mut guard = lock_sessions(&sessions);
      for session in guard.sessions.values_mut() {
        if !session.active || session.status != TerminalSessionStatus::Working {
          continue;
        }
        let last_activity = match session.last_activity_at {
          Some(value) => value,
          None => continue,
        };
        if now.saturating_sub(last_activity) >= WORKING_SILENCE_TIMEOUT_MS {
          session.status = TerminalSessionStatus::Online;
          updates.push(build_status_payload(session));
          if session.chat_pending {
            session.chat_pending = false;
            if let Some(tx) = session.semantic_tx.clone() {
              semantic_flushes.push(tx);
            }
          }
        }
      }
    }
    for tx in semantic_flushes {
      let _ = tx.send(SemanticEvent::Flush {
        message_type: "info",
        source: "pty",
      });
    }
    for payload in updates {
      let _ = app.emit("terminal-status-change", payload);
    }
  });
}

struct InitialWriteState {
  session_id: String,
  payload: String,
  writer: Arc<Mutex<Box<dyn Write + Send>>>,
  sessions: Arc<Mutex<SessionRegistry>>,
  app: AppHandle,
  sent: AtomicBool,
}

impl InitialWriteState {
  fn send_if_needed(&self, reason: &str) {
    if self.sent.swap(true, Ordering::SeqCst) {
      return;
    }
    mark_session_working_on_input(
      &self.sessions,
      &self.app,
      &self.session_id,
      &self.payload,
      None,
    );
    let now = now_millis().unwrap_or(0);
    let (should_write, buffered) = {
      let mut guard = lock_sessions(&self.sessions);
      let session = match guard.sessions.get_mut(&self.session_id) {
        Some(session) => session,
        None => return,
      };
      handle_buffered_write(session, self.payload.clone(), now)
    };
    if should_write {
      if let Err(err) = flush_input_buffer(&self.writer, buffered) {
        log::warn!(
          "terminal initial write failed session_id={} err={}",
          self.session_id,
          err
        );
      }
    }
    log::info!("terminal initial write session_id={} reason={}", self.session_id, reason);
  }

  fn schedule(self: &Arc<Self>, delay_ms: u64, reason: &'static str) {
    let state = Arc::clone(self);
    thread::spawn(move || {
      if delay_ms > 0 {
        thread::sleep(Duration::from_millis(delay_ms));
      }
      state.send_if_needed(reason);
    });
  }
}

fn panic_message(err: Box<dyn Any + Send>) -> String {
  if let Some(message) = err.downcast_ref::<&str>() {
    (*message).to_string()
  } else if let Some(message) = err.downcast_ref::<String>() {
    message.clone()
  } else {
    "unknown panic".to_string()
  }
}

fn mark_session_broken(
  app: &AppHandle,
  sessions: &Arc<Mutex<SessionRegistry>>,
  session_id: &str,
  reason: String,
) {
  log::error!(
    "terminal reader crashed session_id={} reason={}",
    session_id,
    reason
  );
  let (status_payload, killer, semantic_tx) = {
    let mut guard = lock_sessions(sessions);
    let session = match guard.sessions.get_mut(session_id) {
      Some(session) => session,
      None => return,
    };
    if session.broken {
      return;
    }
    session.broken = true;
    session.active = false;
    session.last_activity_at = None;
    let status_payload = if session.status != TerminalSessionStatus::Offline {
      session.status = TerminalSessionStatus::Offline;
      Some(build_status_payload(session))
    } else {
      None
    };
    let killer = session.handle.take().map(|handle| handle.killer);
    let semantic_tx = session.semantic_tx.take();
    (status_payload, killer, semantic_tx)
  };
  let _ = app.emit(
    "terminal-error",
    TerminalErrorPayload {
      session_id: session_id.to_string(),
      error: format!("terminal reader crashed: {reason}"),
      fatal: true,
    },
  );
  if let Some(payload) = status_payload {
    let _ = app.emit("terminal-status-change", payload);
  }
  if let Some(mut killer) = killer {
    let _ = killer.kill();
  }
  if let Some(tx) = semantic_tx {
    let _ = tx.send(SemanticEvent::Shutdown);
  }
}

fn spawn_pty_reader(
  mut reader: Box<dyn Read + Send>,
  app: AppHandle,
  sessions: Arc<Mutex<SessionRegistry>>,
  session_id: String,
  initial_write: Option<Arc<InitialWriteState>>,
  initial_write_delay_ms: u64,
) {
  thread::spawn(move || {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      let mut buffer = [0u8; 8192];
    let mut initial_scheduled = false;
    let mut osc_carry = Vec::new();
    let mut pending_output = String::new();
    let mut pending_output_seq = 0u64;
    let mut pending_semantic: Vec<u8> = Vec::new();
    let mut last_output_flush = Instant::now();
    let mut last_semantic_flush = Instant::now();
    let mut stats_last_log = Instant::now();
    let mut stats_bytes_read: u64 = 0;
    let mut stats_bytes_emitted: u64 = 0;
    let mut stats_bytes_filtered: u64 = 0;
    let mut flow_paused = false;
    loop {
      if let Some((unacked, ui_active)) = get_flow_control_state(&sessions, &session_id) {
        if ui_active {
          flow_paused = false;
        } else if flow_paused {
          if unacked <= FLOW_CONTROL_LOW_WATERMARK {
            flow_paused = false;
          } else {
            thread::sleep(Duration::from_millis(FLOW_CONTROL_SLEEP_MS));
            continue;
          }
        } else if unacked >= FLOW_CONTROL_HIGH_WATERMARK {
          flow_paused = true;
          thread::sleep(Duration::from_millis(FLOW_CONTROL_SLEEP_MS));
          continue;
        }
      }
      let size = match reader.read(&mut buffer) {
        Ok(0) => break,
        Ok(size) => size,
        Err(_) => break,
      };
      if let Some(state) = initial_write.as_ref() {
        if !initial_scheduled {
          initial_scheduled = true;
          state.schedule(initial_write_delay_ms, "first-data");
        }
      }
      stats_bytes_read = stats_bytes_read.saturating_add(size as u64);
      let chunk = &buffer[..size];
      let raw_for_detect = merge_with_carry(&mut osc_carry, chunk);
      let detect_str = String::from_utf8_lossy(&raw_for_detect).to_string();
      let osc_finished = detect_str.contains(OSC_COMMAND_FINISHED_PREFIX);
      let filtered = strip_osc_633(&raw_for_detect, &mut osc_carry);
      if filtered.len() < raw_for_detect.len() {
        stats_bytes_filtered =
          stats_bytes_filtered.saturating_add((raw_for_detect.len() - filtered.len()) as u64);
      }
      let filtered_str = String::from_utf8_lossy(&filtered).to_string();
      let data_len = filtered_str.len();
      let (status_payload, first_output, output_bytes_total, semantic_tx, flush_needed, shim_error, output_seq, osc_flush, output_window_label, ui_active, unacked_bytes) = {
        let mut guard = lock_sessions(&sessions);
        let (status_payload, first_output, output_bytes_total, semantic_tx, flush_needed, shim_error, output_seq, osc_flush, output_window_label, ui_active, unacked_bytes) = {
          let session = match guard.sessions.get_mut(&session_id) {
            Some(session) => session,
            None => break,
          };
          let first_output = session.output_seq == 0;
          if !filtered.is_empty() {
            session.snapshot.apply_output(&filtered);
          }
          session.output_seq = session.output_seq.saturating_add(1);
          session.output_bytes_total = session
            .output_bytes_total
            .saturating_add(filtered.len() as u64);
          let output_seq = session.output_seq;
          let output_bytes_total = session.output_bytes_total;
          let semantic_tx = session.semantic_tx.clone();
          let output_window_label = session.output_window_label.clone();
          let ui_active = session.ui_active;
          let unacked_bytes = session.unacked_bytes;
          let mut flush_needed = false;
          let mut osc_flush = false;
          let now = now_millis().unwrap_or(0);
          let shim_error = if session.shell_ready {
            None
          } else {
            let error = detect_shell_ready(session, &detect_str, now);
            if session.shell_ready {
              flush_needed = true;
            }
            error
          };
          let mut status_payload = None;
          if osc_finished {
            if session.status != TerminalSessionStatus::Online {
              session.status = TerminalSessionStatus::Online;
              status_payload = Some(build_status_payload(session));
            }
            if session.chat_pending {
              session.chat_pending = false;
              osc_flush = true;
            }
          } else if session.status != TerminalSessionStatus::Working {
            session.status = TerminalSessionStatus::Working;
            status_payload = Some(build_status_payload(session));
          }
          if let Ok(now) = now_millis() {
            session.last_activity_at = Some(now);
          }
          (
            status_payload,
            first_output,
            output_bytes_total,
            semantic_tx,
            flush_needed,
            shim_error,
            output_seq,
            osc_flush,
            output_window_label,
            ui_active,
            unacked_bytes,
          )
        };
        (
          status_payload,
          first_output,
          output_bytes_total,
          semantic_tx,
          flush_needed,
          shim_error,
          output_seq,
          osc_flush,
          output_window_label,
          ui_active,
          unacked_bytes,
        )
      };
      if !filtered_str.is_empty() {
        pending_output.push_str(&filtered_str);
        pending_output_seq = output_seq;
      }
      if !filtered.is_empty() {
        pending_semantic.extend_from_slice(&filtered);
      }
      let now = Instant::now();
      let should_flush_output = !pending_output.is_empty()
        && (pending_output.len() >= OUTPUT_EMIT_MAX_BYTES
          || now.duration_since(last_output_flush) >= Duration::from_millis(OUTPUT_EMIT_INTERVAL_MS));
      if should_flush_output {
        let payload = TerminalOutputPayload {
          session_id: session_id.clone(),
          data: pending_output.clone(),
          seq: pending_output_seq,
        };
        let payload_len = payload.data.len();
        if terminal_trace_detail() {
          let target = output_window_label.as_deref().unwrap_or("<broadcast>");
          let unacked_after = unacked_bytes.saturating_add(payload_len);
          log::info!(
            "terminal_output_emit session_id={} seq={} data_len={} unacked={} ui_active={} flow_paused={} target={}",
            session_id,
            pending_output_seq,
            payload_len,
            unacked_after,
            ui_active,
            flow_paused,
            target
          );
        }
        stats_bytes_emitted = stats_bytes_emitted.saturating_add(pending_output.len() as u64);
        pending_output.clear();
        last_output_flush = now;
        emit_terminal_output(&app, output_window_label.as_deref(), payload);
        add_unacked_bytes(&sessions, &session_id, payload_len);
      }
      if let Some(payload) = status_payload {
        let _ = app.emit("terminal-status-change", payload);
      }
      if let Some(tx) = semantic_tx.as_ref() {
        let should_flush_semantic = !pending_semantic.is_empty()
          && (pending_semantic.len() >= SEMANTIC_EMIT_MAX_BYTES
            || now.duration_since(last_semantic_flush)
              >= Duration::from_millis(SEMANTIC_EMIT_INTERVAL_MS));
        if should_flush_semantic {
          let payload = std::mem::take(&mut pending_semantic);
          let _ = tx.send(SemanticEvent::Output(payload));
          last_semantic_flush = now;
        }
      }
      if osc_flush {
        if let Some(tx) = semantic_tx.as_ref() {
          let _ = tx.send(SemanticEvent::Flush {
            message_type: "info",
            source: "osc",
          });
        }
      }
      if let Some(error) = shim_error {
        let _ = app.emit(
          "terminal-error",
          TerminalErrorPayload {
            session_id: session_id.clone(),
            error,
            fatal: true,
          },
        );
      }
      if flush_needed {
        let (buffered, writer) = {
          let mut guard = lock_sessions(&sessions);
          let session = match guard.sessions.get_mut(&session_id) {
            Some(session) => session,
            None => break,
          };
          let buffered: Vec<String> = session.input_buffer.drain(..).collect();
          let writer = session.handle.as_ref().map(|handle| Arc::clone(&handle.writer));
          (buffered, writer)
        };
        if let Some(writer) = writer {
          if let Err(err) = flush_input_buffer(&writer, buffered) {
            log::warn!("terminal buffer flush failed session_id={} err={}", session_id, err);
          }
        }
      }
      if first_output {
        log::info!(
          "terminal first output session_id={} data_len={} total_bytes={}",
          session_id,
          data_len,
          output_bytes_total
        );
      }
      if terminal_trace_enabled()
        && now.duration_since(stats_last_log) >= Duration::from_millis(STATS_LOG_INTERVAL_MS)
      {
        log::info!(
          "terminal io stats session_id={} read_bytes={} emit_bytes={} filtered_bytes={}",
          session_id,
          stats_bytes_read,
          stats_bytes_emitted,
          stats_bytes_filtered
        );
        stats_last_log = now;
        stats_bytes_read = 0;
        stats_bytes_emitted = 0;
        stats_bytes_filtered = 0;
      }
    }
    if !pending_output.is_empty() {
      let payload = TerminalOutputPayload {
        session_id: session_id.clone(),
        data: pending_output,
        seq: pending_output_seq,
      };
      let payload_len = payload.data.len();
      let output_window_label = {
        let guard = lock_sessions(&sessions);
        guard
          .sessions
          .get(&session_id)
          .and_then(|session| session.output_window_label.clone())
      };
      if terminal_trace_detail() {
        let target = output_window_label.as_deref().unwrap_or("<broadcast>");
        log::info!(
          "terminal_output_emit session_id={} seq={} data_len={} target={}",
          session_id,
          pending_output_seq,
          payload_len,
          target
        );
      }
      emit_terminal_output(&app, output_window_label.as_deref(), payload);
      add_unacked_bytes(&sessions, &session_id, payload_len);
    }
    if !pending_semantic.is_empty() {
      let guard = lock_sessions(&sessions);
      if let Some(session) = guard.sessions.get(&session_id) {
        if let Some(tx) = session.semantic_tx.clone() {
          let _ = tx.send(SemanticEvent::Output(pending_semantic));
        }
      }
    }
  }));
  if let Err(err) = result {
    let reason = panic_message(err);
    mark_session_broken(&app, &sessions, &session_id, reason);
  }
  });
}

fn terminal_trace_enabled() -> bool {
  match std::env::var("GOLUTRA_TERMINAL_TRACE") {
    Ok(value) => matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"),
    Err(_) => false,
  }
}

fn terminal_trace_detail() -> bool {
  match std::env::var("GOLUTRA_TERMINAL_TRACE_DETAIL") {
    Ok(value) => matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"),
    Err(_) => false,
  }
}

fn emit_terminal_output(
  app: &AppHandle,
  output_window_label: Option<&str>,
  payload: TerminalOutputPayload,
) {
  if let Some(label) = output_window_label {
    if let Some(window) = app.get_webview_window(label) {
      let _ = window.emit("terminal-output", payload);
      return;
    }
  }
  let _ = app.emit("terminal-output", payload);
}

fn merge_with_carry(carry: &mut Vec<u8>, chunk: &[u8]) -> Vec<u8> {
  if carry.is_empty() {
    return chunk.to_vec();
  }
  let mut combined = Vec::with_capacity(carry.len() + chunk.len());
  combined.extend_from_slice(carry);
  combined.extend_from_slice(chunk);
  carry.clear();
  combined
}

fn strip_osc_633(input: &[u8], carry: &mut Vec<u8>) -> Vec<u8> {
  let mut output = Vec::with_capacity(input.len());
  let mut idx = 0;
  while idx < input.len() {
    if input[idx] == 0x1b
      && idx + 5 < input.len()
      && input[idx + 1] == b']'
      && input[idx + 2] == b'6'
      && input[idx + 3] == b'3'
      && input[idx + 4] == b'3'
      && input[idx + 5] == b';'
    {
      let mut end = None;
      let mut j = idx + 6;
      while j < input.len() {
        if input[j] == 0x07 {
          end = Some(j + 1);
          break;
        }
        if input[j] == 0x1b && j + 1 < input.len() && input[j + 1] == b'\\' {
          end = Some(j + 2);
          break;
        }
        j += 1;
      }
      if let Some(end) = end {
        idx = end;
        continue;
      } else {
        carry.extend_from_slice(&input[idx..]);
        break;
      }
    } else {
      output.push(input[idx]);
      idx += 1;
    }
  }
  output
}

fn spawn_exit_watcher(
  mut child: Box<dyn Child + Send + Sync>,
  app: AppHandle,
  sessions: Arc<Mutex<SessionRegistry>>,
  session_id: String,
) {
  thread::spawn(move || {
    let (code, signal, reason) = match child.wait() {
      Ok(status) => {
        if let Some(signal) = status.signal() {
          (None, Some(signal.to_string()), format!("signal {signal}"))
        } else {
          let code = i32::try_from(status.exit_code()).ok();
          let reason = code.map(|value| format!("code {value}")).unwrap_or_else(|| "unknown".to_string());
          (code, None, reason)
        }
      }
      Err(err) => {
        log::warn!("terminal child wait failed session_id={} err={}", session_id, err);
        (None, Some("error".to_string()), "error".to_string())
      }
    };
    let (exit_payload, status_payload, semantic_tx, notice_bytes) = {
      let mut guard = lock_sessions(&sessions);
      let (exit_payload, status_payload, semantic_tx, notice_bytes) = {
        let session = match guard.sessions.get_mut(&session_id) {
          Some(session) => session,
          None => return,
        };
        session.active = false;
        session.last_activity_at = None;
        session.handle = None;
        let status_payload = if session.status != TerminalSessionStatus::Offline {
          session.status = TerminalSessionStatus::Offline;
          Some(build_status_payload(session))
        } else {
          None
        };
        let notice_bytes = format!("\r\n\x1b[31m[Process exited with {reason}]\x1b[0m").into_bytes();
        session.snapshot.apply_output(&notice_bytes);
        session.output_bytes_total = session
          .output_bytes_total
          .saturating_add(notice_bytes.len() as u64);
        let semantic_tx = session.semantic_tx.clone();
        (
          TerminalExitPayload {
            session_id: session_id.clone(),
            code,
            signal,
          },
          status_payload,
          semantic_tx,
          notice_bytes,
        )
      };
      (exit_payload, status_payload, semantic_tx, notice_bytes)
    };
    let _ = app.emit("terminal-exit", exit_payload);
    if let Some(payload) = status_payload {
      let _ = app.emit("terminal-status-change", payload);
    }
    if let Some(tx) = semantic_tx {
      let _ = tx.send(SemanticEvent::Output(notice_bytes));
      let _ = tx.send(SemanticEvent::Flush {
        message_type: "system",
        source: "system",
      });
    }
  });
}

#[tauri::command]
pub(crate) fn terminal_create(
  app: AppHandle,
  window: Window,
  state: State<'_, TerminalManager>,
  cols: Option<u16>,
  rows: Option<u16>,
  cwd: Option<String>,
  member_id: Option<String>,
  workspace_id: Option<String>,
  keep_alive: Option<bool>,
  session_id: Option<String>,
  terminal_type: Option<String>,
  terminal_command: Option<String>,
  terminal_path: Option<String>,
) -> Result<String, String> {
  let requested_id = session_id
    .as_deref()
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty());
  let command = terminal_command
    .as_deref()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(|value| value.to_string());
  let terminal_type = resolve_terminal_type(terminal_type.as_deref(), command.as_deref());
  let terminal_path = terminal_path
    .as_deref()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(|value| value.to_string());
  let sanitized_command = command
    .as_deref()
    .map(|value| value.replace('\r', "\\r").replace('\n', "\\n"))
    .unwrap_or_else(|| "<none>".to_string());
  let sanitized_path = terminal_path.as_deref().unwrap_or("<none>");
  log::info!(
    "terminal_create type={} command={} path={}",
    terminal_type.as_str(),
    sanitized_command,
    sanitized_path
  );
  let session_id = requested_id
    .unwrap_or_else(|| format!("term-{}", SESSION_COUNTER.fetch_add(1, Ordering::Relaxed)));
  let keep_alive = keep_alive.unwrap_or(false);
  let owner_window_label = if keep_alive {
    None
  } else {
    Some(window.label().to_string())
  };
  let output_window_label = Some(window.label().to_string());
  {
    let guard = state
      .sessions
      .lock()
      .map_err(|_| "terminal session lock poisoned".to_string())?;
    if guard.sessions.contains_key(&session_id) {
      return Err("terminal session already exists".to_string());
    }
  }

  let cols = cols.unwrap_or(80).max(1);
  let rows = rows.unwrap_or(24).max(1);
  let spawned = match terminal_type {
    TerminalType::Shell => spawn_shell(cols, rows, cwd)?,
    _ => {
      let binary = terminal_type
        .default_binary()
        .ok_or_else(|| "terminal binary missing".to_string())?;
      let resolved = lookup_binary(binary, terminal_path.as_deref())?;
      spawn_command(cols, rows, cwd, &resolved, &[])?
    }
  };
  let writer = Arc::clone(&spawned.handle.writer);
  let mut cleanup_killer = spawned.child.clone_killer();
  let semantic_tx = spawn_semantic_worker(
    app.clone(),
    session_id.clone(),
    member_id.clone(),
    workspace_id.clone(),
    rows,
    cols,
  );
  let status_payload = match register_session(
    &state,
    &session_id,
    member_id,
    workspace_id,
    rows,
    cols,
    terminal_type,
    keep_alive,
    owner_window_label,
    output_window_label,
    semantic_tx.clone(),
    spawned.handle,
  ) {
    Ok(payload) => payload,
    Err(err) => {
      let _ = cleanup_killer.kill();
      let _ = semantic_tx.send(SemanticEvent::Shutdown);
      return Err(err);
    }
  };

  let initial_payload = if matches!(terminal_type, TerminalType::Shell) {
    command.as_deref().map(|value| {
      if value.ends_with('\n') || value.ends_with('\r') {
        value.to_string()
      } else {
        format!("{value}\r")
      }
    })
  } else {
    None
  };
  let (initial_timeout_ms, initial_delay_ms) = if cfg!(windows) { (1200, 300) } else { (500, 0) };
  let initial_write = initial_payload.map(|payload| {
    Arc::new(InitialWriteState {
      session_id: session_id.clone(),
      payload,
      writer,
      sessions: state.sessions.clone(),
      app: app.clone(),
      sent: AtomicBool::new(false),
    })
  });
  if let Some(state) = initial_write.as_ref() {
    state.schedule(initial_timeout_ms, "timeout");
  }

  spawn_pty_reader(
    spawned.reader,
    app.clone(),
    state.sessions.clone(),
    session_id.clone(),
    initial_write,
    initial_delay_ms,
  );
  spawn_exit_watcher(spawned.child, app.clone(), state.sessions.clone(), session_id.clone());

  let _ = app.emit("terminal-status-change", status_payload);
  Ok(session_id)
}

#[tauri::command]
pub(crate) fn terminal_attach(
  window: Window,
  state: State<'_, TerminalManager>,
  session_id: String,
) -> Result<TerminalSnapshotPayload, String> {
  let output_window_label = window.label().to_string();
  let (data, seq, history, data_len) = {
    let mut guard = lock_sessions(&state.sessions);
    let session = guard
      .sessions
      .get_mut(&session_id)
      .ok_or_else(|| "terminal session not found".to_string())?;
    session.output_window_label = Some(output_window_label.clone());
    let snapshot = session.snapshot.snapshot();
    let data_len = snapshot.len();
    let data = String::from_utf8_lossy(&snapshot).to_string();
    session.unacked_bytes = session.unacked_bytes.saturating_add(data_len);
    (data, session.output_seq, None, data_len)
  };
  log::info!(
    "terminal_attach session_id={} seq={} data_len={}",
    session_id,
    seq,
    data_len
  );
  if terminal_trace_detail() {
    log::info!(
      "terminal_attach_window session_id={} window={}",
      session_id,
      output_window_label
    );
  }
  Ok(TerminalSnapshotPayload {
    session_id: session_id.clone(),
    data,
    seq,
    history,
  })
}

#[tauri::command]
pub(crate) fn terminal_write(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  session_id: String,
  data: String,
) -> Result<(), String> {
  ensure_session_active(&state, &session_id)?;
  mark_session_working_on_input(&state.sessions, &app, &session_id, &data, None);
  let now = now_millis()?;
  let (should_write, buffered, writer, shell_ready) = {
    let mut guard = lock_sessions(&state.sessions);
    let session = guard
      .sessions
      .get_mut(&session_id)
      .ok_or_else(|| "terminal session not found".to_string())?;
    let shell_ready = session.shell_ready;
    let handle = session
      .handle
      .as_ref()
      .ok_or_else(|| "terminal session handle missing".to_string())?;
    let writer = Arc::clone(&handle.writer);
    let (should_write, buffered) = handle_buffered_write(session, data.clone(), now);
    (should_write, buffered, writer, shell_ready)
  };
  if terminal_trace_detail() {
    let buffered_bytes: usize = buffered.iter().map(|entry| entry.len()).sum();
    log::info!(
      "terminal_write session_id={} data_len={} buffered_bytes={} write_now={} shell_ready={}",
      session_id,
      data.len(),
      buffered_bytes,
      should_write,
      shell_ready
    );
  }
  if !should_write {
    return Ok(());
  }
  flush_input_buffer(&writer, buffered)?;
  Ok(())
}

#[tauri::command]
pub(crate) fn terminal_set_active(
  state: State<'_, TerminalManager>,
  session_id: String,
  active: bool,
) -> Result<(), String> {
  let mut guard = lock_sessions(&state.sessions);
  let session = guard
    .sessions
    .get_mut(&session_id)
    .ok_or_else(|| "terminal session not found".to_string())?;
  session.ui_active = active;
  if terminal_trace_detail() {
    log::info!(
      "terminal_set_active session_id={} active={}",
      session_id,
      active
    );
  }
  Ok(())
}

#[tauri::command]
pub(crate) fn terminal_set_member_status(
  state: State<'_, TerminalManager>,
  member_id: String,
  status: String,
) -> Result<(), String> {
  let normalized = status.trim().to_lowercase();
  let mut guard = lock_sessions(&state.sessions);
  if normalized.is_empty() {
    guard.member_statuses.remove(&member_id);
    return Ok(());
  }
  if matches!(normalized.as_str(), "online" | "working" | "dnd" | "offline") {
    guard.member_statuses.insert(member_id, normalized);
    return Ok(());
  }
  guard.member_statuses.remove(&member_id);
  Ok(())
}

#[tauri::command]
pub(crate) fn terminal_ack(
  state: State<'_, TerminalManager>,
  session_id: String,
  count: usize,
) -> Result<(), String> {
  subtract_unacked_bytes(&state.sessions, &session_id, count);
  if terminal_trace_detail() {
    let current = {
      let guard = lock_sessions(&state.sessions);
      guard
        .sessions
        .get(&session_id)
        .map(|session| session.unacked_bytes)
        .unwrap_or(0)
    };
    log::info!(
      "terminal_ack session_id={} count={} unacked={}",
      session_id,
      count,
      current
    );
  }
  Ok(())
}

#[tauri::command]
pub(crate) fn terminal_dispatch(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  session_id: String,
  data: String,
  context: TerminalDispatchContext,
) -> Result<(), String> {
  if is_member_dnd(&state.sessions, &session_id) {
    if terminal_trace_detail() {
      log::info!("terminal_dispatch skipped dnd session_id={}", session_id);
    }
    return Ok(());
  }
  ensure_session_active(&state, &session_id)?;
  let chat_context = TerminalChatContext {
    conversation_id: context.conversation_id,
    conversation_type: context.conversation_type,
    sender_id: context.sender_id,
    sender_name: context.sender_name,
  };
  mark_session_working_on_input(&state.sessions, &app, &session_id, &data, Some(chat_context));
  let now = now_millis()?;
  let (should_write, buffered, writer, shell_ready, data_len) = {
    let mut guard = lock_sessions(&state.sessions);
    let session = guard
      .sessions
      .get_mut(&session_id)
      .ok_or_else(|| "terminal session not found".to_string())?;
    let shell_ready = session.shell_ready;
    let handle = session
      .handle
      .as_ref()
      .ok_or_else(|| "terminal session handle missing".to_string())?;
    let writer = Arc::clone(&handle.writer);
    let decorated = append_command_finished_marker(&data, session.terminal_type);
    let data_len = decorated.len();
    let (should_write, buffered) = handle_buffered_write(session, decorated, now);
    (should_write, buffered, writer, shell_ready, data_len)
  };
  if terminal_trace_detail() {
    let buffered_bytes: usize = buffered.iter().map(|entry| entry.len()).sum();
    log::info!(
      "terminal_dispatch session_id={} data_len={} buffered_bytes={} write_now={} shell_ready={}",
      session_id,
      data_len,
      buffered_bytes,
      should_write,
      shell_ready
    );
  }
  if !should_write {
    return Ok(());
  }
  flush_input_buffer(&writer, buffered)?;
  Ok(())
}

#[tauri::command]
pub(crate) fn terminal_resize(
  state: State<'_, TerminalManager>,
  session_id: String,
  cols: u16,
  rows: u16,
) -> Result<(), String> {
  ensure_session_active(&state, &session_id)?;
  let rows = rows.max(1);
  let cols = cols.max(1);
  let semantic_tx = {
    let mut guard = state
      .sessions
      .lock()
      .map_err(|_| "terminal session lock poisoned".to_string())?;
    let session = guard
      .sessions
      .get_mut(&session_id)
      .ok_or_else(|| "terminal session not found".to_string())?;
    session.screen_rows = rows;
    session.screen_cols = cols;
    session.snapshot.set_size(rows, cols);
    let handle = session
      .handle
      .as_ref()
      .ok_or_else(|| "terminal session handle missing".to_string())?;
    resize_pty(handle, session.screen_rows, session.screen_cols)?;
    session.semantic_tx.clone()
  };
  if let Some(tx) = semantic_tx {
    let _ = tx.send(SemanticEvent::Resize { rows, cols });
  }
  Ok(())
}

#[tauri::command]
pub(crate) fn terminal_close(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  session_id: String,
  preserve: Option<bool>,
) -> Result<(), String> {
  let preserve = preserve.unwrap_or(false);
  let (status_payload, killer, semantic_tx) = {
    let mut guard = state
      .sessions
      .lock()
      .map_err(|_| "terminal session lock poisoned".to_string())?;
    let session = guard
      .sessions
      .get_mut(&session_id)
      .ok_or_else(|| "terminal session not found".to_string())?;
    let mut status_payload = None;
    let mut killer = None;
    let mut semantic_tx = None;
    if preserve {
      session.active = false;
      session.last_activity_at = None;
      if session.status != TerminalSessionStatus::Offline {
        session.status = TerminalSessionStatus::Offline;
        status_payload = Some(build_status_payload(session));
      }
      if let Some(handle) = session.handle.take() {
        killer = Some(handle.killer);
      }
      semantic_tx = session.semantic_tx.take();
    } else {
      let removed = guard.sessions.remove(&session_id);
      if let Some(removed) = removed {
        if let Some(handle) = removed.handle {
          killer = Some(handle.killer);
        }
        semantic_tx = removed.semantic_tx;
      }
    }
    (status_payload, killer, semantic_tx)
  };
  if let Some(mut killer) = killer {
    thread::spawn(move || {
      let _ = killer.kill();
    });
  }
  if let Some(tx) = semantic_tx {
    let _ = tx.send(SemanticEvent::Shutdown);
  }
  if let Some(payload) = status_payload {
    let _ = app.emit("terminal-status-change", payload);
  }
  Ok(())
}
