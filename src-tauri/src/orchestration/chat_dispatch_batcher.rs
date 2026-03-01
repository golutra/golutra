//! 聊天派发批次器：仅以语义 flush 完成作为释放门槛。

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::ports::terminal_dispatch_gate::TerminalDispatchGate;
use crate::terminal_engine::session::{terminal_dispatch, TerminalDispatchContext, TerminalManager};

// 合并派发分隔符：保持输入边界，避免 CLI 把多条指令拼接成一行。
const DISPATCH_BATCH_SEPARATOR: &str = "\n\n";
// 与终端派发一致的回车补发策略，避免输入模式误判。
const COMMAND_CONFIRM_DELAY_MS: u64 = 100;
const COMMAND_CONFIRM_SUFFIX: &str = "\r";

#[derive(Clone, Debug)]
struct DispatchBatch {
  text: String,
  context: TerminalDispatchContext,
  message_ids: Vec<String>,
}

impl DispatchBatch {
  fn new(text: String, context: TerminalDispatchContext) -> Self {
    let mut message_ids = Vec::new();
    if let Some(message_id) = context.message_id.clone() {
      message_ids.push(message_id);
    }
    Self {
      text,
      context,
      message_ids,
    }
  }

  fn merge(&mut self, mut next: DispatchBatch) {
    if !self.text.is_empty() && !next.text.is_empty() {
      self.text.push_str(DISPATCH_BATCH_SEPARATOR);
    }
    self.text.push_str(&next.text);
    if self.context.message_id.is_none() {
      self.context.message_id = next.context.message_id.take();
    }
    self.message_ids.append(&mut next.message_ids);
  }
}

#[derive(Default)]
struct DispatchQueue {
  inflight: Option<DispatchBatch>,
  pending: VecDeque<DispatchBatch>,
}

/// 终端派发批次器：用于合并输入并按语义 flush 释放。
pub(crate) struct ChatDispatchBatcher {
  queues: Mutex<std::collections::HashMap<String, DispatchQueue>>,
}

impl ChatDispatchBatcher {
  pub(crate) fn new() -> Self {
    Self {
      queues: Mutex::new(std::collections::HashMap::new()),
    }
  }

  pub(crate) fn enqueue_for_terminal(
    &self,
    app: &AppHandle,
    terminal_id: String,
    text: String,
    context: TerminalDispatchContext,
  ) -> Result<(), String> {
    let terminal_state = app.state::<TerminalManager>();
    if terminal_state.is_terminal_dnd(&terminal_id) {
      return Ok(());
    }
    let batch = DispatchBatch::new(text, context);
    let mut dispatch_now: Option<DispatchBatch> = None;
    {
      let mut guard = self
        .queues
        .lock()
        .map_err(|_| "chat dispatch batcher lock poisoned".to_string())?;
      let queue = guard.entry(terminal_id.clone()).or_default();
      if has_message_id_conflict(queue, &batch) {
        return Ok(());
      }
      if let Some(last) = queue.pending.back_mut() {
        if can_merge_context(&last.context, &batch.context) {
          last.merge(batch);
        } else {
          queue.pending.push_back(batch);
        }
      } else {
        queue.pending.push_back(batch);
      }
      if queue.inflight.is_none() {
        if let Some(next) = queue.pending.pop_front() {
          queue.inflight = Some(next.clone());
          dispatch_now = Some(next);
        }
      }
    }
    if let Some(batch) = dispatch_now {
      if let Err(err) = dispatch_batch(app, &terminal_id, &batch) {
        restore_failed_dispatch(&self.queues, &terminal_id, batch);
        return Err(err);
      }
    }
    Ok(())
  }

  fn handle_semantic_flush_complete(&self, app: &AppHandle, terminal_id: &str) {
    let mut dispatch_now: Option<DispatchBatch> = None;
    {
      let mut guard = match self.queues.lock() {
        Ok(guard) => guard,
        Err(err) => err.into_inner(),
      };
      let Some(queue) = guard.get_mut(terminal_id) else {
        return;
      };
      queue.inflight = None;
      if let Some(next) = queue.pending.pop_front() {
        queue.inflight = Some(next.clone());
        dispatch_now = Some(next);
      }
      if queue.inflight.is_none() && queue.pending.is_empty() {
        guard.remove(terminal_id);
      }
    }
    if let Some(batch) = dispatch_now {
      if let Err(err) = dispatch_batch(app, terminal_id, &batch) {
        restore_failed_dispatch(&self.queues, terminal_id, batch);
        log::warn!(
          "chat dispatch batch resend failed terminal_id={} err={}",
          terminal_id,
          err
        );
      }
    }
  }
}

impl TerminalDispatchGate for ChatDispatchBatcher {
  fn on_semantic_flush_complete(&self, app: &AppHandle, terminal_id: &str) {
    self.handle_semantic_flush_complete(app, terminal_id);
  }
}

fn can_merge_context(base: &TerminalDispatchContext, next: &TerminalDispatchContext) -> bool {
  base.conversation_id == next.conversation_id
    && base.conversation_type == next.conversation_type
    && base.sender_id == next.sender_id
    && base.sender_name == next.sender_name
}

fn has_message_id_conflict(queue: &DispatchQueue, batch: &DispatchBatch) -> bool {
  let Some(message_id) = batch.context.message_id.as_ref() else {
    return false;
  };
  if queue
    .inflight
    .as_ref()
    .is_some_and(|batch| batch.message_ids.iter().any(|id| id == message_id))
  {
    return true;
  }
  queue
    .pending
    .iter()
    .any(|batch| batch.message_ids.iter().any(|id| id == message_id))
}

fn dispatch_batch(
  app: &AppHandle,
  terminal_id: &str,
  batch: &DispatchBatch,
) -> Result<(), String> {
  let terminal_state = app.state::<TerminalManager>();
  terminal_dispatch(
    app.clone(),
    terminal_state,
    terminal_id.to_string(),
    batch.text.clone(),
    batch.context.clone(),
  )?;
  std::thread::sleep(Duration::from_millis(COMMAND_CONFIRM_DELAY_MS));
  terminal_dispatch(
    app.clone(),
    app.state::<TerminalManager>(),
    terminal_id.to_string(),
    COMMAND_CONFIRM_SUFFIX.to_string(),
    batch.context.clone(),
  )?;
  Ok(())
}

fn restore_failed_dispatch(
  queues: &Mutex<std::collections::HashMap<String, DispatchQueue>>,
  terminal_id: &str,
  batch: DispatchBatch,
) {
  let mut guard = match queues.lock() {
    Ok(guard) => guard,
    Err(err) => err.into_inner(),
  };
  let queue = guard.entry(terminal_id.to_string()).or_default();
  queue.inflight = None;
  queue.pending.push_front(batch);
}
