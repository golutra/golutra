//! 平台状态命令：对前端暴露更新与激活状态。

use tauri::State;

use crate::platform::{ActivationState, ActivationStatusPayload, UpdaterState, UpdaterStatusPayload};

#[tauri::command]
pub(crate) fn platform_get_updater_status(
  state: State<'_, UpdaterState>,
) -> UpdaterStatusPayload {
  state.snapshot()
}

#[tauri::command]
pub(crate) fn platform_get_activation_status(
  state: State<'_, ActivationState>,
) -> ActivationStatusPayload {
  state.snapshot()
}
