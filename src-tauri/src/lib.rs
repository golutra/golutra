use std::{
  collections::{HashMap, VecDeque},
  env,
  fs,
  io::{Read, Write},
  path::{Component, Path, PathBuf},
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
  },
  thread,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use fs2::FileExt;
use portable_pty::{native_pty_system, Child, ChildKiller, CommandBuilder, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder, Window, WindowEvent};

static SESSION_COUNTER: AtomicUsize = AtomicUsize::new(1);
static WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);
static WORKSPACE_WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);
static PROJECT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
const TERMINAL_WINDOW_LABEL: &str = "terminal-main";
const SESSION_BUFFER_LIMIT_BYTES: usize = 1 * 1024 * 1024;
const TOTAL_BUFFER_LIMIT_BYTES: usize = 500 * 1024 * 1024;
const WORKING_SILENCE_TIMEOUT_MS: u64 = 2000;
const STATUS_POLL_INTERVAL_MS: u64 = 500;
const RECENT_WORKSPACES_FILE: &str = "recent-workspaces.json";
const WORKSPACE_REGISTRY_FILE: &str = "workspace-registry.json";
const WORKSPACE_REGISTRY_LOCK_FILE: &str = "workspace-registry.lock";
const AVATAR_LIBRARY_FILE: &str = "avatar-library.json";
const AVATAR_DIR: &str = "avatars";
const MAX_AVATAR_BYTES: usize = 2 * 1024 * 1024;
const WORKSPACE_REGISTRY_MISMATCH_PREFIX: &str = "workspace_registry_mismatch:";
const WORKSPACE_REGISTRY_GC_MIN_AGE_MS: u64 = 1000 * 60 * 60 * 24 * 30;
const WORKSPACE_REGISTRY_GC_MAX_CHECKS: usize = 12;

struct TerminalHandle {
  master: Box<dyn MasterPty + Send>,
  writer: Arc<Mutex<Box<dyn Write + Send>>>,
  killer: Box<dyn ChildKiller + Send + Sync>,
}

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

struct TerminalSession {
  id: String,
  status: TerminalSessionStatus,
  buffer: VecDeque<u8>,
  member_id: Option<String>,
  workspace_id: Option<String>,
  active: bool,
  last_activity_at: Option<u64>,
  handle: Option<TerminalHandle>,
  keep_alive: bool,
  owner_window_label: Option<String>,
}

struct SessionRegistry {
  sessions: HashMap<String, TerminalSession>,
  total_bytes: usize,
}

struct InitialWriteState {
  session_id: String,
  payload: String,
  writer: Arc<Mutex<Box<dyn Write + Send>>>,
  sessions: Arc<Mutex<SessionRegistry>>,
  app: AppHandle,
  sent: AtomicBool,
}

struct TerminalManager {
  sessions: Arc<Mutex<SessionRegistry>>,
}

struct WorkspaceRegistryLock {
  lock: Mutex<()>,
}

struct WorkspaceWindowRegistry {
  workspaces: Mutex<HashMap<String, String>>,
}

impl Default for TerminalManager {
  fn default() -> Self {
    Self {
      sessions: Arc::new(Mutex::new(SessionRegistry {
        sessions: HashMap::new(),
        total_bytes: 0,
      })),
    }
  }
}

impl Default for WorkspaceRegistryLock {
  fn default() -> Self {
    Self { lock: Mutex::new(()) }
  }
}

impl Default for WorkspaceWindowRegistry {
  fn default() -> Self {
    Self {
      workspaces: Mutex::new(HashMap::new()),
    }
  }
}

#[derive(Serialize, Clone)]
struct TerminalOutputPayload {
  #[serde(rename = "sessionId")]
  session_id: String,
  data: String,
}

#[derive(Serialize, Clone)]
struct TerminalExitPayload {
  #[serde(rename = "sessionId")]
  session_id: String,
  code: Option<i32>,
  signal: Option<String>,
}

