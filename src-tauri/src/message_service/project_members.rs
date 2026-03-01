//! 项目成员存储：封装成员列表的读写与清理操作。

use serde_json::{json, Map, Value};
use ulid::Ulid;

use crate::message_service::project_data;
use crate::runtime::StorageManager;

const DEFAULT_OWNER_ID: &str = "01J00000000000000000000000";
const DEFAULT_OWNER_NAME: &str = "Owner";
const DEFAULT_OWNER_ROLE_KEY: &str = "members.roles.owner";
const DEFAULT_OWNER_ROLE_TYPE: &str = "owner";
const DEFAULT_MEMBER_STATUS: &str = "online";
const DEFAULT_VERSION: i64 = 1;
const DEFAULT_WORKSPACE_NAME: &str = "workspace";
const MEMBER_SEQUENCE_KEY: &str = "memberSequence";

const ROLE_ASSISTANT: &str = "assistant";
const ROLE_MEMBER: &str = "member";
const ROLE_KEY_ASSISTANT: &str = "members.roles.aiAssistant";
const ROLE_KEY_MEMBER: &str = "members.roles.member";

const AVATAR_PRESET_IDS: [&str; 5] = ["orbit", "ember", "mint", "canyon", "storm"];
const CSS_AVATAR_PREFIX: &str = "css:";

pub(crate) struct ProjectMemberStore<'a> {
    storage: &'a StorageManager,
}

pub(crate) struct ProjectMemberPurgeResult {
    pub(crate) removed_count: usize,
    pub(crate) removed_member_ids: Vec<String>,
    pub(crate) warning: Option<String>,
}

pub(crate) struct ProjectMemberInviteResult {
    pub(crate) members: Vec<Value>,
    pub(crate) created_members: Vec<Value>,
    pub(crate) storage: String,
    pub(crate) warning: Option<String>,
}

impl<'a> ProjectMemberStore<'a> {
    pub(crate) fn new(storage: &'a StorageManager) -> Self {
        Self { storage }
    }

    pub(crate) fn invite_members(
        &self,
        workspace_id: &str,
        workspace_path: &str,
        workspace_name: &str,
        role_type: &str,
        terminal_label: &str,
        terminal_type: Option<&str>,
        command: Option<&str>,
        instance_count: u32,
        unlimited_access: bool,
        sandboxed: bool,
    ) -> Result<ProjectMemberInviteResult, String> {
        let role_type = role_type.trim().to_lowercase();
        if role_type != ROLE_ASSISTANT && role_type != ROLE_MEMBER {
            return Err("unsupported member role".to_string());
        }
        let instance_count = normalize_instance_count(instance_count);
        let base_name = build_member_base_name(workspace_name, &role_type, terminal_label);

        let read_result = project_data::read_project_data(self.storage, workspace_path, workspace_id)?;
        let mut warning = read_result.warning;
        let mut payload = read_result.data.unwrap_or_else(|| json!({}));
        if !payload.is_object() {
            payload = json!({});
        }
        let object = payload
            .as_object_mut()
            .ok_or_else(|| "project data is not a JSON object".to_string())?;
        object
            .entry("projectId")
            .or_insert_with(|| Value::String(workspace_id.to_string()));
        object
            .entry("version")
            .or_insert_with(|| Value::Number(DEFAULT_VERSION.into()));
        let member_sequence_snapshot = read_member_sequence_snapshot(object);
        let (members_snapshot, created_snapshot, next_index) = {
            let members_value = object
                .entry("members")
                .or_insert_with(|| Value::Array(Vec::new()));
            let members_array = members_value
                .as_array_mut()
                .ok_or_else(|| "project members is not a JSON array".to_string())?;

            // 兜底补齐默认 owner，避免成员列表为空时破坏项目结构。
            ensure_default_owner(members_array);

            let roster_names = extract_member_names(members_array);
            let start_index =
                resolve_member_sequence_start(&base_name, &roster_names, &member_sequence_snapshot);

            let mut created_members = Vec::new();
            for offset in 0..instance_count {
                let name = format!("{}-{}", base_name, start_index + offset as usize);
                let avatar_seed_prefix = if role_type == ROLE_MEMBER {
                    "member"
                } else {
                    "assistant"
                };
                let avatar = build_seeded_avatar(&format!("{avatar_seed_prefix}:{name}"));
                let role_key = if role_type == ROLE_MEMBER {
                    ROLE_KEY_MEMBER
                } else {
                    ROLE_KEY_ASSISTANT
                };

                let mut member = serde_json::Map::new();
                member.insert("id".to_string(), Value::String(Ulid::new().to_string()));
                member.insert("name".to_string(), Value::String(name));
                member.insert("role".to_string(), Value::String(String::new()));
                member.insert("roleKey".to_string(), Value::String(role_key.to_string()));
                member.insert("roleType".to_string(), Value::String(role_type.clone()));
                member.insert("avatar".to_string(), Value::String(avatar));
                member.insert(
                    "status".to_string(),
                    Value::String(DEFAULT_MEMBER_STATUS.to_string()),
                );
                member.insert("autoStartTerminal".to_string(), Value::Bool(true));
                member.insert("unlimitedAccess".to_string(), Value::Bool(unlimited_access));
                member.insert("sandboxed".to_string(), Value::Bool(sandboxed));
                if let Some(value) = terminal_type {
                    member.insert("terminalType".to_string(), Value::String(value.to_string()));
                }
                if let Some(value) = command {
                    member.insert("terminalCommand".to_string(), Value::String(value.to_string()));
                }

                let member_value = Value::Object(member);
                members_array.push(member_value.clone());
                created_members.push(member_value);
            }

            let next_index = start_index + instance_count as usize;
            let members_snapshot = members_array.clone();
            let created_snapshot = created_members.clone();
            (members_snapshot, created_snapshot, next_index)
        };

        let member_sequence = ensure_member_sequence(object)?;
        store_member_sequence(member_sequence, &base_name, next_index);
        let write_result = project_data::write_project_data(
            self.storage,
            workspace_path,
            workspace_id,
            false,
            payload,
        )?;

        if let Some(write_warning) = write_result.warning {
            warning = Some(match warning {
                Some(existing) => format!("{existing}; {write_warning}"),
                None => write_warning,
            });
        }

        Ok(ProjectMemberInviteResult {
            members: members_snapshot,
            created_members: created_snapshot,
            storage: write_result.storage,
            warning,
        })
    }

