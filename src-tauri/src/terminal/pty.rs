use std::{
  env,
  io::{Read, Write},
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use portable_pty::{native_pty_system, Child, ChildKiller, CommandBuilder, MasterPty, PtySize};
#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::GetShortPathNameW;

pub(crate) struct TerminalHandle {
  pub(crate) master: Box<dyn MasterPty + Send>,
  pub(crate) writer: Arc<Mutex<Box<dyn Write + Send>>>,
  pub(crate) killer: Box<dyn ChildKiller + Send + Sync>,
}

pub(crate) struct SpawnedPty {
  pub(crate) child: Box<dyn Child + Send + Sync>,
  pub(crate) handle: TerminalHandle,
  pub(crate) reader: Box<dyn Read + Send>,
}

struct CommandSpec {
  program: String,
  args: Vec<String>,
  cwd: Option<String>,
}

#[cfg(windows)]
const CMD_COMPAT_PATH_LIMIT: usize = 260;

fn resolve_shell_candidate(candidate: &str) -> Option<String> {
  let trimmed = candidate.trim();
  if trimmed.is_empty() {
    return None;
  }
  lookup_binary(trimmed, None).ok()
}

fn default_shell_spec() -> (String, Vec<String>) {
  if cfg!(windows) {
    let candidates = ["powershell", "pwsh", "cmd"];
    for candidate in candidates {
      if let Some(path) = resolve_shell_candidate(candidate) {
        let args = if is_powershell_program(&path) {
          vec!["-NoLogo".to_string()]
        } else {
          Vec::new()
        };
        return (path, args);
      }
    }
    if let Ok(comspec) = env::var("COMSPEC") {
      if let Some(path) = resolve_shell_candidate(&comspec) {
        return (path, Vec::new());
      }
    }
    return ("cmd.exe".to_string(), Vec::new());
  }

  if let Ok(shell) = env::var("SHELL") {
    if let Some(path) = resolve_shell_candidate(&shell) {
      return (path, Vec::new());
    }
  }
  let candidates = ["zsh", "bash", "sh"];
  for candidate in candidates {
    if let Some(path) = resolve_shell_candidate(candidate) {
      return (path, Vec::new());
    }
  }
  ("sh".to_string(), Vec::new())
}

fn shell_name(program: &str) -> Option<String> {
  Path::new(program)
    .file_name()
    .and_then(|value| value.to_str())
    .map(|value| value.to_lowercase())
}

fn build_bash_prompt_command() -> String {
  "printf '\\033]633;D;%s\\a' \"$?\"".to_string()
}

fn build_shell_integration_env(program: &str, args: &mut Vec<String>) -> Vec<(String, String)> {
  let mut envs = Vec::new();
  let name = match shell_name(program) {
    Some(value) => value,
    None => return envs,
  };
  if cfg!(windows) && is_powershell_program(program) {
    if !args.iter().any(|arg| arg.eq_ignore_ascii_case("-Command")) {
      let script = r#"if (-not $global:__golutra_prompt) { $global:__golutra_prompt = $function:prompt; function global:prompt { $exitCode = if ($LASTEXITCODE -ne $null) { $LASTEXITCODE } elseif ($?) { 0 } else { 1 }; $promptText = if ($global:__golutra_prompt) { & $global:__golutra_prompt } else { "PS $PWD> " }; $esc = [char]27; $bel = [char]7; [Console]::Write("$esc]633;D;$exitCode$bel"); [Console]::Write("$esc]633;A$bel"); $global:LASTEXITCODE = $exitCode; return $promptText } }"#;
      if !args.iter().any(|arg| arg.eq_ignore_ascii_case("-NoExit")) {
        args.push("-NoExit".to_string());
      }
      args.push("-Command".to_string());
      args.push(script.to_string());
    }
    envs.push(("GOLUTRA_SHELL_INTEGRATION".to_string(), "1".to_string()));
    return envs;
  }
  if name == "bash" || name == "bash.exe" {
    let base = build_bash_prompt_command();
    let existing = std::env::var("PROMPT_COMMAND").ok();
    let combined = match existing {
      Some(value) if !value.trim().is_empty() && !value.contains("633;D") => {
        format!("{base};{value}")
      }
      Some(value) if value.contains("633;D") => value,
      _ => base,
    };
    envs.push(("PROMPT_COMMAND".to_string(), combined));
    envs.push(("GOLUTRA_SHELL_INTEGRATION".to_string(), "1".to_string()));
  }
  envs
}

fn shim_binary_name() -> &'static str {
  if cfg!(windows) {
    "shim.exe"
  } else {
    "shim"
  }
}

