//! 终端默认成员配置与能力注册入口。

pub(crate) mod ai_shared;
pub(crate) mod claude;
pub(crate) mod codex;
pub(crate) mod gemini;
pub(crate) mod onboarding;
pub(crate) mod opencode;
pub(crate) mod qwen;
pub(crate) mod registry;
pub(crate) mod shell;

pub(crate) use registry::{
    apply_resume_command, apply_unlimited_access_command, resolve_default_command_for_invite,
    resolve_default_member,
};
