use std::{
  collections::HashMap,
  fs,
  path::{Component, Path, PathBuf},
  sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
  },
  time::{SystemTime, UNIX_EPOCH},
};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_log::{Target, TargetKind};

mod chat_db;
mod terminal;
use chat_db::{
  chat_clear_all_messages, chat_clear_conversation, chat_create_group, chat_delete_conversation, chat_ensure_direct,
  chat_get_messages, chat_list_conversations, chat_mark_conversation_read_latest, chat_rename_conversation,
  chat_repair_messages, chat_send_message, chat_set_conversation_members, chat_set_conversation_settings,
  chat_ulid_new, ChatDbManager,
};
use terminal::{
  cleanup_ephemeral_sessions_for_window, has_active_sessions, shutdown_sessions, spawn_status_poller,
  terminal_ack, terminal_attach, terminal_close, terminal_create, terminal_dispatch, terminal_resize,
  terminal_set_active, terminal_set_member_status, terminal_write, TerminalManager,
};

static WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);
static WORKSPACE_WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);
static PROJECT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
const TERMINAL_WINDOW_LABEL: &str = "terminal-main";
const RECENT_WORKSPACES_FILE: &str = "recent-workspaces.json";
const WORKSPACE_REGISTRY_FILE: &str = "workspace-registry.json";
const WORKSPACE_REGISTRY_LOCK_FILE: &str = "workspace-registry.lock";
const AVATAR_LIBRARY_FILE: &str = "avatar-library.json";
const AVATAR_DIR: &str = "avatars";
const MAX_AVATAR_BYTES: usize = 2 * 1024 * 1024;
const WORKSPACE_REGISTRY_MISMATCH_PREFIX: &str = "workspace_registry_mismatch:";
const WORKSPACE_REGISTRY_GC_MIN_AGE_MS: u64 = 1000 * 60 * 60 * 24 * 30;
const WORKSPACE_REGISTRY_GC_MAX_CHECKS: usize = 12;

fn find_repo_root(path: &Path) -> Option<PathBuf> {
  for ancestor in path.ancestors() {
    if ancestor.file_name().and_then(|name| name.to_str()) == Some("src-tauri") {
      return ancestor.parent().map(|parent| parent.to_path_buf());
    }
  }
  None
}

fn resolve_log_dir() -> PathBuf {
  if let Ok(value) = std::env::var("GOLUTRA_LOG_DIR") {
    return PathBuf::from(value);
  }
  let cwd = std::env::current_dir().ok();
  let exe_dir = std::env::current_exe()
    .ok()
    .and_then(|path| path.parent().map(|parent| parent.to_path_buf()));
  let root = cwd
    .as_ref()
    .and_then(|path| find_repo_root(path))
    .or_else(|| exe_dir.as_ref().and_then(|path| find_repo_root(path)))
    .or_else(|| cwd.clone())
    .unwrap_or_else(|| PathBuf::from("."));
  root.join("log")
}

struct WorkspaceRegistryLock {
  lock: Mutex<()>,
}

struct WorkspaceWindowRegistry {
  workspaces: Mutex<HashMap<String, String>>,
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
    workspace_path: Option<String>,
  ) -> Result<TerminalWindowOpenResult, String> {
  let reuse = reuse.unwrap_or(true);
  let workspace_id = workspace_id
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty());
  let workspace_name = workspace_name
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty());
  let workspace_path = workspace_path
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
  let init_payload = json!({
    "id": workspace_id.as_deref(),
    "name": workspace_name.as_deref(),
    "path": workspace_path.as_deref()
  });
  let init_script = format!("window.__NEXUS_VIEW__ = 'terminal'; window.__NEXUS_WORKSPACE__ = {init_payload};");
  let window_builder = WebviewWindowBuilder::new(&app, label.clone(), WebviewUrl::App("index.html".into()))
    .initialization_script(&init_script)
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

pub(crate) fn resolve_app_data_path(app: &AppHandle, relative_path: &str) -> Result<PathBuf, String> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(TerminalManager::default())
    .manage(ChatDbManager::default())
    .manage(WorkspaceRegistryLock::default())
    .manage(WorkspaceWindowRegistry::default())
    .setup(|app| {
      if cfg!(debug_assertions) {
        let log_dir = resolve_log_dir();
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .targets([
              Target::new(TargetKind::Stdout),
              Target::new(TargetKind::Folder {
                path: log_dir,
                file_name: Some("sidecar-debug".to_string()),
              }),
            ])
            .build(),
        )?;
      }
      let manager = app.state::<TerminalManager>();
      spawn_status_poller(app.handle().clone(), &manager);
      Ok(())
    })
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
      terminal_open_window,
      workspace_selection_open_window,
      terminal_create,
      terminal_attach,
      terminal_write,
      terminal_ack,
      terminal_set_active,
      terminal_set_member_status,
      terminal_dispatch,
      terminal_resize,
      terminal_close,
      chat_ulid_new,
      chat_repair_messages,
      chat_clear_all_messages,
      chat_list_conversations,
      chat_get_messages,
      chat_mark_conversation_read_latest,
      chat_send_message,
      chat_create_group,
      chat_ensure_direct,
      chat_set_conversation_settings,
      chat_rename_conversation,
      chat_clear_conversation,
      chat_delete_conversation,
      chat_set_conversation_members,
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
    .build(tauri::generate_context!())
    .expect("error while building tauri application")
    .run(|app, event| {
      if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
        let state = app.state::<TerminalManager>();
        let _ = shutdown_sessions(&state);
      }
    });
}