fn resolve_shim_path() -> Result<String, String> {
  if let Some(value) = env::var_os("GOLUTRA_SHIM_PATH") {
    let path = PathBuf::from(value);
    if path.is_file() {
      log::info!("terminal shim path resolved via env: {}", path.to_string_lossy());
      return Ok(path.to_string_lossy().to_string());
    }
    return Err(format!(
      "shim binary not found at path: {}",
      path.to_string_lossy()
    ));
  }
  let exe = env::current_exe().map_err(|err| format!("failed to resolve shim path: {err}"))?;
  let name = shim_binary_name();
  if let Some(parent) = exe.parent() {
    let candidate = parent.join(name);
    if candidate.is_file() {
      log::info!("terminal shim path resolved: {}", candidate.to_string_lossy());
      return Ok(candidate.to_string_lossy().to_string());
    }
    let candidate = parent.join("resources").join(name);
    if candidate.is_file() {
      log::info!("terminal shim path resolved: {}", candidate.to_string_lossy());
      return Ok(candidate.to_string_lossy().to_string());
    }
  }
  Err(format!("shim binary not found: {name}"))
}

fn spawn_with_command(
  mut cmd: CommandBuilder,
  cols: u16,
  rows: u16,
  cwd: Option<String>,
) -> Result<SpawnedPty, String> {
  let pty_system = native_pty_system();
  let pair = pty_system
    .openpty(PtySize {
      rows,
      cols,
      pixel_width: 0,
      pixel_height: 0,
    })
    .map_err(|err| format!("failed to open pty: {err}"))?;

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

  let cleanup_killer = child.clone_killer();
  let handle = TerminalHandle {
    master,
    writer: Arc::clone(&writer),
    killer: cleanup_killer.clone_killer(),
  };

  Ok(SpawnedPty {
    child,
    handle,
    reader,
  })
}

fn spawn_with_shim(
  cols: u16,
  rows: u16,
  cwd: Option<String>,
  program: &str,
  args: &[String],
  extra_env: Vec<(String, String)>,
) -> Result<SpawnedPty, String> {
  let shim = resolve_shim_path()?;
  log::info!(
    "terminal spawn shim={} program={} args_len={}",
    shim,
    program,
    args.len()
  );
  let mut cmd = CommandBuilder::new(shim);
  for (key, value) in extra_env {
    cmd.env(key, value);
  }
  cmd.arg(program);
  if !args.is_empty() {
    cmd.args(args);
  }
  spawn_with_command(cmd, cols, rows, cwd)
}

#[cfg(windows)]
fn strip_windows_extended_prefix(value: &str) -> Option<String> {
  if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
    return Some(format!(r"\\{}", rest));
  }
  value.strip_prefix(r"\\?\").map(|rest| rest.to_string())
}

#[cfg(windows)]
fn to_wide_null(value: &str) -> Vec<u16> {
  OsStr::new(value).encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn try_short_path(value: &str) -> Option<String> {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return None;
  }
  let candidate = strip_windows_extended_prefix(trimmed).unwrap_or_else(|| trimmed.to_string());
  let wide = to_wide_null(&candidate);
  let required = unsafe { GetShortPathNameW(wide.as_ptr(), std::ptr::null_mut(), 0) };
  if required == 0 {
    return None;
  }
  let mut buffer = vec![0u16; required as usize];
  let written = unsafe { GetShortPathNameW(wide.as_ptr(), buffer.as_mut_ptr(), required) };
  if written == 0 {
    return None;
  }
  buffer.truncate(written as usize);
  String::from_utf16(&buffer).ok()
}

