//! 项目成员命令：将成员创建下沉到后端统一处理。

use tauri::{AppHandle, Manager};

use crate::application::chat as chat_app;
use crate::application::project::{
    project_invite_members, project_purge_terminal_members, ProjectInviteMembersRequest,
    ProjectInviteMembersResult, ProjectPurgeTerminalMembersRequest,
    ProjectPurgeTerminalMembersResult,
};
use crate::message_service::chat_db::ChatDbManager;
use crate::runtime::StorageManager;
use crate::terminal_engine::session::{terminal_close_by_member_ids, TerminalManager};

#[tauri::command]
/// 邀请并创建项目成员（后端统一生成）。
/// 输入：workspaceId + invite payload。
/// 输出：更新后的成员列表与本次创建成员。
/// 错误：项目数据读取/写入失败或参数不合法。
pub(crate) fn project_members_invite(
    app: AppHandle,
    workspace_id: String,
    payload: ProjectInviteMembersRequest,
) -> Result<ProjectInviteMembersResult, String> {
    let storage = app.state::<StorageManager>();
    project_invite_members(
        storage.inner(),
        &workspace_id,
        payload,
    )
}

#[tauri::command]
/// 清理终端好友并重置序号。
/// 输入：workspaceId 与清理范围（current/all）。
/// 输出：清理汇总统计。
/// 错误：工作区解析或项目数据读写失败。
pub(crate) fn project_members_purge_terminal(
    app: AppHandle,
    workspace_id: String,
    payload: ProjectPurgeTerminalMembersRequest,
) -> Result<ProjectPurgeTerminalMembersResult, String> {
    let storage = app.state::<StorageManager>();
    let mut outcome = project_purge_terminal_members(storage.inner(), &workspace_id, payload)?;
    if !outcome.removed_workspaces.is_empty() {
        let terminal_state = app.state::<TerminalManager>();
        for workspace in &outcome.removed_workspaces {
            let warnings = terminal_close_by_member_ids(
                app.clone(),
                terminal_state.clone(),
                workspace.workspace_id.as_str(),
                &workspace.removed_member_ids,
                true,
            );
            outcome.result.warnings.extend(warnings);
            let chat_state = app.state::<ChatDbManager>();
            match chat_app::chat_delete_member_conversations(
                chat_state,
                workspace.workspace_id.clone(),
                workspace.removed_member_ids.clone(),
            ) {
                Ok(result) => {
                    if !result.warnings.is_empty() {
                        outcome.result.warnings.extend(result.warnings);
                    }
                }
                Err(err) => {
                    outcome
                        .result
                        .warnings
                        .push(format!("workspace {}: {err}", workspace.workspace_id));
                }
            }
        }
    }
    Ok(outcome.result)
}
