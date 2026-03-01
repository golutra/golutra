//! PTY 与命令启动：负责选择 shell/命令、处理 Windows 路径兼容、加载 shim 与集成脚本。
//! 边界：只负责进程与 IO 组织，不管理会话状态与输出解析。

use std::{
  collections::HashSet,
  env,
  io::{Read, Write},
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use portable_pty::{native_pty_system, Child, ChildKiller, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::GetShortPathNameW;

/// PTY 句柄集合：共享 writer 需加锁以满足多线程写入。
pub(crate) struct TerminalHandle {
  pub(crate) master: Box<dyn MasterPty + Send>,
  pub(crate) writer: Arc<Mutex<Box<dyn Write + Send>>>,
  pub(crate) killer: Box<dyn ChildKiller + Send + Sync>,
}

/// 启动后的 PTY 产物，包含子进程与读写端。
pub(crate) struct SpawnedPty {
  pub(crate) child: Box<dyn Child + Send + Sync>,
  pub(crate) handle: TerminalHandle,
  pub(crate) reader: Box<dyn Read + Send>,
}

// 启动参数的归一化结果，避免平台差异导致的行为不一致。
struct CommandSpec {
  program: String,
  args: Vec<String>,
  cwd: Option<String>,
}

#[cfg(windows)]
// 兼容旧版 CMD/Win32 MAX_PATH 限制，超长路径需要降级到短路径。
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
  ("sh".to_string(), Vec::new())
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TerminalEnvironmentOption {
  pub(crate) id: String,
  pub(crate) label: String,
  pub(crate) path: String,
}

fn normalize_terminal_env_key(path: &str) -> String {
  if cfg!(windows) {
    return path.to_lowercase();
  }
  path.to_string()
}

/// 枚举当前系统可用的终端环境，供设置页选择默认 shell。
pub(crate) fn list_terminal_environments() -> Vec<TerminalEnvironmentOption> {
  let candidates: Vec<(&str, &str)> = if cfg!(windows) {
    vec![
      ("powershell", "Windows PowerShell"),
      ("pwsh", "PowerShell"),
      ("cmd", "Command Prompt"),
    ]
  } else {
    vec![("zsh", "Zsh"), ("bash", "Bash"), ("sh", "Sh")]
  };
  let mut seen = HashSet::new();
  let mut options = Vec::new();
  for (id, label) in candidates {
    if let Ok(path) = lookup_binary(id, None) {
      let key = normalize_terminal_env_key(&path);
      if seen.insert(key) {
        options.push(TerminalEnvironmentOption {
          id: id.to_string(),
          label: label.to_string(),
          path,
        });
      }
    }
  }
  options
}

fn shim_binary_name() -> &'static str {
  if cfg!(windows) {
    "shim.exe"
  } else {
    "shim"
  }
}

fn resolve_shim_path() -> Result<String, String> {
  // 允许通过环境变量替换 shim，便于调试或热修复。
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
    // 固定 TERM，避免宿主环境缺失导致终端能力识别错误。
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
) -> Result<SpawnedPty, String> {
  let shim = resolve_shim_path()?;
  // 通过 shim 统一发送 OSC 就绪与退出信号，避免平台差异。
  log::info!(
    "terminal spawn shim={} program={} args_len={}",
    shim,
    program,
    args.len()
  );
  let mut cmd = CommandBuilder::new(shim);
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
  // CMD 对长路径/UNC 前缀兼容性差，优先转换为短路径。
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
  // PowerShell 对长路径相对友好，但仍需处理扩展前缀以提升兼容性。
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
      // .cmd/.bat 需要通过 cmd.exe /c 执行，且路径需兼容性归一化。
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
      // .ps1 通过 powershell.exe 执行，避免执行策略阻止启动。
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

/// 启动默认 shell 的 PTY 会话。
/// 输入：`cols/rows` 为终端尺寸，`cwd` 为可选工作目录；`shell_path` 可选用于覆盖默认 shell。
/// 返回：包含子进程与读写端的 `SpawnedPty`。
/// 错误：PTy 创建失败、shell 不可用或 shim 不可用。
pub(crate) fn spawn_shell(
  cols: u16,
  rows: u16,
  cwd: Option<String>,
  shell_path: Option<&str>,
) -> Result<SpawnedPty, String> {
  // shell_path 指定时优先解析该可执行，避免默认探测覆盖用户选择。
  let (shell, args) = if let Some(value) = shell_path.map(str::trim).filter(|value| !value.is_empty()) {
    let resolved = lookup_binary(value, None)?;
    let args = if is_powershell_program(&resolved) {
      vec!["-NoLogo".to_string()]
    } else {
      Vec::new()
    };
    (resolved, args)
  } else {
    // 选择系统默认 shell，shim 负责就绪与退出信号。
    default_shell_spec()
  };
  let spec = build_command_spec(&shell, &args, cwd);
  spawn_with_shim(cols, rows, spec.cwd, &spec.program, &spec.args)
}

/// 启动指定命令的 PTY 会话。
/// 输入：`program/args` 为目标命令，`cwd` 为可选工作目录。
/// 返回：包含子进程与读写端的 `SpawnedPty`。
/// 错误：PTy 创建失败、命令不可用或 shim 不可用。
pub(crate) fn spawn_command(
  cols: u16,
  rows: u16,
  cwd: Option<String>,
  program: &str,
  args: &[String],
) -> Result<SpawnedPty, String> {
  let spec = build_command_spec(program, args, cwd);
  spawn_with_shim(cols, rows, spec.cwd, &spec.program, &spec.args)
}

/// 调整 PTY 尺寸。
/// 约束：尺寸最小为 1 行 1 列，避免底层报错。
/// 错误：PTY resize 失败或句柄不可用。
pub(crate) fn resize_pty(handle: &TerminalHandle, rows: u16, cols: u16) -> Result<(), String> {
  // 保证最小尺寸为 1，避免 PTY 在 0 行列下报错。
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
  // 按 Windows 常见可执行扩展优先级查找，避免误选脚本类型。
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

/// 解析可执行文件路径。
/// 输入：`name` 为命令名，`configured_path` 为可选显式路径。
/// 返回：可执行文件绝对路径。
/// 错误：路径不存在或在搜索路径中找不到。
pub(crate) fn lookup_binary(name: &str, configured_path: Option<&str>) -> Result<String, String> {
  // 查找顺序：显式路径 > PATH > 常见安装目录，确保可预测且可调试。
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