#[cfg(windows)]
fn normalize_windows_cmd_path(value: &str) -> String {
  let trimmed = value.trim();
  if let Some(short) = try_short_path(trimmed) {
    return short;
  }
  strip_windows_extended_prefix(trimmed).unwrap_or_else(|| trimmed.to_string())
}

#[cfg(not(windows))]
fn normalize_windows_cmd_path(value: &str) -> String {
  value.trim().to_string()
}

#[cfg(windows)]
fn normalize_windows_cmd_cwd(cwd: Option<&str>) -> Option<String> {
  let trimmed = cwd.map(str::trim).filter(|value| !value.is_empty())?;
  Some(normalize_windows_cmd_path(trimmed))
}

#[cfg(not(windows))]
fn normalize_windows_cmd_cwd(cwd: Option<&str>) -> Option<String> {
  cwd.map(|value| value.to_string())
}

#[cfg(windows)]
fn normalize_windows_powershell_path(value: &str) -> String {
  let trimmed = value.trim();
  if let Some(short) = try_short_path(trimmed) {
    return short;
  }
  if let Some(stripped) = strip_windows_extended_prefix(trimmed) {
    if stripped.len() <= CMD_COMPAT_PATH_LIMIT {
      return stripped;
    }
  }
  trimmed.to_string()
}

#[cfg(not(windows))]
fn normalize_windows_powershell_path(value: &str) -> String {
  value.trim().to_string()
}

#[cfg(windows)]
fn normalize_windows_powershell_cwd(cwd: Option<&str>) -> Option<String> {
  let trimmed = cwd.map(str::trim).filter(|value| !value.is_empty())?;
  Some(normalize_windows_powershell_path(trimmed))
}

#[cfg(not(windows))]
fn normalize_windows_powershell_cwd(cwd: Option<&str>) -> Option<String> {
  cwd.map(|value| value.to_string())
}

#[cfg(windows)]
fn is_cmd_program(program: &str) -> bool {
  Path::new(program)
    .file_name()
    .and_then(|value| value.to_str())
    .map(|value| value.eq_ignore_ascii_case("cmd.exe"))
    .unwrap_or(false)
}

