//! 通用过滤：默认不改动内容。

use super::super::rules::prompt_block::{extract_last_bullet_block, extract_recent_bullet_block};
use super::super::types::{FilterContext, FilterDecision, FilterMode, FilterResult, TerminalFilterProfile};

pub(crate) fn apply_snapshot(
  profile: TerminalFilterProfile,
  _context: &FilterContext<'_>,
  lines: &[String],
) -> FilterResult {
  let result = match _context.mode {
    FilterMode::Stream => extract_recent_bullet_block(lines, _context.last_input_lines, 2),
    FilterMode::Final => extract_last_bullet_block(lines, _context.last_input_lines),
  };
  match result.decision {
    FilterDecision::Allow => {
      let lines = result.lines.unwrap_or_else(|| lines.to_vec());
      FilterResult::allow_with_lines(profile, result.reason, lines)
    }
    FilterDecision::Drop => FilterResult {
      decision: FilterDecision::Drop,
      reason: result.reason,
      profile,
      lines: None,
    },
    FilterDecision::Defer => FilterResult {
      decision: FilterDecision::Defer,
      reason: result.reason,
      profile,
      lines: None,
    },
  }
}
