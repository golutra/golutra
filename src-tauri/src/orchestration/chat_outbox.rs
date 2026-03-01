//! 聊天派发 Outbox Worker：异步派发与重试控制。

use std::thread;
use std::time::Duration;

use serde_json::json;
use tauri::{AppHandle, Manager, WebviewWindow};

use crate::contracts::chat_dispatch::ChatDispatchPayload;
use crate::message_service::chat_db::{
  chat_outbox_claim_due, chat_outbox_mark_failed, chat_outbox_mark_sent,
  chat_update_message_status, list_workspace_ids, ChatDbManager, MessageStatus,
};
use crate::platform::{diagnostics_log_backend_event, DiagnosticsState};
use crate::runtime::StorageManager;
use crate::terminal_engine::TerminalManager;
use crate::ui_gateway::MAIN_WINDOW_LABEL;
use crate::{now_millis};

use super::dispatch::orchestrate_chat_dispatch;

const OUTBOX_POLL_INTERVAL_MS: u64 = 280;
const OUTBOX_CLAIM_LIMIT: usize = 8;
const OUTBOX_LEASE_MS: u64 = 8000;
const OUTBOX_MAX_ATTEMPTS: u32 = 6;
const OUTBOX_BACKOFF_BASE_MS: u64 = 800;
const OUTBOX_BACKOFF_MAX_MS: u64 = 30_000;

pub(crate) fn spawn_chat_outbox_worker(app: AppHandle) {
  thread::spawn(move || loop {
    let now = match now_millis() {
      Ok(value) => value,
      Err(err) => {
        log::warn!("chat outbox time read failed err={}", err);
        thread::sleep(Duration::from_millis(OUTBOX_POLL_INTERVAL_MS));
        continue;
      }
    };
    let workspace_ids: Vec<String> = match list_workspace_ids(app.state::<ChatDbManager>().inner()) {
      Ok(value) => value,
      Err(err) => {
        log::warn!("chat outbox list workspace failed err={}", err);
        thread::sleep(Duration::from_millis(OUTBOX_POLL_INTERVAL_MS));
        continue;
      }
    };
    for workspace_id in workspace_ids {
      let tasks = match chat_outbox_claim_due(
        app.state::<ChatDbManager>().inner(),
        workspace_id.as_str(),
        now,
        OUTBOX_CLAIM_LIMIT,
        OUTBOX_LEASE_MS,
      ) {
        Ok(value) => value,
        Err(err) => {
          log::warn!(
            "chat outbox claim failed workspace_id={} err={}",
            workspace_id,
            err
          );
          continue;
        }
      };
      if tasks.is_empty() {
        continue;
      }
      let Some(window) = resolve_dispatch_window(&app) else {
        for task in tasks {
          handle_dispatch_error(
            &app,
            &workspace_id,
            task.message_id.as_str(),
            &task.payload,
            task.attempts,
            "no_window",
          );
        }
        continue;
      };
      for task in tasks {
        dispatch_outbox_task(&app, &window, &workspace_id, task);
      }
    }
    thread::sleep(Duration::from_millis(OUTBOX_POLL_INTERVAL_MS));
  });
}

fn resolve_dispatch_window(app: &AppHandle) -> Option<WebviewWindow> {
  if let Ok(guard) = app.state::<crate::runtime::state::AppState>().active_main_window.lock() {
    if let Some(label) = guard.as_ref() {
      if let Some(window) = app.get_webview_window(label) {
        return Some(window);
      }
    }
  }
  if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
    return Some(window);
  }
  app.webview_windows().values().next().cloned()
}

fn dispatch_outbox_task(
  app: &AppHandle,
  window: &WebviewWindow,
  workspace_id: &str,
  task: crate::message_service::chat_db::ChatOutboxTask,
) {
  let payload = task.payload.clone();
  let dispatch_result = orchestrate_chat_dispatch(
    app,
    window,
    app.state::<TerminalManager>(),
    app.state::<ChatDbManager>(),
    app.state::<StorageManager>().inner(),
    payload.clone(),
  );
  match dispatch_result {
    Ok(_) => {
      if let Err(err) = chat_outbox_mark_sent(
        app.state::<ChatDbManager>().inner(),
        workspace_id,
        task.message_id.as_str(),
      ) {
        log::warn!(
          "chat outbox mark sent failed workspace_id={} message_id={} err={}",
          workspace_id,
          task.message_id,
          err
        );
      }
      let _ = chat_update_message_status(
        app,
        app.state::<ChatDbManager>().inner(),
        workspace_id,
        payload.conversation_id.as_str(),
        task.message_id.as_str(),
        MessageStatus::Sent,
      );
    }
    Err(err) => {
      handle_dispatch_error(
        app,
        workspace_id,
        task.message_id.as_str(),
        &payload,
        task.attempts,
        err.as_str(),
      );
    }
  }
}

fn handle_dispatch_error(
  app: &AppHandle,
  workspace_id: &str,
  message_id: &str,
  payload: &ChatDispatchPayload,
  attempts: u32,
  error: &str,
) {
  let now = now_millis().unwrap_or(0);
  let mark_dead = attempts >= OUTBOX_MAX_ATTEMPTS;
  let backoff = compute_backoff_ms(attempts);
  let next_attempt_at = now.saturating_add(backoff);
  if let Err(err) = chat_outbox_mark_failed(
    app.state::<ChatDbManager>().inner(),
    workspace_id,
    message_id,
    next_attempt_at,
    error,
    mark_dead,
  ) {
    log::warn!(
      "chat outbox mark failed workspace_id={} message_id={} err={}",
      workspace_id,
      message_id,
      err
    );
  }
  if mark_dead {
    let _ = chat_update_message_status(
      app,
      app.state::<ChatDbManager>().inner(),
      workspace_id,
      payload.conversation_id.as_str(),
      message_id,
      MessageStatus::Failed,
    );
  }
  diagnostics_log_backend_event(
    &app.state::<DiagnosticsState>(),
    Some(payload.sender_id.clone()),
    None,
    Some(payload.conversation_id.clone()),
    None,
    Some(workspace_id.to_string()),
    "chat_outbox_dispatch_error",
    json!({
      "workspaceId": workspace_id,
      "conversationId": payload.conversation_id,
      "messageId": message_id,
      "attempts": attempts,
      "maxAttempts": OUTBOX_MAX_ATTEMPTS,
      "error": error,
      "nextAttemptAt": if mark_dead { None } else { Some(next_attempt_at) }
    }),
  );
}

fn compute_backoff_ms(attempts: u32) -> u64 {
  let factor = attempts.max(1).saturating_sub(1).min(6);
  let scale = 2u64.saturating_pow(factor);
  let delay = OUTBOX_BACKOFF_BASE_MS.saturating_mul(scale);
  delay.min(OUTBOX_BACKOFF_MAX_MS)
}
