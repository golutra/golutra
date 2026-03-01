//! 会话级过滤运行时：保存 profile，便于后续扩展流式状态。

use super::profiles;
use super::registry::resolve_profile;
use super::types::{FilterContext, FilterResult, TerminalFilterProfile};

pub(crate) struct FilterRuntime {
  profile: TerminalFilterProfile,
}

impl FilterRuntime {
  pub(crate) fn new(terminal_type: &str) -> Self {
    Self {
      profile: resolve_profile(terminal_type),
    }
  }

  pub(crate) fn apply_snapshot(
    &self,
    context: &FilterContext<'_>,
    lines: &[String],
  ) -> FilterResult {
    profiles::apply_snapshot(self.profile, context, lines)
  }
}