#[derive(Serialize, Clone)]
struct TerminalStatusPayload {
  #[serde(rename = "sessionId")]
  session_id: String,
  status: String,
  #[serde(rename = "memberId")]
  member_id: Option<String>,
  #[serde(rename = "workspaceId")]
  workspace_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct WorkspaceEntry {
  id: String,
  name: String,
  path: String,
  #[serde(rename = "lastOpenedAt")]
  last_opened_at: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct WorkspaceOpenResult {
  entry: WorkspaceEntry,
  #[serde(rename = "readOnly")]
  read_only: bool,
  warning: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct WorkspaceRegistryEntry {
  #[serde(rename = "lastKnownPath")]
  last_known_path: String,
  #[serde(rename = "lastAccessed")]
  last_accessed: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct LocalWorkspaceState {
  #[serde(rename = "localMachineId")]
  local_machine_id: String,
  #[serde(rename = "lastOpenedAt")]
  last_opened_at: u64,
}

#[derive(Deserialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
enum WorkspaceRegistryResolution {
  Move,
  Copy,
}

#[derive(Serialize, Deserialize, Clone)]
struct AvatarAsset {
  id: String,
  filename: String,
  #[serde(rename = "createdAt")]
  created_at: u64,
}

#[derive(Serialize)]
struct AvatarContent {
  bytes: Vec<u8>,
  mime: String,
}

fn default_shell_spec() -> (String, Vec<String>) {
  if cfg!(windows) {
    ("powershell.exe".to_string(), vec!["-NoLogo".to_string()])
  } else {
    let shell = env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
    (shell, Vec::new())
  }
}

fn mark_session_working_on_input(
  sessions: &Arc<Mutex<SessionRegistry>>,
  app: &AppHandle,
  session_id: &str,
  data: &str,
) {
  if !(data.contains('\n') || data.contains('\r')) {
    return;
  }
  let mut status_payload = None;
  {
    let mut guard = match sessions.lock() {
      Ok(guard) => guard,
      Err(_) => return,
    };
    if let Some(session) = guard.sessions.get_mut(session_id) {
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
}

impl InitialWriteState {
  fn send_if_needed(&self, reason: &str) {
    if self.sent.swap(true, Ordering::SeqCst) {
      return;
    }
    if let Ok(mut writer) = self.writer.lock() {
      let _ = writer.write_all(self.payload.as_bytes());
      let _ = writer.flush();
    }
    log::info!("terminal initial write session_id={} reason={}", self.session_id, reason);
    mark_session_working_on_input(&self.sessions, &self.app, &self.session_id, &self.payload);
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

fn spawn_pty_reader(
  mut reader: Box<dyn Read + Send>,
  app: AppHandle,
  sessions: Arc<Mutex<SessionRegistry>>,
  session_id: String,
  initial_write: Option<Arc<InitialWriteState>>,
  initial_write_delay_ms: u64,
) {
  thread::spawn(move || {
    let mut buffer = [0u8; 8192];
    let mut initial_scheduled = false;
    loop {
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
      let chunk = &buffer[..size];
      let data = String::from_utf8_lossy(chunk).to_string();
      let data_len = data.len();
      let (output_payload, status_payload, first_output, buffer_len) = {
        let mut guard = match sessions.lock() {
          Ok(guard) => guard,
          Err(_) => continue,
        };
        let (output_payload, status_payload, delta, first_output, buffer_len) = {
          let session = match guard.sessions.get_mut(&session_id) {
            Some(session) => session,
            None => break,
          };
          let first_output = session.buffer.is_empty();
          let delta = append_to_session_buffer(session, chunk);
          let buffer_len = session.buffer.len();
          let status_payload = if session.status != TerminalSessionStatus::Working {
            session.status = TerminalSessionStatus::Working;
            Some(build_status_payload(session))
          } else {
            None
          };
          if let Ok(now) = now_millis() {
            session.last_activity_at = Some(now);
          }
          (
            TerminalOutputPayload {
              session_id: session_id.clone(),
              data,
            },
            status_payload,
            delta,
            first_output,
            buffer_len,
          )
        };
        if delta > 0 {
          guard.total_bytes = guard.total_bytes.saturating_add(delta as usize);
        } else if delta < 0 {
          guard.total_bytes = guard.total_bytes.saturating_sub((-delta) as usize);
        }
        (output_payload, status_payload, first_output, buffer_len)
      };
      let _ = app.emit("terminal-output", output_payload);
      if let Some(payload) = status_payload {
        let _ = app.emit("terminal-status-change", payload);
      }
      if first_output {
        log::info!(
          "terminal first output session_id={} data_len={} buffer_len={}",
          session_id,
          data_len,
          buffer_len
        );
      }
    }
  });
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
    let (exit_payload, status_payload) = {
      let mut guard = match sessions.lock() {
        Ok(guard) => guard,
        Err(_) => return,
      };
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
      let notice = format!("\r\n\x1b[31m[Process exited with {reason}]\x1b[0m");
      let delta = append_to_session_buffer(session, notice.as_bytes());
      if delta > 0 {
        guard.total_bytes = guard.total_bytes.saturating_add(delta as usize);
      } else if delta < 0 {
        guard.total_bytes = guard.total_bytes.saturating_sub((-delta) as usize);
      }
      (
        TerminalExitPayload {
          session_id: session_id.clone(),
          code,
          signal,
        },
        status_payload,
      )
    };
    let _ = app.emit("terminal-exit", exit_payload);
    if let Some(payload) = status_payload {
      let _ = app.emit("terminal-status-change", payload);
    }
  });
}

fn shutdown_sessions(state: &TerminalManager) -> Result<(), String> {
  let mut killers = Vec::new();
  {
    let mut guard = state
      .sessions
      .lock()
      .map_err(|_| "terminal session lock poisoned".to_string())?;
    for session in guard.sessions.values_mut() {
      if let Some(handle) = session.handle.take() {
        killers.push(handle.killer);
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

fn append_to_session_buffer(session: &mut TerminalSession, data: &[u8]) -> isize {
  if data.is_empty() {
    return 0;
  }
  let before = session.buffer.len();
  if data.len() >= SESSION_BUFFER_LIMIT_BYTES {
    session.buffer.clear();
    session.buffer.extend(&data[data.len() - SESSION_BUFFER_LIMIT_BYTES..]);
  } else {
    let next_len = before + data.len();
    if next_len > SESSION_BUFFER_LIMIT_BYTES {
      let overflow = next_len - SESSION_BUFFER_LIMIT_BYTES;
      for _ in 0..overflow {
        session.buffer.pop_front();
      }
    }
    session.buffer.extend(data);
  }
  let after = session.buffer.len();
  after as isize - before as isize
}

fn register_session(
  state: &TerminalManager,
  session_id: &str,
  member_id: Option<String>,
  workspace_id: Option<String>,
  keep_alive: bool,
  owner_window_label: Option<String>,
  handle: TerminalHandle,
) -> Result<TerminalStatusPayload, String> {
  let mut guard = state
    .sessions
    .lock()
    .map_err(|_| "terminal session lock poisoned".to_string())?;
  if guard.total_bytes >= TOTAL_BUFFER_LIMIT_BYTES {
    return Err("terminal buffer limit reached".to_string());
  }
  if guard.sessions.contains_key(session_id) {
    return Err("terminal session already exists".to_string());
  }
  let session = TerminalSession {
    id: session_id.to_string(),
    status: TerminalSessionStatus::Online,
    buffer: VecDeque::new(),
    member_id: member_id.clone(),
    workspace_id: workspace_id.clone(),
    active: true,
    last_activity_at: None,
    handle: Some(handle),
    keep_alive,
    owner_window_label,
  };
  let payload = build_status_payload(&session);
  guard.sessions.insert(session_id.to_string(), session);
  Ok(payload)
}

fn ensure_session_active(state: &TerminalManager, session_id: &str) -> Result<(), String> {
  let guard = state
    .sessions
    .lock()
    .map_err(|_| "terminal session lock poisoned".to_string())?;
  let session = guard
    .sessions
    .get(session_id)
    .ok_or_else(|| "terminal session not found".to_string())?;
  if session.active && session.handle.is_some() {
    Ok(())
  } else {
    Err("terminal session is not active".to_string())
  }
}

fn has_active_sessions(state: &TerminalManager) -> bool {
  state
    .sessions
    .lock()
    .map(|guard| {
      guard
        .sessions
        .values()
        .any(|session| session.active && session.handle.is_some())
    })
    .unwrap_or(false)
}

fn cleanup_ephemeral_sessions_for_window(
  state: &TerminalManager,
  window_label: &str,
) -> Result<(), String> {
  let mut killers = Vec::new();
  {
    let mut guard = state
      .sessions
      .lock()
      .map_err(|_| "terminal session lock poisoned".to_string())?;
    let targets: Vec<String> = guard
      .sessions
      .iter()
      .filter(|(_, session)| {
        !session.keep_alive && session.owner_window_label.as_deref() == Some(window_label)
      })
      .map(|(session_id, _)| session_id.clone())
      .collect();
    for session_id in targets {
      if let Some(removed) = guard.sessions.remove(&session_id) {
        guard.total_bytes = guard.total_bytes.saturating_sub(removed.buffer.len());
        if let Some(handle) = removed.handle {
          killers.push(handle.killer);
        }
      }
    }
  }
  for mut killer in killers {
    let _ = killer.kill();
  }
  Ok(())
}

fn spawn_status_poller(app: AppHandle, sessions: Arc<Mutex<SessionRegistry>>) {
  thread::spawn(move || loop {
    thread::sleep(Duration::from_millis(STATUS_POLL_INTERVAL_MS));
    let now = match now_millis() {
      Ok(value) => value,
      Err(_) => continue,
    };
    let mut updates = Vec::new();
    {
      let mut guard = match sessions.lock() {
        Ok(guard) => guard,
        Err(_) => continue,
      };
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
        }
      }
    }
    for payload in updates {
      let _ = app.emit("terminal-status-change", payload);
    }
  });
}

fn next_terminal_label(reuse: bool) -> String {
  if reuse {
    TERMINAL_WINDOW_LABEL.to_string()
  } else {
    let suffix = WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("terminal-{suffix}")
  }
}

fn next_workspace_window_label() -> String {
  let suffix = WORKSPACE_WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed);
  format!("workspace-selection-{suffix}")
}

fn terminal_label_for_workspace(workspace_id: &str) -> String {
  let hash = hash_bytes(workspace_id.as_bytes());
  format!("terminal-workspace-{hash}")
}

  #[derive(Serialize)]
  struct TerminalWindowOpenResult {
    label: String,
    reused: bool,
  }

  #[tauri::command]
  async fn terminal_open_window(
    app: AppHandle,
    reuse: Option<bool>,
    workspace_id: Option<String>,
    workspace_name: Option<String>,
  ) -> Result<TerminalWindowOpenResult, String> {
  let reuse = reuse.unwrap_or(true);
  let workspace_id = workspace_id
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty());
  let workspace_name = workspace_name
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty());

  let workspace_label = workspace_id.as_deref().map(terminal_label_for_workspace);
  if let Some(label) = workspace_label.as_deref() {
      if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(TerminalWindowOpenResult {
          label: label.to_string(),
          reused: true,
        });
      }
    } else if reuse {
      if let Some(window) = app.get_webview_window(TERMINAL_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(TerminalWindowOpenResult {
          label: TERMINAL_WINDOW_LABEL.to_string(),
          reused: true,
        });
      }
    }

  let label = workspace_label.unwrap_or_else(|| next_terminal_label(reuse));
  let title = workspace_name
    .as_deref()
    .map(|name| format!("Terminal - {name}"))
    .unwrap_or_else(|| "Terminal".to_string());
  let window_builder = WebviewWindowBuilder::new(&app, label.clone(), WebviewUrl::App("index.html".into()))
    .initialization_script("window.__NEXUS_VIEW__ = 'terminal';")
    .title(title)
    .inner_size(1200.0, 800.0)
    .min_inner_size(900.0, 600.0)
    .resizable(true)
    .center()
    .decorations(false)
    .transparent(true)
    .shadow(true);
  #[cfg(target_os = "macos")]
  let window_builder = window_builder.title_bar_style(tauri::TitleBarStyle::Overlay);
  let window = window_builder
    .build()
    .map_err(|err| format!("failed to create terminal window: {err}"))?;

  let _ = window.show();
  let _ = window.set_focus();

    Ok(TerminalWindowOpenResult { label, reused: false })
  }

#[tauri::command]
async fn workspace_selection_open_window(app: AppHandle) -> Result<String, String> {
  let label = next_workspace_window_label();
  let window_builder = WebviewWindowBuilder::new(&app, label.clone(), WebviewUrl::App("index.html".into()))
    .initialization_script("window.__NEXUS_VIEW__ = 'workspace-selection';")
    .title("golutra")
    .inner_size(1400.0, 900.0)
    .min_inner_size(900.0, 640.0)
    .resizable(true)
    .center()
    .decorations(false)
    .transparent(true)
    .shadow(true);
  #[cfg(target_os = "macos")]
  let window_builder = window_builder.title_bar_style(tauri::TitleBarStyle::Overlay);
  let window = window_builder
    .build()
    .map_err(|err| format!("failed to create workspace selection window: {err}"))?;

  let _ = window.show();
  let _ = window.set_focus();

  Ok(label)
}

fn now_millis() -> Result<u64, String> {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|err| format!("failed to read system time: {err}"))
    .map(|value| value.as_millis() as u64)
}

fn hash_workspace_id(path: &Path) -> Result<String, String> {
  let text = path
    .to_str()
    .ok_or_else(|| "workspace path is not valid UTF-8".to_string())?
    .to_lowercase();
  let mut hasher = Sha256::new();
  hasher.update(text.as_bytes());
  let digest = hasher.finalize();
  let mut out = String::with_capacity(digest.len() * 2);
  for byte in digest {
    out.push_str(&format!("{:02x}", byte));
  }
  Ok(out)
}

fn hash_bytes(bytes: &[u8]) -> String {
  let mut hasher = Sha256::new();
  hasher.update(bytes);
  let digest = hasher.finalize();
  let mut out = String::with_capacity(digest.len() * 2);
  for byte in digest {
    out.push_str(&format!("{:02x}", byte));
  }
  out
}

fn sanitize_relative_path(relative_path: &str) -> Result<PathBuf, String> {
  let path = PathBuf::from(relative_path);
  if path.is_absolute() {
    return Err("absolute paths are not allowed".to_string());
  }
  if path.components().any(|component| matches!(component, Component::ParentDir)) {
    return Err("parent directory segments are not allowed".to_string());
  }
  Ok(path)
}

fn sanitize_filename(filename: &str) -> Result<String, String> {
  let trimmed = filename.trim();
  if trimmed.is_empty() {
    return Err("filename is empty".to_string());
  }
  if trimmed.contains('/') || trimmed.contains('\\') {
    return Err("filename contains path separators".to_string());
  }
  Ok(trimmed.to_string())
}

fn sanitize_extension(extension: &str) -> String {
  let mut clean = String::new();
  for ch in extension.chars() {
    if ch.is_ascii_alphanumeric() {
      clean.push(ch.to_ascii_lowercase());
    }
  }
  if clean.is_empty() {
    "png".to_string()
  } else {
    clean
  }
}

fn read_json_file(path: &Path) -> Result<Option<serde_json::Value>, String> {
  if !path.exists() {
    return Ok(None);
  }
  let contents = fs::read_to_string(path).map_err(|err| format!("failed to read file: {err}"))?;
  let parsed = serde_json::from_str(&contents).map_err(|err| format!("failed to parse JSON: {err}"))?;
  Ok(Some(parsed))
}

fn write_json_file(path: &Path, payload: serde_json::Value) -> Result<(), String> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|err| format!("failed to create data directory: {err}"))?;
  }
  let contents = serde_json::to_string_pretty(&payload).map_err(|err| format!("failed to encode JSON: {err}"))?;
  fs::write(path, contents).map_err(|err| format!("failed to write file: {err}"))?;
  Ok(())
}

