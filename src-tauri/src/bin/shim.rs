use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};

const OSC_READY: &str = "\x1b]633;A\x07";
const OSC_EXIT_PREFIX: &str = "\x1b]633;D;";
const SHIM_LAUNCH_ERROR_MARKER: &str = "SHIM_LAUNCH_ERROR";

#[cfg(windows)]
fn force_utf8_console() {
  use windows_sys::Win32::System::Console::{SetConsoleCP, SetConsoleOutputCP};
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
      eprintln!("{SHIM_LAUNCH_ERROR_MARKER}: no target command");
      std::process::exit(101);
    }
  };
  let target_args: Vec<String> = args.collect();

  print!("{OSC_READY}");
  let _ = io::stdout().flush();

  let child = Command::new(&target)
    .args(&target_args)
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn();

  match child {
    Ok(mut child) => {
      let status = match child.wait() {
        Ok(status) => status,
        Err(err) => {
          eprintln!("{SHIM_LAUNCH_ERROR_MARKER}: wait error='{}'", err);
          std::process::exit(103);
        }
      };
      let code = status.code().unwrap_or(0);
      print!("{OSC_EXIT_PREFIX}{code}\x07");
      let _ = io::stdout().flush();
      std::process::exit(code);
    }
    Err(err) => {
      eprintln!(
        "{SHIM_LAUNCH_ERROR_MARKER}: command='{}' error='{}'",
        target, err
      );
      std::process::exit(102);
    }
  }
}
