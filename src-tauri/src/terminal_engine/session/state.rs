//! 终端会话状态子域：封装会话类型与注册表，避免与 IO 逻辑耦合。

use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::Write,
    sync::{atomic::AtomicUsize, mpsc, Arc, Mutex},
};

use serde::Deserialize;

use super::super::emulator::{
    create_emulator_with_writer, EmulatorConfig, SnapshotSegments, TerminalEmulator,
};
use crate::ports::terminal_dispatch_gate::{
    default_terminal_dispatch_gate, TerminalDispatchGate,
};
use crate::ports::terminal_event::{default_terminal_event_port, TerminalEventPort};
use crate::ports::terminal_message::{default_terminal_message_pipeline, TerminalMessagePipeline};
use crate::ports::terminal_session::{
    default_terminal_session_repository, TerminalSessionRepository,
};
use crate::runtime::TerminalHandle;

use super::trigger::TriggerBus;
use super::{SemanticEvent, SESSION_SCROLLBACK_LINES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 终端状态机：用于 UI 状态指示与成员状态同步。
pub(super) enum TerminalSessionStatus {
    Connecting,
    Online,
    Working,
    Offline,
}

impl TerminalSessionStatus {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            TerminalSessionStatus::Connecting => "connecting",
            TerminalSessionStatus::Online => "online",
            TerminalSessionStatus::Working => "working",
            TerminalSessionStatus::Offline => "offline",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 启动后流程状态：避免稳定检测重复触发。
pub(super) enum PostReadyState {
    Idle,
    Starting,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 启动后流程模式：控制是否执行 post_ready。
pub(super) enum PostReadyMode {
    Invite,
    Disabled,
}

impl PostReadyMode {
    pub(super) fn from_str(value: Option<&str>) -> Self {
        match value.unwrap_or("").trim().to_lowercase().as_str() {
            "invite" => PostReadyMode::Invite,
            _ => PostReadyMode::Disabled,
        }
    }

    pub(super) fn should_run(self) -> bool {
        matches!(self, PostReadyMode::Invite)
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            PostReadyMode::Invite => "invite",
            PostReadyMode::Disabled => "none",
        }
    }
}

/// 启动后注入队列元素：支持按事件选择是否等待画面稳定。
#[derive(Clone, Debug)]
pub(super) enum PostReadyAction {
    Input(String),
    Delay {
        ms: u64,
        started_at: Option<u64>,
    },
    ExtractSessionId {
        keyword: String,
    },
    WaitForPattern {
        pattern: String,
        attempts: u32,
        started_at: Option<u64>,
    },
    Introduction {
        prompt_type: String,
    },
}

#[derive(Clone, Copy, Debug)]
/// 输出速率采样点：用于自适应稳定检测。
pub(super) struct OutputRateSample {
    pub timestamp: u64,
    pub bytes_total: u64,
}

pub(super) struct PostReadyQueueItem {
    pub(super) action: PostReadyAction,
    pub(super) require_stable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 终端类型：决定启动方式与默认二进制。
pub(super) enum TerminalType {
    Shell,
    Codex,
    Gemini,
    Claude,
    Opencode,
    Qwen,
}

impl TerminalType {
    pub(super) fn from_str(value: Option<&str>) -> Option<Self> {
        match value?.trim().to_lowercase().as_str() {
            "shell" => Some(TerminalType::Shell),
            "codex" => Some(TerminalType::Codex),
            "gemini" => Some(TerminalType::Gemini),
            "claude" => Some(TerminalType::Claude),
            "opencode" => Some(TerminalType::Opencode),
            "qwen" => Some(TerminalType::Qwen),
            _ => None,
        }
    }

    pub(super) fn as_str(&self) -> &'static str {
        match self {
            TerminalType::Shell => "shell",
            TerminalType::Codex => "codex",
            TerminalType::Gemini => "gemini",
            TerminalType::Claude => "claude",
            TerminalType::Opencode => "opencode",
            TerminalType::Qwen => "qwen",
        }
    }

    pub(super) fn default_binary(&self) -> Option<&'static str> {
        match self {
            TerminalType::Shell => None,
            TerminalType::Codex => Some("codex"),
            TerminalType::Gemini => Some("gemini"),
            TerminalType::Claude => Some("claude"),
            TerminalType::Opencode => Some("opencode"),
            TerminalType::Qwen => Some("qwen"),
        }
    }
}

// 会话快照持有模拟器，支持在 attach 时回放视图。
pub(super) struct TerminalSnapshot {
    pub(super) emulator: Box<dyn TerminalEmulator>,
}

impl TerminalSnapshot {
    pub(super) fn new(
        rows: u16,
        cols: u16,
        writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
    ) -> Self {
        let response_writer =
            writer.map(|writer| Box::new(PtyResponseWriter { writer }) as Box<dyn Write + Send>);
        let emulator = create_emulator_with_writer(
            EmulatorConfig {
                rows,
                cols,
                scrollback_limit: SESSION_SCROLLBACK_LINES,
            },
            response_writer,
        );
        Self { emulator }
    }

    pub(super) fn apply_output(&mut self, bytes: &[u8]) {
        self.emulator.apply_output(bytes);
    }

    pub(super) fn set_size(&mut self, rows: u16, cols: u16) {
        self.emulator.set_size(rows, cols);
    }

    pub(super) fn snapshot_segments(&self) -> SnapshotSegments {
        self.emulator.snapshot_ansi_segments()
    }

    pub(super) fn snapshot_lines(&self) -> Vec<String> {
        self.emulator.snapshot_lines()
    }

    pub(super) fn cursor_position(&self) -> (u16, u16) {
        self.emulator.cursor_position()
    }
}

// 通过响应写入器捕获终端查询响应，避免丢失 cursor 等信息。
struct PtyResponseWriter {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl Write for PtyResponseWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut writer = self.writer.lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "terminal writer lock poisoned")
        })?;
        writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut writer = self.writer.lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "terminal writer lock poisoned")
        })?;
        writer.flush()
    }
}

