//! 启动后流程：稳定后执行默认成员的后置注入。

use std::sync::{Arc, Mutex};

use crate::now_millis;
use crate::ports::terminal_event::TerminalEventPort;
use crate::ports::terminal_session::TerminalSessionRepository;
use crate::terminal_engine::default_members::registry::TerminalPostReadyStep;
use crate::terminal_engine::default_members::resolve_default_member;

use super::keyboard_input::{parse_session_id_from_lines, send_post_ready_input, ENTER_INPUT_DELAY_MS};
use super::stability::is_output_stable;
use super::state::{PostReadyAction, PostReadyQueueItem};
use super::{lock_sessions, unlock_session_status_by_member, PostReadyState, SessionRegistry};

const POST_READY_STEP_REASON: &str = "post-ready-input";
const EXTRACT_SESSION_ID_TIMEOUT_MS: u64 = 2000;
const EXTRACT_SESSION_ID_MAX_RESTARTS: u32 = 3;

pub(super) fn maybe_start_post_ready(
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &dyn TerminalEventPort,
    terminal_id: &str,
    terminal_type: &str,
) -> Result<(), String> {
    // 启动顺序：shell_ready + 稳定门禁通过后进入 post_ready。
    let plan = resolve_default_member(terminal_type).map(|member| member.post_ready_plan);
    let plan = match plan {
        Some(plan) => plan,
        None => {
            finish_post_ready(sessions, event_port, terminal_id)?;
            return Ok(());
        }
    };
    let should_finish = {
        let mut guard = lock_sessions(sessions);
        let session = guard
            .sessions
            .get_mut(terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        if !session.post_ready_mode.should_run() {
            session.post_ready_state = PostReadyState::Done;
            session.post_ready_queue.clear();
            return Ok(());
        }
        if session.post_ready_state != PostReadyState::Idle {
            return Ok(());
        }
        if !session.active || session.handle.is_none() {
            return Err("terminal session is not active".to_string());
        }
        if plan.post_ready_steps.is_empty() {
            true
        } else {
            session.post_ready_state = PostReadyState::Starting;
            false
        }
    };
    if should_finish {
        finish_post_ready(sessions, event_port, terminal_id)?;
        return Ok(());
    }
    queue_post_ready_steps(sessions, terminal_id, plan.post_ready_steps)?;
    Ok(())
}

pub(super) fn maybe_step_post_ready(
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &dyn TerminalEventPort,
    session_repository: &dyn TerminalSessionRepository,
    settings_service: Option<&dyn crate::ports::settings::TerminalSettingsPort>,
    terminal_id: &str,
    force: bool,
) -> Result<(), String> {
    // 逐条下发启动后输入：每个输入可选择是否等待画面稳定。
    let (action, require_stable) = {
        let now = now_millis()?;
        let mut guard = lock_sessions(sessions);
        let session = guard
            .sessions
            .get_mut(terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        if session.post_ready_state != PostReadyState::Starting {
            return Ok(());
        }
        if session.post_ready_restart_pending {
            return Ok(());
        }
        if session.post_ready_queue.is_empty() {
            drop(guard);
            finish_post_ready(sessions, event_port, terminal_id)?;
            return Ok(());
        }
        if session.flow_paused {
            return Ok(());
        }
        let item = match session.post_ready_queue.front() {
            Some(item) => item,
            None => return Ok(()),
        };
        if item.require_stable && !is_output_stable(session, force, now) {
            return Ok(());
        }
        let item = match session.post_ready_queue.pop_front() {
            Some(item) => item,
            None => return Ok(()),
        };
        (item.action, item.require_stable)
    };

    let mut finish_post_ready_now = false;
    let result = match action {
        PostReadyAction::Input(input) => send_post_ready_input(
            sessions,
            event_port,
            terminal_id,
            input.as_str(),
            POST_READY_STEP_REASON,
        )
        .map_err(|err| (PostReadyAction::Input(input), err)),
        PostReadyAction::Delay { ms, started_at } => {
            let now = now_millis()?;
            let started_at = started_at.unwrap_or(now);
            if now.saturating_sub(started_at) >= ms {
                Ok(())
            } else {
                Err((
                    PostReadyAction::Delay {
                        ms,
                        started_at: Some(started_at),
                    },
                    String::new(),
                ))
            }
        }
        PostReadyAction::Introduction { prompt_type } => {
            let type_enum = match prompt_type.as_str() {
                crate::terminal_engine::default_members::onboarding::PROMPT_TYPE_ONBOARDING => {
                    crate::terminal_engine::default_members::onboarding::PromptType::Onboarding
                }
                _ => crate::terminal_engine::default_members::onboarding::PromptType::Onboarding,
            };
            let language = settings_service.and_then(|s| s.get_language());
            let member_id = {
                let guard = super::lock_sessions(sessions);
                guard
                    .sessions
                    .get(terminal_id)
                    .and_then(|s| s.member_id.clone())
            };
            match member_id {
                Some(member_id) => {
                    let input = crate::terminal_engine::default_members::onboarding::generate_prompt(
                        type_enum,
                        member_id.as_str(),
                        language.as_deref(),
                    );
                    send_post_ready_input(
                        sessions,
                        event_port,
                        terminal_id,
                        input.as_str(),
                        POST_READY_STEP_REASON,
                    )
                    .map_err(|err| (PostReadyAction::Introduction { prompt_type }, err))
                }
                None => Err((
                    PostReadyAction::Introduction { prompt_type },
                    "terminal session missing member_id".to_string(),
                )),
            }
        }
        PostReadyAction::ExtractSessionId { keyword } => {
            let extracted = {
                let guard = lock_sessions(sessions);
                let session = guard
                    .sessions
                    .get(terminal_id)
                    .ok_or_else(|| "terminal session not found".to_string())?;
                let lines = session.snapshot.snapshot_lines();
                parse_session_id_from_lines(&lines, &keyword)
            };
            if let Some(value) = extracted.as_ref() {
                let (workspace_id, member_id, previous) = {
                    let mut guard = lock_sessions(sessions);
                    let session = guard
                        .sessions
                        .get_mut(terminal_id)
                        .ok_or_else(|| "terminal session not found".to_string())?;
                    let previous = session.remote_session_id.clone();
                    session.remote_session_id = Some(value.clone());
                    session.post_ready_session_id_started_at = None;
                    session.post_ready_session_id_restart_count = 0;
                    session.post_ready_restart_pending = false;
                    (
                        session.workspace_id.clone(),
                        session.member_id.clone(),
                        previous,
                    )
                };
                if let Some(workspace_id) = workspace_id.as_deref() {
                    if previous.as_deref() != Some(value.as_str()) {
                        let Some(member_id) = member_id.as_deref() else {
                            log::warn!(
                                "terminal session map skipped missing member_id terminal_id={}",
                                terminal_id
                            );
                            return Ok(());
                        };
                        session_repository.upsert_terminal_session(
                            workspace_id,
                            member_id,
                            value,
                        )?;
                        log::info!(
                            "terminal session map updated member_id={} session_id={}",
                            member_id,
                            value
                        );
                    }
                } else {
                    log::warn!(
                        "terminal session map skipped missing workspace_id terminal_id={}",
                        terminal_id
                    );
                }
                Ok(())
            } else {
                let now = now_millis()?;
                let mut guard = lock_sessions(sessions);
                let session = guard
                    .sessions
                    .get_mut(terminal_id)
                    .ok_or_else(|| "terminal session not found".to_string())?;
                let started_at = session.post_ready_session_id_started_at.get_or_insert(now);
                let elapsed = now.saturating_sub(*started_at);
                if elapsed < EXTRACT_SESSION_ID_TIMEOUT_MS {
                    Err((PostReadyAction::ExtractSessionId { keyword }, String::new()))
                } else if session.post_ready_session_id_restart_count < EXTRACT_SESSION_ID_MAX_RESTARTS
                {
                    session.post_ready_session_id_restart_count =
                        session.post_ready_session_id_restart_count.saturating_add(1);
                    session.post_ready_session_id_started_at = None;
                    session.post_ready_restart_pending = true;
                    session.post_ready_state = PostReadyState::Idle;
                    session.post_ready_queue.clear();
                    session.flow_paused = true;
                    session.shell_ready = false;
                    log::warn!(
                        "terminal post_ready session id timeout restart scheduled terminal_id={} attempt={}",
                        terminal_id,
                        session.post_ready_session_id_restart_count
                    );
                    Ok(())
                } else {
                    session.post_ready_session_id_started_at = None;
                    session.post_ready_restart_pending = false;
                    session.post_ready_state = PostReadyState::Done;
                    session.post_ready_queue.clear();
                    session.flow_paused = false;
                    log::warn!(
                        "terminal post_ready session id timeout skip post_ready terminal_id={} attempts={}",
                        terminal_id,
                        session.post_ready_session_id_restart_count
                    );
                    finish_post_ready_now = true;
                    Ok(())
                }
            }
        }
        PostReadyAction::WaitForPattern {
            pattern,
            attempts,
            started_at,
        } => {
            let now = now_millis()?;
            let started_at = started_at.unwrap_or(now);
            const WAIT_PATTERN_MAX_ATTEMPTS: u32 = 30;
            const WAIT_PATTERN_TIMEOUT_MS: u64 = 30_000;
            if attempts >= WAIT_PATTERN_MAX_ATTEMPTS
                || now.saturating_sub(started_at) >= WAIT_PATTERN_TIMEOUT_MS
            {
                log::warn!(
          "terminal wait pattern timeout terminal_id={} pattern={} attempts={} elapsed_ms={}",
          terminal_id,
          pattern,
          attempts,
          now.saturating_sub(started_at)
        );
                // 超时后跳过，继续后续步骤
                return Ok(());
            }
            let has_pattern = {
                let guard = lock_sessions(sessions);
                let session = guard
                    .sessions
                    .get(terminal_id)
                    .ok_or_else(|| "terminal session not found".to_string())?;
                let lines = session.snapshot.snapshot_lines();
                lines.iter().any(|line| line.contains(pattern.as_str()))
            };
            if has_pattern {
                log::info!(
                    "terminal wait pattern matched terminal_id={} pattern={}",
                    terminal_id,
                    pattern
                );
                Ok(())
            } else {
                // 还没匹配到，重新入队继续等待
                Err((
                    PostReadyAction::WaitForPattern {
                        pattern,
                        attempts: attempts.saturating_add(1),
                        started_at: Some(started_at),
                    },
                    String::new(),
                ))
            }
        }
    };
    if finish_post_ready_now {
        finish_post_ready(sessions, event_port, terminal_id)?;
        return Ok(());
    }
    if let Err((action, err)) = result {
        let mut guard = lock_sessions(sessions);
        if let Some(session) = guard.sessions.get_mut(terminal_id) {
            session.post_ready_queue.push_front(PostReadyQueueItem {
                action,
                require_stable,
            });
        }
        if !err.is_empty() {
            return Err(err);
        }
    } else {
        let should_finish = {
            let guard = lock_sessions(sessions);
            guard
                .sessions
                .get(terminal_id)
                .map(|session| {
                    session.post_ready_state == PostReadyState::Starting
                        && session.post_ready_queue.is_empty()
                })
                .unwrap_or(false)
        };
        if should_finish {
            finish_post_ready(sessions, event_port, terminal_id)?;
        }
    }
    Ok(())
}

// post_ready 结束即启动链路稳定，解锁 Connecting 门禁并广播 Online。
fn finish_post_ready(
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &dyn TerminalEventPort,
    terminal_id: &str,
) -> Result<(), String> {
    let (member_id, should_unlock) = {
        let mut guard = lock_sessions(sessions);
        let session = guard
            .sessions
            .get_mut(terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        session.post_ready_state = PostReadyState::Done;
        (session.member_id.clone(), session.status_locked)
    };
    if !should_unlock {
        return Ok(());
    }
    let Some(member_id) = member_id.as_deref() else {
        log::warn!(
            "terminal post_ready unlock skipped missing member_id terminal_id={}",
            terminal_id
        );
        return Ok(());
    };
    match unlock_session_status_by_member(sessions, member_id) {
        Ok(payload) => {
            if let Some(payload) = payload {
                let _ = event_port.emit_status(payload);
            }
        }
        Err(err) => {
            log::warn!(
                "terminal post_ready unlock failed member_id={} terminal_id={} err={}",
                member_id,
                terminal_id,
                err
            );
        }
    }
    Ok(())
}

fn queue_post_ready_steps(
    sessions: &Arc<Mutex<SessionRegistry>>,
    terminal_id: &str,
    steps: &[TerminalPostReadyStep],
) -> Result<(), String> {
    // 启动后注入使用队列分步发送，避免一次性写入导致状态误判。
    let mut guard = lock_sessions(sessions);
    let session = guard
        .sessions
        .get_mut(terminal_id)
        .ok_or_else(|| "terminal session not found".to_string())?;
    session.post_ready_queue.clear();
    for step in steps {
        match step {
            TerminalPostReadyStep::Input {
                input,
                require_stable,
            } => {
                if *input == "\r" {
                    session.post_ready_queue.push_back(PostReadyQueueItem {
                        action: PostReadyAction::Delay {
                            ms: ENTER_INPUT_DELAY_MS,
                            started_at: None,
                        },
                        require_stable: false,
                    });
                }
                session.post_ready_queue.push_back(PostReadyQueueItem {
                    action: PostReadyAction::Input((*input).to_string()),
                    require_stable: *require_stable,
                });
            }
            TerminalPostReadyStep::ExtractSessionId {
                keyword,
                require_stable,
            } => {
                session.post_ready_queue.push_back(PostReadyQueueItem {
                    action: PostReadyAction::ExtractSessionId {
                        keyword: (*keyword).to_string(),
                    },
                    require_stable: *require_stable,
                });
            }
            TerminalPostReadyStep::WaitForPattern {
                pattern,
                require_stable,
            } => {
                session.post_ready_queue.push_back(PostReadyQueueItem {
                    action: PostReadyAction::WaitForPattern {
                        pattern: (*pattern).to_string(),
                        attempts: 0,
                        started_at: None,
                    },
                    require_stable: *require_stable,
                });
            }
            TerminalPostReadyStep::Introduction {
                prompt_type,
                require_stable,
            } => {
                session.post_ready_queue.push_back(PostReadyQueueItem {
                    action: PostReadyAction::Introduction {
                        prompt_type: (*prompt_type).to_string(),
                    },
                    require_stable: *require_stable,
                });
            }
        }
    }
    session.post_ready_state = PostReadyState::Starting;
    Ok(())
}

