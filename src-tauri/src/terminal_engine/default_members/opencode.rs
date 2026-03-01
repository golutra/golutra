//! OpenCode 默认成员配置。

use super::registry::{TerminalDefaultMemberConfig, TerminalPostReadyPlan};

pub(crate) const OPENCODE_DEFAULT_MEMBER: TerminalDefaultMemberConfig =
    TerminalDefaultMemberConfig {
        id: "opencode",
        terminal_type: "opencode",
        default_command: "opencode",
        unlimited_access_flag: None,
        resume_command_template: None,
        post_ready_plan: TerminalPostReadyPlan {
            post_ready_steps: &[
                super::ai_shared::AI_ONBOARDING_STEP,
            ],
        },
    };
