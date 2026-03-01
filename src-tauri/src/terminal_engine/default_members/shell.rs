//! Shell 默认成员配置。

use super::registry::{TerminalDefaultMemberConfig, TerminalPostReadyPlan};

pub(crate) const SHELL_DEFAULT_MEMBER: TerminalDefaultMemberConfig = TerminalDefaultMemberConfig {
    id: "terminal",
    terminal_type: "shell",
    default_command: "",
    unlimited_access_flag: None,
    resume_command_template: None,
    post_ready_plan: TerminalPostReadyPlan::EMPTY,
};
