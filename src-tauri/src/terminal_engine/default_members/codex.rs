//! Codex 默认成员配置。

use super::registry::{TerminalDefaultMemberConfig, TerminalPostReadyPlan, TerminalPostReadyStep};

pub(crate) const CODEX_DEFAULT_MEMBER: TerminalDefaultMemberConfig = TerminalDefaultMemberConfig {
    id: "codex",
    terminal_type: "codex",
    default_command: "codex",
    unlimited_access_flag: Some("--dangerously-bypass-approvals-and-sandbox"),
    resume_command_template: Some("resume {session_id}"),
    post_ready_plan: TerminalPostReadyPlan {
        post_ready_steps: &[
            TerminalPostReadyStep::Input {
                input: "/status",
                require_stable: true,
            },
            super::ai_shared::ENTER_STEP,
            // 检测 "model" 特征判断 Codex 已完全加载
            TerminalPostReadyStep::WaitForPattern {
                pattern: "model",
                require_stable: false,
            },
            TerminalPostReadyStep::ExtractSessionId {
                keyword: "session:",
                require_stable: true,
            },
            super::ai_shared::AI_ONBOARDING_STEP,
            super::ai_shared::ENTER_STEP,
        ],
    },
};
