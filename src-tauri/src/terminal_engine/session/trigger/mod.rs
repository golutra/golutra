//! 触发式调度：事件驱动规则评估，避免无意义轮询。

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::mpsc;

use super::polling::RuleMask;

mod dispatcher;

pub(crate) use dispatcher::{plan_trigger, TriggerTargets};

#[derive(Clone, Debug)]
pub(crate) enum FactEvent {
  OutputUpdated { terminal_id: String, observed_at: u64 },
  ShellReady { terminal_id: String },
  ChatPending { terminal_id: String, observed_at: u64 },
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum DeferredStage {
  Stable,
  Silence,
  Debounce,
  PostReadyTick,
  ChatPendingForce,
}

#[derive(Clone, Debug)]
pub(crate) enum GuardianEvent {
  StatusFallbackTick,
  Deferred {
    terminal_id: String,
    mask: RuleMask,
  },
}

#[derive(Clone, Debug)]
pub(crate) enum TriggerEvent {
  Fact(FactEvent),
  Guardian(GuardianEvent),
}

#[derive(Clone)]
pub(crate) struct TriggerBus {
  sender: mpsc::Sender<TriggerEvent>,
}

impl TriggerBus {
  pub(crate) fn new(sender: mpsc::Sender<TriggerEvent>) -> Self {
    Self { sender }
  }

  pub(crate) fn emit(&self, event: TriggerEvent) {
    let _ = self.sender.send(event);
  }

  pub(crate) fn emit_fact(&self, event: FactEvent) {
    self.emit(TriggerEvent::Fact(event));
  }
}

#[derive(Clone, Debug)]
pub(crate) struct ScheduledTrigger {
  pub(crate) due_at: u64,
  pub(crate) event: TriggerEvent,
  key: Option<TriggerKey>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct TriggerKey {
  terminal_id: String,
  mask: RuleMask,
  stage: DeferredStage,
}

#[derive(Default)]
pub(crate) struct TriggerScheduler {
  scheduled: BinaryHeap<ScheduledTrigger>,
  latest_due: HashMap<TriggerKey, u64>,
}

impl TriggerScheduler {
  pub(crate) fn schedule(&mut self, trigger: ScheduledTrigger) {
    if let Some(key) = trigger.key.as_ref() {
      let entry = self.latest_due.entry(key.clone()).or_insert(0);
      if *entry >= trigger.due_at {
        return;
      }
      *entry = trigger.due_at;
    }
    self.scheduled.push(trigger);
  }

  pub(crate) fn next_due_at(&self) -> Option<u64> {
    self.scheduled.peek().map(|item| item.due_at)
  }

  pub(crate) fn pop_due(&mut self, now: u64) -> Vec<TriggerEvent> {
    let mut ready = Vec::new();
    while let Some(item) = self.scheduled.peek() {
      if item.due_at > now {
        break;
      }
      let item = self.scheduled.pop().expect("scheduled not empty");
      if let Some(key) = item.key.as_ref() {
        if let Some(latest) = self.latest_due.get(key) {
          if *latest != item.due_at {
            continue;
          }
        }
        self.latest_due.remove(key);
      }
      ready.push(item.event);
    }
    ready
  }
}

impl ScheduledTrigger {
  fn new(
    due_at: u64,
    event: TriggerEvent,
    key: Option<TriggerKey>,
  ) -> Self {
    Self { due_at, event, key }
  }
}

impl ScheduledTrigger {
  pub(crate) fn immediate(due_at: u64, event: TriggerEvent) -> Self {
    ScheduledTrigger::new(due_at, event, None)
  }

  pub(crate) fn deferred(
    due_at: u64,
    terminal_id: String,
    mask: RuleMask,
    stage: DeferredStage,
  ) -> Self {
    let key = Some(TriggerKey {
      terminal_id: terminal_id.clone(),
      mask,
      stage: stage.clone(),
    });
    ScheduledTrigger::new(
      due_at,
      TriggerEvent::Guardian(GuardianEvent::Deferred { terminal_id, mask }),
      key,
    )
  }
}

impl PartialEq for ScheduledTrigger {
  fn eq(&self, other: &Self) -> bool {
    self.due_at == other.due_at
  }
}

impl Eq for ScheduledTrigger {}

impl PartialOrd for ScheduledTrigger {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for ScheduledTrigger {
  fn cmp(&self, other: &Self) -> Ordering {
    // BinaryHeap 是最大堆，这里反转排序确保最早到期优先。
    other.due_at.cmp(&self.due_at)
  }
}