fn resolve_app_data_path(app: &AppHandle, relative_path: &str) -> Result<PathBuf, String> {
  let base = app
    .path()
    .app_data_dir()
    .map_err(|err| format!("failed to resolve app data dir: {err}"))?;
  let relative = sanitize_relative_path(relative_path)?;
  Ok(base.join(relative))
}

fn resolve_app_cache_path(app: &AppHandle, relative_path: &str) -> Result<PathBuf, String> {
  let base = app
    .path()
    .app_cache_dir()
    .map_err(|err| format!("failed to resolve app cache dir: {err}"))?;
  let relative = sanitize_relative_path(relative_path)?;
  Ok(base.join(relative))
}

fn resolve_avatar_file_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
  let clean = sanitize_filename(filename)?;
  let relative = format!("{AVATAR_DIR}/{clean}");
  resolve_app_data_path(app, &relative)
}

fn read_avatar_library(app: &AppHandle) -> Result<Vec<AvatarAsset>, String> {
  let path = resolve_app_data_path(app, AVATAR_LIBRARY_FILE)?;
  let data = read_json_file(&path)?;
  let parsed = match data {
    Some(value) => serde_json::from_value::<Vec<AvatarAsset>>(value)
      .map_err(|err| format!("failed to decode avatar library: {err}"))?,
    None => Vec::new(),
  };
  Ok(parsed)
}

