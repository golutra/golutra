//! 聊天 Outbox：持久化派发任务，支持重试与恢复。

use redb::ReadableTable;

use crate::contracts::chat_dispatch::ChatDispatchPayload;

use super::store::{
  decode, encode, now_millis, open_db, parse_ulid, CHAT_OUTBOX_SCHEDULE, CHAT_OUTBOX_TASKS,
};
use super::types::{ChatOutboxStatus, ChatOutboxTask};
use super::ChatDbManager;

fn remove_schedule_entry(
  schedule: &mut redb::Table<(u64, u128), ()>,
  scheduled_at: u64,
  message_id: u128,
) {
  let _ = schedule.remove((scheduled_at, message_id));
}

fn insert_schedule_entry(
  schedule: &mut redb::Table<(u64, u128), ()>,
  scheduled_at: u64,
  message_id: u128,
) -> Result<(), String> {
  schedule
    .insert((scheduled_at, message_id), ())
    .map_err(|err| format!("failed to insert chat_outbox_schedule: {err}"))?;
  Ok(())
}

pub(crate) fn chat_outbox_enqueue(
  state: &ChatDbManager,
  workspace_id: &str,
  message_id: &str,
  mut payload: ChatDispatchPayload,
) -> Result<ChatOutboxTask, String> {
  let message_id_u128 = parse_ulid(message_id)?;
  let payload_message_id = payload.message_id.as_deref().map(str::trim).unwrap_or("");
  if payload_message_id.is_empty() {
    payload.message_id = Some(message_id.to_string());
  }
  let now = now_millis()?;
  let db = open_db(state, workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat outbox write: {err}"))?;

  let task = {
    let mut schedule = txn
      .open_table(CHAT_OUTBOX_SCHEDULE)
      .map_err(|err| format!("failed to open chat_outbox_schedule: {err}"))?;
    let mut tasks = txn
      .open_table(CHAT_OUTBOX_TASKS)
      .map_err(|err| format!("failed to open chat_outbox_tasks: {err}"))?;

    let existing = tasks
      .get(message_id_u128)
      .map_err(|err| format!("failed to read chat_outbox_tasks: {err}"))?
      .and_then(|value| decode::<ChatOutboxTask>(value.value()).ok());

    if let Some(existing) = existing.as_ref() {
      remove_schedule_entry(&mut schedule, existing.next_attempt_at, message_id_u128);
    }

    let task = ChatOutboxTask {
      message_id: message_id.to_string(),
      payload,
      status: ChatOutboxStatus::Pending,
      attempts: existing.as_ref().map(|value| value.attempts).unwrap_or(0),
      created_at: existing.as_ref().map(|value| value.created_at).unwrap_or(now),
      updated_at: now,
      next_attempt_at: now,
      sending_since: None,
      last_error: None,
    };

    let payload = encode(&task)?;
    tasks
      .insert(message_id_u128, payload.as_slice())
      .map_err(|err| format!("failed to insert chat_outbox_tasks: {err}"))?;
    insert_schedule_entry(&mut schedule, task.next_attempt_at, message_id_u128)?;
    task
  };

  txn
    .commit()
    .map_err(|err| format!("failed to commit chat outbox enqueue: {err}"))?;

  Ok(task)
}

pub(crate) fn chat_outbox_claim_due(
  state: &ChatDbManager,
  workspace_id: &str,
  now: u64,
  limit: usize,
  lease_ms: u64,
) -> Result<Vec<ChatOutboxTask>, String> {
  let db = open_db(state, workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat outbox write: {err}"))?;

  let claimed = {
    let mut schedule = txn
      .open_table(CHAT_OUTBOX_SCHEDULE)
      .map_err(|err| format!("failed to open chat_outbox_schedule: {err}"))?;
    let mut tasks = txn
      .open_table(CHAT_OUTBOX_TASKS)
      .map_err(|err| format!("failed to open chat_outbox_tasks: {err}"))?;

    let mut claimed = Vec::new();
    let mut candidates = Vec::new();
    let start = (0u64, 0u128);
    let end = (now, u128::MAX);
    for entry in schedule
      .range(start..=end)
      .map_err(|err| format!("failed to scan chat_outbox_schedule: {err}"))?
    {
      let (key, _) = entry.map_err(|err| format!("failed to decode chat_outbox_schedule: {err}"))?;
      candidates.push(key.value());
      if candidates.len() >= limit {
        break;
      }
    }

    for (scheduled_at, message_id) in candidates {
      remove_schedule_entry(&mut schedule, scheduled_at, message_id);
      let task = match tasks
        .get(message_id)
        .map_err(|err| format!("failed to read chat_outbox_tasks: {err}"))?
      {
        Some(value) => decode::<ChatOutboxTask>(value.value())?,
        None => continue,
      };

      if matches!(task.status, ChatOutboxStatus::Sent | ChatOutboxStatus::Dead) {
        continue;
      }

      if let Some(sending_since) = task.sending_since {
        if task.status == ChatOutboxStatus::Sending && sending_since.saturating_add(lease_ms) > now {
          let mut updated = task;
          updated.next_attempt_at = sending_since.saturating_add(lease_ms);
          let payload = encode(&updated)?;
          tasks
            .insert(message_id, payload.as_slice())
            .map_err(|err| format!("failed to update chat_outbox_tasks: {err}"))?;
          insert_schedule_entry(&mut schedule, updated.next_attempt_at, message_id)?;
          continue;
        }
      }

      let mut updated = task;
      updated.status = ChatOutboxStatus::Sending;
      updated.attempts = updated.attempts.saturating_add(1);
      updated.updated_at = now;
      updated.sending_since = Some(now);
      updated.next_attempt_at = now.saturating_add(lease_ms);
      updated.last_error = None;

      let payload = encode(&updated)?;
      tasks
        .insert(message_id, payload.as_slice())
        .map_err(|err| format!("failed to update chat_outbox_tasks: {err}"))?;
      insert_schedule_entry(&mut schedule, updated.next_attempt_at, message_id)?;
      claimed.push(updated);
    }
    claimed
  };

  txn
    .commit()
    .map_err(|err| format!("failed to commit chat outbox claim: {err}"))?;

  Ok(claimed)
}

pub(crate) fn chat_outbox_mark_sent(
  state: &ChatDbManager,
  workspace_id: &str,
  message_id: &str,
) -> Result<(), String> {
  let message_id_u128 = parse_ulid(message_id)?;
  let db = open_db(state, workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat outbox write: {err}"))?;
  {
    let mut schedule = txn
      .open_table(CHAT_OUTBOX_SCHEDULE)
      .map_err(|err| format!("failed to open chat_outbox_schedule: {err}"))?;
    let mut tasks = txn
      .open_table(CHAT_OUTBOX_TASKS)
      .map_err(|err| format!("failed to open chat_outbox_tasks: {err}"))?;

    let mut task: ChatOutboxTask = {
      let value = tasks
        .get(message_id_u128)
        .map_err(|err| format!("failed to read chat_outbox_tasks: {err}"))?;
      let Some(value) = value else {
        return Ok(());
      };
      decode(value.value())?
    };
    remove_schedule_entry(&mut schedule, task.next_attempt_at, message_id_u128);
    task.status = ChatOutboxStatus::Sent;
    task.updated_at = now_millis()?;
    task.sending_since = None;
    task.last_error = None;
    let payload = encode(&task)?;
    tasks
      .insert(message_id_u128, payload.as_slice())
      .map_err(|err| format!("failed to update chat_outbox_tasks: {err}"))?;
  }
  txn
    .commit()
    .map_err(|err| format!("failed to commit chat outbox sent: {err}"))?;
  Ok(())
}

pub(crate) fn chat_outbox_mark_failed(
  state: &ChatDbManager,
  workspace_id: &str,
  message_id: &str,
  next_attempt_at: u64,
  error: &str,
  mark_dead: bool,
) -> Result<(), String> {
  let message_id_u128 = parse_ulid(message_id)?;
  let db = open_db(state, workspace_id)?;
  let txn = db
    .begin_write()
    .map_err(|err| format!("failed to open chat outbox write: {err}"))?;
  {
    let mut schedule = txn
      .open_table(CHAT_OUTBOX_SCHEDULE)
      .map_err(|err| format!("failed to open chat_outbox_schedule: {err}"))?;
    let mut tasks = txn
      .open_table(CHAT_OUTBOX_TASKS)
      .map_err(|err| format!("failed to open chat_outbox_tasks: {err}"))?;

    let mut task: ChatOutboxTask = {
      let value = tasks
        .get(message_id_u128)
        .map_err(|err| format!("failed to read chat_outbox_tasks: {err}"))?;
      let Some(value) = value else {
        return Ok(());
      };
      decode(value.value())?
    };
    remove_schedule_entry(&mut schedule, task.next_attempt_at, message_id_u128);
    task.updated_at = now_millis()?;
    task.sending_since = None;
    task.last_error = Some(error.to_string());
    if mark_dead {
      task.status = ChatOutboxStatus::Dead;
    } else {
      task.status = ChatOutboxStatus::Failed;
      task.next_attempt_at = next_attempt_at;
      insert_schedule_entry(&mut schedule, task.next_attempt_at, message_id_u128)?;
    }
    let payload = encode(&task)?;
    tasks
      .insert(message_id_u128, payload.as_slice())
      .map_err(|err| format!("failed to update chat_outbox_tasks: {err}"))?;
  }
  txn
    .commit()
    .map_err(|err| format!("failed to commit chat outbox failed: {err}"))?;
  Ok(())
}
