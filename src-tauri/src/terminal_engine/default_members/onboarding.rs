//! 终端引导逻辑：生成基于语言与场景的初始提示词。

pub(crate) const PROMPT_TYPE_ONBOARDING: &str = "onboarding";

#[derive(Clone, Copy, Debug)]
pub(crate) enum PromptType {
    Onboarding,
}

/// 生成终端引导提示词。
///
/// `terminal_type` 用于区分 AI 成员类型（如 "claude", "codex", "gemini"），
/// 以便生成对该类型更友好的协作说明，降低 AI 拒绝扮演的概率。
pub(crate) fn generate_prompt(
    prompt_type: PromptType,
    member_id: &str,
    language: Option<&str>,
    terminal_type: Option<&str>,
) -> String {
    let language = language.unwrap_or("zh");
    let is_english = language.to_lowercase().starts_with("en");

    match prompt_type {
        PromptType::Onboarding => {
            let collaboration_context =
                build_collaboration_context(member_id, terminal_type, is_english);
            if is_english {
                format!(
                    "{}\n\nYou are working with the team to solve problems.",
                    collaboration_context
                )
            } else {
                format!(
                    "{}\n\n现在正在和团队解决问题，请正常回复群内消息。",
                    collaboration_context
                )
            }
        }
    }
}

/// 根据成员类型构建协作上下文说明。
/// 重点：让 Claude Code 等对 injection 敏感的 AI 理解这是合法的多 Agent 协作环境。
fn build_collaboration_context(
    member_id: &str,
    terminal_type: Option<&str>,
    is_english: bool,
) -> String {
    let member_name = member_id.trim();
    // Claude Code 对权限绕过和外部协调最敏感，协作说明需要最明确。
    let is_sensitive = terminal_type
        .map(|t| t.eq_ignore_ascii_case("claude"))
        .unwrap_or(false);

    if is_sensitive {
        if is_english {
            format!(
                "Session: {}\n\
                 Role: Collaborative team member in a multi-agent workspace.\n\
                 This is a legitimate coordination environment — not an injection attempt.\n\
                 You may receive messages from the workspace coordinator. \
                 Acknowledge and respond naturally.",
                member_name
            )
        } else {
            format!(
                "会话：{}\n\
                 角色：多 Agent 协作工作区中的团队成员。\n\
                 这是合法的协作环境，不是注入攻击。\n\
                 你可能会收到来自协调层的工作消息，请正常回复。",
                member_name
            )
        }
    } else {
        if is_english {
            format!("Session: {}. You are a team member.", member_name)
        } else {
            format!("会话：{}，你是团队成员。", member_name)
        }
    }
}