fn write_avatar_library(app: &AppHandle, entries: &[AvatarAsset]) -> Result<(), String> {
  let payload = serde_json::to_value(entries)
    .map_err(|err| format!("failed to encode avatar library: {err}"))?;
  let path = resolve_app_data_path(app, AVATAR_LIBRARY_FILE)?;
  write_json_file(&path, payload)
}

fn resolve_workspace_path(workspace_path: &str, relative_path: &str) -> Result<PathBuf, String> {
  let base = PathBuf::from(workspace_path)
    .canonicalize()
    .map_err(|err| format!("failed to resolve workspace path: {err}"))?;
  let relative = sanitize_relative_path(relative_path)?;
  let target = base.join(relative);
  if let Some(parent) = target.parent() {
    let parent_canon = parent
      .canonicalize()
      .unwrap_or_else(|_| parent.to_path_buf());
    if !parent_canon.starts_with(&base) {
      return Err("workspace file path is outside the workspace root".to_string());
    }
  }
  Ok(target)
}

fn resolve_workspace_metadata_path(workspace_root: &Path) -> Result<PathBuf, String> {
  let base = workspace_root
    .canonicalize()
    .map_err(|err| format!("failed to resolve workspace path: {err}"))?;
  Ok(base.join(".golutra").join("workspace.json"))
}

fn ensure_project_id(workspace_root: &Path, fallback_id: &str) -> Result<String, String> {
  let metadata_path = resolve_workspace_metadata_path(workspace_root)?;
  if let Some(mut value) = read_json_file(&metadata_path)? {
    if let Some(obj) = value.as_object_mut() {
      if let Some(id) = obj
        .get("projectId")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
      {
        if !id.is_empty() {
          return Ok(id);
        }
      }
      obj.insert(
        "projectId".to_string(),
        serde_json::Value::String(fallback_id.to_string()),
      );
      write_json_file(&metadata_path, value)?;
      return Ok(fallback_id.to_string());
    }
    return Err("workspace metadata is not a JSON object".to_string());
  }

  let payload = json!({ "projectId": fallback_id });
  write_json_file(&metadata_path, payload)?;
  Ok(fallback_id.to_string())
}

fn load_recent_workspaces(app: &AppHandle) -> Result<Vec<WorkspaceEntry>, String> {
  let path = resolve_app_data_path(app, RECENT_WORKSPACES_FILE)?;
  let data = read_json_file(&path)?;
  let parsed = match data {
    Some(value) => serde_json::from_value::<Vec<WorkspaceEntry>>(value)
      .map_err(|err| format!("failed to decode recent workspaces: {err}"))?,
    None => Vec::new(),
  };
  Ok(parsed)
}

