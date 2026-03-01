//! 诊断日志收集：承载前后端链路数据并写入 create_chat.log。
//! 边界：只做数据记录与一致性标记，不影响业务流程。

use std::{
  collections::HashMap,
  fs::{create_dir_all, OpenOptions},
  io::Write,
  path::PathBuf,
  sync::Mutex,
  time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::AppHandle;

use super::gate::backend_passive_enabled;
use crate::platform::resolve_log_dir;

const LOG_FILE_NAME: &str = "create_chat.log";
const STRICT_KEYS: [&str; 8] = [
  "memberId",
  "member_id",
  "terminalId",
  "terminal_id",
  "terminalType",
  "terminal_type",
  "terminalCommand",
  "terminal_command",
];
const SOFT_KEYS: [&str; 4] = ["windowLabel", "window_label", "conversationId", "conversation_id"];

#[derive(Default)]
struct MemberContext {
  last_values: HashMap<String, String>,
}

struct DiagnosticsRun {
  members: HashMap<String, MemberContext>,
}

impl DiagnosticsRun {
  fn new() -> Self {
    Self {
      members: HashMap::new(),
    }
  }
}

pub struct DiagnosticsState {
  runs: Mutex<HashMap<String, DiagnosticsRun>>,
  member_index: Mutex<HashMap<String, String>>,
  session_index: Mutex<HashMap<String, String>>,
  conversation_index: Mutex<HashMap<String, String>>,
  window_index: Mutex<HashMap<String, String>>,
  workspace_index: Mutex<HashMap<String, String>>,
  backend_run_id: Mutex<Option<String>>,
  log_path: PathBuf,
}

impl DiagnosticsState {
  pub fn new() -> Self {
    let log_path = resolve_log_dir().join(LOG_FILE_NAME);
    Self {
      runs: Mutex::new(HashMap::new()),
      member_index: Mutex::new(HashMap::new()),
      session_index: Mutex::new(HashMap::new()),
      conversation_index: Mutex::new(HashMap::new()),
      window_index: Mutex::new(HashMap::new()),
      workspace_index: Mutex::new(HashMap::new()),
      backend_run_id: Mutex::new(None),
      log_path,
    }
  }

  fn now_millis() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|value| value.as_millis() as u64)
      .unwrap_or(0)
  }

  fn resolve_run_id(
    &self,
    member_id: Option<&str>,
    terminal_id: Option<&str>,
    conversation_id: Option<&str>,
    window_label: Option<&str>,
    workspace_id: Option<&str>,
  ) -> Option<String> {
    if let Some(member_id) = member_id {
      if let Ok(index) = self.member_index.lock() {
        if let Some(run_id) = index.get(member_id) {
          return Some(run_id.clone());
        }
      }
    }
    if let Some(terminal_id) = terminal_id {
      if let Ok(index) = self.session_index.lock() {
        if let Some(run_id) = index.get(terminal_id) {
          return Some(run_id.clone());
        }
      }
    }
    if let Some(conversation_id) = conversation_id {
      if let Ok(index) = self.conversation_index.lock() {
        if let Some(run_id) = index.get(conversation_id) {
          return Some(run_id.clone());
        }
      }
    }
    if let Some(window_label) = window_label {
      if let Ok(index) = self.window_index.lock() {
        if let Some(run_id) = index.get(window_label) {
          return Some(run_id.clone());
        }
      }
    }
    if let Some(workspace_id) = workspace_id {
      if let Ok(index) = self.workspace_index.lock() {
        if let Some(run_id) = index.get(workspace_id) {
          return Some(run_id.clone());
        }
      }
    }
    None
  }

  fn write_log(&self, entry: &DiagnosticsLogEntry) {
    if let Some(parent) = self.log_path.parent() {
      let _ = create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new()
      .create(true)
      .append(true)
      .open(&self.log_path)
    {
      if let Ok(line) = serde_json::to_string(entry) {
        let _ = writeln!(file, "{line}");
      }
    }
  }

  fn track_member(&self, run_id: &str, member_id: &str) {
    if let Ok(mut index) = self.member_index.lock() {
      index.insert(member_id.to_string(), run_id.to_string());
    }
    if let Ok(mut runs) = self.runs.lock() {
      let run = runs.entry(run_id.to_string()).or_insert_with(DiagnosticsRun::new);
      run.members.entry(member_id.to_string()).or_default();
    }
  }

  fn update_index(&self, index: &Mutex<HashMap<String, String>>, run_id: &str, key: &str) {
    if key.trim().is_empty() {
      return;
    }
    if let Ok(mut map) = index.lock() {
      map.insert(key.to_string(), run_id.to_string());
    }
  }

  fn clear_run_from_index(&self, index: &Mutex<HashMap<String, String>>, run_id: &str) {
    let mut remove_keys = Vec::new();
    if let Ok(map) = index.lock() {
      for (key, value) in map.iter() {
        if value == run_id {
          remove_keys.push(key.clone());
        }
      }
    }
    if remove_keys.is_empty() {
      return;
    }
    if let Ok(mut map) = index.lock() {
      for key in remove_keys {
        map.remove(&key);
      }
    }
  }

  fn build_changes(
    &self,
    run_id: &str,
    member_id: Option<&str>,
    payload: &Value,
  ) -> (Value, Vec<String>) {
    let mut errors = Vec::new();
    let mut changes = serde_json::Map::new();
    let object = match payload.as_object() {
      Some(value) => value,
      None => return (Value::Object(changes), errors),
    };
    let member_key = member_id.unwrap_or("unknown");
    // 仅对已注册的终端成员启用严格变更校验，避免跨链路噪音。
    let is_tracked_member = if member_key == "unknown" {
      false
    } else if let Ok(index) = self.member_index.lock() {
      index.get(member_key).map(|value| value == run_id).unwrap_or(false)
    } else {
      false
    };
    let mut runs = match self.runs.lock() {
      Ok(guard) => guard,
      Err(_) => return (Value::Object(changes), errors),
    };
    let run = match runs.get_mut(run_id) {
      Some(run) => run,
      None => {
        errors.push("run missing".to_string());
        return (Value::Object(changes), errors);
      }
    };
    let member = run.members.entry(member_key.to_string()).or_default();

    let track_value = |key: &str, value: &Value, strict: bool, changes: &mut serde_json::Map<String, Value>, errors: &mut Vec<String>, member: &mut MemberContext| {
      let next = match value.as_str() {
        Some(value) => value.to_string(),
        None => match value {
          Value::Number(num) => num.to_string(),
          Value::Bool(value) => value.to_string(),
          _ => return,
        },
      };
      let prev = member.last_values.get(key).cloned();
      let changed = match prev.as_ref() {
        Some(prev) => prev != &next,
        None => true,
      };
      if strict && prev.is_some() && changed {
        errors.push(format!("{key} changed: {prev:?} -> {next}"));
      }
      member.last_values.insert(key.to_string(), next.clone());
      changes.insert(
        key.to_string(),
        json!({
          "prev": prev,
          "next": next,
          "changed": changed,
          "strict": strict
        }),
      );
    };

    for key in STRICT_KEYS {
      if let Some(value) = object.get(key) {
        track_value(key, value, is_tracked_member, &mut changes, &mut errors, member);
      }
    }
    for key in SOFT_KEYS {
      if let Some(value) = object.get(key) {
        track_value(key, value, false, &mut changes, &mut errors, member);
      }
    }

    (Value::Object(changes), errors)
  }

  fn log_event(
    &self,
    run_id: &str,
    round: Option<u8>,
    member_id: Option<String>,
    source: &str,
    step_id: &str,
    payload: Value,
    client_ts: Option<u64>,
    seq: Option<u64>,
  ) {
    let (changes, errors) = self.build_changes(run_id, member_id.as_deref(), &payload);
    let entry = DiagnosticsLogEntry {
      ts: Self::now_millis(),
      client_ts,
      seq,
      event: "trace".to_string(),
      run_id: run_id.to_string(),
      round,
      member_id,
      source: source.to_string(),
      step_id: step_id.to_string(),
      payload,
      changes,
      errors,
    };
    self.write_log(&entry);
  }

  // 后端被动监控：没有前端 run 时，自动创建一个后端 run 兜底。
  fn ensure_backend_run(&self, workspace_id: Option<&str>) -> Option<String> {
    if !backend_passive_enabled() {
      return None;
    }
    let mut guard = match self.backend_run_id.lock() {
      Ok(value) => value,
      Err(_) => return None,
    };
    if let Some(run_id) = guard.as_ref() {
      return Some(run_id.clone());
    }
    let run_id = format!("backend-{}", Self::now_millis());
    if let Ok(mut runs) = self.runs.lock() {
      runs.insert(run_id.clone(), DiagnosticsRun::new());
    } else {
      return None;
    }
    if let Some(workspace_id) = workspace_id {
      self.update_index(&self.workspace_index, &run_id, workspace_id);
    }
    let entry = DiagnosticsLogEntry {
      ts: DiagnosticsState::now_millis(),
      client_ts: None,
      seq: None,
      event: "run-start".to_string(),
      run_id: run_id.clone(),
      round: None,
      member_id: None,
      source: "backend".to_string(),
      step_id: "run-start".to_string(),
      payload: json!({
        "label": "backend-passive",
        "workspaceId": workspace_id,
        "appLabel": "backend",
      }),
      changes: Value::Null,
      errors: Vec::new(),
    };
    self.write_log(&entry);
    *guard = Some(run_id.clone());
    Some(run_id)
  }
}