    pub(crate) fn purge_terminal_members(
        &self,
        workspace_id: &str,
        workspace_path: &str,
    ) -> Result<ProjectMemberPurgeResult, String> {
        let read_result = project_data::read_project_data(self.storage, workspace_path, workspace_id)?;
        let mut warning = read_result.warning;
        let payload = match read_result.data {
            Some(value) => value,
            None => {
                return Ok(ProjectMemberPurgeResult {
                    removed_count: 0,
                    removed_member_ids: Vec::new(),
                    warning,
                });
            }
        };
        let mut payload = payload;
        if !payload.is_object() {
            payload = json!({});
        }
        let object = payload
            .as_object_mut()
            .ok_or_else(|| "project data is not a JSON object".to_string())?;
        let members_value = object
            .entry("members")
            .or_insert_with(|| Value::Array(Vec::new()));
        let members_array = members_value
            .as_array_mut()
            .ok_or_else(|| "project members is not a JSON array".to_string())?;
        ensure_default_owner(members_array);

        let mut removed_count = 0usize;
        let mut removed_member_ids = Vec::new();
        members_array.retain(|value| {
            if !is_terminal_member(value) || is_owner_member(value) {
                return true;
            }
            if let Some(id) = value.get("id").and_then(|id| id.as_str()) {
                removed_member_ids.push(id.to_string());
            }
            removed_count += 1;
            false
        });
        ensure_default_owner(members_array);
        reset_member_sequence(object);

        let write_result = project_data::write_project_data(
            self.storage,
            workspace_path,
            workspace_id,
            false,
            payload,
        )?;
        if let Some(write_warning) = write_result.warning {
            warning = Some(match warning {
                Some(existing) => format!("{existing}; {write_warning}"),
                None => write_warning,
            });
        }
        Ok(ProjectMemberPurgeResult {
            removed_count,
            removed_member_ids,
            warning,
        })
    }
}

fn ensure_default_owner(members: &mut Vec<Value>) {
    let has_owner = members.iter().any(|value| {
        value
            .get("id")
            .and_then(|id| id.as_str())
            .map(|id| id == DEFAULT_OWNER_ID)
            .unwrap_or(false)
    });
    if has_owner {
        return;
    }
    members.push(json!({
        "id": DEFAULT_OWNER_ID,
        "name": DEFAULT_OWNER_NAME,
        "role": "",
        "roleKey": DEFAULT_OWNER_ROLE_KEY,
        "roleType": DEFAULT_OWNER_ROLE_TYPE,
        "avatar": default_avatar(),
        "status": DEFAULT_MEMBER_STATUS,
    }));
}

fn normalize_instance_count(value: u32) -> u32 {
    if value == 0 {
        1
    } else {
        value
    }
}

fn extract_member_names(members: &[Value]) -> Vec<String> {
    members
        .iter()
        .filter_map(|value| {
            value
                .get("name")
                .and_then(|name| name.as_str())
                .map(|name| name.to_string())
        })
        .collect()
}

fn ensure_member_sequence(object: &mut Map<String, Value>) -> Result<&mut Map<String, Value>, String> {
    let value = object
        .entry(MEMBER_SEQUENCE_KEY)
        .or_insert_with(|| Value::Object(Map::new()));
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
    value
        .as_object_mut()
        .ok_or_else(|| "project member sequence is not a JSON object".to_string())
}