fn save_recent_workspaces(app: &AppHandle, entries: &[WorkspaceEntry]) -> Result<(), String> {
  let payload = serde_json::to_value(entries).map_err(|err| format!("failed to encode recent workspaces: {err}"))?;
  let path = resolve_app_data_path(app, RECENT_WORKSPACES_FILE)?;
  write_json_file(&path, payload)
}

fn normalize_registry_path(path: &str) -> String {
  if cfg!(windows) {
    path.to_lowercase()
  } else {
    path.to_string()
  }
}

fn is_probably_remote_path(path: &str) -> bool {
  if cfg!(windows) {
    let lower = path.to_lowercase();
    if lower.starts_with("\\\\?\\unc\\") {
      return true;
    }
    if lower.starts_with("\\\\?\\") {
      return false;
    }
    return lower.starts_with("\\\\") || lower.starts_with("//");
  }
  path.starts_with("//")
}

fn gc_workspace_registry(registry: &mut HashMap<String, WorkspaceRegistryEntry>, now: u64) {
  let mut candidates: Vec<(String, String, u64)> = registry
    .iter()
    .filter_map(|(id, entry)| {
      let age = now.saturating_sub(entry.last_accessed);
      if age < WORKSPACE_REGISTRY_GC_MIN_AGE_MS {
        return None;
      }
      if is_probably_remote_path(&entry.last_known_path) {
        return None;
      }
      Some((id.clone(), entry.last_known_path.clone(), entry.last_accessed))
    })
    .collect();

  candidates.sort_by_key(|(_, _, last_accessed)| *last_accessed);
  for (id, path, _) in candidates.into_iter().take(WORKSPACE_REGISTRY_GC_MAX_CHECKS) {
    if !Path::new(&path).exists() {
      registry.remove(&id);
    }
  }
}

fn with_workspace_registry<T>(
  app: &AppHandle,
  f: impl FnOnce(&mut HashMap<String, WorkspaceRegistryEntry>) -> Result<T, String>,
) -> Result<T, String> {
  let state = app.state::<WorkspaceRegistryLock>();
  let _guard = state
    .lock
    .lock()
    .map_err(|_| "workspace registry lock poisoned".to_string())?;
  let lock_path = resolve_app_data_path(app, WORKSPACE_REGISTRY_LOCK_FILE)?;
  if let Some(parent) = lock_path.parent() {
    fs::create_dir_all(parent).map_err(|err| format!("failed to create data directory: {err}"))?;
  }
  let lock_file = fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open(&lock_path)
    .map_err(|err| format!("failed to open workspace registry lock file: {err}"))?;
  lock_file
    .lock_exclusive()
    .map_err(|err| format!("failed to lock workspace registry: {err}"))?;

  let registry_path = resolve_app_data_path(app, WORKSPACE_REGISTRY_FILE)?;
  let data = read_json_file(&registry_path)?;
  let mut registry = match data {
    Some(value) => serde_json::from_value::<HashMap<String, WorkspaceRegistryEntry>>(value)
      .map_err(|err| format!("failed to decode workspace registry: {err}"))?,
    None => HashMap::new(),
  };

  let result = f(&mut registry);
  if result.is_ok() {
    let payload = serde_json::to_value(&registry).map_err(|err| format!("failed to encode workspace registry: {err}"))?;
    write_json_file(&registry_path, payload)?;
  }

  let _ = lock_file.unlock();
  result
}

fn workspace_registry_mismatch_error(project_id: &str, last_known_path: &str, current_path: &str) -> String {
  let payload = json!({
    "projectId": project_id,
    "lastKnownPath": last_known_path,
    "currentPath": current_path
  });
  format!("{WORKSPACE_REGISTRY_MISMATCH_PREFIX}{}", payload.to_string())
}

fn generate_project_id(seed: &str) -> Result<String, String> {
  let now = now_millis()?;
  let counter = PROJECT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
  Ok(hash_bytes(format!("{seed}-{now}-{counter}").as_bytes()))
}

fn update_workspace_project_id(metadata_path: &Path, project_id: &str) -> Result<(), String> {
  let payload = match read_json_file(metadata_path)? {
    Some(mut value) => {
      if let Some(obj) = value.as_object_mut() {
        obj.insert(
          "projectId".to_string(),
          serde_json::Value::String(project_id.to_string()),
        );
        value
      } else {
        json!({ "projectId": project_id })
      }
    }
    None => json!({ "projectId": project_id }),
  };
  write_json_file(metadata_path, payload)
}

fn resolve_workspace_local_state_path(workspace_path: &Path) -> PathBuf {
  workspace_path.join(".golutra").join("local.json")
}

fn write_workspace_local_state(workspace_path: &Path, now: u64) -> Result<(), String> {
  let path = resolve_workspace_local_state_path(workspace_path);
  let existing = read_json_file(&path)?;
  let existing_state = existing
    .and_then(|value| serde_json::from_value::<LocalWorkspaceState>(value).ok());
  let local_machine_id = match existing_state
    .as_ref()
    .map(|state| state.local_machine_id.trim())
    .filter(|value| !value.is_empty())
  {
    Some(value) => value.to_string(),
    None => generate_project_id(&format!("local-{}", workspace_path.to_string_lossy()))?,
  };
  let next = LocalWorkspaceState {
    local_machine_id,
    last_opened_at: now,
  };
  let payload = serde_json::to_value(next).map_err(|err| format!("failed to encode local workspace state: {err}"))?;
  write_json_file(&path, payload)
}

#[tauri::command]
fn workspace_recent_list(app: AppHandle) -> Result<Vec<WorkspaceEntry>, String> {
  load_recent_workspaces(&app)
}

