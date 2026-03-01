use std::env;
use std::io::{BufRead, BufReader, BufWriter, Write};

use interprocess::local_socket::LocalSocketStream;
use serde::{Deserialize, Serialize};

use app_lib::COMMAND_IPC_NAME;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandIpcRequest {
  mode: String,
  request_id: Option<String>,
  args: Vec<String>,
  async_exec: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandIpcResponse {
  ok: bool,
  request_id: Option<String>,
  result: Option<CommandResultPayload>,
  error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandResultPayload {
  status: String,
  message: Option<String>,
  data: Option<serde_json::Value>,
}

fn main() {
  let mut args: Vec<String> = env::args().skip(1).collect();
  if args.is_empty() {
    print_help();
    std::process::exit(1);
  }
  let mode = match args.first().map(|value| value.as_str()) {
    Some("wait") => "wait",
    Some("send") => "run",
    Some("help") | Some("--help") | Some("-h") => {
      print_help();
      return;
    }
    _ => "run",
  };
  let request = if mode == "wait" {
    args.remove(0);
    let request_id = args.pop().filter(|value| !value.trim().is_empty());
    if request_id.is_none() {
      eprintln!("request_id is required");
      std::process::exit(1);
    }
    CommandIpcRequest {
      mode: "wait".to_string(),
      request_id,
      args: Vec::new(),
      async_exec: None,
    }
  } else {
    let async_exec = if let Some(index) = args.iter().position(|value| value == "--async") {
      args.remove(index);
      true
    } else {
      false
    };
    CommandIpcRequest {
      mode: "run".to_string(),
      request_id: None,
      args,
      async_exec: Some(async_exec),
    }
  };

  let mut stream = LocalSocketStream::connect(COMMAND_IPC_NAME).unwrap_or_else(|err| {
    eprintln!("failed to connect golutra ipc: {err}");
    std::process::exit(1);
  });
  {
    let mut writer = BufWriter::new(&mut stream);
    let payload = serde_json::to_string(&request).unwrap_or_else(|err| {
      eprintln!("failed to encode request: {err}");
      std::process::exit(1);
    });
    writer
      .write_all(payload.as_bytes())
      .and_then(|_| writer.write_all(b"\n"))
      .and_then(|_| writer.flush())
      .unwrap_or_else(|err| {
        eprintln!("failed to send request: {err}");
        std::process::exit(1);
      });
  }
  let response_line = {
    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap_or_else(|err| {
      eprintln!("failed to read response: {err}");
      std::process::exit(1);
    });
    line
  };
  let response: CommandIpcResponse = serde_json::from_str(response_line.trim_end()).unwrap_or_else(|err| {
    eprintln!("failed to decode response: {err}");
    std::process::exit(1);
  });
  if !response.ok {
    if let Some(error) = response.error {
      eprintln!("{error}");
    } else {
      eprintln!("command failed");
    }
    std::process::exit(1);
  }
  if mode == "run" && request.async_exec == Some(true) {
    if let Some(request_id) = response.request_id {
      println!("{request_id}");
    }
    return;
  }
  if let Some(result) = response.result {
    let payload = serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.status.clone());
    println!("{payload}");
  } else if let Some(request_id) = response.request_id {
    println!("{request_id}");
  }
}

fn print_help() {
  println!(
    "golutra command usage:\n  golutra send [--async] [--workspace <id>] <command>\n  golutra wait <request_id>\n\nExamples:\n  golutra send --workspace <id> send --sender <id> --conversation <id> --text \"hello\"\n  golutra send --workspace <id> a -> b #conversation-id hello\n  golutra wait <request_id>"
  );
}