/// 终端会话状态：承载 IO 状态、快照与语义通道。
/// 约束：`unacked_bytes` 用于流控，`ui_active` 仅代表前端订阅状态。
pub(crate) struct TerminalSession {
    pub(crate) id: String,
    pub(super) terminal_type: TerminalType,
    pub(super) status: TerminalSessionStatus,
    pub(crate) output_bytes_total: u64,
    pub(crate) output_seq: u64,
    pub(crate) unacked_bytes: usize,
    pub(crate) screen_rows: u16,
    pub(crate) screen_cols: u16,
    pub(crate) member_id: Option<String>,
    pub(crate) member_name: Option<String>,
    pub(crate) workspace_id: Option<String>,
    // 状态门禁：锁定时保持 Connecting，禁止自动切到 Working/Online。
    pub(crate) status_locked: bool,
    pub(crate) active: bool,
    pub(crate) last_activity_at: Option<u64>,
    pub(crate) last_output_at: Option<u64>,
    pub(crate) last_read_at: Option<u64>,
    pub(crate) last_applied_at: Option<u64>,
    // 非进度提示输出的最新时间，用于避免 Working 动画阻塞语义 flush。
    pub(super) idle_candidate_at: Option<u64>,
    // 命令提交后的 Working 意图窗口，避免布局期/背景输出误触发。
    pub(super) working_intent_until: Option<u64>,
    pub(super) chat_pending_since: Option<u64>,
    pub(super) chat_candidate_at: Option<u64>,
    pub(super) semantic_active: bool,
    // chat 流式开关快照：决定是否发送语义流式与增量输出。
    pub(super) chat_stream_enabled: bool,
    pub(super) flow_paused: bool,
    // 为了解决切换标签/attach/resize 误触发 Working，布局期短暂抑制状态触发。
    pub(super) redraw_suppression_until: Option<u64>,
    pub(crate) broken: bool,
    pub(crate) chat_pending: bool,
    pub(super) post_ready_state: PostReadyState,
    pub(super) post_ready_mode: PostReadyMode,
    // 启动后注入队列，按输入事件逐条下发。
    pub(super) post_ready_queue: VecDeque<PostReadyQueueItem>,
    // /status 输出解析出的远端会话 ID。
    pub(super) remote_session_id: Option<String>,
    // post_ready 解析 /status 的超时起点。
    pub(super) post_ready_session_id_started_at: Option<u64>,
    // post_ready 解析 /status 的重启次数。
    pub(super) post_ready_session_id_restart_count: u32,
    // post_ready 解析 /status 触发的重启请求。
    pub(super) post_ready_restart_pending: bool,
    // 会话进程世代号，用于避免旧进程退出覆盖新会话状态。
    pub(super) spawn_epoch: u64,
    pub(crate) ui_active: bool,
    pub(crate) handle: Option<TerminalHandle>,
    pub(super) snapshot: TerminalSnapshot,
    pub(super) semantic_tx: Option<mpsc::Sender<SemanticEvent>>,
    pub(crate) keep_alive: bool,
    pub(crate) launch_cwd: Option<String>,
    pub(crate) launch_command: Option<String>,
    pub(crate) launch_path: Option<String>,
    pub(crate) launch_strict_shell: bool,
    pub(crate) owner_window_label: Option<String>,
    pub(crate) output_window_label: Option<String>,
    pub(crate) shell_ready: bool,
    pub(crate) created_at: u64,
    pub(crate) input_buffer: VecDeque<String>,
    pub(crate) ready_probe_bytes: usize,
    pub(crate) pending_output_chunks: Arc<AtomicUsize>,
    /// 输出速率采样环形缓冲区，用于自适应稳定检测。
    pub(super) output_rate_samples: VecDeque<OutputRateSample>,
    // 终端派发队列：Working/离线时缓存待派发指令。
    pub(super) dispatch_queue: VecDeque<DispatchQueueItem>,
    pub(super) dispatch_inflight: bool,
    pub(super) dispatch_inflight_message_id: Option<String>,
    // 合并派发时记录所有 message_id，用于去重与确认回写。
    pub(super) dispatch_inflight_message_ids: Vec<String>,
    pub(super) dispatch_recent_message_ids: VecDeque<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// 终端派发上下文：用于将终端输出回写到指定聊天会话。
pub(crate) struct TerminalDispatchContext {
    pub(crate) conversation_id: String,
    pub(crate) conversation_type: String,
    pub(crate) sender_id: String,
    pub(crate) sender_name: String,
    #[serde(rename = "messageId")]
    pub(crate) message_id: Option<String>,
    #[serde(rename = "clientTraceId")]
    pub(crate) client_trace_id: Option<String>,
    #[serde(rename = "timestamp")]
    pub(crate) client_timestamp: Option<u64>,
}

#[derive(Clone, Debug)]
/// 终端派发包：用于队列化输入与上下文。
pub(crate) struct TerminalDispatchEnvelope {
    pub(crate) text: String,
    pub(crate) context: TerminalDispatchContext,
    // 合并派发时携带的 message_id 列表，便于去重与回写确认。
    pub(crate) batched_message_ids: Vec<String>,
}

#[derive(Clone, Debug)]
pub(super) struct DispatchQueueItem {
    pub(super) envelope: TerminalDispatchEnvelope,
    pub(super) enqueued_at: u64,
}

// 进程内会话注册表，集中管理 session 状态与成员状态。
pub(crate) struct SessionRegistry {
    pub(super) sessions: HashMap<String, TerminalSession>,
    pub(super) member_statuses: HashMap<String, String>,
    pub(super) member_sessions: HashMap<String, String>,
    pub(super) pending_status_locks: HashSet<String>,
    // 触发总线仅由调度器注入，避免业务层直接依赖调度实现。
    pub(super) trigger_bus: Option<Arc<TriggerBus>>,
    // 仅追踪 Working 会话，避免状态回落时全量扫描；仅供 Guardian 规则使用。
    pub(super) working_sessions: Arc<Mutex<HashSet<String>>>,
}

/// 终端会话管理器：Tauri State 持有的全局入口。
pub(crate) struct TerminalManager {
    pub(super) sessions: Arc<Mutex<SessionRegistry>>,
    // 终端消息端口：从组合根注入，避免引擎直接依赖消息服务。
    message_pipeline: Mutex<Arc<dyn TerminalMessagePipeline>>,
    // 终端事件端口：从组合根注入，避免引擎直接依赖 Tauri IPC。
    event_port: Mutex<Arc<dyn TerminalEventPort>>,
    // 终端派发门禁：语义 flush 完成后释放派发队列。
    dispatch_gate: Mutex<Arc<dyn TerminalDispatchGate>>,
    // 终端会话映射端口：负责落库远端 session_id 对应关系。
    pub(super) session_repository: Mutex<Arc<dyn TerminalSessionRepository>>,
    pub(super) settings_service:
        Mutex<Option<Arc<dyn crate::ports::settings::TerminalSettingsPort>>>,
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(SessionRegistry {
                sessions: HashMap::new(),
                member_statuses: HashMap::new(),
                member_sessions: HashMap::new(),
                pending_status_locks: HashSet::new(),
                trigger_bus: None,
                working_sessions: Arc::new(Mutex::new(HashSet::new())),
            })),
            message_pipeline: Mutex::new(default_terminal_message_pipeline()),
            event_port: Mutex::new(default_terminal_event_port()),
            dispatch_gate: Mutex::new(default_terminal_dispatch_gate()),
            session_repository: Mutex::new(default_terminal_session_repository()),
            settings_service: Mutex::new(None),
        }
    }
}