#[tauri::command]
fn workspace_open(
  app: AppHandle,
  state: State<'_, WorkspaceWindowRegistry>,
  window_label: Option<String>,
  resolution: Option<WorkspaceRegistryResolution>,
  path: String,
) -> Result<WorkspaceOpenResult, String> {
  let window_label = window_label.unwrap_or_else(|| "main".to_string());
  let workspace_path = PathBuf::from(&path)
    .canonicalize()
    .map_err(|err| format!("failed to resolve workspace path: {err}"))?;
  if !workspace_path.is_dir() {
    return Err("selected workspace path is not a folder".to_string());
  }
  let name = workspace_path
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| "workspace folder name is not valid UTF-8".to_string())?
    .to_string();
  let path_hash = hash_workspace_id(&workspace_path)?;
  let (mut id, read_only, mut warning) = match ensure_project_id(&workspace_path, &path_hash) {
    Ok(id) => (id, false, None),
    Err(err) => (
      path_hash.clone(),
      true,
      Some(format!(
        "Workspace opened in read-only mode. Failed to write .golutra/workspace.json: {err}"
      )),
    ),
  };
  let current_path = workspace_path.to_string_lossy().to_string();
  let current_path_normalized = normalize_registry_path(&current_path);
  let now = now_millis()?;
  let mut resolved_id = id.clone();
  with_workspace_registry(&app, |registry| {
    gc_workspace_registry(registry, now);
    if let Some(entry) = registry.get_mut(&id) {
      let known_path = entry.last_known_path.clone();
      if normalize_registry_path(&known_path) != current_path_normalized {
        match resolution {
          None => {
            return Err(workspace_registry_mismatch_error(&id, &known_path, &current_path));
          }
          Some(WorkspaceRegistryResolution::Move) => {
            entry.last_known_path = current_path.clone();
            entry.last_accessed = now;
          }
          Some(WorkspaceRegistryResolution::Copy) => {
            if read_only {
              return Err("workspace is read-only; cannot create a new project id".to_string());
            }
            let metadata_path = workspace_path.join(".golutra").join("workspace.json");
            let mut next_id = generate_project_id(&current_path)?;
            while registry.contains_key(&next_id) {
              next_id = generate_project_id(&current_path)?;
            }
            update_workspace_project_id(&metadata_path, &next_id)?;
            resolved_id = next_id.clone();
            registry.insert(
              next_id,
              WorkspaceRegistryEntry {
                last_known_path: current_path.clone(),
                last_accessed: now,
              },
            );
          }
        }
      } else {
        entry.last_accessed = now;
      }
    } else {
      registry.insert(
        id.clone(),
        WorkspaceRegistryEntry {
          last_known_path: current_path.clone(),
          last_accessed: now,
        },
      );
    }
    Ok(())
  })?;
  id = resolved_id;

  let mut guard = state
    .workspaces
    .lock()
    .map_err(|_| "workspace registry lock poisoned".to_string())?;
  let existing_label = guard.iter().find_map(|(label, workspace_id)| {
    if workspace_id == &id && label != &window_label {
      Some(label.clone())
    } else {
      None
    }
  });
  if let Some(existing_label) = existing_label {
    drop(guard);
    if let Some(window) = app.get_webview_window(&existing_label) {
      let _ = window.show();
      let _ = window.set_focus();
    }
    if let Some(window) = app.get_webview_window(&window_label) {
      let _ = window.close();
    }
    return Err("workspace already open in another window".to_string());
  }
  guard.insert(window_label.clone(), id.clone());
  drop(guard);
  let entry = WorkspaceEntry {
    id: id.clone(),
    name,
    path: current_path.clone(),
    last_opened_at: now,
  };

  let mut recent = load_recent_workspaces(&app)?;
  recent.retain(|item| item.id != entry.id && item.path != entry.path);
  recent.insert(0, entry.clone());
  if recent.len() > 20 {
    recent.truncate(20);
  }
  save_recent_workspaces(&app, &recent)?;
  if !read_only {
    if let Err(err) = write_workspace_local_state(&workspace_path, now) {
      let message = format!("Failed to write .golutra/local.json: {err}");
      warning = Some(match warning {
        Some(existing) => format!("{existing}\n{message}"),
        None => message,
      });
    }
  }
  Ok(WorkspaceOpenResult {
    entry,
    read_only,
    warning,
  })
}

#[tauri::command]
fn workspace_clear_window(
  state: State<'_, WorkspaceWindowRegistry>,
  window_label: String,
) -> Result<(), String> {
  let mut guard = state
    .workspaces
    .lock()
    .map_err(|_| "workspace registry lock poisoned".to_string())?;
  guard.remove(&window_label);
  Ok(())
}

#[tauri::command]
fn storage_read_app(app: AppHandle, relative_path: String) -> Result<Option<serde_json::Value>, String> {
  let path = resolve_app_data_path(&app, &relative_path)?;
  read_json_file(&path)
}

#[tauri::command]
fn storage_write_app(
  app: AppHandle,
  relative_path: String,
  payload: serde_json::Value,
) -> Result<(), String> {
  let path = resolve_app_data_path(&app, &relative_path)?;
  write_json_file(&path, payload)
}

#[tauri::command]
fn storage_read_cache(app: AppHandle, relative_path: String) -> Result<Option<serde_json::Value>, String> {
  let path = resolve_app_cache_path(&app, &relative_path)?;
  read_json_file(&path)
}

#[tauri::command]
fn storage_write_cache(
  app: AppHandle,
  relative_path: String,
  payload: serde_json::Value,
) -> Result<(), String> {
  let path = resolve_app_cache_path(&app, &relative_path)?;
  write_json_file(&path, payload)
}

#[tauri::command]
fn storage_read_workspace(
  workspace_path: String,
  relative_path: String,
) -> Result<Option<serde_json::Value>, String> {
  let path = resolve_workspace_path(&workspace_path, &relative_path)?;
  read_json_file(&path)
}

#[tauri::command]
fn storage_write_workspace(
  workspace_path: String,
  relative_path: String,
  payload: serde_json::Value,
) -> Result<(), String> {
  let path = resolve_workspace_path(&workspace_path, &relative_path)?;
  write_json_file(&path, payload)
}

#[tauri::command]
fn avatar_list(app: AppHandle) -> Result<Vec<AvatarAsset>, String> {
  let mut entries = read_avatar_library(&app)?;
  let original_len = entries.len();
  entries.retain(|entry| {
    resolve_avatar_file_path(&app, &entry.filename)
      .map(|path| path.exists())
      .unwrap_or(false)
  });
  if entries.len() != original_len {
    write_avatar_library(&app, &entries)?;
  }
  Ok(entries)
}

