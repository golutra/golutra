//! 快照服务层：集中处理终端快照获取与基础归一化。

use super::state::TerminalSnapshot;
use crate::terminal_engine::emulator::SnapshotMetrics;
use crate::platform::backend_passive_enabled;

pub(super) fn snapshot_lines(snapshot: &TerminalSnapshot) -> Vec<String> {
  let lines = snapshot.snapshot_lines();
  normalize_lines(lines)
}

pub(super) struct SnapshotTextPayload {
  pub(super) data: String,
  pub(super) history: Option<String>,
  pub(super) metrics: SnapshotMetrics,
}

pub(super) fn snapshot_text_with_history(snapshot: &TerminalSnapshot) -> SnapshotTextPayload {
  let segments = snapshot.snapshot_segments();
  let data = String::from_utf8_lossy(&segments.data).to_string();
  let history = segments.history.map(|raw| String::from_utf8_lossy(&raw).to_string());
  let data = normalize_text(data);
  let history = history
    .map(normalize_text)
    .and_then(|value| if value.is_empty() { None } else { Some(value) });
  SnapshotTextPayload {
    data,
    history,
    metrics: segments.metrics,
  }
}

pub(super) fn normalize_lines(lines: Vec<String>) -> Vec<String> {
  // [TODO/terminal, 2026-01-26] 补齐 ANSI 清理与空行归一化逻辑，避免快照噪音影响语义。
  lines
}

pub(super) fn normalize_text(text: String) -> String {
  // [TODO/terminal, 2026-01-26] 统一 ANSI 清理与空行归一化规则，并与行级快照保持一致。
  text
}

pub(super) fn merge_semantic_lines(lines: &[String], _terminal_cols: u16) -> Vec<String> {
  let mut merged: Vec<String> = Vec::new();
  let mut current = String::new();
  let mut in_code_block = false;
  for raw in lines {
    let text = raw.trim_end();
    let trimmed = text.trim();
    if trimmed.is_empty() {
      if !current.is_empty() {
        merged.push(std::mem::take(&mut current));
      }
      merged.push(String::new());
      continue;
    }
    if is_code_fence_line(trimmed) {
      if !current.is_empty() {
        merged.push(std::mem::take(&mut current));
      }
      merged.push(text.to_string());
      in_code_block = !in_code_block;
      continue;
    }
    if in_code_block {
      if !current.is_empty() {
        merged.push(std::mem::take(&mut current));
      }
      merged.push(text.to_string());
      continue;
    }
    if is_hard_break_line(text, trimmed) {
      if !current.is_empty() {
        merged.push(std::mem::take(&mut current));
      }
      merged.push(text.to_string());
      continue;
    }
    if current.is_empty() {
      current.push_str(text);
    } else {
      current.push_str(text.trim_start());
    }
  }
  if !current.is_empty() {
    merged.push(current);
  }
  if backend_passive_enabled() && log::log_enabled!(log::Level::Info) {
    let merged_text = merged.join("\n");
    log::info!(
      "terminal_semantic_merge line_count={} content={}",
      merged.len(),
      merged_text
    );
  }
  merged
}

fn is_hard_break_line(raw: &str, trimmed: &str) -> bool {
  is_indented_code_line(raw)
    || is_prompt_line(trimmed)
    || is_list_line(trimmed)
    || is_table_line(trimmed)
    || is_separator_line(trimmed)
    || is_paragraph_start_line(trimmed)
}

fn is_code_fence_line(line: &str) -> bool {
  line.starts_with("```") || line.starts_with("~~~")
}

fn is_prompt_line(line: &str) -> bool {
  let trimmed = line.trim_start();
  if trimmed.is_empty() {
    return false;
  }
  if trimmed.starts_with('›') {
    return true;
  }
  if trimmed == ">" || trimmed.starts_with(">>>") || trimmed.starts_with(">> ") {
    return true;
  }
  if trimmed == "$" || trimmed.starts_with("$ ") {
    return true;
  }
  if trimmed.starts_with("PS ") && trimmed.ends_with('>') {
    return true;
  }
  false
}

fn is_list_line(line: &str) -> bool {
  let trimmed = line.trim_start();
  if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
    return true;
  }
  is_numbered_list_line(trimmed)
}

fn is_table_line(line: &str) -> bool {
  let trimmed = line.trim();
  if trimmed.is_empty() {
    return false;
  }
  if trimmed.starts_with('+') && trimmed.ends_with('+') && trimmed.contains('-') {
    return true;
  }
  let bytes = trimmed.as_bytes();
  if bytes.len() < 2 {
    return false;
  }
  if bytes[0] == b'|' && bytes[bytes.len() - 1] == b'|' {
    return bytes[1..bytes.len().saturating_sub(1)]
      .iter()
      .any(|value| *value == b'|');
  }
  false
}

fn is_separator_line(line: &str) -> bool {
  let trimmed = line.trim();
  if trimmed.len() < 3 {
    return false;
  }
  trimmed
    .chars()
    .all(|ch| ch == '-' || ch == '=' || ch == '_' || ch == '*' || ch == '.')
}

fn is_paragraph_start_line(line: &str) -> bool {
  let trimmed = line.trim_start();
  if trimmed.starts_with('#') {
    return true;
  }
  if trimmed.starts_with('>') {
    return true;
  }
  false
}

fn is_numbered_list_line(line: &str) -> bool {
  let bytes = line.as_bytes();
  let mut index = 0usize;
  while index < bytes.len() && bytes[index].is_ascii_digit() {
    index += 1;
  }
  if index == 0 || index > 2 {
    return false;
  }
  if index + 2 > bytes.len() {
    return false;
  }
  let marker = bytes[index];
  if marker != b'.' && marker != b')' {
    return false;
  }
  if bytes[index + 1] != b' ' {
    return false;
  }
  index + 2 < bytes.len()
}

fn is_indented_code_line(line: &str) -> bool {
  let bytes = line.as_bytes();
  if bytes.is_empty() {
    return false;
  }
  if bytes[0] == b'\t' {
    return true;
  }
  let mut count = 0usize;
  for value in bytes {
    if *value == b' ' {
      count = count.saturating_add(1);
    } else {
      break;
    }
  }
  count >= 4
}
