//! Skills UI gateway: import folders into the global skills library.

use std::{
  fs,
  path::{Path, PathBuf},
};

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use super::app::open_folder_in_file_manager;
use crate::runtime::{storage, StorageManager};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SkillFolderImportResult {
  folder_name: String,
  dest_path: String,
}

#[tauri::command]
pub(crate) async fn skills_import_folder(
  app: AppHandle,
) -> Result<Option<SkillFolderImportResult>, String> {
  #[cfg(not(desktop))]
  {
    let _ = app;
    return Err("skill folder import is only supported on desktop".to_string());
  }
  #[cfg(desktop)]
  {
    let selection = tauri::async_runtime::spawn_blocking({
      let app = app.clone();
      move || app.dialog().file().blocking_pick_folder()
    })
    .await
    .map_err(|_| "failed to open folder picker".to_string())?;

    let selection = match selection {
      Some(value) => value,
      None => return Ok(None),
    };

    let source_path = selection
      .into_path()
      .map_err(|err| format!("failed to resolve selected folder: {err}"))?;
    if !source_path.exists() {
      return Err("selected folder does not exist".to_string());
    }
    if !source_path.is_dir() {
      return Err("selected path is not a folder".to_string());
    }

    let folder_name = source_path
      .file_name()
      .and_then(|name| name.to_str())
      .ok_or_else(|| "selected folder name is not valid UTF-8".to_string())?
      .to_string();

    let storage = app.state::<StorageManager>();
    let skills_root = storage::resolve_app_data_path(storage.inner(), "skills")?;
    fs::create_dir_all(&skills_root)
      .map_err(|err| format!("failed to create skills directory: {err}"))?;
    let skills_root = skills_root
      .canonicalize()
      .map_err(|err| format!("failed to resolve skills directory: {err}"))?;
    let source_canonical = source_path
      .canonicalize()
      .map_err(|err| format!("failed to resolve selected folder: {err}"))?;
    if source_canonical.starts_with(&skills_root) {
      return Err("selected folder is already inside the skills library".to_string());
    }

    let destination = unique_destination(&skills_root, &folder_name);
    storage::copy_dir_recursive(&source_path, &destination)?;

    Ok(Some(SkillFolderImportResult {
      folder_name,
      dest_path: destination.to_string_lossy().to_string(),
    }))
  }
}

#[tauri::command]
pub(crate) fn skills_remove_folder(app: AppHandle, path: String) -> Result<bool, String> {
  let trimmed = path.trim();
  if trimmed.is_empty() {
    return Err("skill folder path is empty".to_string());
  }
  let storage = app.state::<StorageManager>();
  let skills_root = storage::resolve_app_data_path(storage.inner(), "skills")?;
  if !skills_root.exists() {
    return Ok(false);
  }
  let skills_root = skills_root
    .canonicalize()
    .map_err(|err| format!("failed to resolve skills directory: {err}"))?;
  let target_path = PathBuf::from(trimmed);
  if !target_path.exists() {
    return Ok(false);
  }
  let target_path = target_path
    .canonicalize()
    .map_err(|err| format!("failed to resolve skill folder: {err}"))?;
  if target_path == skills_root {
    return Err("refusing to delete the skills root".to_string());
  }
  if !target_path.starts_with(&skills_root) {
    return Err("skill folder is outside the skills library".to_string());
  }
  storage::remove_dir_recursive(&target_path)
}

#[tauri::command]
pub(crate) fn skills_open_folder(app: AppHandle, path: String) -> Result<(), String> {
  let trimmed = path.trim();
  if trimmed.is_empty() {
    return Err("skill folder path is empty".to_string());
  }
  let storage = app.state::<StorageManager>();
  let skills_root = storage::resolve_app_data_path(storage.inner(), "skills")?;
  if !skills_root.exists() {
    return Err("skills library does not exist".to_string());
  }
  let skills_root = skills_root
    .canonicalize()
    .map_err(|err| format!("failed to resolve skills directory: {err}"))?;
  let target_path = PathBuf::from(trimmed);
  if !target_path.exists() {
    return Err("skill folder does not exist".to_string());
  }
  if !target_path.is_dir() {
    return Err("skill path is not a folder".to_string());
  }
  let target_path = target_path
    .canonicalize()
    .map_err(|err| format!("failed to resolve skill folder: {err}"))?;
  if !target_path.starts_with(&skills_root) {
    return Err("skill folder is outside the skills library".to_string());
  }
  open_folder_in_file_manager(&target_path)
}

fn unique_destination(root: &Path, folder_name: &str) -> PathBuf {
  let base = root.join(folder_name);
  if !base.exists() {
    return base;
  }
  let mut index = 1;
  loop {
    let candidate = root.join(format!("{folder_name}-{index}"));
    if !candidate.exists() {
      return candidate;
    }
    index += 1;
  }
}
