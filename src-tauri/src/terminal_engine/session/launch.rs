//! 终端启动策略：封装默认终端与兜底链路，避免命令入口堆积分支。

use std::collections::HashSet;

use serde::Deserialize;

use crate::runtime::{
  list_terminal_environments, lookup_binary, read_app_json, spawn_command, spawn_shell,
  StorageManager,
};
use crate::runtime::pty::SpawnedPty;

use crate::terminal_engine::errors::{TerminalError, TerminalErrorCode};

use super::state::TerminalType;

const GLOBAL_SETTINGS_PATH: &str = "global-settings.json";

#[derive(Clone, Default)]
pub(crate) struct DefaultTerminalPreference {
  pub(crate) name: Option<String>,
  pub(crate) path: Option<String>,
}

pub(crate) struct TerminalLaunchResult {
  pub(crate) spawned: SpawnedPty,
  pub(crate) terminal_type: TerminalType,
  pub(crate) fallback_used: bool,
  pub(crate) default_terminal: DefaultTerminalPreference,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredGlobalSettings {
  members: Option<StoredMemberSettings>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredMemberSettings {
  default_terminal_name: Option<String>,
  default_terminal_path: Option<String>,
}

struct ShellCandidate {
  label: String,
  path: Option<String>,
}

fn normalize_preference_value(value: Option<String>) -> Option<String> {
  let trimmed = value.unwrap_or_default().trim().to_string();
  if trimmed.is_empty() {
    return None;
  }
  Some(trimmed)
}

fn load_default_terminal_preference_from_settings(
  storage: &StorageManager,
) -> Result<Option<DefaultTerminalPreference>, String> {
  let raw = read_app_json(storage, GLOBAL_SETTINGS_PATH)?;
  let Some(raw) = raw else {
    return Ok(None);
  };
  let parsed: StoredGlobalSettings = serde_json::from_value(raw)
    .map_err(|err| format!("failed to parse global settings: {err}"))?;
  let members = match parsed.members {
    Some(value) => value,
    None => {
      return Ok(Some(DefaultTerminalPreference::default()));
    }
  };
  Ok(Some(DefaultTerminalPreference {
    name: normalize_preference_value(members.default_terminal_name),
    path: normalize_preference_value(members.default_terminal_path),
  }))
}

fn load_default_terminal_preference(storage: &StorageManager) -> DefaultTerminalPreference {
  match load_default_terminal_preference_from_settings(storage) {
    Ok(Some(value)) => return value,
    Ok(None) => {}
    Err(err) => {
      log::warn!("failed to read global settings: {err}");
    }
  }
  DefaultTerminalPreference::default()
}

fn normalize_candidate_key(path: Option<&str>) -> String {
  let key = path.unwrap_or("<system-default>");
  if cfg!(windows) {
    return key.to_lowercase();
  }
  key.to_string()
}

fn push_candidate(
  list: &mut Vec<ShellCandidate>,
  seen: &mut HashSet<String>,
  label: impl Into<String>,
  path: Option<String>,
) {
  let key = normalize_candidate_key(path.as_deref());
  if !seen.insert(key) {
    return;
  }
  list.push(ShellCandidate {
    label: label.into(),
    path,
  });
}

fn build_shell_candidates(
  primary_path: Option<&str>,
  secondary_default_path: Option<&str>,
) -> Vec<ShellCandidate> {
  let mut list = Vec::new();
  let mut seen = HashSet::new();
  let primary_label = if primary_path.is_some() {
    "primary-shell"
  } else {
    "system-default"
  };
  push_candidate(
    &mut list,
    &mut seen,
    primary_label,
    primary_path.map(|value| value.to_string()),
  );
  if let Some(path) = secondary_default_path {
    push_candidate(
      &mut list,
      &mut seen,
      "default-terminal",
      Some(path.to_string()),
    );
  }
  for option in list_terminal_environments() {
    push_candidate(
      &mut list,
      &mut seen,
      format!("fallback-{}", option.label),
      Some(option.path),
    );
  }
  // 补齐系统默认候选，确保无法解析路径时仍可兜底。
  push_candidate(&mut list, &mut seen, "system-default-fallback", None);
  list
}

fn spawn_shell_with_candidates(
  cols: u16,
  rows: u16,
  cwd: Option<String>,
  candidates: Vec<ShellCandidate>,
) -> Result<(SpawnedPty, bool), String> {
  let mut errors = Vec::new();
  for (index, candidate) in candidates.iter().enumerate() {
    let attempt = spawn_shell(cols, rows, cwd.clone(), candidate.path.as_deref());
    match attempt {
      Ok(spawned) => {
        let fallback_used = index > 0;
        if fallback_used {
          log::warn!(
            "terminal fallback used label={} path={}",
            candidate.label,
            candidate.path.as_deref().unwrap_or("<system-default>")
          );
        }
        return Ok((spawned, fallback_used));
      }
      Err(err) => {
        errors.push(format!(
          "{}({}): {}",
          candidate.label,
          candidate.path.as_deref().unwrap_or("<system-default>"),
          err
        ));
      }
    }
  }
  Err(format!("shell launch failed: {}", errors.join(" | ")))
}

fn spawn_shell_strict(
  cols: u16,
  rows: u16,
  cwd: Option<String>,
  terminal_path: Option<&str>,
) -> Result<SpawnedPty, TerminalError> {
  let trimmed = terminal_path.map(str::trim).filter(|value| !value.is_empty());
  if let Some(path) = trimmed {
    if let Err(err) = lookup_binary(path, Some(path)) {
      return Err(TerminalError::with_path(
        TerminalErrorCode::ShellBinaryNotFound,
        err,
        Some(path.to_string()),
      ));
    }
  }
  spawn_shell(cols, rows, cwd, trimmed).map_err(|err| {
    TerminalError::new(TerminalErrorCode::ShellLaunchFailed, err)
  })
}

fn split_command(command: &str) -> (String, Vec<String>) {
  let mut parts = command.split_whitespace();
  let program = parts.next().unwrap_or("").to_string();
  let args = parts.map(|part| part.to_string()).collect();
  (program, args)
}

fn spawn_direct_terminal_with_command(
  cols: u16,
  rows: u16,
  cwd: Option<String>,
  terminal_type: TerminalType,
  terminal_path: Option<&str>,
  terminal_command: Option<&str>,
) -> Result<SpawnedPty, String> {
  let default_binary = terminal_type
    .default_binary()
    .ok_or_else(|| "terminal binary missing".to_string())?;
  let command = terminal_command
    .map(str::trim)
    .filter(|value| !value.is_empty());
  let (program, args) = if let Some(command) = command {
    split_command(command)
  } else {
    (default_binary.to_string(), Vec::new())
  };
  let resolved = lookup_binary(&program, terminal_path)?;
  spawn_command(cols, rows, cwd, &resolved, &args)
}

/// 启动终端并在失败时执行兜底链路。
/// 规则：非 shell 先直连二进制（支持 `terminal_command` 参数）；失败后退到默认终端；仍失败则按候选列表尝试。
/// 当 `strict_shell` 为 true 时，shell 仅尝试指定路径/系统默认，不执行兜底。
pub(crate) fn launch_terminal_with_fallback(
  storage: &StorageManager,
  cols: u16,
  rows: u16,
  cwd: Option<String>,
  terminal_type: TerminalType,
  terminal_path: Option<&str>,
  terminal_command: Option<&str>,
  strict_shell: bool,
) -> Result<TerminalLaunchResult, TerminalError> {
  let default_terminal = load_default_terminal_preference(storage);
  if terminal_type == TerminalType::Shell {
    if strict_shell {
      let spawned = spawn_shell_strict(cols, rows, cwd, terminal_path)?;
      return Ok(TerminalLaunchResult {
        spawned,
        terminal_type: TerminalType::Shell,
        fallback_used: false,
        default_terminal,
      });
    }
    let default_terminal_path = default_terminal.path.as_deref();
    let primary_path = terminal_path.or(default_terminal_path);
    let secondary_default = if terminal_path.is_some() {
      default_terminal_path
    } else {
      None
    };
    let candidates = build_shell_candidates(primary_path, secondary_default);
    let (spawned, fallback_used) = spawn_shell_with_candidates(cols, rows, cwd, candidates).map_err(|err| {
      TerminalError::new(TerminalErrorCode::ShellLaunchFailed, err)
    })?;
    return Ok(TerminalLaunchResult {
      spawned,
      terminal_type: TerminalType::Shell,
      fallback_used,
      default_terminal,
    });
  }

  let direct_result = spawn_direct_terminal_with_command(
    cols,
    rows,
    cwd.clone(),
    terminal_type,
    terminal_path,
    terminal_command,
  )
    .map_err(|err| TerminalError::new(TerminalErrorCode::TerminalLaunchFailed, err));
  match direct_result {
    Ok(spawned) => Ok(TerminalLaunchResult {
      spawned,
      terminal_type,
      fallback_used: false,
      default_terminal,
    }),
    Err(primary_err) => {
      log::warn!(
        "terminal direct launch failed type={} err={}",
        terminal_type.as_str(),
        primary_err.message
      );
      let default_terminal_path = default_terminal.path.as_deref();
      let candidates = build_shell_candidates(default_terminal_path, None);
      let (spawned, _) = spawn_shell_with_candidates(cols, rows, cwd, candidates).map_err(|fallback_err| {
        TerminalError::new(
          TerminalErrorCode::TerminalLaunchFailed,
          format!("terminal launch failed: primary={}; fallback={fallback_err}", primary_err.message),
        )
      })?;
      Ok(TerminalLaunchResult {
        spawned,
        terminal_type: TerminalType::Shell,
        fallback_used: true,
        default_terminal,
      })
    }
  }
}
