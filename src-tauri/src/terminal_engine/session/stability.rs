//! 终端输出稳定性检测模块。
//! 包含自适应速率检测算法，用于判断终端是否处于静止状态。

use super::state::{OutputRateSample, TerminalSession};

/// 采样间隔 (ms)
const RATE_SAMPLE_INTERVAL_MS: u64 = 100;
/// 采样缓冲区大小
const RATE_SAMPLE_COUNT: usize = 10;
/// 速率阈值 (bytes/ms)：低于此值认为输出减缓
const RATE_THRESHOLD_BYTES_PER_MS: f64 = 1.0;
/// 连续低速样本数：达到此数量认为稳定
const RATE_STABLE_SAMPLES: usize = 3;
/// 样本不足时的轻量兜底等待（ms），避免静默启动卡死。
const FALLBACK_STABLE_MS: u64 = 400;

/// 记录输出速率采样点。
/// 在每次输出时调用，按间隔采样以控制缓冲区大小。
pub(crate) fn record_output_sample(session: &mut TerminalSession, now: u64) {
    let last_sample_at = session
        .output_rate_samples
        .back()
        .map(|s| s.timestamp)
        .unwrap_or(0);
    if now.saturating_sub(last_sample_at) >= RATE_SAMPLE_INTERVAL_MS {
        session.output_rate_samples.push_back(OutputRateSample {
            timestamp: now,
            bytes_total: session.output_bytes_total,
        });
        while session.output_rate_samples.len() > RATE_SAMPLE_COUNT {
            session.output_rate_samples.pop_front();
        }
    }
}

/// 检测输出速率是否稳定（自适应）。
/// 当最近 N 个采样间隔的速率都低于阈值时，认为稳定。
fn is_output_rate_stable(session: &TerminalSession) -> bool {
    let samples = &session.output_rate_samples;
    if samples.len() < RATE_STABLE_SAMPLES + 1 {
        return false; // 样本不足
    }
    // 检查最近 N 个间隔的速率
    let recent: Vec<_> = samples.iter().rev().take(RATE_STABLE_SAMPLES + 1).collect();
    for window in recent.windows(2) {
        let (newer, older) = (window[0], window[1]);
        let duration_ms = newer.timestamp.saturating_sub(older.timestamp);
        let bytes_delta = newer.bytes_total.saturating_sub(older.bytes_total);
        if duration_ms == 0 {
            continue;
        }
        let rate = bytes_delta as f64 / duration_ms as f64;
        if rate > RATE_THRESHOLD_BYTES_PER_MS {
            return false; // 速率仍然较高
        }
    }
    true
}

/// 判断输出是否稳定（支持两种模式）。
/// 优先使用速率检测，fallback 到固定时间检测。
pub(crate) fn is_output_stable(session: &TerminalSession, force: bool, now: u64) -> bool {
    // 强制检查模式：Trigger 触发时，无视速率和 Shell 状态，强制通过。
    if force {
        return true;
    }
    // Shell 未就绪前，必须等待超时或显式就绪
    if !session.shell_ready
        && now.saturating_sub(session.created_at) < super::SHELL_READY_TIMEOUT_MS
    {
        return false;
    }
    // 仅使用速率检测；样本不足时用短兜底，避免静默启动卡死。
    if is_output_rate_stable(session) {
        return true;
    }
    if session.output_rate_samples.len() < RATE_STABLE_SAMPLES + 1 {
        let last_applied_at = session
            .last_applied_at
            .or(session.last_output_at)
            .unwrap_or(session.created_at);
        return now.saturating_sub(last_applied_at) >= FALLBACK_STABLE_MS;
    }
    false
}
