//! 终端 IPC 载荷模型：定义事件与快照对外传输的序列化格式。
//! 边界：仅承载数据，不包含业务逻辑或持久化策略。

use serde::Serialize;

#[derive(Serialize, Clone)]
/// 终端输出事件载荷，用于前端按序增量渲染。
/// 约束：`seq` 对同一会话单调递增；`data` 已过滤控制序列以避免 UI 噪声。
pub(crate) struct TerminalOutputPayload {
  #[serde(rename = "terminalId")]
  pub(crate) terminal_id: String,
  pub(crate) data: String,
  pub(crate) seq: u64,
}

#[derive(Serialize, Clone)]
/// 进程退出事件载荷。
/// 约束：`code`/`signal` 可能为空，用于跨平台统一退出语义。
pub(crate) struct TerminalExitPayload {
  #[serde(rename = "terminalId")]
  pub(crate) terminal_id: String,
  pub(crate) code: Option<i32>,
  pub(crate) signal: Option<String>,
}

#[derive(Serialize, Clone)]
/// 会话状态变化载荷，用于驱动 UI 的活动指示与成员状态同步。
/// 约束：`status` 目前是字符串枚举，需与前端状态机保持一致。
pub(crate) struct TerminalStatusPayload {
  #[serde(rename = "terminalId")]
  pub(crate) terminal_id: String,
  pub(crate) status: String,
  #[serde(rename = "memberId")]
  pub(crate) member_id: Option<String>,
  #[serde(rename = "workspaceId")]
  pub(crate) workspace_id: Option<String>,
}

#[derive(Serialize, Clone)]
/// 会话快照载荷，用于 attach 时一次性恢复可视内容。
/// 约束：`history` 仅在后端提供额外缓冲时出现。
pub(crate) struct TerminalSnapshotPayload {
  #[serde(rename = "terminalId")]
  pub(crate) terminal_id: String,
  pub(crate) data: String,
  pub(crate) seq: u64,
  pub(crate) rows: u16,
  pub(crate) cols: u16,
  #[serde(rename = "cursorRow")]
  pub(crate) cursor_row: u16,
  #[serde(rename = "cursorCol")]
  pub(crate) cursor_col: u16,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) history: Option<String>,
}

#[derive(Serialize, Clone)]
/// 会话快照文本行载荷，便于前端做屏幕一致性校验。
pub(crate) struct TerminalSnapshotLinesPayload {
  #[serde(rename = "terminalId")]
  pub(crate) terminal_id: String,
  pub(crate) seq: u64,
  pub(crate) lines: Vec<String>,
}

#[derive(Serialize, Clone)]
/// 会话错误载荷，用于提示 UI 并决定是否禁止输入。
/// 约束：`fatal=true` 代表会话已不可恢复，需要重新打开。
pub(crate) struct TerminalErrorPayload {
  #[serde(rename = "terminalId")]
  pub(crate) terminal_id: String,
  pub(crate) error: String,
  pub(crate) fatal: bool,
}