impl Default for DiagnosticsState {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Serialize)]
struct DiagnosticsLogEntry {
  ts: u64,
  #[serde(rename = "clientTs")]
  client_ts: Option<u64>,
  seq: Option<u64>,
  event: String,
  #[serde(rename = "runId")]
  run_id: String,
  round: Option<u8>,
  #[serde(rename = "memberId")]
  member_id: Option<String>,
  source: String,
  #[serde(rename = "stepId")]
  step_id: String,
  payload: Value,
  changes: Value,
  errors: Vec<String>,
}

#[derive(Deserialize)]
pub struct FrontendLogEntry {
  #[serde(rename = "runId")]
  run_id: String,
  round: Option<u8>,
  #[serde(rename = "memberId")]
  member_id: Option<String>,
  #[serde(rename = "stepId")]
  step_id: String,
  payload: Value,
  #[serde(rename = "clientTs")]
  client_ts: Option<u64>,
  seq: Option<u64>,
}

pub fn diagnostics_start_run(
  app: AppHandle,
  state: tauri::State<'_, DiagnosticsState>,
  run_id: String,
  workspace_id: Option<String>,
  label: Option<String>,
) -> Result<(), String> {
  let mut runs = state
    .runs
    .lock()
    .map_err(|_| "diagnostics run lock poisoned".to_string())?;
  runs.insert(run_id.clone(), DiagnosticsRun::new());
  drop(runs);
  if let Some(workspace_id) = workspace_id.as_deref() {
    state.update_index(&state.workspace_index, &run_id, workspace_id);
  }
  let entry = DiagnosticsLogEntry {
    ts: DiagnosticsState::now_millis(),
    client_ts: None,
    seq: None,
    event: "run-start".to_string(),
    run_id: run_id.clone(),
    round: None,
    member_id: None,
    source: "backend".to_string(),
    step_id: "run-start".to_string(),
    payload: json!({
      "label": label,
      "workspaceId": workspace_id,
      "appLabel": app.package_info().name.clone(),
    }),
    changes: Value::Null,
    errors: Vec::new(),
  };
  state.write_log(&entry);
  Ok(())
}

