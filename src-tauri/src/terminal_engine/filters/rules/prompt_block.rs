//! 基于提示符分段的对话块提取规则。

use super::super::types::FilterDecision;

pub(crate) struct PromptBlockResult {
  pub(crate) decision: FilterDecision,
  pub(crate) reason: &'static str,
  pub(crate) lines: Option<Vec<String>>,
}

/// 规则：按提示符定位对话块，保留块内最近 N 条 bullet 段落。
/// - 若缺少完整边界（只看到一个符合条件的 `›`），返回 Defer 等待下一次刷新。
/// - 若块内没有命中 `•`/`✦`，返回 Drop。
/// - 命中后从“最近 N 条 bullet 的第一条”开始截取，直到下一条 `›` 之前的块末尾。
/// - 提取时会去掉 `•/✦` 及其后的空格前缀，其余内容保持原样。
pub(crate) fn extract_recent_bullet_block(
  lines: &[String],
  input_lines: Option<&[String]>,
  max_bullets: usize,
) -> PromptBlockResult {
  let max_bullets = max_bullets.max(1);
  extract_bullet_block(lines, input_lines, max_bullets)
}

/// 规则：先用 `›` 到首个 `•/✦` 的内容匹配输入，再用该 `›` 到下一条 `›` 作为分段边界。
/// - 若缺少完整边界（只看到一个符合条件的 `›`），返回 Defer 等待下一次刷新。
/// - 若块内没有命中 `•`/`✦`，返回 Drop。
/// - 命中后仅保留最后一段 bullet（从最后一条 `•/✦` 开始到块末尾）。
/// - 提取时会去掉 `•/✦` 及其后的空格前缀，其余内容保持原样。
pub(crate) fn extract_last_bullet_block(
  lines: &[String],
  input_lines: Option<&[String]>,
) -> PromptBlockResult {
  extract_bullet_block(lines, input_lines, 1)
}

#[derive(Clone, Copy)]
struct PromptMatch {
  start: usize,
  end: usize,
}

fn find_prompt_matches_any(lines: &[String]) -> Vec<PromptMatch> {
  let mut matches = Vec::new();
  for (index, line) in lines.iter().enumerate() {
    if is_non_empty_prompt(line) {
      matches.push(PromptMatch { start: index, end: index });
    }
  }
  matches
}

fn find_prompt_block_by_input(
  lines: &[String],
  input_lines: &[String],
) -> Option<(PromptMatch, PromptMatch)> {
  if input_lines.is_empty() {
    return None;
  }
  let prompt_indices = find_prompt_matches_any(lines);
  if prompt_indices.len() < 2 {
    return None;
  }
  // 从尾部往前找最近的匹配块，避免把旧输入错误匹配到当前输出。
  for idx in (0..prompt_indices.len() - 1).rev() {
    let start = prompt_indices[idx].start;
    let end = prompt_indices[idx + 1].start;
    if end <= start + 1 {
      continue;
    }
    let block = &lines[start + 1..end];
    let Some(first_bullet_offset) = block.iter().position(|line| is_bullet_line(line)) else {
      continue;
    };
    let segment_end = start + 1 + first_bullet_offset;
    if segment_end <= start {
      continue;
    }
    let segment = &lines[start..segment_end];
    if matches_input_segment(segment, input_lines) {
      let start_match = PromptMatch { start, end: start };
      let end_match = PromptMatch { start: end, end };
      return Some((start_match, end_match));
    }
  }
  None
}

fn match_input_from_prompt(
  lines: &[String],
  start: usize,
  input_lines: &[String],
) -> Option<usize> {
  if input_lines.is_empty() {
    return None;
  }
  let mut non_empty = 0;
  for end in start..lines.len() {
    let line = lines[end].trim_end();
    if end > start && (is_non_empty_prompt(line) || is_bullet_line(line)) {
      return None;
    }
    if !line.trim().is_empty() {
      non_empty += 1;
    }
    if non_empty < input_lines.len() {
      continue;
    }
    let segment = &lines[start..=end];
    if matches_input_segment(segment, input_lines) {
      return Some(end);
    }
    return None;
  }
  None
}

fn find_prompt_match_by_input_tail(
  lines: &[String],
  input_lines: &[String],
) -> Option<usize> {
  for idx in (0..lines.len()).rev() {
    if !is_non_empty_prompt(lines[idx].as_str()) {
      continue;
    }
    if match_input_from_prompt(lines, idx, input_lines).is_some() {
      return Some(idx);
    }
  }
  None
}

fn extract_bullet_block_before_prompt(
  lines: &[String],
  prompt_index: usize,
) -> Option<Vec<String>> {
  if prompt_index == 0 {
    return None;
  }
  let mut block_end = None;
  let mut block_start = None;
  // 只提取紧贴当前 prompt 之前的最近一段 bullet，避免误拿更早的回复。
  let mut idx = prompt_index;
  while idx > 0 {
    idx -= 1;
    let line = lines[idx].trim_end();
    if line.trim().is_empty() {
      if block_end.is_some() {
        continue;
      }
      continue;
    }
    if is_bullet_line(line) {
      if block_end.is_none() {
        block_end = Some(idx);
      }
      block_start = Some(idx);
      continue;
    }
    break;
  }
  let (block_start, block_end) = match (block_start, block_end) {
    (Some(start), Some(end)) => (start, end),
    _ => return None,
  };
  let mut extracted = Vec::new();
  for line in lines[block_start..=block_end].iter() {
    if line.trim().is_empty() {
      continue;
    }
    if is_bullet_line(line) {
      extracted.push(strip_bullet_prefix(line));
    } else {
      extracted.push(line.clone());
    }
  }
  if extracted.is_empty() {
    None
  } else {
    Some(extracted)
  }
}