impl TerminalManager {
    /// 注入消息端口，确保语义线程可直接持有端口引用。
    pub(crate) fn set_message_pipeline(&self, pipeline: Arc<dyn TerminalMessagePipeline>) {
        if let Ok(mut guard) = self.message_pipeline.lock() {
            *guard = pipeline;
        } else {
            log::warn!("terminal message pipeline lock poisoned");
        }
    }

    pub(crate) fn set_settings_service(
        &self,
        service: Arc<dyn crate::ports::settings::TerminalSettingsPort>,
    ) {
        if let Ok(mut guard) = self.settings_service.lock() {
            *guard = Some(service);
        } else {
            log::warn!("terminal settings service lock poisoned");
        }
    }

    /// 获取消息端口快照，避免使用 AppHandle 取全局状态。
    pub(crate) fn message_pipeline(&self) -> Arc<dyn TerminalMessagePipeline> {
        match self.message_pipeline.lock() {
            Ok(guard) => Arc::clone(&guard),
            Err(err) => {
                log::warn!("terminal message pipeline lock poisoned; recovering");
                Arc::clone(&err.into_inner())
            }
        }
    }

    /// 注入终端事件端口，统一由适配层负责事件发送。
    pub(crate) fn set_event_port(&self, port: Arc<dyn TerminalEventPort>) {
        if let Ok(mut guard) = self.event_port.lock() {
            *guard = port;
        } else {
            log::warn!("terminal event port lock poisoned");
        }
    }

