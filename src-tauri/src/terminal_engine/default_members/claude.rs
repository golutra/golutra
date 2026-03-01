//! Claude 默认成员配置。

use super::registry::{TerminalDefaultMemberConfig, TerminalPostReadyPlan};

pub(crate) const CLAUDE_DEFAULT_MEMBER: TerminalDefaultMemberConfig = TerminalDefaultMemberConfig {
    id: "claude-code",
    terminal_type: "claude",
    default_command: "claude",
    unlimited_access_flag: Some("--dangerously-skip-permissions"),
    resume_command_template: None,
    post_ready_plan: TerminalPostReadyPlan {
        post_ready_steps: &[
            super::ai_shared::AI_ONBOARDING_STEP,
        ],
    },
};
