//! 终端引导逻辑：生成基于语言与场景的初始提示词。

pub(crate) const PROMPT_TYPE_ONBOARDING: &str = "onboarding";

#[derive(Clone, Copy, Debug)]
pub(crate) enum PromptType {
    Onboarding,
}

pub(crate) fn generate_prompt(
    prompt_type: PromptType,
    terminal_id: &str,
    language: Option<&str>,
) -> String {
    let language = language.unwrap_or("zh");
    let is_english = language.to_lowercase().starts_with("en");

    match prompt_type {
        PromptType::Onboarding => {
            if is_english {
                format!(
                    "{}, this is your name. You are working with the team to solve problems.",
                    terminal_id
                )
            } else {
                format!("{}，这是你的名字，现在正在和团队解决问题", terminal_id)
            }
        }
    }
}