pub fn diagnostics_end_run(
  state: tauri::State<'_, DiagnosticsState>,
  run_id: String,
  status: Option<String>,
) -> Result<(), String> {
  let mut runs = state
    .runs
    .lock()
    .map_err(|_| "diagnostics run lock poisoned".to_string())?;
  runs.remove(&run_id);
  drop(runs);
  state.clear_run_from_index(&state.member_index, &run_id);
  state.clear_run_from_index(&state.session_index, &run_id);
  state.clear_run_from_index(&state.conversation_index, &run_id);
  state.clear_run_from_index(&state.window_index, &run_id);
  state.clear_run_from_index(&state.workspace_index, &run_id);
  let entry = DiagnosticsLogEntry {
    ts: DiagnosticsState::now_millis(),
    client_ts: None,
    seq: None,
    event: "run-end".to_string(),
    run_id: run_id.clone(),
    round: None,
    member_id: None,
    source: "backend".to_string(),
    step_id: "run-end".to_string(),
    payload: json!({ "status": status.unwrap_or_else(|| "unknown".to_string()) }),
    changes: Value::Null,
    errors: Vec::new(),
  };
  state.write_log(&entry);
  Ok(())
}

pub fn diagnostics_register_member(
  state: tauri::State<'_, DiagnosticsState>,
  run_id: String,
  member_id: String,
  terminal_type: Option<String>,
  terminal_command: Option<String>,
  name: Option<String>,
) -> Result<(), String> {
  state.track_member(&run_id, &member_id);
  let payload = json!({
    "memberId": member_id,
    "terminalType": terminal_type,
    "terminalCommand": terminal_command,
    "name": name,
  });
  state.log_event(
    &run_id,
    None,
    Some(member_id),
    "backend",
    "diagnostics.register-member",
    payload,
    None,
    None,
  );
  Ok(())
}

