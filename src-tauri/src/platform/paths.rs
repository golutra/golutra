//! 平台路径解析：集中管理日志等基础路径，避免基础层反向依赖组合根。

use std::path::{Path, PathBuf};

// 开发态日志落地到仓库根目录，便于统一收集与排查。
fn find_repo_root(path: &Path) -> Option<PathBuf> {
  for ancestor in path.ancestors() {
    if ancestor.file_name().and_then(|name| name.to_str()) == Some("src-tauri") {
      return ancestor.parent().map(|parent| parent.to_path_buf());
    }
  }
  None
}

// 日志目录：支持环境变量覆盖，避免在只读目录写入。
pub(crate) fn resolve_log_dir() -> PathBuf {
  if let Ok(value) = std::env::var("GOLUTRA_LOG_DIR") {
    return PathBuf::from(value);
  }
  let cwd = std::env::current_dir().ok();
  let exe_dir = std::env::current_exe()
    .ok()
    .and_then(|path| path.parent().map(|parent| parent.to_path_buf()));
  let root = cwd
    .as_ref()
    .and_then(|path| find_repo_root(path))
    .or_else(|| exe_dir.as_ref().and_then(|path| find_repo_root(path)))
    .or_else(|| cwd.clone())
    .unwrap_or_else(|| PathBuf::from("."));
  root.join("log")
}
