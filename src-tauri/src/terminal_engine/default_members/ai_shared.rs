//! AI 成员共享配置。

use super::onboarding::PROMPT_TYPE_ONBOARDING;
use super::registry::TerminalPostReadyStep;

pub(crate) const AI_ONBOARDING_STEP: TerminalPostReadyStep = TerminalPostReadyStep::Introduction {
    prompt_type: PROMPT_TYPE_ONBOARDING,
    require_stable: true,
};

pub(crate) const ENTER_STEP: TerminalPostReadyStep = TerminalPostReadyStep::Input {
    input: "\r",
    require_stable: true,
};