pub fn diagnostics_register_session(
  state: tauri::State<'_, DiagnosticsState>,
  run_id: String,
  terminal_id: String,
  member_id: Option<String>,
) -> Result<(), String> {
  state.update_index(&state.session_index, &run_id, &terminal_id);
  if let Some(member_id) = member_id.as_deref() {
    state.track_member(&run_id, member_id);
  }
  let payload = json!({
    "terminalId": terminal_id,
    "memberId": member_id,
  });
  state.log_event(
    &run_id,
    None,
    member_id,
    "backend",
    "diagnostics.register-session",
    payload,
    None,
    None,
  );
  Ok(())
}

pub fn diagnostics_register_conversation(
  state: tauri::State<'_, DiagnosticsState>,
  run_id: String,
  conversation_id: String,
  member_id: Option<String>,
) -> Result<(), String> {
  state.update_index(&state.conversation_index, &run_id, &conversation_id);
  if let Some(member_id) = member_id.as_deref() {
    state.track_member(&run_id, member_id);
  }
  let payload = json!({
    "conversationId": conversation_id,
    "memberId": member_id,
  });
  state.log_event(
    &run_id,
    None,
    member_id,
    "backend",
    "diagnostics.register-conversation",
    payload,
    None,
    None,
  );
  Ok(())
}

pub fn diagnostics_register_window(
  state: tauri::State<'_, DiagnosticsState>,
  run_id: String,
  window_label: String,
) -> Result<(), String> {
  state.update_index(&state.window_index, &run_id, &window_label);
  let payload = json!({ "windowLabel": window_label });
  state.log_event(
    &run_id,
    None,
    None,
    "backend",
    "diagnostics.register-window",
    payload,
    None,
    None,
  );
  Ok(())
}

pub fn diagnostics_log_frontend_event(
  state: tauri::State<'_, DiagnosticsState>,
  run_id: String,
  round: Option<u8>,
  member_id: Option<String>,
  step_id: String,
  payload: Value,
  client_ts: Option<u64>,
  seq: Option<u64>,
) -> Result<(), String> {
  state.log_event(
    &run_id,
    round,
    member_id,
    "frontend",
    &step_id,
    payload,
    client_ts,
    seq,
  );
  Ok(())
}