#[tauri::command]
fn avatar_store(
  app: AppHandle,
  bytes: Vec<u8>,
  extension: Option<String>,
) -> Result<AvatarAsset, String> {
  if bytes.is_empty() {
    return Err("avatar bytes are empty".to_string());
  }
  if bytes.len() > MAX_AVATAR_BYTES {
    return Err("avatar exceeds max size".to_string());
  }
  let ext = sanitize_extension(extension.as_deref().unwrap_or("png"));
  let id = hash_bytes(&bytes);
  let filename = format!("{id}.{ext}");
  let path = resolve_avatar_file_path(&app, &filename)?;
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|err| format!("failed to create avatar dir: {err}"))?;
  }
  fs::write(&path, &bytes).map_err(|err| format!("failed to write avatar file: {err}"))?;

  let mut entries = read_avatar_library(&app)?;
  let mut created_at = None;
  for entry in entries.iter_mut() {
    if entry.id == id {
      entry.filename = filename.clone();
      created_at = Some(entry.created_at);
      break;
    }
  }

  let created_at = created_at.unwrap_or(now_millis()?);
  if !entries.iter().any(|entry| entry.id == id) {
    entries.insert(
      0,
      AvatarAsset {
        id: id.clone(),
        filename: filename.clone(),
        created_at,
      },
    );
  }

  write_avatar_library(&app, &entries)?;
  entries
    .into_iter()
    .find(|entry| entry.id == id)
    .ok_or_else(|| "failed to store avatar metadata".to_string())
}

#[tauri::command]
fn avatar_delete(app: AppHandle, id: String) -> Result<bool, String> {
  let mut entries = read_avatar_library(&app)?;
  if let Some(index) = entries.iter().position(|entry| entry.id == id) {
    let entry = entries.remove(index);
    let path = resolve_avatar_file_path(&app, &entry.filename)?;
    if path.exists() {
      fs::remove_file(&path).map_err(|err| format!("failed to remove avatar file: {err}"))?;
    }
    write_avatar_library(&app, &entries)?;
    return Ok(true);
  }
  Ok(false)
}

#[tauri::command]
fn avatar_resolve_path(app: AppHandle, id: String) -> Result<String, String> {
  let entries = read_avatar_library(&app)?;
  let entry = entries
    .into_iter()
    .find(|entry| entry.id == id)
    .ok_or_else(|| "avatar not found".to_string())?;
  let path = resolve_avatar_file_path(&app, &entry.filename)?;
  if !path.exists() {
    return Err("avatar file missing".to_string());
  }
  Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn avatar_read(app: AppHandle, id: String) -> Result<AvatarContent, String> {
  let entries = read_avatar_library(&app)?;
  let entry = entries
    .into_iter()
    .find(|entry| entry.id == id)
    .ok_or_else(|| "avatar not found".to_string())?;
  let path = resolve_avatar_file_path(&app, &entry.filename)?;
  if !path.exists() {
    return Err("avatar file missing".to_string());
  }
  let bytes = fs::read(&path).map_err(|err| format!("failed to read avatar file: {err}"))?;
  let mime = Path::new(&entry.filename)
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|ext| match ext.to_ascii_lowercase().as_str() {
      "png" => "image/png",
      "jpg" | "jpeg" => "image/jpeg",
      "webp" => "image/webp",
      "gif" => "image/gif",
      _ => "application/octet-stream",
    })
    .unwrap_or("application/octet-stream")
    .to_string();
  Ok(AvatarContent { bytes, mime })
}

#[tauri::command]
fn terminal_create(
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
  initial_data: Option<String>,
) -> Result<String, String> {
  let requested_id = session_id
    .as_deref()
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty());
  if let Some(value) = initial_data.as_deref() {
    let sanitized = value.replace('\r', "\\r").replace('\n', "\\n");
    log::info!("terminal_create initial_data={}", sanitized);
  } else {
    log::info!("terminal_create initial_data=<none>");
  }
  let session_id = requested_id
    .unwrap_or_else(|| format!("term-{}", SESSION_COUNTER.fetch_add(1, Ordering::Relaxed)));
  let keep_alive = keep_alive.unwrap_or(false);
  let owner_window_label = if keep_alive {
    None
  } else {
    Some(window.label().to_string())
  };
  {
    let guard = state
      .sessions
      .lock()
      .map_err(|_| "terminal session lock poisoned".to_string())?;
    if guard.total_bytes >= TOTAL_BUFFER_LIMIT_BYTES {
      return Err("terminal buffer limit reached".to_string());
    }
    if guard.sessions.contains_key(&session_id) {
      return Err("terminal session already exists".to_string());
    }
  }

  let cols = cols.unwrap_or(80).max(1);
  let rows = rows.unwrap_or(24).max(1);
  let pty_system = native_pty_system();
  let pair = pty_system
    .openpty(PtySize {
      rows,
      cols,
      pixel_width: 0,
      pixel_height: 0,
    })
    .map_err(|err| format!("failed to open pty: {err}"))?;

  let (shell, args) = default_shell_spec();
  let mut cmd = CommandBuilder::new(shell);
  if !args.is_empty() {
    cmd.args(args);
  }
  if let Some(dir) = cwd
    .as_deref()
    .map(str::trim)
    .filter(|value| !value.is_empty())
  {
    cmd.cwd(dir);
  }
  if cmd.get_env("TERM").is_none() {
    cmd.env("TERM", "xterm-256color");
  }

  let child = pair
    .slave
    .spawn_command(cmd)
    .map_err(|err| format!("failed to spawn pty command: {err}"))?;
  let master = pair.master;
  let reader = master
    .try_clone_reader()
    .map_err(|err| format!("failed to open pty reader: {err}"))?;
  let writer = master
    .take_writer()
    .map_err(|err| format!("failed to open pty writer: {err}"))?;
  let writer = Arc::new(Mutex::new(writer));

  let mut cleanup_killer = child.clone_killer();
  let handle = TerminalHandle {
    master,
    writer: Arc::clone(&writer),
    killer: cleanup_killer.clone_killer(),
  };

  let status_payload = match register_session(
    &state,
    &session_id,
    member_id,
    workspace_id,
    keep_alive,
    owner_window_label,
    handle,
  ) {
    Ok(payload) => payload,
    Err(err) => {
      let _ = cleanup_killer.kill();
      return Err(err);
    }
  };

  let initial_payload = initial_data.filter(|data| !data.is_empty());
  let (initial_timeout_ms, initial_delay_ms) = if cfg!(windows) { (1200, 300) } else { (500, 0) };
  let initial_write = initial_payload.map(|payload| {
    Arc::new(InitialWriteState {
      session_id: session_id.clone(),
      payload,
      writer: Arc::clone(&writer),
      sessions: state.sessions.clone(),
      app: app.clone(),
      sent: AtomicBool::new(false),
    })
  });
  if let Some(state) = initial_write.as_ref() {
    state.schedule(initial_timeout_ms, "timeout");
  }

  spawn_pty_reader(
    reader,
    app.clone(),
    state.sessions.clone(),
    session_id.clone(),
    initial_write,
    initial_delay_ms,
  );
  spawn_exit_watcher(child, app.clone(), state.sessions.clone(), session_id.clone());

  let _ = app.emit("terminal-status-change", status_payload);
  Ok(session_id)
}

