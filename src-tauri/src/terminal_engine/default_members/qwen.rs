//! Qwen Code 默认成员配置。

use super::registry::{TerminalDefaultMemberConfig, TerminalPostReadyPlan};

pub(crate) const QWEN_DEFAULT_MEMBER: TerminalDefaultMemberConfig = TerminalDefaultMemberConfig {
    id: "qwen-code",
    terminal_type: "qwen",
    default_command: "qwen",
    unlimited_access_flag: Some("--yolo"),
    resume_command_template: None,
    post_ready_plan: TerminalPostReadyPlan {
        post_ready_steps: &[
            super::ai_shared::AI_ONBOARDING_STEP,
        ],
    },
};
