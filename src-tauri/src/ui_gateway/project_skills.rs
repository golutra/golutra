//! Project skills UI gateway: manage workspace skill entries.

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
    managed: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum ProjectSkillConflict {
    NameConflict,
    TargetConflict,
}

#[derive(Debug, PartialEq, Eq)]
struct ExistingProjectSkill {
    name: String,
    canonical_target: Option<PathBuf>,
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
        let managed = metadata.file_type().is_symlink();
        if !managed && !metadata.is_dir() {
            continue;
        }
        let name = entry
            .file_name()
            .to_str()
            .map(|value| value.to_string())
            .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
        let target_path = if managed {
            resolve_link_target_path(&path, &skills_root)?
        } else {
            canonicalize_path(&path)
        };
        results.push(ProjectSkillLink {
            name,
            link_path: path.to_string_lossy().to_string(),
            target_path: target_path.to_string_lossy().to_string(),
            managed,
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

    let folder_name = source_canonical
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "selected skill folder name is not valid UTF-8".to_string())?;
    let existing_skills = collect_existing_project_skills(&skills_root)?;
    match detect_project_skill_conflict(&existing_skills, folder_name, &source_canonical) {
        Some(ProjectSkillConflict::TargetConflict) => {
            return Err("skill is already linked to this workspace".to_string());
        }
        Some(ProjectSkillConflict::NameConflict) => {
            return Err("a project skill with the same name already exists".to_string());
        }
        None => {}
    }

    let destination = skills_root.join(folder_name);
    storage::create_dir_symlink(&source_canonical, &destination)?;

    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| {
            destination
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        });
    Ok(ProjectSkillLink {
        name,
        link_path: destination.to_string_lossy().to_string(),
        target_path: source_canonical.to_string_lossy().to_string(),
        managed: true,
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

fn collect_existing_project_skills(root: &Path) -> Result<Vec<ExistingProjectSkill>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(root)
        .map_err(|err| format!("failed to read project skills directory: {err}"))?;
    let mut results = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read project skill entry: {err}"))?;
        let entry_path = entry.path();
        let metadata = fs::symlink_metadata(&entry_path)
            .map_err(|err| format!("failed to read project skill metadata: {err}"))?;
        let name = entry
            .file_name()
            .to_str()
            .map(|value| value.to_string())
            .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
        let canonical_target = if metadata.file_type().is_symlink() {
            Some(canonicalize_path(&resolve_link_target_path(
                &entry_path,
                root,
            )?))
        } else if metadata.is_dir() || metadata.is_file() {
            Some(canonicalize_path(&entry_path))
        } else {
            None
        };
        results.push(ExistingProjectSkill {
            name,
            canonical_target,
        });
    }
    Ok(results)
}

fn detect_project_skill_conflict(
    existing_skills: &[ExistingProjectSkill],
    folder_name: &str,
    source_canonical: &Path,
) -> Option<ProjectSkillConflict> {
    if existing_skills.iter().any(|entry| {
        entry
            .canonical_target
            .as_ref()
            .map(|target| target == source_canonical)
            .unwrap_or(false)
    }) {
        return Some(ProjectSkillConflict::TargetConflict);
    }
    if existing_skills
        .iter()
        .any(|entry| entry.name.eq_ignore_ascii_case(folder_name))
    {
        return Some(ProjectSkillConflict::NameConflict);
    }
    None
}

fn resolve_link_target_path(path: &Path, root: &Path) -> Result<PathBuf, String> {
    let target =
        fs::read_link(path).map_err(|err| format!("failed to read project skill link: {err}"))?;
    Ok(if target.is_absolute() {
        target
    } else {
        path.parent().unwrap_or(root).join(target)
    })
}

fn canonicalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_project_skill_conflict_flags_existing_name() {
        let existing = vec![ExistingProjectSkill {
            name: "skill-vetter".to_string(),
            canonical_target: Some(PathBuf::from(
                "D:/Program Files/golutra/.golutra/skills/skill-vetter",
            )),
        }];

        let conflict = detect_project_skill_conflict(
            &existing,
            "skill-vetter",
            Path::new("C:/Users/will/AppData/Roaming/com.golutra/skills/skill-vetter"),
        );

        assert_eq!(conflict, Some(ProjectSkillConflict::NameConflict));
    }

    #[test]
    fn detect_project_skill_conflict_flags_existing_target_alias() {
        let source = PathBuf::from("C:/Users/will/AppData/Roaming/com.golutra/skills/skill-vetter");
        let existing = vec![ExistingProjectSkill {
            name: "skill-vetter-1".to_string(),
            canonical_target: Some(source.clone()),
        }];

        let conflict = detect_project_skill_conflict(&existing, "skill-vetter", &source);

        assert_eq!(conflict, Some(ProjectSkillConflict::TargetConflict));
    }

    #[test]
    fn detect_project_skill_conflict_allows_distinct_name_and_target() {
        let existing = vec![ExistingProjectSkill {
            name: "skill-vetter".to_string(),
            canonical_target: Some(PathBuf::from(
                "D:/Program Files/golutra/.golutra/skills/skill-vetter",
            )),
        }];

        let conflict = detect_project_skill_conflict(
            &existing,
            "skillnet-phase1",
            Path::new("C:/Users/will/AppData/Roaming/com.golutra/skills/skillnet-phase1"),
        );

        assert_eq!(conflict, None);
    }
}
