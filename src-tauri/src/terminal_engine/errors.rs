//! 终端错误编码：统一错误码与序列化格式，便于前端按码翻译展示。

use serde::Serialize;

#[derive(Clone, Copy, Debug)]
pub(crate) enum TerminalErrorCode {
  ShellBinaryNotFound,
  ShellLaunchFailed,
  TerminalLaunchFailed,
}

impl TerminalErrorCode {
  pub(crate) const fn as_str(self) -> &'static str {
    match self {
      TerminalErrorCode::ShellBinaryNotFound => "terminal.shell.binary_not_found",
      TerminalErrorCode::ShellLaunchFailed => "terminal.shell.launch_failed",
      TerminalErrorCode::TerminalLaunchFailed => "terminal.launch_failed",
    }
  }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TerminalErrorDetail {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) path: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct TerminalError {
  pub(crate) code: TerminalErrorCode,
  pub(crate) message: String,
  pub(crate) detail: Option<TerminalErrorDetail>,
}

#[derive(Serialize)]
struct TerminalErrorPayload<'a> {
  code: &'a str,
  message: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  detail: Option<&'a TerminalErrorDetail>,
}

impl TerminalError {
  pub(crate) fn new(code: TerminalErrorCode, message: impl Into<String>) -> Self {
    Self {
      code,
      message: message.into(),
      detail: None,
    }
  }

  pub(crate) fn with_path(
    code: TerminalErrorCode,
    message: impl Into<String>,
    path: Option<String>,
  ) -> Self {
    Self {
      code,
      message: message.into(),
      detail: Some(TerminalErrorDetail { path }),
    }
  }

  pub(crate) fn to_invoke_error(&self) -> String {
    let payload = TerminalErrorPayload {
      code: self.code.as_str(),
      message: self.message.as_str(),
      detail: self.detail.as_ref(),
    };
    serde_json::to_string(&payload).unwrap_or_else(|_| {
      format!("{}: {}", self.code.as_str(), self.message.as_str())
    })
  }
}
