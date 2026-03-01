//! 触发式调度：事件驱动规则评估，状态回落保持定时触发。
//! 约束：调度器只做触发与调度，不持有会话级业务状态。

use std::{
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use tauri::AppHandle;

use crate::now_millis;

use super::polling::{
    build_poll_actions, collect_poll_snapshots_by_ids, collect_poll_snapshots_by_working_set,
    dispatch_poll_actions, RuleMask,
};
use super::polling::snapshot::SessionPollSnapshot;
use super::trigger::{
    plan_trigger, DeferredStage, GuardianEvent, ScheduledTrigger, TriggerBus, TriggerEvent,
    TriggerScheduler, TriggerTargets,
};
use super::{lock_sessions, PostReadyState, SessionRegistry, TerminalManager, POST_READY_TICK_MS};
use crate::ports::terminal_event::TerminalEventPort;
use crate::ports::terminal_session::TerminalSessionRepository;

/// 启动状态驱动线程：只在触发事件时评估对应规则。
pub(crate) fn spawn_status_poller(app: AppHandle, manager: &TerminalManager) {
    let sessions = Arc::clone(&manager.sessions);
    let event_port = manager.event_port();
    let session_repository = manager.session_repository();
    let settings_service = {
        let guard = manager
            .settings_service
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        guard.clone()
    };
    let (tx, rx) = mpsc::channel();
    let trigger_bus = Arc::new(TriggerBus::new(tx));
    {
        let mut guard = lock_sessions(&sessions);
        guard.trigger_bus = Some(Arc::clone(&trigger_bus));
    }
    thread::spawn(move || {
        let mut scheduler = TriggerScheduler::default();
        if let Ok(now) = now_millis() {
            scheduler.schedule(ScheduledTrigger::immediate(
                now,
                TriggerEvent::Guardian(GuardianEvent::StatusFallbackTick),
            ));
        }
        loop {
            let now = match now_millis() {
                Ok(value) => value,
                Err(_) => continue,
            };
            for event in scheduler.pop_due(now) {
                handle_trigger_event(
                    event,
                    now,
                    &app,
                    &sessions,
                    &event_port,
                    &session_repository,
                    settings_service.clone(),
                    &mut scheduler,
                );
            }
            let timeout_ms = scheduler
                .next_due_at()
                .map(|due_at| due_at.saturating_sub(now))
                .unwrap_or(500);
            match rx.recv_timeout(Duration::from_millis(timeout_ms)) {
                Ok(event) => {
                    let event_now = match now_millis() {
                        Ok(value) => value,
                        Err(_) => now,
                    };
                    handle_trigger_event(
                        event,
                        event_now,
                        &app,
                        &sessions,
                        &event_port,
                        &session_repository,
                        settings_service.clone(),
                        &mut scheduler,
                    )
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}

fn handle_trigger_event(
    event: TriggerEvent,
    now: u64,
    app: &AppHandle,
    sessions: &Arc<std::sync::Mutex<SessionRegistry>>,
    event_port: &Arc<dyn TerminalEventPort>,
    session_repository: &Arc<dyn TerminalSessionRepository>,
    settings_service: Option<Arc<dyn crate::ports::settings::TerminalSettingsPort>>,
    scheduler: &mut TriggerScheduler,
) {
    let plan = plan_trigger(event, now);
    for scheduled in plan.schedules {
        scheduler.schedule(scheduled);
    }
    if plan.mask.is_empty() {
        return;
    }
    let snapshots = match plan.targets {
        TriggerTargets::WorkingSet => collect_poll_snapshots_by_working_set(sessions),
        TriggerTargets::SessionIds(ids) => collect_poll_snapshots_by_ids(sessions, &ids),
    };
    if snapshots.is_empty() {
        return;
    }
    let actions = build_poll_actions(&snapshots, now, plan.mask);
    dispatch_poll_actions(
        app,
        sessions,
        event_port,
        session_repository,
        settings_service,
        actions,
    );
    if plan.mask.contains(RuleMask::POST_READY) {
        schedule_post_ready_ticks(&snapshots, now, scheduler);
    }
}

fn schedule_post_ready_ticks(
    snapshots: &[SessionPollSnapshot],
    now: u64,
    scheduler: &mut TriggerScheduler,
) {
    for snapshot in snapshots {
        if snapshot.post_ready_state != PostReadyState::Starting {
            continue;
        }
        scheduler.schedule(ScheduledTrigger::deferred(
            now.saturating_add(POST_READY_TICK_MS),
            snapshot.terminal_id.clone(),
            RuleMask::POST_READY,
            DeferredStage::PostReadyTick,
        ));
    }
}
