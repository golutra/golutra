//! Project skills UI gateway: manage workspace skill symlinks.

use std::{
  fs,
  path::{Path, PathBuf},
};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::runtime::{storage, StorageManager};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectSkillLink {
  name: String,
  link_path: String,
  target_path: String,
}

#[tauri::command]
pub(crate) fn project_skills_list(workspace_path: String) -> Result<Vec<ProjectSkillLink>, String> {
  let skills_root = storage::resolve_workspace_path(&workspace_path, ".golutra/skills")?;
  if !skills_root.exists() {
    return Ok(Vec::new());
  }
  let entries = fs::read_dir(&skills_root)
    .map_err(|err| format!("failed to read project skills directory: {err}"))?;
  let mut results = Vec::new();
  for entry in entries {
    let entry = entry.map_err(|err| format!("failed to read project skill entry: {err}"))?;
    let path = entry.path();
    let metadata = fs::symlink_metadata(&path)
      .map_err(|err| format!("failed to read project skill metadata: {err}"))?;
    if !metadata.file_type().is_symlink() {
      continue;
    }
    let name = entry
      .file_name()
      .to_str()
      .map(|value| value.to_string())
      .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
    let target = fs::read_link(&path)
      .map_err(|err| format!("failed to read project skill link: {err}"))?;
    let target_path = if target.is_absolute() {
      target
    } else {
      path.parent()
        .unwrap_or(&skills_root)
        .join(target)
    };
    results.push(ProjectSkillLink {
      name,
      link_path: path.to_string_lossy().to_string(),
      target_path: target_path.to_string_lossy().to_string(),
    });
  }
  Ok(results)
}

#[tauri::command]
pub(crate) fn project_skills_link(
  app: AppHandle,
  workspace_path: String,
  source_path: String,
) -> Result<ProjectSkillLink, String> {
  let storage = app.state::<StorageManager>();
  let skills_root = storage::resolve_workspace_path(&workspace_path, ".golutra/skills")?;
  fs::create_dir_all(&skills_root)
    .map_err(|err| format!("failed to create project skills directory: {err}"))?;
  let skills_root = skills_root
    .canonicalize()
    .map_err(|err| format!("failed to resolve project skills directory: {err}"))?;

  let trimmed_source = source_path.trim();
  if trimmed_source.is_empty() {
    return Err("selected skill path is empty".to_string());
  }
  let source_path = PathBuf::from(trimmed_source);
  if !source_path.exists() {
    return Err("selected skill folder does not exist".to_string());
  }
  if !source_path.is_dir() {
    return Err("selected skill path is not a folder".to_string());
  }
  let source_canonical = source_path
    .canonicalize()
    .map_err(|err| format!("failed to resolve selected skill folder: {err}"))?;

  let library_root = storage::resolve_app_data_path(storage.inner(), "skills")?;
  let library_root = library_root
    .canonicalize()
    .map_err(|err| format!("failed to resolve skills library: {err}"))?;
  if !source_canonical.starts_with(&library_root) {
    return Err("selected skill is not inside the skills library".to_string());
  }

  if skills_root.exists() {
    let entries = fs::read_dir(&skills_root)
      .map_err(|err| format!("failed to read project skills directory: {err}"))?;
    for entry in entries {
      let entry = entry.map_err(|err| format!("failed to read project skill entry: {err}"))?;
      let entry_path = entry.path();
      let metadata = fs::symlink_metadata(&entry_path)
        .map_err(|err| format!("failed to read project skill metadata: {err}"))?;
      if !metadata.file_type().is_symlink() {
        continue;
      }
      let target = fs::read_link(&entry_path)
        .map_err(|err| format!("failed to read project skill link: {err}"))?;
      let target_path = if target.is_absolute() {
        target
      } else {
        entry_path
          .parent()
          .unwrap_or(&skills_root)
          .join(target)
      };
      let target_canonical = target_path
        .canonicalize()
        .unwrap_or_else(|_| target_path);
      if target_canonical == source_canonical {
        return Err("skill is already linked to this workspace".to_string());
      }
    }
  }

  let folder_name = source_canonical
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| "selected skill folder name is not valid UTF-8".to_string())?;
  let destination = unique_destination(&skills_root, folder_name);
  storage::create_dir_symlink(&source_canonical, &destination)?;

  let name = destination
    .file_name()
    .and_then(|value| value.to_str())
    .map(|value| value.to_string())
    .unwrap_or_else(|| destination.file_name().unwrap().to_string_lossy().to_string());
  Ok(ProjectSkillLink {
    name,
    link_path: destination.to_string_lossy().to_string(),
    target_path: source_canonical.to_string_lossy().to_string(),
  })
}

#[tauri::command]
pub(crate) fn project_skills_unlink(
  workspace_path: String,
  link_name: String,
) -> Result<bool, String> {
  let trimmed = link_name.trim();
  if trimmed.is_empty() {
    return Err("link name is empty".to_string());
  }
  if trimmed.contains('/') || trimmed.contains('\\') {
    return Err("link name is invalid".to_string());
  }
  let skills_root = storage::resolve_workspace_path(&workspace_path, ".golutra/skills")?;
  if !skills_root.exists() {
    return Ok(false);
  }
  let target_path = skills_root.join(trimmed);
  if !target_path.exists() {
    return Ok(false);
  }
  let metadata = fs::symlink_metadata(&target_path)
    .map_err(|err| format!("failed to read project skill metadata: {err}"))?;
  if !metadata.file_type().is_symlink() {
    return Err("project skill is not a symlink".to_string());
  }
  storage::remove_symlink(&target_path)
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