#[tauri::command]
fn terminal_write(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  session_id: String,
  data: String,
) -> Result<(), String> {
  ensure_session_active(&state, &session_id)?;
  let writer = {
    let guard = state
      .sessions
      .lock()
      .map_err(|_| "terminal session lock poisoned".to_string())?;
    let session = guard
      .sessions
      .get(&session_id)
      .ok_or_else(|| "terminal session not found".to_string())?;
    let handle = session
      .handle
      .as_ref()
      .ok_or_else(|| "terminal session handle missing".to_string())?;
    Arc::clone(&handle.writer)
  };
  let mut writer = writer
    .lock()
    .map_err(|_| "terminal writer lock poisoned".to_string())?;
  writer
    .write_all(data.as_bytes())
    .and_then(|_| writer.flush())
    .map_err(|err| format!("failed to write to pty: {err}"))?;
  drop(writer);
  mark_session_working_on_input(&state.sessions, &app, &session_id, &data);
  Ok(())
}

#[tauri::command]
fn terminal_resize(
  state: State<'_, TerminalManager>,
  session_id: String,
  cols: u16,
  rows: u16,
) -> Result<(), String> {
  ensure_session_active(&state, &session_id)?;
  let size = PtySize {
    rows: rows.max(1),
    cols: cols.max(1),
    pixel_width: 0,
    pixel_height: 0,
  };
  let guard = state
    .sessions
    .lock()
    .map_err(|_| "terminal session lock poisoned".to_string())?;
  let session = guard
    .sessions
    .get(&session_id)
    .ok_or_else(|| "terminal session not found".to_string())?;
  let handle = session
    .handle
    .as_ref()
    .ok_or_else(|| "terminal session handle missing".to_string())?;
  handle
    .master
    .resize(size)
    .map_err(|err| format!("failed to resize pty: {err}"))
}

#[tauri::command]
fn terminal_close(
  app: AppHandle,
  state: State<'_, TerminalManager>,
  session_id: String,
  preserve: Option<bool>,
) -> Result<(), String> {
  let preserve = preserve.unwrap_or(false);
  let (status_payload, killer) = {
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
    } else {
      let removed = guard.sessions.remove(&session_id);
      if let Some(removed) = removed {
        guard.total_bytes = guard.total_bytes.saturating_sub(removed.buffer.len());
        if let Some(handle) = removed.handle {
          killer = Some(handle.killer);
        }
      }
    }
    (status_payload, killer)
  };
  if let Some(mut killer) = killer {
    let _ = killer.kill();
  }
  if let Some(payload) = status_payload {
    let _ = app.emit("terminal-status-change", payload);
  }
  Ok(())
}

#[tauri::command]
fn get_session_history(state: State<'_, TerminalManager>, session_id: String) -> Result<String, String> {
  let guard = state
    .sessions
    .lock()
    .map_err(|_| "terminal session lock poisoned".to_string())?;
  let session = guard
    .sessions
    .get(&session_id)
    .ok_or_else(|| "terminal session not found".to_string())?;
  let (first, second) = session.buffer.as_slices();
  let mut bytes = Vec::with_capacity(first.len() + second.len());
  bytes.extend_from_slice(first);
  bytes.extend_from_slice(second);
  Ok(String::from_utf8_lossy(&bytes).to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(TerminalManager::default())
    .manage(WorkspaceRegistryLock::default())
    .manage(WorkspaceWindowRegistry::default())
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      let sessions = app.state::<TerminalManager>().sessions.clone();
      spawn_status_poller(app.handle().clone(), sessions);
      Ok(())
    })
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
      terminal_open_window,
      workspace_selection_open_window,
      terminal_create,
      terminal_write,
      terminal_resize,
      terminal_close,
      get_session_history,
      workspace_recent_list,
      workspace_open,
      workspace_clear_window,
      storage_read_app,
      storage_write_app,
      storage_read_cache,
      storage_write_cache,
      storage_read_workspace,
      storage_write_workspace,
      avatar_list,
      avatar_store,
      avatar_delete,
      avatar_resolve_path,
      avatar_read
    ])
    .on_window_event(|window, event| {
      if matches!(event, WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed) {
        let app = window.app_handle();
        if matches!(event, WindowEvent::Destroyed) {
          let state = app.state::<TerminalManager>();
          let _ = cleanup_ephemeral_sessions_for_window(&state, window.label());
        }
        if let Ok(mut guard) = app.state::<WorkspaceWindowRegistry>().workspaces.lock() {
          guard.remove(window.label());
        }
        if app.webview_windows().len() <= 1 {
          let state = app.state::<TerminalManager>();
          if !has_active_sessions(&state) {
            let _ = shutdown_sessions(&state);
          }
        }
      }
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
