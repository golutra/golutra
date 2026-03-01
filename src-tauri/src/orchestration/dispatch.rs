use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, State, WebviewWindow};

use crate::contracts::chat_dispatch::{ChatDispatchMentions, ChatDispatchPayload};
use crate::message_service::chat_db::{chat_get_conversation_member_ids, ChatDbManager};
use crate::message_service::project_data;
use crate::platform::{diagnostics_log_backend_event, DiagnosticsState};
use crate::runtime::StorageManager;
use crate::terminal_engine::session::{
    terminal_create, terminal_dispatch_chat, TerminalDispatchContext, TerminalManager,
};
use super::chat_dispatch_batcher::ChatDispatchBatcher;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationMemberConfig {
    pub id: String,
    pub name: String,
    pub terminal_type: String,
    pub terminal_command: Option<String>,
    pub terminal_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationPayload {
    pub text: String,
    pub context: TerminalDispatchContext,
    pub targets: Vec<OrchestrationMemberConfig>,
    pub workspace_id: Option<String>,
    pub cwd: Option<String>,
}

#[derive(Clone, Debug)]
struct MemberTerminalConfig {
    id: String,
    name: String,
    terminal_type: Option<String>,
    terminal_command: Option<String>,
    terminal_path: Option<String>,
}

/// 编排并执行终端消息派发。
/// 接收一组目标成员配置，自动确保会话存在，并并行（或串行）派发消息。
pub fn orchestrate_dispatch_impl(
    app: &AppHandle,
    window: &WebviewWindow,
    state: &State<'_, TerminalManager>,
    payload: OrchestrationPayload,
) -> Result<(), String> {
    for target in payload.targets {
        let terminal_id = ensure_backend_session(
            app,
            window,
            state,
            &target,
            payload.workspace_id.as_deref(),
            payload.cwd.as_deref(),
        )?;
        terminal_dispatch_chat(
            app.clone(),
            state.clone(),
            terminal_id,
            payload.text.clone(),
            payload.context.clone(),
        )?;
    }
    Ok(())
}

/// 发送聊天消息后，按 mention 规则编排派发到终端。
pub fn orchestrate_chat_dispatch(
    app: &AppHandle,
    window: &WebviewWindow,
    terminal_state: State<'_, TerminalManager>,
    chat_state: State<'_, ChatDbManager>,
    storage: &StorageManager,
    payload: ChatDispatchPayload,
) -> Result<(), String> {
    let member_ids = chat_get_conversation_member_ids(
        chat_state,
        payload.workspace_id.clone(),
        payload.conversation_id.clone(),
    )?;
    let member_set: HashSet<String> = member_ids.iter().cloned().collect();
    if member_set.is_empty() {
        log_chat_dispatch_skip(
            app,
            &payload,
            "no_conversation_members",
            json!({
                "memberCount": member_ids.len()
            }),
        );
        return Ok(());
    }

    let project_data = project_data::read_project_data(
        storage,
        &payload.workspace_path,
        &payload.workspace_id,
    )?;
    let member_configs = match project_data.data {
        Some(value) => collect_member_configs(&value),
        None => HashMap::new(),
    };
    let member_config_count = member_configs.len();

    let default_mentions = ChatDispatchMentions {
        mention_ids: Vec::new(),
        mention_all: false,
    };
    let mentions = payload.mentions.as_ref().unwrap_or(&default_mentions);
    let mut targets = resolve_targets(
        payload.conversation_type.as_str(),
        &member_ids,
        &member_set,
        payload.sender_id.as_str(),
        mentions,
    );
    if targets.is_empty() {
        log_chat_dispatch_skip(
            app,
            &payload,
            "no_targets",
            json!({
                "memberCount": member_ids.len()
            }),
        );
        return Ok(());
    }
    let targets_before_filter = targets.len();
    targets.retain(|id| member_configs.contains_key(id));
    if targets.is_empty() {
        log_chat_dispatch_skip(
            app,
            &payload,
            "targets_missing_config",
            json!({
                "targetsBeforeFilter": targets_before_filter,
                "memberConfigCount": member_config_count
            }),
        );
        return Ok(());
    }

    let context = TerminalDispatchContext {
        conversation_id: payload.conversation_id.clone(),
        conversation_type: payload.conversation_type.clone(),
        sender_id: payload.sender_id.clone(),
        sender_name: payload.sender_name.clone(),
        message_id: payload.message_id.clone(),
        client_trace_id: payload.client_trace_id.clone(),
        client_timestamp: payload.timestamp,
    };

    let batcher = app.state::<Arc<ChatDispatchBatcher>>();
    let mut dispatched_count = 0usize;
    let mut skipped_missing_terminal_config = 0usize;
    for target_id in targets {
        let Some(config) = member_configs.get(&target_id) else {
            continue;
        };
        if !has_terminal_config(config) {
            skipped_missing_terminal_config = skipped_missing_terminal_config.saturating_add(1);
            continue;
        }
        let terminal_id = ensure_backend_member_session(
            app,
            window,
            &terminal_state,
            config,
            &payload.workspace_id,
            &payload.workspace_path,
        )?;
        batcher.enqueue_for_terminal(app, terminal_id, payload.text.clone(), context.clone())?;
        dispatched_count = dispatched_count.saturating_add(1);
    }
    if dispatched_count == 0 {
        log_chat_dispatch_skip(
            app,
            &payload,
            "targets_no_terminal_config",
            json!({
                "targetsBeforeFilter": targets_before_filter,
                "memberConfigCount": member_config_count,
                "skippedMissingTerminalConfig": skipped_missing_terminal_config
            }),
        );
    }
    Ok(())
}

fn resolve_targets(
    conversation_type: &str,
    member_ids: &[String],
    member_set: &HashSet<String>,
    sender_id: &str,
    mentions: &ChatDispatchMentions,
) -> Vec<String> {
    let mut targets = Vec::new();
    if conversation_type == "dm" {
        if let Some(target_id) = member_ids.iter().find(|id| id.as_str() != sender_id) {
            if member_set.contains(target_id) {
                targets.push(target_id.clone());
            }
        }
        return targets;
    }

    if mentions.mention_all {
        targets.extend(member_ids.iter().cloned());
    } else {
        targets.extend(mentions.mention_ids.iter().cloned());
    }
    let mut unique = HashSet::new();
    targets
        .into_iter()
        .filter(|id| id.as_str() != sender_id)
        .filter(|id| member_set.contains(id))
        .filter(|id| unique.insert(id.clone()))
        .collect()
}

fn collect_member_configs(payload: &Value) -> HashMap<String, MemberTerminalConfig> {
    let mut map = HashMap::new();
    let Some(members) = payload.get("members").and_then(|value| value.as_array()) else {
        return map;
    };
    for member in members {
        let Some(obj) = member.as_object() else {
            continue;
        };
        let id = obj
            .get("id")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim();
        if id.is_empty() {
            continue;
        }
        let name = obj
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim();
        let terminal_type = normalize_string(obj.get("terminalType"));
        let terminal_command = normalize_string(obj.get("terminalCommand"));
        let terminal_path = normalize_string(obj.get("terminalPath"));
        map.insert(
            id.to_string(),
            MemberTerminalConfig {
                id: id.to_string(),
                name: name.to_string(),
                terminal_type,
                terminal_command,
                terminal_path,
            },
        );
    }
    map
}

fn normalize_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn has_terminal_config(config: &MemberTerminalConfig) -> bool {
    config
        .terminal_type
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || config
            .terminal_command
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        || config
            .terminal_path
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
}

fn ensure_backend_session(
    app: &AppHandle,
    window: &WebviewWindow,
    state: &State<'_, TerminalManager>,
    config: &OrchestrationMemberConfig,
    workspace_id: Option<&str>,
    cwd: Option<&str>,
) -> Result<String, String> {
    if let Some(session_id) =
        state.find_session_id_by_member(config.id.as_str(), workspace_id)
    {
        return Ok(session_id);
    }

    terminal_create(
        app.clone(),
        window.clone(),
        state.clone(),
        None,
        None,
        cwd.map(|value| value.to_string()),
        Some(config.id.clone()),
        workspace_id.map(|value| value.to_string()),
        Some(true),
        None,
        Some(config.terminal_type.clone()),
        config.terminal_command.clone(),
        config.terminal_path.clone(),
        None,
        Some("none".to_string()),
        Some(config.name.clone()),
        None,
        None,
        None,
        None,
    )
}

fn ensure_backend_member_session(
    app: &AppHandle,
    window: &WebviewWindow,
    state: &State<'_, TerminalManager>,
    config: &MemberTerminalConfig,
    workspace_id: &str,
    workspace_path: &str,
) -> Result<String, String> {
    if let Some(session_id) =
        state.find_session_id_by_member(config.id.as_str(), Some(workspace_id))
    {
        return Ok(session_id);
    }

    terminal_create(
        app.clone(),
        window.clone(),
        state.clone(),
        None,
        None,
        Some(workspace_path.to_string()),
        Some(config.id.clone()),
        Some(workspace_id.to_string()),
        Some(true),
        None,
        config.terminal_type.clone(),
        config.terminal_command.clone(),
        config.terminal_path.clone(),
        None,
        Some("none".to_string()),
        Some(config.name.clone()),
        None,
        None,
        None,
        None,
    )
}

fn log_chat_dispatch_skip(
    app: &AppHandle,
    payload: &ChatDispatchPayload,
    reason: &str,
    detail: Value,
) {
    let mentions = payload.mentions.as_ref();
    let mention_all = mentions.map(|value| value.mention_all).unwrap_or(false);
    let mention_ids_len = mentions.map(|value| value.mention_ids.len()).unwrap_or(0);
    diagnostics_log_backend_event(
        &app.state::<DiagnosticsState>(),
        Some(payload.sender_id.clone()),
        None,
        Some(payload.conversation_id.clone()),
        None,
        Some(payload.workspace_id.clone()),
        "chat_dispatch_skip",
        json!({
            "reason": reason,
            "conversationType": payload.conversation_type,
            "mentionAll": mention_all,
            "mentionIdsLen": mention_ids_len,
            "detail": detail
        }),
    );
}
