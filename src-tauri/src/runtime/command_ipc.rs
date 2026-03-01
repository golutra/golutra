//! 本地命令 IPC：提供外部终端命令接入。

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::Arc;
use std::thread;

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use ulid::Ulid;

use crate::application::command::{execute_terminal_command, parse_terminal_command};
use crate::message_service::chat_db::ChatDbManager;
use crate::runtime::command_center::{CommandCenter, CommandResultPayload};

#[cfg(windows)]
pub const COMMAND_IPC_NAME: &str = r"\\.\pipe\golutra-command";

#[cfg(not(windows))]
pub const COMMAND_IPC_NAME: &str = "/tmp/golutra-command.sock";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommandIpcRequest {
  pub(crate) mode: String,
  pub(crate) request_id: Option<String>,
  pub(crate) args: Vec<String>,
  pub(crate) async_exec: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommandIpcResponse {
  pub(crate) ok: bool,
  pub(crate) request_id: Option<String>,
  pub(crate) result: Option<CommandResultPayload>,
  pub(crate) error: Option<String>,
}

pub(crate) fn spawn_command_ipc_server(
  app: AppHandle,
  command_center: Arc<CommandCenter>,
) -> Result<(), String> {
  #[cfg(not(windows))]
  {
    let _ = std::fs::remove_file(COMMAND_IPC_NAME);
  }
  let listener = LocalSocketListener::bind(COMMAND_IPC_NAME)
    .map_err(|err| format!("command ipc bind failed: {err}"))?;
  thread::spawn(move || {
    for incoming in listener.incoming() {
      let Ok(stream) = incoming else {
        continue;
      };
      let app = app.clone();
      let command_center = Arc::clone(&command_center);
      thread::spawn(move || {
        if let Err(err) = handle_connection(app, command_center, stream) {
          log::warn!("command ipc connection failed: {err}");
        }
      });
    }
  });
  Ok(())
}

fn handle_connection(
  app: AppHandle,
  command_center: Arc<CommandCenter>,
  stream: LocalSocketStream,
) -> Result<(), String> {
  let mut stream = stream;
  let request: CommandIpcRequest = {
    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    reader
      .read_line(&mut line)
      .map_err(|err| format!("command ipc read failed: {err}"))?;
    serde_json::from_str(&line.trim_end())
      .map_err(|err| format!("command ipc decode failed: {err}"))?
  };
  let response = match request.mode.as_str() {
    "run" => handle_run_request(app, command_center, request),
    "wait" => handle_wait_request(command_center, request),
    _ => CommandIpcResponse {
      ok: false,
      request_id: None,
      result: None,
      error: Some("unsupported command mode".to_string()),
    },
  };
  let mut writer = BufWriter::new(&mut stream);
  let payload = serde_json::to_string(&response)
    .map_err(|err| format!("command ipc encode failed: {err}"))?;
  writer
    .write_all(payload.as_bytes())
    .and_then(|_| writer.write_all(b"\n"))
    .map_err(|err| format!("command ipc write failed: {err}"))?;
  writer.flush().map_err(|err| format!("command ipc flush failed: {err}"))?;
  Ok(())
}

fn handle_run_request(
  app: AppHandle,
  command_center: Arc<CommandCenter>,
  request: CommandIpcRequest,
) -> CommandIpcResponse {
  let args = request.args;
  let async_exec = request.async_exec.unwrap_or(false);
  let request_id = request
    .request_id
    .filter(|value| !value.trim().is_empty())
    .unwrap_or_else(|| Ulid::new().to_string());
  if async_exec {
    command_center.create_slot(&request_id);
    let command_center = Arc::clone(&command_center);
    let app = app.clone();
    let request_id_for_task = request_id.clone();
    thread::spawn(move || {
      let result = execute_command_request(app, args);
      let _ = command_center.complete(&request_id_for_task, result);
    });
    return CommandIpcResponse {
      ok: true,
      request_id: Some(request_id),
      result: None,
      error: None,
    };
  }
  let result = execute_command_request(app, args);
  CommandIpcResponse {
    ok: result.status == "ok",
    request_id: Some(request_id),
    result: Some(result),
    error: None,
  }
}

fn handle_wait_request(
  command_center: Arc<CommandCenter>,
  request: CommandIpcRequest,
) -> CommandIpcResponse {
  let request_id = match request.request_id {
    Some(value) if !value.trim().is_empty() => value,
    _ => {
      return CommandIpcResponse {
        ok: false,
        request_id: None,
        result: None,
        error: Some("request_id is required".to_string()),
      }
    }
  };
  match command_center.wait_result(&request_id) {
    Ok(result) => CommandIpcResponse {
      ok: result.status == "ok",
      request_id: Some(request_id),
      result: Some(result),
      error: None,
    },
    Err(err) => CommandIpcResponse {
      ok: false,
      request_id: Some(request_id),
      result: None,
      error: Some(err),
    },
  }
}

fn execute_command_request(app: AppHandle, args: Vec<String>) -> CommandResultPayload {
  let command = match parse_terminal_command(&args) {
    Ok(command) => command,
    Err(err) => {
      return CommandResultPayload {
        status: "error".to_string(),
        message: Some(err),
        data: None,
      };
    }
  };
  let chat_state = app.state::<ChatDbManager>();
  match execute_terminal_command(app.clone(), chat_state, command) {
    Ok(result) => result,
    Err(err) => CommandResultPayload {
      status: "error".to_string(),
      message: Some(err),
      data: None,
    },
  }
}
