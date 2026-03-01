//! 终端会话映射存储：维护 member_id 与远端 session_id 的一一对应关系。

use redb::ReadableTable;

use super::store::{decode, encode, open_db, TERMINAL_SESSION_INDEX, TERMINAL_SESSION_MAP};
use super::types::{TerminalSessionIndexEntry, TerminalSessionMapEntry};
use super::ChatDbManager;

pub fn terminal_session_upsert(
    state: &ChatDbManager,
    workspace_id: &str,
    member_id: &str,
    session_id: &str,
) -> Result<TerminalSessionMapEntry, String> {
    let session_id = session_id.trim();
    if session_id.is_empty() {
        return Err("session_id is required".to_string());
    }
    let member_id = member_id.trim();
    if member_id.is_empty() {
        return Err("member_id is required".to_string());
    }
    let db = open_db(state, workspace_id)?;
    let txn = db
        .begin_write()
        .map_err(|err| format!("failed to open terminal session write transaction: {err}"))?;
    let mut map_table = txn
        .open_table(TERMINAL_SESSION_MAP)
        .map_err(|err| format!("failed to open terminal_session_map: {err}"))?;
    let mut index_table = txn
        .open_table(TERMINAL_SESSION_INDEX)
        .map_err(|err| format!("failed to open terminal_session_index: {err}"))?;

    let existing = map_table
        .get(member_id)
        .map_err(|err| format!("failed to read terminal_session_map: {err}"))?
        .map(|entry| decode::<TerminalSessionMapEntry>(entry.value()))
        .transpose()?;

    if let Some(entry) = existing.as_ref() {
        if entry.session_id != session_id {
            let _ = index_table
                .remove(entry.session_id.as_str())
                .map_err(|err| format!("failed to remove terminal_session_index: {err}"))?;
        }
    }

    let index_existing = index_table
        .get(session_id)
        .map_err(|err| format!("failed to read terminal_session_index: {err}"))?
        .map(|entry| decode::<TerminalSessionIndexEntry>(entry.value()))
        .transpose()?;
    if let Some(index_entry) = index_existing.as_ref() {
        if index_entry.member_id != member_id {
            let _ = map_table
                .remove(index_entry.member_id.as_str())
                .map_err(|err| format!("failed to remove terminal_session_map: {err}"))?;
        }
    }

    let entry = TerminalSessionMapEntry {
        member_id: member_id.to_string(),
        session_id: session_id.to_string(),
    };
    let index_entry = TerminalSessionIndexEntry {
        member_id: member_id.to_string(),
    };

    map_table
        .insert(member_id, encode(&entry)?.as_slice())
        .map_err(|err| format!("failed to write terminal_session_map: {err}"))?;
    index_table
        .insert(session_id, encode(&index_entry)?.as_slice())
        .map_err(|err| format!("failed to write terminal_session_index: {err}"))?;
    drop(map_table);
    drop(index_table);
    txn.commit()
        .map_err(|err| format!("failed to commit terminal session map: {err}"))?;
    Ok(entry)
}

pub fn terminal_session_delete_by_member_id(
    state: &ChatDbManager,
    workspace_id: &str,
    member_id: &str,
) -> Result<(), String> {
    let db = open_db(state, workspace_id)?;
    let txn = db
        .begin_write()
        .map_err(|err| format!("failed to open terminal session write transaction: {err}"))?;
    let mut map_table = txn
        .open_table(TERMINAL_SESSION_MAP)
        .map_err(|err| format!("failed to open terminal_session_map: {err}"))?;
    let mut index_table = txn
        .open_table(TERMINAL_SESSION_INDEX)
        .map_err(|err| format!("failed to open terminal_session_index: {err}"))?;
    let entry = map_table
        .remove(member_id)
        .map_err(|err| format!("failed to remove terminal_session_map: {err}"))?
        .map(|value| decode::<TerminalSessionMapEntry>(value.value()))
        .transpose()?;
    if let Some(entry) = entry {
        let _ = index_table
            .remove(entry.session_id.as_str())
            .map_err(|err| format!("failed to remove terminal_session_index: {err}"))?;
    }
    drop(map_table);
    drop(index_table);
    txn.commit()
        .map_err(|err| format!("failed to commit terminal session delete: {err}"))?;
    Ok(())
}

/// 查询 member_id 对应的 session_id。
pub fn terminal_session_get_by_member_id(
    state: &ChatDbManager,
    workspace_id: &str,
    member_id: &str,
) -> Result<Option<String>, String> {
    let db = open_db(state, workspace_id)?;
    let txn = db
        .begin_read()
        .map_err(|err| format!("failed to open terminal session read transaction: {err}"))?;
    let table = txn
        .open_table(TERMINAL_SESSION_MAP)
        .map_err(|err| format!("failed to open terminal_session_map: {err}"))?;
    let entry = table
        .get(member_id)
        .map_err(|err| format!("failed to read terminal_session_map: {err}"))?
        .map(|entry| decode::<TerminalSessionMapEntry>(entry.value()))
        .transpose()?;
    Ok(entry.map(|e| e.session_id))
}
