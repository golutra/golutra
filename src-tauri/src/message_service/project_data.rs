//! 项目数据服务：负责工作区项目数据的读取与持久化规则。

use serde::Serialize;
use serde_json::Value;

use crate::runtime::{storage, StorageManager};

const PROJECT_DATA_PATH: &str = ".golutra/workspace.json";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
/// 项目数据读取结果：包含来源与可选告警信息。
pub(crate) struct ProjectDataReadResult {
  pub(crate) data: Option<Value>,
  pub(crate) source: String,
  pub(crate) warning: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
/// 项目数据写入结果：标识实际写入的存储位置与告警信息。
pub(crate) struct ProjectDataWriteResult {
  pub(crate) storage: String,
  pub(crate) warning: Option<String>,
}

fn project_data_app_path(workspace_id: &str) -> String {
  format!("{workspace_id}/project.json")
}

/// 读取工作区项目数据。
/// 输入：工作区路径与 ID。
/// 输出：读取结果（含来源与告警）。
/// 错误：路径非法或读取失败。
pub(crate) fn read_project_data(
  storage: &StorageManager,
  workspace_path: &str,
  workspace_id: &str,
) -> Result<ProjectDataReadResult, String> {
  let workspace_id = workspace_id.trim();
  if workspace_id.is_empty() {
    return Err("workspace id is empty".to_string());
  }
  let workspace_path = workspace_path.trim();
  if workspace_path.is_empty() {
    return Err("workspace path is empty".to_string());
  }
  let mut warning = None;
  let workspace_payload = storage::resolve_workspace_path(workspace_path, PROJECT_DATA_PATH)
    .and_then(|path| storage::read_json_file(&path));
  match workspace_payload {
    Ok(Some(value)) => {
      return Ok(ProjectDataReadResult {
        data: Some(value),
        source: "workspace".to_string(),
        warning: None,
      });
    }
    Ok(None) => {}
    Err(err) => {
      warning = Some(format!("workspace read failed: {err}"));
    }
  }

  let app_payload = storage::read_app_json(storage, &project_data_app_path(workspace_id));
  match app_payload {
    Ok(Some(value)) => Ok(ProjectDataReadResult {
      data: Some(value),
      source: "app".to_string(),
      warning,
    }),
    Ok(None) => Ok(ProjectDataReadResult {
      data: None,
      source: "none".to_string(),
      warning,
    }),
    Err(err) => {
      if let Some(existing) = warning {
        Err(format!("{existing}; app read failed: {err}"))
      } else {
        Err(format!("app read failed: {err}"))
      }
    }
  }
}

/// 写入工作区项目数据。
/// 输入：工作区路径与 ID、只读标记、待写入 payload。
/// 输出：写入结果（含写入位置与告警）。
/// 错误：路径非法或写入失败。
pub(crate) fn write_project_data(
  storage: &StorageManager,
  workspace_path: &str,
  workspace_id: &str,
  read_only: bool,
  payload: Value,
) -> Result<ProjectDataWriteResult, String> {
  let workspace_id = workspace_id.trim();
  if workspace_id.is_empty() {
    return Err("workspace id is empty".to_string());
  }
  let workspace_path = workspace_path.trim();
  if workspace_path.is_empty() {
    return Err("workspace path is empty".to_string());
  }

  let app_path = project_data_app_path(workspace_id);
  if read_only {
    storage::write_app_json(storage, &app_path, payload)?;
    return Ok(ProjectDataWriteResult {
      storage: "app".to_string(),
      warning: None,
    });
  }

  let fallback_payload = payload.clone();
  let workspace_result = storage::resolve_workspace_path(workspace_path, PROJECT_DATA_PATH)
    .and_then(|path| storage::write_json_file(&path, payload));
  match workspace_result {
    Ok(()) => Ok(ProjectDataWriteResult {
      storage: "workspace".to_string(),
      warning: None,
    }),
    Err(err) => {
      let warning = format!("workspace write failed: {err}");
      storage::write_app_json(storage, &app_path, fallback_payload).map_err(|fallback_err| {
        format!("{warning}; app write failed: {fallback_err}")
      })?;
      Ok(ProjectDataWriteResult {
        storage: "app".to_string(),
        warning: Some(warning),
      })
    }
  }
}
