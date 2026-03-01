//! 项目数据命令：UI 层入口转发到业务服务层。

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::message_service::project_data;
use crate::message_service::project_data::{ProjectDataReadResult, ProjectDataWriteResult};
use crate::runtime::StorageManager;

#[tauri::command]
/// 读取项目数据。
/// 输入：工作区 ID 与路径。
/// 输出：项目数据读取结果。
/// 错误：路径非法或读取失败。
pub(crate) fn project_data_read(
  app: AppHandle,
  workspace_id: String,
  workspace_path: String,
) -> Result<ProjectDataReadResult, String> {
  let storage = app.state::<StorageManager>();
  project_data::read_project_data(storage.inner(), &workspace_path, &workspace_id)
}

#[tauri::command]
/// 写入项目数据。
/// 输入：工作区 ID 与路径、只读标记、payload。
/// 输出：项目数据写入结果。
/// 错误：路径非法或写入失败。
pub(crate) fn project_data_write(
  app: AppHandle,
  workspace_id: String,
  workspace_path: String,
  read_only: bool,
  payload: Value,
) -> Result<ProjectDataWriteResult, String> {
  let storage = app.state::<StorageManager>();
  project_data::write_project_data(
    storage.inner(),
    &workspace_path,
    &workspace_id,
    read_only,
    payload,
  )
}
