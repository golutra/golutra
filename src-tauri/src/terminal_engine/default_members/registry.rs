//! 终端默认成员注册与命令处理。

#[derive(Clone, Copy, Debug)]
pub(crate) enum TerminalPostReadyStep {
    Input {
        input: &'static str,
        require_stable: bool,
    },
    ExtractSessionId {
        keyword: &'static str,
        require_stable: bool,
    },
    /// 等待快照中出现指定特征字符串。
    WaitForPattern {
        pattern: &'static str,
        require_stable: bool,
    },
    /// 插入基于场景的动态引导提示词。
    Introduction {
        prompt_type: &'static str,
        require_stable: bool,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TerminalPostReadyPlan {
    pub(crate) post_ready_steps: &'static [TerminalPostReadyStep],
}

impl TerminalPostReadyPlan {
    pub(crate) const EMPTY: TerminalPostReadyPlan = TerminalPostReadyPlan {
        post_ready_steps: &[],
    };
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TerminalDefaultMemberConfig {
    pub(crate) id: &'static str,
    pub(crate) terminal_type: &'static str,
    pub(crate) default_command: &'static str,
    pub(crate) unlimited_access_flag: Option<&'static str>,
    /// 会话恢复命令模板，{session_id} 将被替换为实际 ID。
    pub(crate) resume_command_template: Option<&'static str>,
    pub(crate) post_ready_plan: TerminalPostReadyPlan,
}

use super::{
    claude::CLAUDE_DEFAULT_MEMBER, codex::CODEX_DEFAULT_MEMBER, gemini::GEMINI_DEFAULT_MEMBER,
    opencode::OPENCODE_DEFAULT_MEMBER, qwen::QWEN_DEFAULT_MEMBER, shell::SHELL_DEFAULT_MEMBER,
};

pub(crate) const DEFAULT_TERMINAL_MEMBERS: [TerminalDefaultMemberConfig; 6] = [
    GEMINI_DEFAULT_MEMBER,
    CODEX_DEFAULT_MEMBER,
    CLAUDE_DEFAULT_MEMBER,
    OPENCODE_DEFAULT_MEMBER,
    QWEN_DEFAULT_MEMBER,
    SHELL_DEFAULT_MEMBER,
];

pub(crate) fn resolve_default_member(
    terminal_type: &str,
) -> Option<&'static TerminalDefaultMemberConfig> {
    let normalized = terminal_type.trim().to_lowercase();
    DEFAULT_TERMINAL_MEMBERS
        .iter()
        .find(|member| member.terminal_type == normalized)
}

fn command_contains_flag(command: &str, flag: &str) -> bool {
    command.split_whitespace().any(|part| part == flag)
}

fn normalize_command(value: Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn should_apply_unlimited_flag(command: Option<&str>, default_command: &str) -> bool {
    if default_command.trim().is_empty() {
        return false;
    }
    match command {
        None => true,
        Some(value) => value.trim() == default_command,
    }
}

pub(crate) fn apply_unlimited_access_command(
    terminal_type: &str,
    command: Option<String>,
    unlimited_access: bool,
) -> Option<String> {
    let command = normalize_command(command);
    if !unlimited_access {
        return command;
    }
    let member = match resolve_default_member(terminal_type) {
        Some(member) => member,
        None => return command,
    };
    let flag = match member.unlimited_access_flag {
        Some(flag) => flag,
        None => return command,
    };
    if !should_apply_unlimited_flag(command.as_deref(), member.default_command) {
        return command;
    }
    let base = command.unwrap_or_else(|| member.default_command.to_string());
    if command_contains_flag(&base, flag) {
        return Some(base);
    }
    let mut next = base;
    if !next.is_empty() {
        next.push(' ');
    }
    next.push_str(flag);
    Some(next)
}

pub(crate) fn resolve_default_command_for_invite(
    terminal_type: &str,
    default_command: Option<String>,
    unlimited_access: bool,
) -> Option<String> {
    let base = normalize_command(default_command)
        .or_else(|| {
            resolve_default_member(terminal_type).map(|member| member.default_command.to_string())
        })
        .filter(|value| !value.trim().is_empty());
    apply_unlimited_access_command(terminal_type, base, unlimited_access)
}

/// 根据 session_id 构建恢复命令。
/// 如果 session_id 存在且配置了 resume_command_template，则返回组装后的命令；
/// 格式：{default_command} {resume_template} {unlimited_access_flag}
/// 否则返回原始命令。
pub(crate) fn apply_resume_command(
    terminal_type: &str,
    command: Option<String>,
    session_id: Option<&str>,
) -> Option<String> {
    let session_id = match session_id {
        Some(id) if !id.trim().is_empty() => id.trim(),
        _ => return command,
    };
    let member = match resolve_default_member(terminal_type) {
        Some(member) => member,
        None => return command,
    };
    let template = match member.resume_command_template {
        Some(template) => template,
        None => return command,
    };
    // 替换 {session_id} 占位符
    let resume_args = template.replace("{session_id}", session_id);
    let mut base = command
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| member.default_command.to_string());
    if base
        .split_whitespace()
        .any(|part| part.eq_ignore_ascii_case("resume"))
        || base.contains(session_id)
    {
        return Some(base);
    }
    if !base.ends_with(' ') {
        base.push(' ');
    }
    base.push_str(resume_args.as_str());
    Some(base)
}
