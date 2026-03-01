//! 触发策略：把事件映射到规则执行范围与后续定时触发。
// 约束：规则层只产出动作，不直接派发 TriggerEvent，避免形成隐形流程。

use super::super::polling::RuleMask;
use super::super::{
    CHAT_IDLE_DEBOUNCE_MS, CHAT_PENDING_FORCE_FLUSH_MS, CHAT_SILENCE_TIMEOUT_MS,
    POST_READY_STABLE_MS, STATUS_POLL_INTERVAL_MS,
};
use super::{DeferredStage, FactEvent, GuardianEvent, ScheduledTrigger, TriggerEvent};

#[derive(Clone, Debug)]
pub(crate) struct TriggerPlan {
    pub(crate) mask: RuleMask,
    pub(crate) targets: TriggerTargets,
    pub(crate) schedules: Vec<ScheduledTrigger>,
}

#[derive(Clone, Debug)]
pub(crate) enum TriggerTargets {
    SessionIds(Vec<String>),
    WorkingSet,
}

pub(crate) fn plan_trigger(event: TriggerEvent, now: u64) -> TriggerPlan {
    match event {
        TriggerEvent::Guardian(GuardianEvent::StatusFallbackTick) => {
            let plan = TriggerPlan {
                mask: RuleMask::STATUS_FALLBACK,
                // 状态回落只允许从 Working 集合采集，避免全量扫描。
                targets: TriggerTargets::WorkingSet,
                schedules: vec![ScheduledTrigger::immediate(
                    now.saturating_add(STATUS_POLL_INTERVAL_MS),
                    TriggerEvent::Guardian(GuardianEvent::StatusFallbackTick),
                )],
            };
            debug_assert!(
                plan.mask.contains(RuleMask::STATUS_FALLBACK),
                "STATUS_FALLBACK must be triggered by Guardian events"
            );
            plan
        }
        TriggerEvent::Guardian(GuardianEvent::Deferred { terminal_id, mask }) => {
            let plan = TriggerPlan {
                mask,
                targets: TriggerTargets::SessionIds(vec![terminal_id]),
                schedules: Vec::new(),
            };
            debug_assert!(
                !plan.mask.contains(RuleMask::STATUS_FALLBACK),
                "STATUS_FALLBACK must only be triggered by Guardian tick"
            );
            plan
        }
        TriggerEvent::Fact(fact) => {
            let plan = plan_fact(fact);
            debug_assert!(
                !plan.mask.contains(RuleMask::STATUS_FALLBACK),
                "STATUS_FALLBACK must only be triggered by Guardian events"
            );
            plan
        }
    }
}

fn plan_fact(event: FactEvent) -> TriggerPlan {
    match event {
        FactEvent::OutputUpdated {
            terminal_id,
            observed_at,
        } => TriggerPlan {
            mask: RuleMask::POST_READY | RuleMask::SEMANTIC_FLUSH,
            targets: TriggerTargets::SessionIds(vec![terminal_id.clone()]),
            schedules: vec![
                ScheduledTrigger::deferred(
                    observed_at.saturating_add(POST_READY_STABLE_MS),
                    terminal_id.clone(),
                    RuleMask::POST_READY,
                    DeferredStage::Stable,
                ),
                ScheduledTrigger::deferred(
                    observed_at.saturating_add(CHAT_SILENCE_TIMEOUT_MS),
                    terminal_id.clone(),
                    RuleMask::SEMANTIC_FLUSH,
                    DeferredStage::Silence,
                ),
                ScheduledTrigger::deferred(
                    observed_at
                        .saturating_add(CHAT_SILENCE_TIMEOUT_MS)
                        .saturating_add(CHAT_IDLE_DEBOUNCE_MS),
                    terminal_id.clone(),
                    RuleMask::SEMANTIC_FLUSH,
                    DeferredStage::Debounce,
                ),
            ],
        },
        FactEvent::ShellReady { terminal_id } => TriggerPlan {
            mask: RuleMask::POST_READY,
            targets: TriggerTargets::SessionIds(vec![terminal_id]),
            schedules: Vec::new(),
        },
        FactEvent::ChatPending {
            terminal_id,
            observed_at,
        } => TriggerPlan {
            mask: RuleMask::SEMANTIC_FLUSH,
            targets: TriggerTargets::SessionIds(vec![terminal_id.clone()]),
            schedules: vec![
                ScheduledTrigger::deferred(
                    observed_at.saturating_add(CHAT_SILENCE_TIMEOUT_MS),
                    terminal_id.clone(),
                    RuleMask::SEMANTIC_FLUSH,
                    DeferredStage::Silence,
                ),
                ScheduledTrigger::deferred(
                    observed_at
                        .saturating_add(CHAT_SILENCE_TIMEOUT_MS)
                        .saturating_add(CHAT_IDLE_DEBOUNCE_MS),
                    terminal_id.clone(),
                    RuleMask::SEMANTIC_FLUSH,
                    DeferredStage::Debounce,
                ),
                ScheduledTrigger::deferred(
                    observed_at.saturating_add(CHAT_PENDING_FORCE_FLUSH_MS),
                    terminal_id,
                    RuleMask::SEMANTIC_FLUSH,
                    DeferredStage::ChatPendingForce,
                ),
            ],
        },
    }
}
