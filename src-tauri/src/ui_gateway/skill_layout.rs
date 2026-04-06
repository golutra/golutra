//! Shared skill layout validation and lightweight normalization helpers.

use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SkillLayout {
    pub(crate) canonical_name: String,
    pub(crate) description: String,
    pub(crate) display_name: String,
}

pub(crate) fn inspect_skill_folder(skill_root: &Path) -> Result<SkillLayout, String> {
    if !skill_root.exists() {
        return Err("skill folder does not exist".to_string());
    }
    if !skill_root.is_dir() {
        return Err("skill path is not a folder".to_string());
    }

    let skill_md_path = skill_root.join("SKILL.md");
    if !skill_md_path.exists() {
        return Err("skill folder must contain SKILL.md".to_string());
    }
    if !skill_md_path.is_file() {
        return Err("SKILL.md must be a file".to_string());
    }

    let contents = fs::read_to_string(&skill_md_path)
        .map_err(|err| format!("failed to read SKILL.md: {err}"))?;
    let frontmatter = parse_skill_frontmatter(&contents)?;
    if !is_kebab_case(&frontmatter.name) {
        return Err("SKILL.md frontmatter name must be lowercase kebab-case".to_string());
    }
    let display_name =
        extract_heading(&contents).unwrap_or_else(|| title_case_name(&frontmatter.name));

    Ok(SkillLayout {
        canonical_name: frontmatter.name,
        description: frontmatter.description,
        display_name,
    })
}

pub(crate) fn ensure_openai_interface_file(
    skill_root: &Path,
    layout: &SkillLayout,
) -> Result<PathBuf, String> {
    let openai_path = skill_root.join("agents").join("openai.yaml");
    if openai_path.exists() {
        if !openai_path.is_file() {
            return Err("agents/openai.yaml must be a file".to_string());
        }
        return Ok(openai_path);
    }

    if let Some(parent) = openai_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create agents directory: {err}"))?;
    }

    let contents = format!(
    "interface:\n  display_name: \"{}\"\n  short_description: \"{}\"\n  default_prompt: \"Use ${} when this skill is needed.\"\n",
    yaml_escape(&layout.display_name),
    yaml_escape(&truncate_text(&layout.description, 160)),
    yaml_escape(&layout.canonical_name),
  );
    fs::write(&openai_path, contents)
        .map_err(|err| format!("failed to write agents/openai.yaml: {err}"))?;
    Ok(openai_path)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SkillFrontmatter {
    name: String,
    description: String,
}

fn parse_skill_frontmatter(contents: &str) -> Result<SkillFrontmatter, String> {
    let mut lines = contents.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Err("SKILL.md must start with YAML frontmatter".to_string());
    }

    let mut closed = false;
    let mut name = None;
    let mut description = None;
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed == "---" {
            closed = true;
            break;
        }
        if trimmed.is_empty() {
            continue;
        }
        let Some((key, raw_value)) = trimmed.split_once(':') else {
            continue;
        };
        let value = normalize_yaml_scalar(raw_value);
        match key.trim() {
            "name" => name = Some(value),
            "description" => description = Some(value),
            _ => {}
        }
    }

    if !closed {
        return Err("SKILL.md frontmatter is not closed".to_string());
    }

    let name = name
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "SKILL.md frontmatter must include name".to_string())?;
    let description = description
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "SKILL.md frontmatter must include description".to_string())?;

    Ok(SkillFrontmatter { name, description })
}

fn normalize_yaml_scalar(raw: &str) -> String {
    let value = raw.trim();
    let unquoted = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|rest| rest.strip_suffix('\''))
        })
        .unwrap_or(value);
    unquoted.trim().to_string()
}

fn extract_heading(contents: &str) -> Option<String> {
    let mut lines = contents.lines();
    if lines.next().map(str::trim) != Some("---") {
        return None;
    }
    for line in lines.by_ref() {
        if line.trim() == "---" {
            break;
        }
    }
    for line in lines {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            let title = title.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

fn is_kebab_case(value: &str) -> bool {
    if value.is_empty() || value.starts_with('-') || value.ends_with('-') || value.contains("--") {
        return false;
    }
    value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn title_case_name(value: &str) -> String {
    value
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = String::new();
                    word.push(first.to_ascii_uppercase());
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_text(value: &str, max_len: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_len {
        return trimmed.to_string();
    }
    let truncated = trimmed
        .chars()
        .take(max_len.saturating_sub(1))
        .collect::<String>();
    format!("{truncated}…")
}

fn yaml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::{ensure_openai_interface_file, inspect_skill_folder};
    use std::{env, fs, path::PathBuf};
    use ulid::Ulid;

    fn temp_dir(name: &str) -> PathBuf {
        let path = env::temp_dir().join(format!("golutra-{name}-{}", Ulid::new()));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn inspect_skill_folder_reads_frontmatter_and_heading() {
        let dir = temp_dir("skill-layout");
        fs::write(
      dir.join("SKILL.md"),
      "---\nname: token-compact\ndescription: Keep shared channels compact.\n---\n\n# Token Compact\n",
    )
    .expect("write skill file");

        let layout = inspect_skill_folder(&dir).expect("inspect skill layout");
        assert_eq!(layout.canonical_name, "token-compact");
        assert_eq!(layout.description, "Keep shared channels compact.");
        assert_eq!(layout.display_name, "Token Compact");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_skill_folder_rejects_invalid_name() {
        let dir = temp_dir("skill-layout-invalid");
        fs::write(
            dir.join("SKILL.md"),
            "---\nname: TokenCompact\ndescription: Keep shared channels compact.\n---\n",
        )
        .expect("write skill file");

        let error = inspect_skill_folder(&dir).expect_err("invalid kebab-case name");
        assert!(error.contains("kebab-case"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_openai_interface_file_bootstraps_missing_metadata() {
        let dir = temp_dir("skill-layout-bootstrap");
        fs::write(
      dir.join("SKILL.md"),
      "---\nname: token-compact\ndescription: Keep shared channels compact.\n---\n\n# Token Compact\n",
    )
    .expect("write skill file");

        let layout = inspect_skill_folder(&dir).expect("inspect skill layout");
        let openai_path =
            ensure_openai_interface_file(&dir, &layout).expect("create openai metadata file");
        let contents = fs::read_to_string(openai_path).expect("read openai metadata file");
        assert!(contents.contains("display_name: \"Token Compact\""));
        assert!(contents.contains("short_description: \"Keep shared channels compact.\""));
        assert!(
            contents.contains("default_prompt: \"Use $token-compact when this skill is needed.\"")
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
