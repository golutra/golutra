//! 存储层：统一 App/Cache/Workspace 目录读写与路径校验。

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use serde_json::Value;

/// 应用存储管理器：保存 App/Cache 根目录供跨层使用。
#[derive(Clone)]
pub(crate) struct StorageManager {
    app_data_dir: PathBuf,
    app_cache_dir: PathBuf,
}

impl StorageManager {
    /// 构造存储管理器，输入为已解析的目录路径。
    pub(crate) fn new(app_data_dir: PathBuf, app_cache_dir: PathBuf) -> Self {
        Self {
            app_data_dir,
            app_cache_dir,
        }
    }
}

/// 清理相对路径，禁止绝对路径与父级穿越。
pub(crate) fn sanitize_relative_path(relative_path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(relative_path);
    if path.is_absolute() {
        return Err("absolute paths are not allowed".to_string());
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("parent directory segments are not allowed".to_string());
    }
    Ok(path)
}

/// 读取 JSON 文件，文件不存在返回 None。
pub(crate) fn read_json_file(path: &Path) -> Result<Option<Value>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path).map_err(|err| format!("failed to read file: {err}"))?;
    let parsed =
        serde_json::from_str(&contents).map_err(|err| format!("failed to parse JSON: {err}"))?;
    Ok(Some(parsed))
}

/// 写入 JSON 文件，自动创建父目录。
pub(crate) fn write_json_file(path: &Path, payload: Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create data directory: {err}"))?;
    }
    let contents = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("failed to encode JSON: {err}"))?;
    fs::write(path, contents).map_err(|err| format!("failed to write file: {err}"))?;
    Ok(())
}

/// 将相对路径解析到应用数据目录。
pub(crate) fn resolve_app_data_path(
    storage: &StorageManager,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let relative = sanitize_relative_path(relative_path)?;
    Ok(storage.app_data_dir.join(relative))
}

/// 将相对路径解析到应用缓存目录。
pub(crate) fn resolve_app_cache_path(
    storage: &StorageManager,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let relative = sanitize_relative_path(relative_path)?;
    Ok(storage.app_cache_dir.join(relative))
}

/// 将相对路径解析到工作区目录并校验边界。
pub(crate) fn resolve_workspace_path(
    workspace_path: &str,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let base = PathBuf::from(workspace_path)
        .canonicalize()
        .map_err(|err| format!("failed to resolve workspace path: {err}"))?;
    let relative = sanitize_relative_path(relative_path)?;
    let target = base.join(relative);
    if let Some(parent) = target.parent() {
        let parent_canon = parent
            .canonicalize()
            .unwrap_or_else(|_| parent.to_path_buf());
        if !parent_canon.starts_with(&base) {
            return Err("workspace file path is outside the workspace root".to_string());
        }
    }
    Ok(target)
}

/// 读取 App 数据 JSON。
pub(crate) fn read_app_json(
    storage: &StorageManager,
    relative_path: &str,
) -> Result<Option<Value>, String> {
    let path = resolve_app_data_path(storage, relative_path)?;
    read_json_file(&path)
}

/// 写入 App 数据 JSON。
pub(crate) fn write_app_json(
    storage: &StorageManager,
    relative_path: &str,
    payload: Value,
) -> Result<(), String> {
    let path = resolve_app_data_path(storage, relative_path)?;
    write_json_file(&path, payload)
}

/// 读取缓存 JSON。
pub(crate) fn read_cache_json(
    storage: &StorageManager,
    relative_path: &str,
) -> Result<Option<Value>, String> {
    let path = resolve_app_cache_path(storage, relative_path)?;
    read_json_file(&path)
}

/// 写入缓存 JSON。
pub(crate) fn write_cache_json(
    storage: &StorageManager,
    relative_path: &str,
    payload: Value,
) -> Result<(), String> {
    let path = resolve_app_cache_path(storage, relative_path)?;
    write_json_file(&path, payload)
}

/// 写入缓存文本并返回绝对路径。
pub(crate) fn write_cache_text(
    storage: &StorageManager,
    relative_path: &str,
    contents: &str,
) -> Result<String, String> {
    let path = resolve_app_cache_path(storage, relative_path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("failed to create cache dir: {err}"))?;
    }
    fs::write(&path, contents).map_err(|err| format!("failed to write cache text: {err}"))?;
    Ok(path.to_string_lossy().to_string())
}

/// Recursively copy a directory tree into a destination directory.
pub(crate) fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.is_dir() {
        return Err("source is not a directory".to_string());
    }
    if destination.exists() && !destination.is_dir() {
        return Err("destination exists and is not a directory".to_string());
    }
    fs::create_dir_all(destination)
        .map_err(|err| format!("failed to create destination directory: {err}"))?;
    let entries =
        fs::read_dir(source).map_err(|err| format!("failed to read source directory: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
        let entry_type = entry
            .file_type()
            .map_err(|err| format!("failed to read entry type: {err}"))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else if entry_type.is_file() {
            fs::copy(&source_path, &destination_path)
                .map_err(|err| format!("failed to copy file: {err}"))?;
        } else if entry_type.is_symlink() {
            return Err(format!(
                "symlink entries are not supported: {}",
                source_path.to_string_lossy()
            ));
        }
    }
    Ok(())
}

/// Recursively remove a directory tree.
pub(crate) fn remove_dir_recursive(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }
    if !path.is_dir() {
        return Err("path is not a directory".to_string());
    }
    fs::remove_dir_all(path).map_err(|err| format!("failed to remove directory: {err}"))?;
    Ok(true)
}

/// Create a directory symlink.
pub(crate) fn create_dir_symlink(target: &Path, link_path: &Path) -> Result<(), String> {
    if let Some(parent) = link_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create symlink parent dir: {err}"))?;
    }
    if link_path.exists() {
        return Err("symlink destination already exists".to_string());
    }
    #[cfg(target_family = "windows")]
    {
        std::os::windows::fs::symlink_dir(target, link_path)
            .map_err(|err| format!("failed to create directory symlink: {err}"))?;
    }
    #[cfg(target_family = "unix")]
    {
        std::os::unix::fs::symlink(target, link_path)
            .map_err(|err| format!("failed to create directory symlink: {err}"))?;
    }
    Ok(())
}

/// Remove a symlink without touching the target.
pub(crate) fn remove_symlink(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|err| format!("failed to read symlink: {err}"))?;
    if !metadata.file_type().is_symlink() {
        return Err("path is not a symlink".to_string());
    }
    if fs::remove_file(path).is_err() {
        fs::remove_dir(path).map_err(|err| format!("failed to remove symlink: {err}"))?;
    }
    Ok(true)
}