    /// 注入派发门禁端口，由语义 flush 释放队列。
    pub(crate) fn set_dispatch_gate(&self, gate: Arc<dyn TerminalDispatchGate>) {
        if let Ok(mut guard) = self.dispatch_gate.lock() {
            *guard = gate;
        } else {
            log::warn!("terminal dispatch gate lock poisoned");
        }
    }

    /// 注入会话映射端口，将远端 session_id 写入持久化存储。
    pub(crate) fn set_session_repository(&self, repository: Arc<dyn TerminalSessionRepository>) {
        if let Ok(mut guard) = self.session_repository.lock() {
            *guard = repository;
        } else {
            log::warn!("terminal session repository lock poisoned");
        }
    }

    /// 获取终端事件端口快照，避免运行时再依赖 AppHandle。
    pub(crate) fn event_port(&self) -> Arc<dyn TerminalEventPort> {
        match self.event_port.lock() {
            Ok(guard) => Arc::clone(&guard),
            Err(err) => {
                log::warn!("terminal event port lock poisoned; recovering");
                Arc::clone(&err.into_inner())
            }
        }
    }

    /// 获取派发门禁端口快照，避免运行时依赖具体实现。
    pub(crate) fn dispatch_gate(&self) -> Arc<dyn TerminalDispatchGate> {
        match self.dispatch_gate.lock() {
            Ok(guard) => Arc::clone(&guard),
            Err(err) => {
                log::warn!("terminal dispatch gate lock poisoned; recovering");
                Arc::clone(&err.into_inner())
            }
        }
    }

    /// 获取会话映射端口快照，避免运行时依赖具体存储实现。
    pub(crate) fn session_repository(&self) -> Arc<dyn TerminalSessionRepository> {
        match self.session_repository.lock() {
            Ok(guard) => Arc::clone(&guard),
            Err(err) => {
                log::warn!("terminal session repository lock poisoned; recovering");
                Arc::clone(&err.into_inner())
            }
        }
    }

    /// 按成员与工作区查找已有会话，避免跨层直接访问注册表细节。
    pub(crate) fn find_session_id_by_member(
        &self,
        member_id: &str,
        workspace_id: Option<&str>,
    ) -> Option<String> {
        let guard = lock_sessions(&self.sessions);
        guard
            .sessions
            .values()
            .find(|session| {
                session.member_id.as_deref() == Some(member_id)
                    && session.workspace_id.as_deref() == workspace_id
            })
            .map(|session| session.id.clone())
    }

    /// 读取终端成员 DND 状态，避免派发进入语义阻塞。
    pub(crate) fn is_terminal_dnd(&self, terminal_id: &str) -> bool {
        let guard = lock_sessions(&self.sessions);
        let session = match guard.sessions.get(terminal_id) {
            Some(session) => session,
            None => return false,
        };
        let member_id = match session.member_id.as_ref() {
            Some(member_id) => member_id,
            None => return false,
        };
        matches!(
            guard
                .member_statuses
                .get(member_id)
                .map(|status| status.as_str()),
            Some("dnd")
        )
    }
}

pub(super) fn lock_sessions<'a>(
    sessions: &'a Arc<Mutex<SessionRegistry>>,
) -> std::sync::MutexGuard<'a, SessionRegistry> {
    match sessions.lock() {
        Ok(guard) => guard,
        Err(err) => {
            // 发生 panic 后尽量恢复，以避免终端全局失效。
            log::warn!("terminal sessions lock poisoned; recovering");
            err.into_inner()
        }
    }
}
