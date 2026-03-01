//! 终端 shim：为 PTY 启动目标命令，并通过 OSC 信号向上游报告就绪与退出。
//! 边界：只负责转发 IO 与退出状态，不解析命令输出或业务语义。

use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

// 与上游终端层的协议：用于解除输入缓冲并标记命令完成。
const OSC_READY: &str = "\x1b]633;A\x07";
const OSC_EXIT_PREFIX: &str = "\x1b]633;D;";
// 约定前缀，便于上游在未就绪前识别启动失败原因。
const SHIM_LAUNCH_ERROR_MARKER: &str = "SHIM_LAUNCH_ERROR";

#[cfg(windows)]
fn force_utf8_console() {
  use windows_sys::Win32::System::Console::{SetConsoleCP, SetConsoleOutputCP};
  // Windows 默认代码页可能导致 UTF-8 输入输出被破坏，统一切换以避免乱码。
  unsafe {
    SetConsoleCP(65001);
    SetConsoleOutputCP(65001);
  }
}

#[cfg(not(windows))]
fn force_utf8_console() {}

fn main() {
  force_utf8_console();
  let mut args = env::args();
  let _shim = args.next();
  let target = match args.next() {
    Some(value) => value,
    None => {
      // 退出码用于上游区分缺参失败。
      eprintln!("{SHIM_LAUNCH_ERROR_MARKER}: no target command");
      std::process::exit(101);
    }
  };
  let target_args: Vec<String> = args.collect();

  // 先发就绪信号，避免上游等待 shell integration 超时。
  print!("{OSC_READY}");
  let _ = io::stdout().flush();

  let child = Command::new(&target)
    .args(&target_args)
    // 继承 IO 以保持交互式终端语义，避免缓冲导致输入延迟。
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn();

  match child {
    Ok(mut child) => {
      let status = match child.wait() {
        Ok(status) => status,
        Err(err) => {
          // 退出码用于上游识别 wait 失败（与 spawn 失败区分）。
          eprintln!("{SHIM_LAUNCH_ERROR_MARKER}: wait error='{}'", err);
          std::process::exit(103);
        }
      };
      let code = status.code().unwrap_or(0);
      // 通过 OSC 携带退出码，供上游更新状态/完成语义。
      print!("{OSC_EXIT_PREFIX}{code}\x07");
      let _ = io::stdout().flush();
      std::process::exit(code);
    }
    Err(err) => {
      // 退出码用于上游区分 spawn 失败（环境/路径问题）。
      eprintln!(
        "{SHIM_LAUNCH_ERROR_MARKER}: command='{}' error='{}'",
        target, err
      );
      std::process::exit(102);
    }
  }
}
