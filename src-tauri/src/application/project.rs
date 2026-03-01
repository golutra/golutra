//! 项目成员用例：统一成员创建与命名规则，供 UI/CLI 复用。

use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::message_service::project_members::{ProjectMemberPurgeResult, ProjectMemberStore};
use crate::runtime::{storage, StorageManager};

const WORKSPACE_REGISTRY_FILE: &str = "workspace-registry.json";
const WORKSPACE_REGISTRY_LOCK_FILE: &str = "workspace-registry.lock";
const DEFAULT_WORKSPACE_NAME: &str = "workspace";
const WORKSPACE_CONTEXT_MISSING_MESSAGE: &str = "请先打开工作区";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectInviteMembersRequest {
    pub(crate) role_type: String,
    pub(crate) command: Option<String>,
    pub(crate) terminal_type: Option<String>,
    pub(crate) instance_count: u32,
    pub(crate) unlimited_access: bool,
    pub(crate) sandboxed: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectPurgeTerminalMembersRequest {
    pub(crate) scope: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectInviteMembersResult {
    pub(crate) members: Vec<Value>,
    pub(crate) created_members: Vec<Value>,
    pub(crate) storage: String,
    pub(crate) warning: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectPurgeTerminalMembersResult {
    pub(crate) scope: String,
    pub(crate) total_removed: u32,
    pub(crate) workspace_count: u32,
    pub(crate) warnings: Vec<String>,
}

pub(crate) struct ProjectPurgeTerminalMembersOutcome {
    pub(crate) result: ProjectPurgeTerminalMembersResult,
    pub(crate) removed_workspaces: Vec<ProjectPurgeRemovedWorkspace>,
}

pub(crate) struct ProjectPurgeRemovedWorkspace {
    pub(crate) workspace_id: String,
    pub(crate) removed_member_ids: Vec<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkspaceRegistryEntry {
    #[serde(rename = "lastKnownPath")]
    last_known_path: String,
}

struct WorkspaceContext {
    workspace_path: String,
    workspace_name: String,
}

// 以工作区为粒度串行化成员创建，避免并发重名与写入竞态。
static WORKSPACE_MEMBER_LOCKS: OnceLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> = OnceLock::new();

pub(crate) fn project_invite_members(
    storage: &StorageManager,
    workspace_id: &str,
    request: ProjectInviteMembersRequest,
) -> Result<ProjectInviteMembersResult, String> {
    let workspace_id = workspace_id.trim();
    if workspace_id.is_empty() {
        return Err("workspace id is empty".to_string());
    }
    let context = resolve_workspace_context(storage, workspace_id)?;
    with_workspace_member_lock(workspace_id, || {
        let identity = resolve_terminal_identity(&request)?;
        let store = ProjectMemberStore::new(storage);
        let result = store.invite_members(
            workspace_id,
            &context.workspace_path,
            &context.workspace_name,
            &request.role_type,
            identity.label.as_str(),
            identity.terminal_type.as_deref(),
            identity.command.as_deref(),
            request.instance_count,
            request.unlimited_access,
            request.sandboxed,
        )?;

        Ok(ProjectInviteMembersResult {
            members: result.members,
            created_members: result.created_members,
            storage: result.storage,
            warning: result.warning,
        })
    })
}

pub(crate) fn project_purge_terminal_members(
    storage: &StorageManager,
    workspace_id: &str,
    request: ProjectPurgeTerminalMembersRequest,
) -> Result<ProjectPurgeTerminalMembersOutcome, String> {
    let scope = request.scope.trim().to_lowercase();
    if scope != "current" && scope != "all" {
        return Err("unsupported purge scope".to_string());
    }
    if scope == "current" {
        let context = resolve_workspace_context(storage, workspace_id)?;
        let store = ProjectMemberStore::new(storage);
        let result = with_workspace_member_lock(workspace_id, || {
            store.purge_terminal_members(workspace_id, &context.workspace_path)
        })?;
        let ProjectMemberPurgeResult {
            removed_count,
            removed_member_ids,
            warning,
        } = result;
        let removed_workspaces = if removed_member_ids.is_empty() {
            Vec::new()
        } else {
            vec![ProjectPurgeRemovedWorkspace {
                workspace_id: workspace_id.to_string(),
                removed_member_ids,
            }]
        };
        return Ok(ProjectPurgeTerminalMembersOutcome {
            result: ProjectPurgeTerminalMembersResult {
                scope,
                total_removed: removed_count as u32,
                workspace_count: 1,
                warnings: warning.into_iter().collect(),
            },
            removed_workspaces,
        });
    }

    let registry = read_workspace_registry_locked(storage)?;
    let registry = registry.ok_or_else(|| WORKSPACE_CONTEXT_MISSING_MESSAGE.to_string())?;
    let entries: HashMap<String, WorkspaceRegistryEntry> =
        serde_json::from_value(registry).map_err(|err| format!("workspace registry decode failed: {err}"))?;
    let store = ProjectMemberStore::new(storage);
    let mut total_removed = 0u32;
    let mut workspace_count = 0u32;
    let mut warnings = Vec::new();
    let mut removed_workspaces = Vec::new();
    for (id, entry) in entries {
        let workspace_path = entry.last_known_path.trim();
        if workspace_path.is_empty() {
            warnings.push(format!("workspace {id} path missing"));
            continue;
        }
        workspace_count += 1;
        match with_workspace_member_lock(&id, || store.purge_terminal_members(&id, workspace_path)) {
            Ok(result) => {
                let ProjectMemberPurgeResult {
                    removed_count,
                    removed_member_ids,
                    warning,
                } = result;
                total_removed += removed_count as u32;
                if let Some(warning) = warning {
                    warnings.push(format!("workspace {id}: {warning}"));
                }
                if !removed_member_ids.is_empty() {
                    removed_workspaces.push(ProjectPurgeRemovedWorkspace {
                        workspace_id: id.clone(),
                        removed_member_ids,
                    });
                }
            }
            Err(err) => warnings.push(format!("workspace {id}: {err}")),
        }
    }
    Ok(ProjectPurgeTerminalMembersOutcome {
        result: ProjectPurgeTerminalMembersResult {
            scope,
            total_removed,
            workspace_count,
            warnings,
        },
        removed_workspaces,
    })
}

struct TerminalIdentity {
    terminal_type: Option<String>,
    command: Option<String>,
    label: String,
}

fn resolve_workspace_context(
    storage: &StorageManager,
    workspace_id: &str,
) -> Result<WorkspaceContext, String> {
    let registry = read_workspace_registry_locked(storage)?;
    let registry = registry.ok_or_else(|| WORKSPACE_CONTEXT_MISSING_MESSAGE.to_string())?;
    let entries: HashMap<String, WorkspaceRegistryEntry> =
        serde_json::from_value(registry).map_err(|err| format!("workspace registry decode failed: {err}"))?;
    let entry = entries
        .get(workspace_id)
        .ok_or_else(|| WORKSPACE_CONTEXT_MISSING_MESSAGE.to_string())?;
    let workspace_path = entry.last_known_path.trim();
    if workspace_path.is_empty() {
        return Err(WORKSPACE_CONTEXT_MISSING_MESSAGE.to_string());
    }
    let workspace_name = Path::new(workspace_path)
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_WORKSPACE_NAME)
        .to_string();
    Ok(WorkspaceContext {
        workspace_path: workspace_path.to_string(),
        workspace_name,
    })
}

fn read_workspace_registry_locked(storage: &StorageManager) -> Result<Option<Value>, String> {
    let lock_path = storage::resolve_app_data_path(storage, WORKSPACE_REGISTRY_LOCK_FILE)?;
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create workspace registry lock dir: {err}"))?;
    }
    let lock_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)
        .map_err(|err| format!("failed to open workspace registry lock file: {err}"))?;
    lock_file
        .lock_exclusive()
        .map_err(|err| format!("failed to lock workspace registry: {err}"))?;

    let registry_path = storage::resolve_app_data_path(storage, WORKSPACE_REGISTRY_FILE)?;
    let registry = storage::read_json_file(&registry_path)?;
    let _ = lock_file.unlock();
    Ok(registry)
}

fn with_workspace_member_lock<T>(
    workspace_id: &str,
    action: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let registry = WORKSPACE_MEMBER_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let lock = {
        let mut guard = match registry.lock() {
            Ok(guard) => guard,
            Err(err) => {
                log::warn!("workspace member lock registry poisoned; recovering");
                err.into_inner()
            }
        };
        guard
            .entry(workspace_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    };
    let _guard = match lock.lock() {
        Ok(guard) => guard,
        Err(err) => {
            log::warn!(
                "workspace member lock poisoned workspace_id={}; recovering",
                workspace_id
            );
            err.into_inner()
        }
    };
    action()
}

fn resolve_terminal_identity(
    request: &ProjectInviteMembersRequest,
) -> Result<TerminalIdentity, String> {
    let command = request.command.as_deref().unwrap_or("").trim().to_string();
    let mut terminal_type = normalize_terminal_type(request.terminal_type.as_deref())?;
    if terminal_type.is_none() && !command.is_empty() {
        terminal_type = Some("shell".to_string());
    }
    if terminal_type.is_none() && command.is_empty() {
        return Err("terminal type or command is required".to_string());
    }
    let label = if !command.is_empty() {
        command.clone()
    } else {
        terminal_type
            .as_deref()
            .unwrap_or("terminal")
            .to_string()
    };
    let command = if command.is_empty() { None } else { Some(command) };
    Ok(TerminalIdentity {
        terminal_type,
        command,
        label,
    })
}

fn normalize_terminal_type(value: Option<&str>) -> Result<Option<String>, String> {
    let trimmed = value.unwrap_or("").trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let lower = trimmed.to_lowercase();
    if matches!(
        lower.as_str(),
        "shell" | "codex" | "gemini" | "claude" | "opencode" | "qwen"
    ) {
        return Ok(Some(lower));
    }
    Err("unsupported terminal type".to_string())
}
