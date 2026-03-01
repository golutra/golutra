//! 命令结果中心：为外部命令提供异步结果等待能力。

use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommandResultPayload {
  pub(crate) status: String,
  pub(crate) message: Option<String>,
  pub(crate) data: Option<serde_json::Value>,
}

#[derive(Default)]
pub(crate) struct CommandCenter {
  slots: Mutex<HashMap<String, Arc<CommandSlot>>>,
}

impl CommandCenter {
  pub(crate) fn new() -> Self {
    Self {
      slots: Mutex::new(HashMap::new()),
    }
  }

  pub(crate) fn create_slot(&self, request_id: &str) -> Arc<CommandSlot> {
    let slot = Arc::new(CommandSlot::new());
    let mut guard = self.slots.lock().unwrap_or_else(|err| err.into_inner());
    guard.insert(request_id.to_string(), Arc::clone(&slot));
    slot
  }

  pub(crate) fn complete(&self, request_id: &str, result: CommandResultPayload) -> bool {
    let slot = {
      let guard = self.slots.lock().unwrap_or_else(|err| err.into_inner());
      guard.get(request_id).cloned()
    };
    let Some(slot) = slot else {
      return false;
    };
    slot.set_result(result);
    true
  }

  pub(crate) fn wait_result(&self, request_id: &str) -> Result<CommandResultPayload, String> {
    let slot = {
      let guard = self.slots.lock().unwrap_or_else(|err| err.into_inner());
      guard
        .get(request_id)
        .cloned()
        .ok_or_else(|| "command request not found".to_string())?
    };
    let result = slot.wait();
    let mut guard = self.slots.lock().unwrap_or_else(|err| err.into_inner());
    guard.remove(request_id);
    Ok(result)
  }
}

pub(crate) struct CommandSlot {
  result: Mutex<Option<CommandResultPayload>>,
  ready: Condvar,
}

impl CommandSlot {
  fn new() -> Self {
    Self {
      result: Mutex::new(None),
      ready: Condvar::new(),
    }
  }

  fn set_result(&self, result: CommandResultPayload) {
    let mut guard = self.result.lock().unwrap_or_else(|err| err.into_inner());
    *guard = Some(result);
    self.ready.notify_all();
  }

  fn wait(&self) -> CommandResultPayload {
    let mut guard = self.result.lock().unwrap_or_else(|err| err.into_inner());
    loop {
      if let Some(result) = guard.clone() {
        return result;
      }
      guard = self.ready.wait(guard).unwrap_or_else(|err| err.into_inner());
    }
  }
}
