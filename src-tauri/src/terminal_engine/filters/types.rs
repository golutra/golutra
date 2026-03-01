//! 过滤类型定义：保持轻量，避免规则与状态交织。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FilterSource {
  Snapshot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FilterMode {
  Stream,
  Final,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TerminalFilterProfile {
  Generic,
  Codex,
  Gemini,
  Claude,
  Shell,
}

impl TerminalFilterProfile {
  pub(crate) fn as_str(&self) -> &'static str {
    match self {
      TerminalFilterProfile::Generic => "generic",
      TerminalFilterProfile::Codex => "codex",
      TerminalFilterProfile::Gemini => "gemini",
      TerminalFilterProfile::Claude => "claude",
      TerminalFilterProfile::Shell => "shell",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FilterDecision {
  Allow,
  #[allow(dead_code)]
  Drop,
  #[allow(dead_code)]
  Defer,
}

impl FilterDecision {
  pub(crate) fn as_str(&self) -> &'static str {
    match self {
      FilterDecision::Allow => "allow",
      FilterDecision::Drop => "drop",
      FilterDecision::Defer => "defer",
    }
  }
}

#[allow(dead_code)]
pub(crate) struct FilterContext<'a> {
  pub(crate) terminal_id: &'a str,
  pub(crate) terminal_type: &'a str,
  pub(crate) last_command: Option<&'a str>,
  pub(crate) last_input_lines: Option<&'a [String]>,
  pub(crate) now_ms: u64,
  pub(crate) source: FilterSource,
  pub(crate) mode: FilterMode,
}

pub(crate) struct FilterResult {
  pub(crate) decision: FilterDecision,
  pub(crate) reason: &'static str,
  pub(crate) profile: TerminalFilterProfile,
  pub(crate) lines: Option<Vec<String>>,
}

impl FilterResult {
  pub(crate) fn allow_with_lines(
    profile: TerminalFilterProfile,
    reason: &'static str,
    lines: Vec<String>,
  ) -> Self {
    Self {
      decision: FilterDecision::Allow,
      reason,
      profile,
      lines: Some(lines),
    }
  }
}