#[cfg(windows)]
fn is_powershell_program(program: &str) -> bool {
  Path::new(program)
    .file_name()
    .and_then(|value| value.to_str())
    .map(|value| value.eq_ignore_ascii_case("powershell.exe") || value.eq_ignore_ascii_case("pwsh.exe"))
    .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_powershell_program(_program: &str) -> bool {
  false
}

#[cfg(windows)]
fn build_command_spec(program: &str, args: &[String], cwd: Option<String>) -> CommandSpec {
  if let Some(ext) = Path::new(program).extension().and_then(|value| value.to_str()) {
    let ext = ext.to_lowercase();
    if ext == "cmd" || ext == "bat" {
      let program = normalize_windows_cmd_path(program);
      let mut cmd_args = vec!["/c".to_string(), program];
      if !args.is_empty() {
        cmd_args.extend_from_slice(args);
      }
      return CommandSpec {
        program: "cmd.exe".to_string(),
        args: cmd_args,
        cwd: normalize_windows_cmd_cwd(cwd.as_deref()),
      };
    }
    if ext == "ps1" {
      let program = normalize_windows_powershell_path(program);
      let mut cmd_args = vec![
        "-NoLogo".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-File".to_string(),
        program,
      ];
      if !args.is_empty() {
        cmd_args.extend_from_slice(args);
      }
      return CommandSpec {
        program: "powershell.exe".to_string(),
        args: cmd_args,
        cwd: normalize_windows_powershell_cwd(cwd.as_deref()),
      };
    }
  }
  let cwd = if is_cmd_program(program) {
    normalize_windows_cmd_cwd(cwd.as_deref())
  } else if is_powershell_program(program) {
    normalize_windows_powershell_cwd(cwd.as_deref())
  } else {
    cwd
  };
  CommandSpec {
    program: program.to_string(),
    args: args.to_vec(),
    cwd,
  }
}

#[cfg(not(windows))]
fn build_command_spec(program: &str, args: &[String], cwd: Option<String>) -> CommandSpec {
  CommandSpec {
    program: program.to_string(),
    args: args.to_vec(),
    cwd,
  }
}

pub(crate) fn spawn_shell(cols: u16, rows: u16, cwd: Option<String>) -> Result<SpawnedPty, String> {
  let (shell, args) = default_shell_spec();
  let mut spec = build_command_spec(&shell, &args, cwd);
  let extra_env = build_shell_integration_env(&spec.program, &mut spec.args);
  spawn_with_shim(cols, rows, spec.cwd, &spec.program, &spec.args, extra_env)
}

pub(crate) fn spawn_command(
  cols: u16,
  rows: u16,
  cwd: Option<String>,
  program: &str,
  args: &[String],
) -> Result<SpawnedPty, String> {
  let spec = build_command_spec(program, args, cwd);
  spawn_with_shim(cols, rows, spec.cwd, &spec.program, &spec.args, Vec::new())
}

pub(crate) fn resize_pty(handle: &TerminalHandle, rows: u16, cols: u16) -> Result<(), String> {
  let size = PtySize {
    rows: rows.max(1),
    cols: cols.max(1),
    pixel_width: 0,
    pixel_height: 0,
  };
  handle
    .master
    .resize(size)
    .map_err(|err| format!("failed to resize pty: {err}"))
}

fn is_path_like(value: &str) -> bool {
  value.contains('/') || value.contains('\\')
}

fn candidate_names(name: &str) -> Vec<String> {
  if !cfg!(windows) {
    return vec![name.to_string()];
  }
  let path = Path::new(name);
  if path.extension().is_some() {
    return vec![name.to_string()];
  }
  vec![
    format!("{name}.exe"),
    format!("{name}.cmd"),
    format!("{name}.bat"),
    format!("{name}.ps1"),
    name.to_string(),
  ]
}

fn resolve_explicit_path(value: &str) -> Option<PathBuf> {
  let path = PathBuf::from(value);
  if path.is_file() {
    return Some(path);
  }
  if !cfg!(windows) {
    return None;
  }
  if path.extension().is_some() {
    return None;
  }
  let parent = path.parent()?;
  let file_name = path.file_name()?.to_str()?;
  for candidate in candidate_names(file_name) {
    let candidate_path = parent.join(candidate);
    if candidate_path.is_file() {
      return Some(candidate_path);
    }
  }
  None
}

#[cfg(not(windows))]
fn find_in_dir(dir: &Path, name: &str) -> Option<PathBuf> {
  for candidate in candidate_names(name) {
    let candidate_path = dir.join(&candidate);
    if candidate_path.is_file() {
      return Some(candidate_path);
    }
  }
  None
}

#[cfg(windows)]
fn find_preferred_in_dirs(dirs: &[PathBuf], name: &str) -> Option<PathBuf> {
  let path = Path::new(name);
  if path.extension().is_some() {
    for dir in dirs {
      let candidate = dir.join(name);
      if candidate.is_file() {
        return Some(candidate);
      }
    }
    return None;
  }
  const EXT_PRIORITY: [&str; 5] = ["exe", "cmd", "bat", "ps1", ""];
  for ext in EXT_PRIORITY {
    for dir in dirs {
      let candidate = if ext.is_empty() {
        dir.join(name)
      } else {
        dir.join(format!("{name}.{ext}"))
      };
      if candidate.is_file() {
        return Some(candidate);
      }
    }
  }
  None
}

#[cfg(not(windows))]
fn find_preferred_in_dirs(dirs: &[PathBuf], name: &str) -> Option<PathBuf> {
  for dir in dirs {
    if let Some(found) = find_in_dir(dir, name) {
      return Some(found);
    }
  }
  None
}

fn push_dir(dirs: &mut Vec<PathBuf>, dir: Option<PathBuf>) {
  if let Some(path) = dir {
    if path.is_dir() && !dirs.iter().any(|existing| existing == &path) {
      dirs.push(path);
    }
  }
}

fn common_binary_dirs() -> Vec<PathBuf> {
  let mut dirs = Vec::new();
  if cfg!(windows) {
    push_dir(
      &mut dirs,
      env::var_os("LOCALAPPDATA").map(|value| PathBuf::from(value).join("Programs")),
    );
    push_dir(
      &mut dirs,
      env::var_os("LOCALAPPDATA").map(|value| PathBuf::from(value).join("Microsoft\\WindowsApps")),
    );
    push_dir(
      &mut dirs,
      env::var_os("APPDATA").map(|value| PathBuf::from(value).join("npm")),
    );
    push_dir(
      &mut dirs,
      env::var_os("LOCALAPPDATA").map(|value| PathBuf::from(value).join("npm")),
    );
    push_dir(&mut dirs, env::var_os("ProgramFiles").map(PathBuf::from));
    push_dir(&mut dirs, env::var_os("ProgramFiles(x86)").map(PathBuf::from));
    push_dir(
      &mut dirs,
      env::var_os("USERPROFILE").map(|value| PathBuf::from(value).join("scoop\\shims")),
    );
    push_dir(&mut dirs, env::var_os("SCOOP").map(|value| PathBuf::from(value).join("shims")));
  } else {
    push_dir(&mut dirs, Some(PathBuf::from("/usr/local/bin")));
    push_dir(&mut dirs, Some(PathBuf::from("/usr/bin")));
    push_dir(&mut dirs, Some(PathBuf::from("/opt/homebrew/bin")));
    push_dir(&mut dirs, Some(PathBuf::from("/opt/bin")));
    push_dir(
      &mut dirs,
      env::var_os("HOME").map(|value| PathBuf::from(value).join(".local/bin")),
    );
    push_dir(
      &mut dirs,
      env::var_os("HOME").map(|value| PathBuf::from(value).join(".cargo/bin")),
    );
    push_dir(
      &mut dirs,
      env::var_os("HOME").map(|value| PathBuf::from(value).join(".bun/bin")),
    );
  }
  dirs
}

pub(crate) fn lookup_binary(name: &str, configured_path: Option<&str>) -> Result<String, String> {
  let configured = configured_path
    .map(str::trim)
    .filter(|value| !value.is_empty());
  if let Some(path) = configured {
    if let Some(resolved) = resolve_explicit_path(path) {
      return Ok(resolved.to_string_lossy().to_string());
    }
    return Err(format!("terminal binary not found at path: {path}"));
  }

  let trimmed = name.trim();
  if trimmed.is_empty() {
    return Err("terminal binary name is empty".to_string());
  }
  if is_path_like(trimmed) {
    if let Some(resolved) = resolve_explicit_path(trimmed) {
      return Ok(resolved.to_string_lossy().to_string());
    }
    return Err(format!("terminal binary not found at path: {trimmed}"));
  }

  let mut search_dirs = Vec::new();
  if let Some(path_list) = env::var_os("PATH") {
    for dir in env::split_paths(&path_list) {
      push_dir(&mut search_dirs, Some(dir));
    }
  }
  for dir in common_binary_dirs() {
    push_dir(&mut search_dirs, Some(dir));
  }
  if let Some(found) = find_preferred_in_dirs(&search_dirs, trimmed) {
    return Ok(found.to_string_lossy().to_string());
  }

  Err(format!("terminal binary not found: {trimmed}"))
}