fn matches_input_segment(segment: &[String], input_lines: &[String]) -> bool {
  // 只允许空格前缀，忽略空行，保持与输入提取规则一致。
  if input_lines.is_empty() {
    return false;
  }
  let mut normalized = Vec::new();
  for (index, line) in segment.iter().enumerate() {
    let normalized_line = if index == 0 {
      let trimmed = line.trim_start();
      if !trimmed.starts_with('›') {
        return false;
      }
      let rest = trimmed.trim_start_matches('›');
      rest.trim_start_matches(' ')
    } else {
      line.trim_start_matches(' ')
    };
    if normalized_line.trim().is_empty() {
      continue;
    }
    normalized.push(normalized_line.trim_end().to_string());
  }
  if normalized.len() != input_lines.len() {
    return false;
  }
  for (actual, expected) in normalized.iter().zip(input_lines.iter()) {
    let expected = expected.trim();
    if expected.is_empty() || actual.trim_end() != expected {
      return false;
    }
  }
  true
}

fn is_non_empty_prompt(line: &str) -> bool {
  let trimmed = line.trim_start();
  if !trimmed.starts_with('›') {
    return false;
  }
  let rest = trimmed.trim_start_matches('›').trim();
  !rest.is_empty()
}

fn is_bullet_line(line: &str) -> bool {
  let trimmed = line.trim_start();
  trimmed.starts_with('•') || trimmed.starts_with('✦')
}

fn strip_bullet_prefix(line: &str) -> String {
  let mut first_non_space = None;
  for (index, ch) in line.char_indices() {
    if !ch.is_whitespace() {
      first_non_space = Some((index, ch));
      break;
    }
  }
  let Some((index, ch)) = first_non_space else {
    return line.to_string();
  };
  if ch != '•' && ch != '✦' {
    return line.to_string();
  }
  let after_bullet = &line[index + ch.len_utf8()..];
  let after_bullet = after_bullet.trim_start_matches(|ch: char| ch.is_whitespace());
  let mut result = String::with_capacity(line.len().saturating_sub(ch.len_utf8()));
  result.push_str(&line[..index]);
  result.push_str(after_bullet);
  result
}

fn extract_bullet_block(
  lines: &[String],
  input_lines: Option<&[String]>,
  max_bullets: usize,
) -> PromptBlockResult {
  let (start_match, end_match) = if let Some(input_lines) = input_lines {
    if input_lines.is_empty() {
      return PromptBlockResult {
        decision: FilterDecision::Defer,
        reason: "prompt-start-missing",
        lines: None,
      };
    }
    // 输入已知时：先在提示符到首个 bullet 的区域匹配输入，再用下一条提示符截断。
    let Some((start_match, end_match)) = find_prompt_block_by_input(lines, input_lines) else {
      if let Some(prompt_index) = find_prompt_match_by_input_tail(lines, input_lines) {
        if let Some(extracted) = extract_bullet_block_before_prompt(lines, prompt_index) {
          return PromptBlockResult {
            decision: FilterDecision::Allow,
            reason: "prompt-input-tail",
            lines: Some(extracted),
          };
        }
      }
      return PromptBlockResult {
        decision: FilterDecision::Defer,
        reason: "prompt-input-unmatched",
        lines: None,
      };
    };
    (start_match, end_match)
  } else {
    let matches = find_prompt_matches_any(lines);
    if matches.len() < 2 {
      return PromptBlockResult {
        decision: FilterDecision::Defer,
        reason: "prompt-boundary-missing",
        lines: None,
      };
    }
    (matches[matches.len() - 2], matches[matches.len() - 1])
  };
  if end_match.start <= start_match.end {
    return PromptBlockResult {
      decision: FilterDecision::Drop,
      reason: "prompt-block-empty",
      lines: None,
    };
  }
  let block_start = start_match.end + 1;
  let block_end = end_match.start;
  if block_end <= block_start {
    return PromptBlockResult {
      decision: FilterDecision::Drop,
      reason: "prompt-block-empty",
      lines: None,
    };
  }
  let block = &lines[block_start..block_end];
  let mut bullet_indices = Vec::new();
  for (index, line) in block.iter().enumerate() {
    if is_bullet_line(line) {
      bullet_indices.push(index);
    }
  }
  let Some(last_index) = bullet_indices.last().copied() else {
    return PromptBlockResult {
      decision: FilterDecision::Drop,
      reason: "prompt-block-no-bullet",
      lines: None,
    };
  };
  let start_index = if bullet_indices.len() >= max_bullets {
    bullet_indices[bullet_indices.len().saturating_sub(max_bullets)]
  } else {
    last_index
  };
  let mut extracted = Vec::with_capacity(block.len().saturating_sub(start_index));
  for line in block[start_index..].iter() {
    if is_bullet_line(line) {
      extracted.push(strip_bullet_prefix(line));
    } else {
      extracted.push(line.clone());
    }
  }
  PromptBlockResult {
    decision: FilterDecision::Allow,
    reason: "prompt-block-bullet-window",
    lines: Some(extracted),
  }
}
