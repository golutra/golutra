//! Profile 入口：按 CLI 类型选择规则集合。

pub(crate) mod generic;

use super::types::{FilterContext, FilterResult, TerminalFilterProfile};

pub(crate) fn apply_snapshot(
  profile: TerminalFilterProfile,
  context: &FilterContext<'_>,
  lines: &[String],
) -> FilterResult {
  generic::apply_snapshot(profile, context, lines)
}
