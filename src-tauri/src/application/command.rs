//! 终端命令解析与执行：供外部命令与 UI 复用。

use std::collections::VecDeque;

use serde_json::json;
use tauri::{AppHandle, State};

use crate::application::chat as chat_app;
use crate::message_service::chat_db::{ChatDbManager, MessageContent};
use crate::runtime::command_center::CommandResultPayload;

const WORKSPACE_ENV_KEY: &str = "GOLUTRA_WORKSPACE_ID";

pub(crate) enum TerminalCommand {
  SendMessage {
    workspace_id: String,
    conversation_id: String,
    sender_id: String,
    text: String,
    is_ai: bool,
  },
  SendDirect {
    workspace_id: String,
    sender_id: String,
    target_id: String,
    text: String,
    is_ai: bool,
  },
}

pub(crate) fn parse_terminal_command(args: &[String]) -> Result<TerminalCommand, String> {
  let mut tokens: VecDeque<String> = args.iter().cloned().collect();
  if tokens.is_empty() {
    return Err("command is empty".to_string());
  }
  let mut workspace_id = None;
  let mut is_ai = false;
  let mut stripped = VecDeque::new();
  while let Some(token) = tokens.pop_front() {
    match token.as_str() {
      "--workspace" => {
        let value = tokens
          .pop_front()
          .ok_or_else(|| "missing value for --workspace".to_string())?;
        workspace_id = Some(value);
      }
      "--ai" => {
        is_ai = true;
      }
      _ => stripped.push_back(token),
    }
  }
  let workspace_id = workspace_id
    .or_else(|| std::env::var(WORKSPACE_ENV_KEY).ok())
    .ok_or_else(|| "workspace_id is required".to_string())?;

  if let Some(index) = stripped.iter().position(|token| token == "->") {
    return parse_arrow_command(&workspace_id, is_ai, stripped, index);
  }
  parse_send_command(&workspace_id, is_ai, stripped)
}

pub(crate) fn execute_terminal_command(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  command: TerminalCommand,
) -> Result<CommandResultPayload, String> {
  match command {
    TerminalCommand::SendMessage {
      workspace_id,
      conversation_id,
      sender_id,
      text,
      is_ai,
    } => {
      let message = chat_app::chat_send_message(
        app,
        state,
        workspace_id.clone(),
        conversation_id.clone(),
        Some(sender_id.clone()),
        None,
        MessageContent::Text { text },
        Some(is_ai),
        None,
      )?;
      Ok(CommandResultPayload {
        status: "ok".to_string(),
        message: Some("message sent".to_string()),
        data: Some(json!({
          "workspaceId": workspace_id,
          "conversationId": conversation_id,
          "messageId": message.id,
          "senderId": sender_id
        })),
      })
    }
    TerminalCommand::SendDirect {
      workspace_id,
      sender_id,
      target_id,
      text,
      is_ai,
    } => {
      let conversation = chat_app::chat_ensure_direct(
        app.clone(),
        state.clone(),
        workspace_id.clone(),
        sender_id.clone(),
        target_id.clone(),
      )?;
      let message = chat_app::chat_send_message(
        app,
        state,
        workspace_id.clone(),
        conversation.id.clone(),
        Some(sender_id.clone()),
        None,
        MessageContent::Text { text },
        Some(is_ai),
        None,
      )?;
      Ok(CommandResultPayload {
        status: "ok".to_string(),
        message: Some("message sent".to_string()),
        data: Some(json!({
          "workspaceId": workspace_id,
          "conversationId": conversation.id,
          "messageId": message.id,
          "senderId": sender_id,
          "targetId": target_id
        })),
      })
    }
  }
}

fn parse_send_command(
  workspace_id: &str,
  is_ai: bool,
  mut tokens: VecDeque<String>,
) -> Result<TerminalCommand, String> {
  if tokens.front().map(|value| value.as_str()) == Some("send") {
    tokens.pop_front();
  }
  let mut conversation_id = None;
  let mut sender_id = None;
  let mut target_id = None;
  let mut text_tokens = Vec::new();
  let mut iter = tokens.into_iter().peekable();
  while let Some(token) = iter.next() {
    match token.as_str() {
      "--conversation" | "--channel" => {
        let value = iter
          .next()
          .ok_or_else(|| "missing value for --conversation".to_string())?;
        conversation_id = Some(value);
      }
      "--sender" | "--from" => {
        let value = iter
          .next()
          .ok_or_else(|| "missing value for --sender".to_string())?;
        sender_id = Some(value);
      }
      "--to" | "--target" => {
        let value = iter
          .next()
          .ok_or_else(|| "missing value for --to".to_string())?;
        target_id = Some(value);
      }
      "--text" => {
        text_tokens.extend(iter.map(|value| value));
        break;
      }
      _ => text_tokens.push(token),
    }
  }
  let text = text_tokens.join(" ").trim().to_string();
  if text.is_empty() {
    return Err("message text is required".to_string());
  }
  let sender_id = sender_id.ok_or_else(|| "sender id is required".to_string())?;
  if let Some(conversation_id) = conversation_id {
    return Ok(TerminalCommand::SendMessage {
      workspace_id: workspace_id.to_string(),
      conversation_id,
      sender_id,
      text,
      is_ai,
    });
  }
  let target_id = target_id.ok_or_else(|| "target id is required".to_string())?;
  Ok(TerminalCommand::SendDirect {
    workspace_id: workspace_id.to_string(),
    sender_id,
    target_id,
    text,
    is_ai,
  })
}

fn parse_arrow_command(
  workspace_id: &str,
  is_ai: bool,
  tokens: VecDeque<String>,
  arrow_index: usize,
) -> Result<TerminalCommand, String> {
  let list: Vec<String> = tokens.into_iter().collect();
  if arrow_index == 0 || arrow_index + 1 >= list.len() {
    return Err("arrow command requires sender and target".to_string());
  }
  let sender_id = list[arrow_index - 1].to_string();
  let target_id = list[arrow_index + 1].to_string();
  let mut channel_index = None;
  for (index, token) in list.iter().enumerate().skip(arrow_index + 2) {
    if token.starts_with('#') {
      channel_index = Some(index);
      break;
    }
  }
  let (conversation_id, message_start) = match channel_index {
    Some(index) => (Some(list[index].trim_start_matches('#').to_string()), index + 1),
    None => (None, arrow_index + 2),
  };
  let text = list
    .iter()
    .skip(message_start)
    .cloned()
    .collect::<Vec<String>>()
    .join(" ")
    .trim()
    .to_string();
  if text.is_empty() {
    return Err("message text is required".to_string());
  }
  if let Some(conversation_id) = conversation_id {
    return Ok(TerminalCommand::SendMessage {
      workspace_id: workspace_id.to_string(),
      conversation_id,
      sender_id,
      text,
      is_ai,
    });
  }
  Ok(TerminalCommand::SendDirect {
    workspace_id: workspace_id.to_string(),
    sender_id,
    target_id,
    text,
    is_ai,
  })
}