pub fn diagnostics_log_frontend_batch(
  state: tauri::State<'_, DiagnosticsState>,
  entries: Vec<FrontendLogEntry>,
) -> Result<(), String> {
  for entry in entries {
    state.log_event(
      &entry.run_id,
      entry.round,
      entry.member_id,
      "frontend",
      &entry.step_id,
      entry.payload,
      entry.client_ts,
      entry.seq,
    );
  }
  Ok(())
}

pub fn diagnostics_log_snapshot_triplet(
  state: tauri::State<'_, DiagnosticsState>,
  run_id: String,
  round: u8,
  member_id: String,
  front_before: Vec<String>,
  backend_stored: Vec<String>,
  front_reopen: Vec<String>,
) -> Result<(), String> {
  let payload = json!({
    "frontBefore": front_before,
    "backendStored": backend_stored,
    "frontReopen": front_reopen,
  });
  state.log_event(
    &run_id,
    Some(round),
    Some(member_id.clone()),
    "backend",
    "snapshot.triplet",
    payload,
    None,
    None,
  );

  let equal_front_backend = front_before == backend_stored;
  let equal_front_reopen = front_before == front_reopen;
  let equal_backend_reopen = backend_stored == front_reopen;
  let summary = json!({
    "frontBackend": equal_front_backend,
    "frontReopen": equal_front_reopen,
    "backendReopen": equal_backend_reopen,
    "counts": {
      "front": front_before.len(),
      "backend": backend_stored.len(),
      "reopen": front_reopen.len(),
    }
  });
  state.log_event(
    &run_id,
    Some(round),
    Some(member_id),
    "backend",
    "snapshot.compare",
    summary,
    None,
    None,
  );
  Ok(())
}

pub fn diagnostics_log_chat_consistency(
  state: tauri::State<'_, DiagnosticsState>,
  run_id: String,
  round: u8,
  member_id: String,
  reply_text: Option<String>,
  reply_missing: Option<bool>,
  terminal_lines: Vec<String>,
) -> Result<(), String> {
  let reply_missing = reply_missing.unwrap_or(false) || reply_text.as_ref().map(|text| text.is_empty()).unwrap_or(true);
  let reply_lines: Vec<&str> = reply_text
    .as_deref()
    .map(|text| text.split('\n').collect())
    .unwrap_or_default();
  let mut matched_lines = 0usize;
  let mut missing_lines = Vec::new();
  for line in &reply_lines {
    if line.is_empty() {
      continue;
    }
    let found = terminal_lines.iter().any(|candidate| candidate.contains(line));
    if found {
      matched_lines = matched_lines.saturating_add(1);
    } else {
      missing_lines.push((*line).to_string());
    }
  }
  let reply_line_count = reply_lines.iter().filter(|line| !line.is_empty()).count();
  let contains = reply_line_count > 0 && missing_lines.is_empty();
  let payload = json!({
    "reply": reply_text,
    "replyMissing": reply_missing,
    "terminalLines": terminal_lines,
    "contains": contains,
    "replyLineCount": reply_line_count,
    "matchedLineCount": matched_lines,
    "missingLines": missing_lines,
  });
  state.log_event(
    &run_id,
    Some(round),
    Some(member_id),
    "backend",
    "chat.consistency",
    payload,
    None,
    None,
  );
  Ok(())
}

pub fn diagnostics_log_backend_event(
  state: &DiagnosticsState,
  member_id: Option<String>,
  terminal_id: Option<String>,
  conversation_id: Option<String>,
  window_label: Option<String>,
  workspace_id: Option<String>,
  step_id: &str,
  payload: Value,
) {
  let run_id = state.resolve_run_id(
    member_id.as_deref(),
    terminal_id.as_deref(),
    conversation_id.as_deref(),
    window_label.as_deref(),
    workspace_id.as_deref(),
  );
  let run_id = match run_id {
    Some(run_id) => run_id,
    None => match state.ensure_backend_run(workspace_id.as_deref()) {
      Some(run_id) => run_id,
      None => return,
    },
  };
  if let Some(member_id) = member_id.as_deref() {
    state.track_member(&run_id, member_id);
  }
  state.log_event(&run_id, None, member_id, "backend", step_id, payload, None, None);
}
