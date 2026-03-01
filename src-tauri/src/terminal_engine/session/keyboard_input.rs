// Stability check logic moved to `stability.rs`

use std::sync::{Arc, Mutex};

use crate::now_millis;
use crate::ports::terminal_event::TerminalEventPort;

use super::{
    flush_input_buffer, handle_buffered_write, lock_sessions, mark_session_working_on_input,
    terminal_trace_detail, SessionRegistry,
};

/// 模拟回车前的输入间隔，避免 CLI 丢提交。
pub(super) const ENTER_INPUT_DELAY_MS: u64 = 100;

/// 发送键盘输入到终端会话。
/// 输入：终端 id、输入字符串与原因标记。
/// 输出：成功或错误。
pub(super) fn send_post_ready_input(
    sessions: &Arc<Mutex<SessionRegistry>>,
    event_port: &dyn TerminalEventPort,
    terminal_id: &str,
    input: &str,
    reason: &str,
) -> Result<(), String> {
    let now = now_millis()?;
    {
        let guard = lock_sessions(sessions);
        let session = guard
            .sessions
            .get(terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        if !session.active || session.handle.is_none() {
            return Err("terminal session is not active".to_string());
        }
        if session.flow_paused {
            return Ok(());
        }
    }
    mark_session_working_on_input(sessions, event_port, terminal_id, input, None, None, None);
    let (should_write, buffered, writer, shell_ready) = {
        let mut guard = lock_sessions(sessions);
        let session = guard
            .sessions
            .get_mut(terminal_id)
            .ok_or_else(|| "terminal session not found".to_string())?;
        if !session.active || session.handle.is_none() {
            return Err("terminal session is not active".to_string());
        }
        let handle = session
            .handle
            .as_ref()
            .ok_or_else(|| "terminal session handle missing".to_string())?;
        let writer = Arc::clone(&handle.writer);
        let (should_write, buffered) = handle_buffered_write(session, input.to_string(), now);
        (should_write, buffered, writer, session.shell_ready)
    };
    if terminal_trace_detail() {
        let buffered_bytes: usize = buffered.iter().map(|entry| entry.len()).sum();
        log::info!(
            "terminal post_ready input terminal_id={} reason={} data_len={} buffered_bytes={} write_now={} shell_ready={}",
            terminal_id,
            reason,
            input.len(),
            buffered_bytes,
            should_write,
            shell_ready
        );
    }
    if !should_write {
        return Ok(());
    }
    flush_input_buffer(&writer, buffered)?;
    Ok(())
}

/// 从快照行中解析远端 session_id。
/// 规则：在关键字之后跳过非字母数字字符，遇到字母数字后开始收集，
/// 允许连字符 `-`，遇到空格/换行即停止。
pub(super) fn parse_session_id_from_lines(lines: &[String], keyword: &str) -> Option<String> {
    for line in lines {
        if let Some(session_id) = parse_session_id_from_line(line, keyword) {
            return Some(session_id);
        }
    }
    None
}

pub(super) fn parse_session_id_from_line(line: &str, keyword: &str) -> Option<String> {
    let mut search_offset = 0usize;
    while let Some(pos) = find_keyword_case_insensitive(line, keyword, search_offset) {
        let start = pos.saturating_add(keyword.len());
        if let Some(session_id) = parse_session_id_after_keyword(&line[start..]) {
            return Some(session_id);
        }
        search_offset = start.saturating_add(keyword.len());
    }
    None
}

fn parse_session_id_after_keyword(value: &str) -> Option<String> {
    let mut started = false;
    let mut session_id = String::new();
    for ch in value.chars() {
        if !started {
            if ch.is_ascii_alphanumeric() {
                started = true;
                session_id.push(ch);
            }
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '-' {
            session_id.push(ch);
            continue;
        }
        if ch.is_whitespace() {
            break;
        }
        break;
    }
    if started && !session_id.is_empty() {
        Some(session_id)
    } else {
        None
    }
}

fn find_keyword_case_insensitive(line: &str, keyword: &str, offset: usize) -> Option<usize> {
    if keyword.is_empty() {
        return None;
    }
    let line_bytes = line.as_bytes();
    let keyword_bytes = keyword.as_bytes();
    if offset >= line_bytes.len() || keyword_bytes.len() > line_bytes.len() {
        return None;
    }
    let end = line_bytes.len().saturating_sub(keyword_bytes.len());
    for index in offset..=end {
        let mut matched = true;
        for (left, right) in line_bytes[index..index + keyword_bytes.len()]
            .iter()
            .zip(keyword_bytes.iter())
        {
            if left.to_ascii_lowercase() != right.to_ascii_lowercase() {
                matched = false;
                break;
            }
        }
        if matched {
            return Some(index);
        }
    }
    None
}