fn read_member_sequence_snapshot(object: &Map<String, Value>) -> Map<String, Value> {
    object
        .get(MEMBER_SEQUENCE_KEY)
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default()
}

fn reset_member_sequence(object: &mut serde_json::Map<String, Value>) {
    object.insert(
        MEMBER_SEQUENCE_KEY.to_string(),
        Value::Object(serde_json::Map::new()),
    );
}

fn is_terminal_member(value: &Value) -> bool {
    let obj = match value.as_object() {
        Some(obj) => obj,
        None => return false,
    };
    let has_value = |key: &str| {
        obj.get(key)
            .and_then(|value| value.as_str())
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    };
    has_value("terminalType") || has_value("terminalCommand") || has_value("terminalPath")
}

fn is_owner_member(value: &Value) -> bool {
    let obj = match value.as_object() {
        Some(obj) => obj,
        None => return false,
    };
    if obj
        .get("roleType")
        .and_then(|value| value.as_str())
        .map(|value| value == DEFAULT_OWNER_ROLE_TYPE)
        .unwrap_or(false)
    {
        return true;
    }
    obj.get("roleKey")
        .and_then(|value| value.as_str())
        .map(|value| value == DEFAULT_OWNER_ROLE_KEY)
        .unwrap_or(false)
}

fn build_member_base_name(workspace_name: &str, role_type: &str, terminal_label: &str) -> String {
    let workspace = normalize_name_segment(workspace_name, DEFAULT_WORKSPACE_NAME);
    let terminal = normalize_name_segment(terminal_label, "terminal");
    format!("{workspace}-{role_type}-{terminal}")
}

fn resolve_member_sequence_start(
    base_name: &str,
    roster_names: &[String],
    member_sequence: &Map<String, Value>,
) -> usize {
    let derived = next_member_index(base_name, roster_names);
    // 优先使用持久化序号，异常/落后时回退扫描结果，保证序号单调递增。
    let stored = member_sequence
        .get(base_name)
        .and_then(parse_member_sequence)
        .unwrap_or(0);
    if stored == 0 {
        derived
    } else if stored < derived {
        derived
    } else {
        stored
    }
}

fn parse_member_sequence(value: &Value) -> Option<usize> {
    let number = value.as_u64()?;
    usize::try_from(number).ok()
}

fn store_member_sequence(
    member_sequence: &mut Map<String, Value>,
    base_name: &str,
    next_index: usize,
) {
    member_sequence.insert(
        base_name.to_string(),
        Value::Number(serde_json::Number::from(next_index as u64)),
    );
}

fn build_seeded_avatar(seed: &str) -> String {
    let preset = pick_avatar_preset_id(seed);
    format!("{CSS_AVATAR_PREFIX}{preset}")
}

fn pick_avatar_preset_id(seed: &str) -> &'static str {
    if seed.is_empty() {
        return AVATAR_PRESET_IDS[0];
    }
    let hash = hash_seed(seed);
    let index = (hash % AVATAR_PRESET_IDS.len() as i64) as usize;
    AVATAR_PRESET_IDS[index]
}

fn hash_seed(seed: &str) -> i64 {
    let mut hash: i32 = 0;
    for ch in seed.chars() {
        let code = ch as i32;
        hash = hash
            .wrapping_shl(5)
            .wrapping_sub(hash)
            .wrapping_add(code);
    }
    let hash_i64 = hash as i64;
    hash_i64.abs()
}

fn default_avatar() -> String {
    format!("{CSS_AVATAR_PREFIX}{}", AVATAR_PRESET_IDS[0])
}

fn normalize_name_segment(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return fallback.to_string();
    }
    let mut result = String::new();
    let mut last_separator = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !last_separator {
                result.push('_');
                last_separator = true;
            }
            continue;
        }
        for lower in ch.to_lowercase() {
            result.push(lower);
        }
        last_separator = false;
    }
    let normalized = result.trim_matches('_');
    if normalized.is_empty() {
        fallback.to_string()
    } else {
        normalized.to_string()
    }
}

fn next_member_index(base: &str, names: &[String]) -> usize {
    let base_lower = base.to_lowercase();
    let prefix = format!("{base_lower}-");
    let mut max_index = 0usize;
    for name in names {
        let lower = name.to_lowercase();
        if !lower.starts_with(&prefix) {
            continue;
        }
        let suffix = &lower[prefix.len()..];
        if suffix.is_empty() || !suffix.chars().all(|ch| ch.is_ascii_digit()) {
            continue;
        }
        if let Ok(value) = suffix.parse::<usize>() {
            if value > max_index {
                max_index = value;
            }
        }
    }
    max_index + 1
}
