//! UI 接口层：窗口/工作区/存储等命令与辅助逻辑。

use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
  process::Command,
  sync::{
    atomic::{AtomicUsize, Ordering},
  },
  time::{SystemTime, UNIX_EPOCH},
};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tauri::{
  App,
  AppHandle,
  Emitter,
  LogicalSize,
  Manager,
  Size,
  State,
  WebviewUrl,
  WebviewWindow,
  WebviewWindowBuilder,
};
#[cfg(desktop)]
use tauri::{
  menu::{Menu, MenuItem},
  tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use super::notification::{NotificationBadgeState, NotificationOpenTerminalPayload, TRAY_ICON_ID};
use crate::message_service::chat_db::{compute_workspace_unread_summary, ChatDbManager};
use crate::platform::{diagnostics_log_backend_event, DiagnosticsState};
use crate::runtime::state::AppState;
use crate::runtime::{storage, StorageManager};

// 窗口与项目标识计数器：用于避免并发创建时的 label/ID 冲突。
static WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);
static MAIN_WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);
static WORKSPACE_WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(1);
static PROJECT_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);
const TERMINAL_WINDOW_LABEL: &str = "terminal-main"; // 主终端窗口固定 label，便于复用。
pub(crate) const MAIN_WINDOW_LABEL: &str = "main"; // 主窗口 label，托盘交互依赖此标识。
const RECENT_WORKSPACES_FILE: &str = "recent-workspaces.json"; // 最近工作区列表。
const WORKSPACE_REGISTRY_FILE: &str = "workspace-registry.json"; // 工作区 ID 与路径映射。
const WORKSPACE_REGISTRY_LOCK_FILE: &str = "workspace-registry.lock"; // 跨进程锁文件。
const AVATAR_LIBRARY_FILE: &str = "avatar-library.json"; // 头像索引清单。
const AVATAR_DIR: &str = "avatars"; // 头像内容存放目录。
const MAX_AVATAR_BYTES: usize = 2 * 1024 * 1024; // 限制单头像大小，避免 UI 传输与磁盘占用失控。
const WORKSPACE_REGISTRY_MISMATCH_PREFIX: &str = "workspace_registry_mismatch:"; // 前端识别路径冲突的标记前缀。
const WORKSPACE_REGISTRY_GC_MIN_AGE_MS: u64 = 1000 * 60 * 60 * 24 * 30; // 仅清理超过 30 天未访问的记录。
const WORKSPACE_REGISTRY_GC_MAX_CHECKS: usize = 12; // 单次最多检查 12 条，避免启动卡顿。
const NOTIFICATION_OPEN_CONVERSATION_EVENT: &str = "notification-open-conversation";
const NOTIFICATION_OPEN_TERMINAL_EVENT: &str = "notification-open-terminal";
const WINDOW_CORNER_RADIUS_PX: f64 = 12.0; // 与前端 window-frame 圆角保持一致。
const CHAT_WINDOW_WIDTH_RATIO: f64 = 0.68;
const CHAT_WINDOW_HEIGHT_RATIO: f64 = 0.78;
const CHAT_WINDOW_SAFE_MARGIN: f64 = 48.0;
const MAIN_WINDOW_DEFAULT_WIDTH: f64 = 1400.0;
const MAIN_WINDOW_DEFAULT_HEIGHT: f64 = 900.0;
const MAIN_WINDOW_MIN_WIDTH: f64 = 720.0;
const MAIN_WINDOW_MIN_HEIGHT: f64 = 520.0;
const MAIN_WINDOW_MAX_WIDTH: f64 = 1600.0;
const MAIN_WINDOW_MAX_HEIGHT: f64 = 1100.0;
const TERMINAL_WINDOW_DEFAULT_WIDTH: f64 = 1200.0;
const TERMINAL_WINDOW_DEFAULT_HEIGHT: f64 = 800.0;
const TERMINAL_WINDOW_MIN_WIDTH: f64 = 720.0;
const TERMINAL_WINDOW_MIN_HEIGHT: f64 = 520.0;
const TERMINAL_WINDOW_MAX_WIDTH: f64 = 1500.0;
const TERMINAL_WINDOW_MAX_HEIGHT: f64 = 1000.0;



#[derive(Serialize, Deserialize, Clone)]
/// 最近工作区条目：用于 UI 展示与快速打开。
pub(crate) struct WorkspaceEntry {
  id: String,
  name: String,
  path: String,
  #[serde(rename = "lastOpenedAt")]
  last_opened_at: u64,
}

#[derive(Serialize, Deserialize, Clone)]
/// 打开工作区的结果：可能包含只读提示与警告信息。
pub(crate) struct WorkspaceOpenResult {
  entry: WorkspaceEntry,
  #[serde(rename = "readOnly")]
  read_only: bool,
  warning: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
/// 工作区注册表条目：用于跟踪 ID 与最近路径。
struct WorkspaceRegistryEntry {
  #[serde(rename = "lastKnownPath")]
  last_known_path: String,
  #[serde(rename = "lastAccessed")]
  last_accessed: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct NotificationConversationOpenPayload {
  conversation_id: String,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotificationOpenAllItem {
  workspace_id: Option<String>,
  workspace_path: Option<String>,
  conversation_id: Option<String>,
  last_message_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
/// 单机本地状态：用于保留设备级标识与最后打开时间。
struct LocalWorkspaceState {
  #[serde(rename = "localMachineId")]
  local_machine_id: String,
  #[serde(rename = "lastOpenedAt")]
  last_opened_at: u64,
}

#[derive(Deserialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
/// 注册表路径冲突时的处理策略。
pub(crate) enum WorkspaceRegistryResolution {
  Move,
  Copy,
}

#[derive(Serialize, Deserialize, Clone)]
/// 头像元数据：用于索引与去重。
pub(crate) struct AvatarAsset {
  id: String,
  filename: String,
  #[serde(rename = "createdAt")]
  created_at: u64,
}

#[derive(Serialize)]
/// 头像内容：用于二进制读取接口。
pub(crate) struct AvatarContent {
  bytes: Vec<u8>,
  mime: String,
}

fn next_terminal_label(reuse: bool) -> String {
  // 终端窗口 label 必须唯一，避免 Tauri 复用错误窗口。
  if reuse {
    TERMINAL_WINDOW_LABEL.to_string()
  } else {
    let suffix = WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("terminal-{suffix}")
  }
}

fn next_main_window_label() -> String {
  let suffix = MAIN_WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed);
  format!("main-{suffix}")
}

fn next_workspace_window_label() -> String {
  // 工作区选择窗口需要独立 label，避免与主窗口冲突。
  let suffix = WORKSPACE_WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed);
  format!("workspace-selection-{suffix}")
}

fn terminal_label_for_workspace(workspace_id: &str) -> String {
  // 用哈希避免直接暴露工作区路径或 ID。
  let hash = hash_bytes(workspace_id.as_bytes());
  format!("terminal-workspace-{hash}")
}

#[cfg(target_os = "windows")]
pub(crate) fn apply_windows_rounding(window: &tauri::WebviewWindow) {
  // Windows 10/11 通过窗口区域裁剪实现圆角，避免系统焦点边框参与绘制。
  use windows_sys::Win32::Graphics::Gdi::{CreateRoundRectRgn, DeleteObject, SetWindowRgn};

  let label = window.label();
  let should_round = label == MAIN_WINDOW_LABEL
    || label.starts_with("main-")
    || label.starts_with("terminal")
    || label.starts_with("workspace-selection");
  if !should_round {
    return;
  }
  let hwnd = match window.hwnd() {
    Ok(handle) => handle.0 as isize,
    Err(_) => return,
  };
  let is_maximized = window.is_maximized().unwrap_or(false);
  let is_fullscreen = window.is_fullscreen().unwrap_or(false);
  let outer = match window.outer_size() {
    Ok(value) => value,
    Err(_) => return,
  };
  let width = outer.width as i32;
  let height = outer.height as i32;
  if width <= 0 || height <= 0 {
    return;
  }
  let left = 0;
  let top = 0;
  let right = width;
  let bottom = height;
  let scale = window.scale_factor().unwrap_or(1.0);
  let base_radius = if is_maximized || is_fullscreen {
    0.0
  } else {
    WINDOW_CORNER_RADIUS_PX
  };
  let radius = (base_radius * scale).round().max(0.0) as i32;
  let diameter = radius.saturating_mul(2);
  unsafe {
    let region = CreateRoundRectRgn(left, top, right + 1, bottom + 1, diameter, diameter);
    if region == 0 {
      return;
    }
    if SetWindowRgn(hwnd, region, 1) == 0 {
      let _ = DeleteObject(region);
    }
  }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn apply_windows_rounding(_window: &tauri::WebviewWindow) {}

fn resolve_main_window_label(app: &AppHandle) -> Option<String> {
  if let Ok(guard) = app.state::<AppState>().active_main_window.lock() {
    if let Some(label) = guard.as_ref() {
      if app.get_webview_window(label).is_some() {
        return Some(label.clone());
      }
    }
  }
  if app.get_webview_window(MAIN_WINDOW_LABEL).is_some() {
    return Some(MAIN_WINDOW_LABEL.to_string());
  }
  app
    .webview_windows()
    .keys()
    .find(|label| label.starts_with("main-"))
    .cloned()
}

fn resolve_active_monitor(app: &AppHandle) -> Option<tauri::Monitor> {
  if let Some(label) = resolve_main_window_label(app) {
    if let Some(window) = app.get_webview_window(&label) {
      if let (Ok(position), Ok(size)) = (window.outer_position(), window.outer_size()) {
        let center_x = position.x as f64 + size.width as f64 * 0.5;
        let center_y = position.y as f64 + size.height as f64 * 0.5;
        if let Ok(monitor) = app.monitor_from_point(center_x, center_y) {
          if monitor.is_some() {
            return monitor;
          }
        }
      }
    }
  }
  app.primary_monitor().ok().flatten()
}

fn resolve_chat_window_size(
  app: &AppHandle,
  min_width: f64,
  min_height: f64,
  max_width: f64,
  max_height: f64,
  fallback_width: f64,
  fallback_height: f64,
) -> (f64, f64) {
  // 按工作区可用大小缩放，避免固定尺寸在高/低分辨率下过大或过小。
  let monitor = match resolve_active_monitor(app) {
    Some(monitor) => monitor,
    None => return (fallback_width, fallback_height),
  };
  let work_area = monitor.work_area();
  let scale_factor = monitor.scale_factor();
  if scale_factor <= 0.0 {
    return (fallback_width, fallback_height);
  }
  let available_width = work_area.size.width as f64 / scale_factor;
  let available_height = work_area.size.height as f64 / scale_factor;
  if available_width <= 0.0 || available_height <= 0.0 {
    return (fallback_width, fallback_height);
  }
  let max_width = max_width.min(available_width - CHAT_WINDOW_SAFE_MARGIN).max(0.0);
  let max_height = max_height.min(available_height - CHAT_WINDOW_SAFE_MARGIN).max(0.0);
  if max_width <= 0.0 || max_height <= 0.0 {
    return (
      fallback_width.min(available_width),
      fallback_height.min(available_height),
    );
  }
  let min_width = min_width.min(max_width);
  let min_height = min_height.min(max_height);
  let target_width = (available_width * CHAT_WINDOW_WIDTH_RATIO).round();
  let target_height = (available_height * CHAT_WINDOW_HEIGHT_RATIO).round();
  (
    target_width.clamp(min_width, max_width),
    target_height.clamp(min_height, max_height),
  )
}

pub(crate) fn apply_main_window_size(app: &AppHandle, window: &WebviewWindow) {
  let (window_width, window_height) = resolve_chat_window_size(
    app,
    MAIN_WINDOW_MIN_WIDTH,
    MAIN_WINDOW_MIN_HEIGHT,
    MAIN_WINDOW_MAX_WIDTH,
    MAIN_WINDOW_MAX_HEIGHT,
    MAIN_WINDOW_DEFAULT_WIDTH,
    MAIN_WINDOW_DEFAULT_HEIGHT,
  );
  let min_width = MAIN_WINDOW_MIN_WIDTH.min(window_width);
  let min_height = MAIN_WINDOW_MIN_HEIGHT.min(window_height);
  let _ = window.set_min_size(Some(Size::Logical(LogicalSize::new(
    min_width,
    min_height,
  ))));
  let _ = window.set_size(Size::Logical(LogicalSize::new(window_width, window_height)));
  let _ = window.center();
}

#[cfg(target_os = "windows")]
fn refresh_main_window_frame(window: &WebviewWindow) {
  // Windows 下 hide/show 不一定触发 resize，手动同步尺寸与圆角区域避免裁切。
  let is_maximized = window.is_maximized().unwrap_or(false);
  let is_fullscreen = window.is_fullscreen().unwrap_or(false);
  if let Ok(size) = window.outer_size() {
    if !is_maximized && !is_fullscreen {
      let bump = tauri::PhysicalSize::new(
        size.width.saturating_add(1),
        size.height.saturating_add(1),
      );
      let _ = window.set_size(Size::Physical(bump));
    }
    let _ = window.set_size(Size::Physical(size));
  }
  apply_windows_rounding(window);
}

#[cfg(not(target_os = "windows"))]
fn refresh_main_window_frame(_window: &WebviewWindow) {}

#[cfg(target_os = "windows")]
pub(crate) fn schedule_main_window_frame_refresh(window: &WebviewWindow) {
  refresh_main_window_frame(window);
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn schedule_main_window_frame_refresh(window: &WebviewWindow) {
  refresh_main_window_frame(window);
}

pub(crate) fn show_main_window(app: &AppHandle) {
  let label = resolve_main_window_label(app);
  if let Some(label) = label {
    if let Some(window) = app.get_webview_window(&label) {
      let _ = window.show();
      let _ = window.unminimize();
      let _ = window.set_focus();
      schedule_main_window_frame_refresh(&window);
      if let Ok(mut guard) = app.state::<AppState>().active_main_window.lock() {
        *guard = Some(label);
      }
    }
  }
}

fn toggle_main_window(app: &AppHandle) {
  let label = resolve_main_window_label(app);
  if let Some(label) = label {
    if let Some(window) = app.get_webview_window(&label) {
      let is_visible = window.is_visible().unwrap_or(false);
      let is_minimized = window.is_minimized().unwrap_or(false);
      if is_visible && !is_minimized {
        let _ = window.hide();
      } else {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        schedule_main_window_frame_refresh(&window);
        if let Ok(mut guard) = app.state::<AppState>().active_main_window.lock() {
          *guard = Some(label);
        }
      }
    }
  }
}

fn show_workspace_window(app: &AppHandle, label: &str, focus: bool) {
  if let Some(window) = app.get_webview_window(label) {
    let _ = window.show();
    let _ = window.unminimize();
    if focus {
      let _ = window.set_focus();
    }
  }
}

fn resolve_active_workspace_window(
  app: &AppHandle,
  state: &AppState,
  workspace_id: &str,
) -> Option<String> {
  let mut guard = state.workspace_windows.lock().ok()?;
  let mut stale = Vec::new();
  let mut target = None;
  for (label, id) in guard.iter() {
    if id != workspace_id {
      continue;
    }
    if app.get_webview_window(label).is_some() {
      target = Some(label.clone());
      break;
    }
    stale.push(label.clone());
  }
  for label in stale {
    guard.remove(&label);
  }
  target
}

#[cfg(desktop)]
pub(crate) fn setup_tray(app: &App) -> tauri::Result<()> {
  let handle = app.handle();
  let show_item = MenuItem::with_id(handle, "tray_show", "显示窗口", true, None::<&str>)?;
  let quit_item = MenuItem::with_id(handle, "tray_quit", "退出", true, None::<&str>)?;
  let menu = Menu::with_items(handle, &[&show_item, &quit_item])?;
  let mut builder = TrayIconBuilder::with_id(TRAY_ICON_ID)
    .menu(&menu)
    .tooltip(handle.package_info().name.clone())
    .show_menu_on_left_click(false)
    .on_menu_event(|app, event| match event.id().as_ref() {
      "tray_show" => show_main_window(app),
      "tray_quit" => app.exit(0),
      _ => {}
    });
  if let Some(icon) = app.default_window_icon().cloned() {
    builder = builder.icon(icon);
  }
  builder
    .on_tray_icon_event(|tray, event| match event {
      TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      }
      | TrayIconEvent::DoubleClick {
        button: MouseButton::Left,
        ..
      } => {
        toggle_main_window(tray.app_handle());
      }
      TrayIconEvent::Enter { rect, .. } | TrayIconEvent::Move { rect, .. } => {
        let notification_state = tray.app_handle().state::<NotificationBadgeState>();
        notification_state.set_tray_hovered(tray.app_handle(), true, Some(rect));
      }
      TrayIconEvent::Leave { rect, .. } => {
        let notification_state = tray.app_handle().state::<NotificationBadgeState>();
        notification_state.set_tray_hovered(tray.app_handle(), false, Some(rect));
      }
      _ => {}
    })
    .build(app)?;
  Ok(())
}

#[derive(Serialize)]
/// 终端窗口打开结果：包含窗口 label 与是否复用标记。
pub(crate) struct TerminalWindowOpenResult {
  label: String,
  reused: bool,
}

fn build_workspace_init_script(
  workspace_id: &str,
  workspace_path: &str,
  conversation_id: Option<&str>,
  ignore_all_unread: bool,
  terminal_payloads: Option<&[NotificationOpenTerminalPayload]>,
) -> Result<String, String> {
  let mut script = String::new();
  let workspace_payload = json!({
    "id": workspace_id,
    "path": workspace_path
  });
  script.push_str(&format!("window.__GOLUTRA_WORKSPACE__ = {workspace_payload};"));
  if let Some(conversation_id) = conversation_id {
    let notification_payload = json!({
      "conversationId": conversation_id
    });
    script.push_str(&format!(
      "window.__GOLUTRA_NOTIFICATION_OPEN__ = {notification_payload};"
    ));
  }
  if ignore_all_unread {
    script.push_str("window.__GOLUTRA_NOTIFICATION_IGNORE_ALL__ = true;");
  }
  if let Some(payloads) = terminal_payloads {
    let payloads = json!(payloads);
    script.push_str(&format!(
      "window.__GOLUTRA_NOTIFICATION_OPEN_TERMINAL__ = {payloads};"
    ));
  }
  Ok(script)
}

fn open_workspace_main_window(
  app: &AppHandle,
  state: &AppState,
  workspace_id: &str,
  workspace_path: &str,
  conversation_id: Option<&str>,
  ignore_all_unread: bool,
  terminal_payloads: Option<&[NotificationOpenTerminalPayload]>,
  focus: bool,
  visible: bool,
) -> Result<String, String> {
  let label = next_main_window_label();
  let init_script = build_workspace_init_script(
    workspace_id,
    workspace_path,
    conversation_id,
    ignore_all_unread,
    terminal_payloads,
  )?;
  let (window_width, window_height) = resolve_chat_window_size(
    app,
    MAIN_WINDOW_MIN_WIDTH,
    MAIN_WINDOW_MIN_HEIGHT,
    MAIN_WINDOW_MAX_WIDTH,
    MAIN_WINDOW_MAX_HEIGHT,
    MAIN_WINDOW_DEFAULT_WIDTH,
    MAIN_WINDOW_DEFAULT_HEIGHT,
  );
  let min_width = MAIN_WINDOW_MIN_WIDTH.min(window_width);
  let min_height = MAIN_WINDOW_MIN_HEIGHT.min(window_height);
  let window_builder = WebviewWindowBuilder::new(app, label.clone(), WebviewUrl::App("index.html".into()))
    .initialization_script(init_script)
    .title("golutra")
    .inner_size(window_width, window_height)
    .min_inner_size(min_width, min_height)
    .resizable(true)
    .center()
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(visible);
  #[cfg(target_os = "macos")]
  let window_builder = window_builder.title_bar_style(tauri::TitleBarStyle::Overlay);
  let window = window_builder
    .build()
    .map_err(|err| format!("failed to create workspace window: {err}"))?;
  apply_windows_rounding(&window);

  if let Ok(mut guard) = state.workspace_windows.lock() {
    guard.insert(label.clone(), workspace_id.to_string());
  }

  if visible {
    let _ = window.show();
    let _ = window.unminimize();
    if focus {
      let _ = window.set_focus();
      if let Ok(mut guard) = state.active_main_window.lock() {
        *guard = Some(label.clone());
      }
    }
  }

  Ok(label)
}

#[tauri::command]
/// 打开或复用终端窗口。
/// 输入：`reuse` 控制是否复用固定窗口；workspace 信息用于关联窗口与上下文。
/// 返回：窗口 label 与是否复用。
/// 错误：窗口创建失败或参数非法。
pub(crate) async fn terminal_open_window(
  app: AppHandle,
  reuse: Option<bool>,
  workspace_id: Option<String>,
  workspace_name: Option<String>,
  workspace_path: Option<String>,
  auto_tab: Option<bool>,
) -> Result<TerminalWindowOpenResult, String> {
  let reuse = reuse.unwrap_or(true);
  let auto_tab = auto_tab.unwrap_or(true);
  let workspace_id = workspace_id
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty());
  let workspace_name = workspace_name
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty());
  let workspace_path = workspace_path
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty());

  let diagnostics = app.state::<DiagnosticsState>();
  let log_open = |label: &str, reused: bool| {
    diagnostics_log_backend_event(
      &diagnostics,
      None,
      None,
      None,
      Some(label.to_string()),
      workspace_id.as_deref().map(|value| value.to_string()),
      "terminal_open_window",
      json!({
        "windowLabel": label,
        "reused": reused,
        "reuse": reuse,
        "autoTab": auto_tab,
        "workspaceId": workspace_id.as_deref(),
        "workspaceName": workspace_name.as_deref(),
        "workspacePath": workspace_path.as_deref()
      }),
    );
  };

  let workspace_label = workspace_id.as_deref().map(terminal_label_for_workspace);
  if let Some(label) = workspace_label.as_deref() {
    if let Some(window) = app.get_webview_window(label) {
      let _ = window.show();
      let _ = window.unminimize();
      let _ = window.set_focus();
      log_open(label, true);
      return Ok(TerminalWindowOpenResult {
        label: label.to_string(),
        reused: true,
      });
    }
  } else if reuse {
    if let Some(window) = app.get_webview_window(TERMINAL_WINDOW_LABEL) {
      let _ = window.show();
      let _ = window.unminimize();
      let _ = window.set_focus();
      log_open(TERMINAL_WINDOW_LABEL, true);
      return Ok(TerminalWindowOpenResult {
        label: TERMINAL_WINDOW_LABEL.to_string(),
        reused: true,
      });
    }
  }

  let label = workspace_label.unwrap_or_else(|| next_terminal_label(reuse));
  let title = workspace_name
    .as_deref()
    .map(|name| format!("{name} - Terminal"))
    .unwrap_or_else(|| "Terminal".to_string());
  let init_payload = json!({
    "id": workspace_id.as_deref(),
    "name": workspace_name.as_deref(),
    "path": workspace_path.as_deref()
  });
  let init_script = format!(
    "window.__GOLUTRA_VIEW__ = 'terminal'; window.__GOLUTRA_WORKSPACE__ = {init_payload}; window.__GOLUTRA_TERMINAL_AUTO_TAB__ = {auto_tab};"
  );
  // 默认尺寸用于容纳终端布局，最小尺寸保证分栏与标题栏不被压缩失真。
  let (window_width, window_height) = resolve_chat_window_size(
    &app,
    TERMINAL_WINDOW_MIN_WIDTH,
    TERMINAL_WINDOW_MIN_HEIGHT,
    TERMINAL_WINDOW_MAX_WIDTH,
    TERMINAL_WINDOW_MAX_HEIGHT,
    TERMINAL_WINDOW_DEFAULT_WIDTH,
    TERMINAL_WINDOW_DEFAULT_HEIGHT,
  );
  let min_width = TERMINAL_WINDOW_MIN_WIDTH.min(window_width);
  let min_height = TERMINAL_WINDOW_MIN_HEIGHT.min(window_height);
  let window_builder = WebviewWindowBuilder::new(&app, label.clone(), WebviewUrl::App("index.html".into()))
    .initialization_script(&init_script)
    .title(title)
    .inner_size(window_width, window_height)
    .min_inner_size(min_width, min_height)
    .resizable(true)
    .center()
    .decorations(false)
    .transparent(true)
    // [HACK/OPTIMIZE, 2026-01-27] 原因: Windows 无边框窗口开启 shadow 会出现 1px 白边; 替代方案: 关闭 transparent/resizable 或自绘边框; 移除条件: 系统或 Tauri 修复该行为。
    .shadow(!cfg!(target_os = "windows"));
  #[cfg(target_os = "macos")]
  let window_builder = window_builder.title_bar_style(tauri::TitleBarStyle::Overlay);
  let window = window_builder
    .build()
    .map_err(|err| format!("failed to create terminal window: {err}"))?;
  apply_windows_rounding(&window);

  let _ = window.show();
  let _ = window.set_focus();

  log_open(&label, false);
  Ok(TerminalWindowOpenResult { label, reused: false })
}

#[tauri::command]
/// 打开工作区选择窗口（独立于主窗口）。
/// 返回：窗口 label。
/// 错误：窗口创建失败。
pub(crate) async fn workspace_selection_open_window(app: AppHandle) -> Result<String, String> {
  let label = next_workspace_window_label();
  let (window_width, window_height) = resolve_chat_window_size(
    &app,
    MAIN_WINDOW_MIN_WIDTH,
    MAIN_WINDOW_MIN_HEIGHT,
    MAIN_WINDOW_MAX_WIDTH,
    MAIN_WINDOW_MAX_HEIGHT,
    MAIN_WINDOW_DEFAULT_WIDTH,
    MAIN_WINDOW_DEFAULT_HEIGHT,
  );
  let min_width = MAIN_WINDOW_MIN_WIDTH.min(window_width);
  let min_height = MAIN_WINDOW_MIN_HEIGHT.min(window_height);
  let window_builder = WebviewWindowBuilder::new(&app, label.clone(), WebviewUrl::App("index.html".into()))
    .initialization_script("window.__GOLUTRA_VIEW__ = 'workspace-selection';")
    .title("golutra")
    .inner_size(window_width, window_height)
    .min_inner_size(min_width, min_height)
    .resizable(true)
    .center()
    .decorations(false)
    .transparent(true)
    // [HACK/OPTIMIZE, 2026-01-28] 原因: Windows 无边框窗口开启 shadow 会出现底部白边; 替代方案: 关闭 transparent/resizable 或自绘边框; 移除条件: 系统或 Tauri 修复该行为。
    .shadow(!cfg!(target_os = "windows"));
  #[cfg(target_os = "macos")]
  let window_builder = window_builder.title_bar_style(tauri::TitleBarStyle::Overlay);
  let window = window_builder
    .build()
    .map_err(|err| format!("failed to create workspace selection window: {err}"))?;
  apply_windows_rounding(&window);

  let _ = window.show();
  let _ = window.set_focus();

  Ok(label)
}

#[tauri::command]
pub(crate) fn notification_open_all_unread(
  app: AppHandle,
  state: State<'_, ChatDbManager>,
  window_state: State<'_, AppState>,
  user_id: String,
  items: Option<Vec<NotificationOpenAllItem>>,
) -> Result<(), String> {
  let mut preview_targets: Vec<(String, String, String, u64)> = Vec::new();
  if let Some(items) = items {
    let mut latest_by_workspace: HashMap<String, (String, String, u64)> = HashMap::new();
    for item in items {
      let workspace_id = item.workspace_id.unwrap_or_default();
      let conversation_id = item.conversation_id.unwrap_or_default();
      let workspace_id = workspace_id.trim();
      let conversation_id = conversation_id.trim();
      if workspace_id.is_empty() || conversation_id.is_empty() {
        continue;
      }
      let workspace_path = item.workspace_path.unwrap_or_default();
      let latest_at = item.last_message_at.unwrap_or(0);
      match latest_by_workspace.get_mut(workspace_id) {
        Some((path, existing_conversation, existing_at)) => {
          if latest_at >= *existing_at {
            *path = workspace_path;
            *existing_conversation = conversation_id.to_string();
            *existing_at = latest_at;
          } else if path.trim().is_empty() && !workspace_path.trim().is_empty() {
            *path = workspace_path;
          }
        }
        None => {
          latest_by_workspace.insert(
            workspace_id.to_string(),
            (workspace_path, conversation_id.to_string(), latest_at),
          );
        }
      }
    }
    for (workspace_id, (workspace_path, conversation_id, latest_at)) in latest_by_workspace {
      preview_targets.push((workspace_id, workspace_path, conversation_id, latest_at));
    }
  }

  let mut db_targets: Vec<(String, String, String, u64)> = Vec::new();
  let user_id = user_id.trim();
  if !user_id.is_empty() {
    let registry = load_workspace_registry_snapshot(&app)?;
    for (workspace_id, entry) in registry {
      let summary = match compute_workspace_unread_summary(&state, &workspace_id, user_id) {
        Ok(summary) => summary,
        Err(error) => {
          log::warn!("notification open all failed to read unread summary: {error}");
          continue;
        }
      };
      if summary.total_unread_count == 0 {
        continue;
      }
      let conversation_id = match summary.latest_conversation_id {
        Some(id) => id,
        None => continue,
      };
      let latest_at = summary.latest_message_at.unwrap_or(0);
      db_targets.push((workspace_id, entry.last_known_path, conversation_id, latest_at));
    }
  }

  let mut targets = if db_targets.is_empty() {
    preview_targets
  } else {
    db_targets
  };
  if targets.is_empty() {
    return Ok(());
  }

  if targets.iter().any(|(_, path, _, _)| path.trim().is_empty()) {
    let registry = load_workspace_registry_snapshot(&app)?;
    for target in targets.iter_mut() {
      if target.1.trim().is_empty() {
        if let Some(entry) = registry.get(&target.0) {
          target.1 = entry.last_known_path.clone();
        }
      }
    }
  }

  targets.sort_by_key(|(_, _, _, latest_at)| *latest_at);

  for (workspace_id, workspace_path, conversation_id, _) in targets {
    let path = workspace_path.trim();
    if path.is_empty() || !Path::new(path).is_dir() {
      continue;
    }
    let label = if let Some(existing) =
      resolve_active_workspace_window(&app, &window_state, &workspace_id)
    {
      show_workspace_window(&app, &existing, true);
      let _ = app.emit_to(existing.as_str(), "notification-ignore-all", ());
      existing
    } else {
      match open_workspace_main_window(
        &app,
        &window_state,
        &workspace_id,
        path,
        Some(&conversation_id),
        true,
        None,
        true,
        true,
      ) {
        Ok(label) => label,
        Err(error) => {
          log::warn!("notification open all failed to open workspace window: {error}");
          continue;
        }
      }
    };
    let payload = NotificationConversationOpenPayload {
      conversation_id: conversation_id.clone(),
    };
    let _ = app.emit_to(label.as_str(), NOTIFICATION_OPEN_CONVERSATION_EVENT, payload);
  }

  Ok(())
}

#[tauri::command]
pub(crate) fn notification_open_unread_conversation(
  app: AppHandle,
  window_state: State<'_, AppState>,
  workspace_id: String,
  conversation_id: String,
) -> Result<(), String> {
  let workspace_id = workspace_id.trim();
  let conversation_id = conversation_id.trim();
  if workspace_id.is_empty() || conversation_id.is_empty() {
    return Ok(());
  }
  if let Some(existing) = resolve_active_workspace_window(&app, &window_state, workspace_id) {
    show_workspace_window(&app, &existing, true);
    let payload = NotificationConversationOpenPayload {
      conversation_id: conversation_id.to_string(),
    };
    let _ = app.emit_to(existing.as_str(), NOTIFICATION_OPEN_CONVERSATION_EVENT, payload);
    return Ok(());
  }
  let registry = load_workspace_registry_snapshot(&app)?;
  let entry = match registry.get(workspace_id) {
    Some(entry) => entry,
    None => return Ok(()),
  };
  let path = entry.last_known_path.trim();
  if path.is_empty() || !Path::new(path).is_dir() {
    return Ok(());
  }
  let label = open_workspace_main_window(
    &app,
    &window_state,
    workspace_id,
    path,
    Some(conversation_id),
    false,
    None,
    true,
    true,
  )?;
  let payload = NotificationConversationOpenPayload {
    conversation_id: conversation_id.to_string(),
  };
  let _ = app.emit_to(label.as_str(), NOTIFICATION_OPEN_CONVERSATION_EVENT, payload);
  Ok(())
}

#[tauri::command]
pub(crate) fn notification_open_terminal(
  app: AppHandle,
  window_state: State<'_, AppState>,
  workspace_id: String,
  conversation_id: String,
  sender_id: String,
) -> Result<(), String> {
  let workspace_id = workspace_id.trim();
  let conversation_id = conversation_id.trim();
  let sender_id = sender_id.trim();
  if workspace_id.is_empty() || conversation_id.is_empty() || sender_id.is_empty() {
    return Ok(());
  }
  let payload = NotificationOpenTerminalPayload {
    workspace_id: workspace_id.to_string(),
    conversation_id: conversation_id.to_string(),
    sender_id: sender_id.to_string(),
  };
  if let Some(existing) = resolve_active_workspace_window(&app, &window_state, workspace_id) {
    let _ = app.emit_to(existing.as_str(), NOTIFICATION_OPEN_TERMINAL_EVENT, payload);
    return Ok(());
  }
  let registry = load_workspace_registry_snapshot(&app)?;
  let entry = match registry.get(workspace_id) {
    Some(entry) => entry,
    None => return Ok(()),
  };
  let path = entry.last_known_path.trim();
  if path.is_empty() || !Path::new(path).is_dir() {
    return Ok(());
  }
  let payloads = vec![payload];
  let _ = open_workspace_main_window(
    &app,
    &window_state,
    workspace_id,
    path,
    Some(conversation_id),
    false,
    Some(&payloads),
    false,
    false,
  )?;
  Ok(())
}

pub(crate) fn now_millis() -> Result<u64, String> {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_err(|err| format!("failed to read system time: {err}"))
    .map(|value| value.as_millis() as u64)
}

fn hash_workspace_id(path: &Path) -> Result<String, String> {
  // 使用路径哈希避免直接暴露本地绝对路径。
  let text = path
    .to_str()
    .ok_or_else(|| "workspace path is not valid UTF-8".to_string())?
    .to_lowercase();
  let mut hasher = Sha256::new();
  hasher.update(text.as_bytes());
  let digest = hasher.finalize();
  let mut out = String::with_capacity(digest.len() * 2);
  for byte in digest {
    out.push_str(&format!("{:02x}", byte));
  }
  Ok(out)
}

fn hash_bytes(bytes: &[u8]) -> String {
  // 统一散列逻辑，既用于窗口 label，也用于资源去重。
  let mut hasher = Sha256::new();
  hasher.update(bytes);
  let digest = hasher.finalize();
  let mut out = String::with_capacity(digest.len() * 2);
  for byte in digest {
    out.push_str(&format!("{:02x}", byte));
  }
  out
}

fn sanitize_filename(filename: &str) -> Result<String, String> {
  // 仅允许纯文件名，避免用户注入路径片段。
  let trimmed = filename.trim();
  if trimmed.is_empty() {
    return Err("filename is empty".to_string());
  }
  if trimmed.contains('/') || trimmed.contains('\\') {
    return Err("filename contains path separators".to_string());
  }
  Ok(trimmed.to_string())
}

fn sanitize_extension(extension: &str) -> String {
  // 仅允许字母数字扩展名，其他字符会被丢弃以避免路径问题。
  let mut clean = String::new();
  for ch in extension.chars() {
    if ch.is_ascii_alphanumeric() {
      clean.push(ch.to_ascii_lowercase());
    }
  }
  if clean.is_empty() {
    "png".to_string()
  } else {
    clean
  }
}

fn read_json_file(path: &Path) -> Result<Option<serde_json::Value>, String> {
  storage::read_json_file(path)
}

fn write_json_file(path: &Path, payload: serde_json::Value) -> Result<(), String> {
  storage::write_json_file(path, payload)
}

/// 将相对路径解析到应用数据目录。
/// 约束：仅允许相对路径且禁止 `..`，避免越界访问。
/// 错误：路径非法或系统目录解析失败。
pub(crate) fn resolve_app_data_path(app: &AppHandle, relative_path: &str) -> Result<PathBuf, String> {
  let storage = app.state::<StorageManager>();
  storage::resolve_app_data_path(storage.inner(), relative_path)
}

fn resolve_avatar_file_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
  // 头像文件固定存放在应用数据目录的子目录。
  let clean = sanitize_filename(filename)?;
  let relative = format!("{AVATAR_DIR}/{clean}");
  resolve_app_data_path(app, &relative)
}

fn read_avatar_library(app: &AppHandle) -> Result<Vec<AvatarAsset>, String> {
  let path = resolve_app_data_path(app, AVATAR_LIBRARY_FILE)?;
  let data = read_json_file(&path)?;
  let parsed = match data {
    Some(value) => serde_json::from_value::<Vec<AvatarAsset>>(value)
      .map_err(|err| format!("failed to decode avatar library: {err}"))?,
    None => Vec::new(),
  };
  Ok(parsed)
}

fn write_avatar_library(app: &AppHandle, entries: &[AvatarAsset]) -> Result<(), String> {
  let payload = serde_json::to_value(entries)
    .map_err(|err| format!("failed to encode avatar library: {err}"))?;
  let path = resolve_app_data_path(app, AVATAR_LIBRARY_FILE)?;
  write_json_file(&path, payload)
}

fn resolve_workspace_path(workspace_path: &str, relative_path: &str) -> Result<PathBuf, String> {
  storage::resolve_workspace_path(workspace_path, relative_path)
}

fn resolve_workspace_metadata_path(workspace_root: &Path) -> Result<PathBuf, String> {
  // 工作区元数据固定落在 .golutra 目录中。
  let base = workspace_root
    .canonicalize()
    .map_err(|err| format!("failed to resolve workspace path: {err}"))?;
  Ok(base.join(".golutra").join("workspace.json"))
}

fn ensure_project_id(workspace_root: &Path, fallback_id: &str) -> Result<String, String> {
  // 读取或写入 projectId，用于稳定标识同一工作区。
  let metadata_path = resolve_workspace_metadata_path(workspace_root)?;
  if let Some(mut value) = read_json_file(&metadata_path)? {
    if let Some(obj) = value.as_object_mut() {
      if let Some(id) = obj
        .get("projectId")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
      {
        if !id.is_empty() {
          return Ok(id);
        }
      }
      obj.insert(
        "projectId".to_string(),
        serde_json::Value::String(fallback_id.to_string()),
      );
      write_json_file(&metadata_path, value)?;
      return Ok(fallback_id.to_string());
    }
    return Err("workspace metadata is not a JSON object".to_string());
  }

  let payload = json!({ "projectId": fallback_id });
  write_json_file(&metadata_path, payload)?;
  Ok(fallback_id.to_string())
}

fn load_recent_workspaces(app: &AppHandle) -> Result<Vec<WorkspaceEntry>, String> {
  let path = resolve_app_data_path(app, RECENT_WORKSPACES_FILE)?;
  let data = read_json_file(&path)?;
  let parsed = match data {
    Some(value) => serde_json::from_value::<Vec<WorkspaceEntry>>(value)
      .map_err(|err| format!("failed to decode recent workspaces: {err}"))?,
    None => Vec::new(),
  };
  Ok(parsed)
}

fn save_recent_workspaces(app: &AppHandle, entries: &[WorkspaceEntry]) -> Result<(), String> {
  let payload = serde_json::to_value(entries).map_err(|err| format!("failed to encode recent workspaces: {err}"))?;
  let path = resolve_app_data_path(app, RECENT_WORKSPACES_FILE)?;
  write_json_file(&path, payload)
}

fn normalize_registry_path(path: &str) -> String {
  // Windows 路径大小写不敏感，统一归一化避免重复项。
  if cfg!(windows) {
    path.to_lowercase()
  } else {
    path.to_string()
  }
}

fn is_probably_remote_path(path: &str) -> bool {
  // 远程路径可用性波动大，避免 GC 误删。
  if cfg!(windows) {
    let lower = path.to_lowercase();
    if lower.starts_with("\\\\?\\unc\\") {
      return true;
    }
    if lower.starts_with("\\\\?\\") {
      return false;
    }
    return lower.starts_with("\\\\") || lower.starts_with("//");
  }
  path.starts_with("//")
}

fn gc_workspace_registry(registry: &mut HashMap<String, WorkspaceRegistryEntry>, now: u64) {
  // 延迟清理策略：优先清理久未访问且非远程路径的条目。
  let mut candidates: Vec<(String, String, u64)> = registry
    .iter()
    .filter_map(|(id, entry)| {
      let age = now.saturating_sub(entry.last_accessed);
      if age < WORKSPACE_REGISTRY_GC_MIN_AGE_MS {
        return None;
      }
      if is_probably_remote_path(&entry.last_known_path) {
        return None;
      }
      Some((id.clone(), entry.last_known_path.clone(), entry.last_accessed))
    })
    .collect();

  candidates.sort_by_key(|(_, _, last_accessed)| *last_accessed);
  for (id, path, _) in candidates.into_iter().take(WORKSPACE_REGISTRY_GC_MAX_CHECKS) {
    if !Path::new(&path).exists() {
      registry.remove(&id);
    }
  }
}

fn with_workspace_registry<T>(
  app: &AppHandle,
  f: impl FnOnce(&mut HashMap<String, WorkspaceRegistryEntry>) -> Result<T, String>,
) -> Result<T, String> {
  // 进程内互斥 + 文件锁，避免多窗口并发写导致损坏。
  let state = app.state::<AppState>();
  let _guard = state
    .workspace_registry_lock
    .lock()
    .map_err(|_| "workspace registry lock poisoned".to_string())?;
  let lock_path = resolve_app_data_path(app, WORKSPACE_REGISTRY_LOCK_FILE)?;
  if let Some(parent) = lock_path.parent() {
    fs::create_dir_all(parent).map_err(|err| format!("failed to create data directory: {err}"))?;
  }
  let lock_file = fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open(&lock_path)
    .map_err(|err| format!("failed to open workspace registry lock file: {err}"))?;
  lock_file
    .lock_exclusive()
    .map_err(|err| format!("failed to lock workspace registry: {err}"))?;

  let registry_path = resolve_app_data_path(app, WORKSPACE_REGISTRY_FILE)?;
  let data = read_json_file(&registry_path)?;
  let mut registry = match data {
    Some(value) => serde_json::from_value::<HashMap<String, WorkspaceRegistryEntry>>(value)
      .map_err(|err| format!("failed to decode workspace registry: {err}"))?,
    None => HashMap::new(),
  };

  let result = f(&mut registry);
  if result.is_ok() {
    let payload = serde_json::to_value(&registry).map_err(|err| format!("failed to encode workspace registry: {err}"))?;
    write_json_file(&registry_path, payload)?;
  }

  let _ = lock_file.unlock();
  result
}

fn load_workspace_registry_snapshot(
  app: &AppHandle,
) -> Result<HashMap<String, WorkspaceRegistryEntry>, String> {
  with_workspace_registry(app, |registry| Ok(registry.clone()))
}

pub(crate) fn list_workspace_registry_ids(app: &AppHandle) -> Result<Vec<String>, String> {
  let registry = load_workspace_registry_snapshot(app)?;
  Ok(registry.keys().cloned().collect())
}

fn workspace_registry_mismatch_error(project_id: &str, last_known_path: &str, current_path: &str) -> String {
  // 前端通过前缀解析冲突详情，便于提示用户选择 Move/Copy。
  let payload = json!({
    "projectId": project_id,
    "lastKnownPath": last_known_path,
    "currentPath": current_path
  });
  format!("{WORKSPACE_REGISTRY_MISMATCH_PREFIX}{}", payload.to_string())
}

fn generate_project_id(seed: &str) -> Result<String, String> {
  // 时间戳 + 计数器降低高频生成冲突概率。
  let now = now_millis()?;
  let counter = PROJECT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
  Ok(hash_bytes(format!("{seed}-{now}-{counter}").as_bytes()))
}

fn update_workspace_project_id(metadata_path: &Path, project_id: &str) -> Result<(), String> {
  let payload = match read_json_file(metadata_path)? {
    Some(mut value) => {
      if let Some(obj) = value.as_object_mut() {
        obj.insert(
          "projectId".to_string(),
          serde_json::Value::String(project_id.to_string()),
        );
        value
      } else {
        json!({ "projectId": project_id })
      }
    }
    None => json!({ "projectId": project_id }),
  };
  write_json_file(metadata_path, payload)
}

fn resolve_workspace_local_state_path(workspace_path: &Path) -> PathBuf {
  // 本地状态与工作区元数据分离，便于同步/迁移。
  workspace_path.join(".golutra").join("local.json")
}


fn write_workspace_local_state(workspace_path: &Path, now: u64) -> Result<(), String> {
  let path = resolve_workspace_local_state_path(workspace_path);
  let existing = read_json_file(&path)?;
  let existing_state = existing
    .and_then(|value| serde_json::from_value::<LocalWorkspaceState>(value).ok());
  let local_machine_id = match existing_state
    .as_ref()
    .map(|state| state.local_machine_id.trim())
    .filter(|value| !value.is_empty())
  {
    Some(value) => value.to_string(),
    None => generate_project_id(&format!("local-{}", workspace_path.to_string_lossy()))?,
  };
  let next = LocalWorkspaceState {
    local_machine_id,
    last_opened_at: now,
  };
  let payload = serde_json::to_value(next).map_err(|err| format!("failed to encode local workspace state: {err}"))?;
  write_json_file(&path, payload)
}

#[tauri::command]
/// 在系统文件管理器中打开工作区目录。
/// 错误：路径为空、路径不存在或系统命令失败。
pub(crate) fn workspace_open_folder(path: String) -> Result<(), String> {
  let trimmed = path.trim();
  if trimmed.is_empty() {
    return Err("workspace path is empty".to_string());
  }
  let workspace_path = PathBuf::from(trimmed);
  if !workspace_path.exists() {
    return Err("workspace path does not exist".to_string());
  }
  if !workspace_path.is_dir() {
    return Err("workspace path is not a folder".to_string());
  }
  open_folder_in_file_manager(&workspace_path)
}

pub(crate) fn open_folder_in_file_manager(path: &Path) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    Command::new("explorer")
      .arg(path)
      .spawn()
      .map_err(|err| format!("failed to open folder in explorer: {err}"))?;
    return Ok(());
  }
  #[cfg(target_os = "macos")]
  {
    Command::new("open")
      .arg(path)
      .spawn()
      .map_err(|err| format!("failed to open folder in Finder: {err}"))?;
    return Ok(());
  }
  #[cfg(all(unix, not(target_os = "macos")))]
  {
    Command::new("xdg-open")
      .arg(path)
      .spawn()
      .map_err(|err| format!("failed to open folder via xdg-open: {err}"))?;
    return Ok(());
  }
  #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
  {
    let _ = path;
    Err("open folder is not supported on this platform".to_string())
  }
}

#[tauri::command]
/// 列出最近打开的工作区。
/// 返回：按最近时间排序的列表。
pub(crate) fn workspace_recent_list(app: AppHandle) -> Result<Vec<WorkspaceEntry>, String> {
  load_recent_workspaces(&app)
}

#[tauri::command]
/// 打开工作区并完成注册表同步。
/// 输入：`resolution` 用于处理路径冲突（Move/Copy）；`window_label` 用于避免重复打开。
/// 返回：工作区条目与只读/警告信息。
/// 错误：路径非法、注册表冲突未解决、写入元数据失败且需要 Copy 等。
pub(crate) fn workspace_open(
  app: AppHandle,
  state: State<'_, AppState>,
  window_label: Option<String>,
  resolution: Option<WorkspaceRegistryResolution>,
  path: String,
) -> Result<WorkspaceOpenResult, String> {
  let window_label = window_label.unwrap_or_else(|| "main".to_string());
  let workspace_path = PathBuf::from(&path)
    .canonicalize()
    .map_err(|err| format!("failed to resolve workspace path: {err}"))?;
  if !workspace_path.is_dir() {
    return Err("selected workspace path is not a folder".to_string());
  }
  let name = workspace_path
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| "workspace folder name is not valid UTF-8".to_string())?
    .to_string();
  let path_hash = hash_workspace_id(&workspace_path)?;
  let (mut id, read_only, mut warning) = match ensure_project_id(&workspace_path, &path_hash) {
    Ok(id) => (id, false, None),
    Err(err) => (
      path_hash.clone(),
      true,
      Some(format!(
        "Workspace opened in read-only mode. Failed to write .golutra/workspace.json: {err}"
      )),
    ),
  };
  let current_path = workspace_path.to_string_lossy().to_string();
  let current_path_normalized = normalize_registry_path(&current_path);
  let now = now_millis()?;
  let mut resolved_id = id.clone();
  with_workspace_registry(&app, |registry| {
    gc_workspace_registry(registry, now);
    if let Some(entry) = registry.get_mut(&id) {
      let known_path = entry.last_known_path.clone();
      if normalize_registry_path(&known_path) != current_path_normalized {
        match resolution {
          None => {
            return Err(workspace_registry_mismatch_error(&id, &known_path, &current_path));
          }
          Some(WorkspaceRegistryResolution::Move) => {
            entry.last_known_path = current_path.clone();
            entry.last_accessed = now;
          }
          Some(WorkspaceRegistryResolution::Copy) => {
            if read_only {
              return Err("workspace is read-only; cannot create a new project id".to_string());
            }
            let metadata_path = workspace_path.join(".golutra").join("workspace.json");
            let mut next_id = generate_project_id(&current_path)?;
            while registry.contains_key(&next_id) {
              next_id = generate_project_id(&current_path)?;
            }
            update_workspace_project_id(&metadata_path, &next_id)?;
            resolved_id = next_id.clone();
            registry.insert(
              next_id,
              WorkspaceRegistryEntry {
                last_known_path: current_path.clone(),
                last_accessed: now,
              },
            );
          }
        }
      } else {
        entry.last_accessed = now;
      }
    } else {
      registry.insert(
        id.clone(),
        WorkspaceRegistryEntry {
          last_known_path: current_path.clone(),
          last_accessed: now,
        },
      );
    }
    Ok(())
  })?;
  id = resolved_id;

  let mut guard = state
    .workspace_windows
    .lock()
    .map_err(|_| "workspace registry lock poisoned".to_string())?;
  let existing_label = guard.iter().find_map(|(label, workspace_id)| {
    if workspace_id == &id && label != &window_label {
      Some(label.clone())
    } else {
      None
    }
  });
  if let Some(existing_label) = existing_label {
    drop(guard);
    if let Some(window) = app.get_webview_window(&existing_label) {
      let _ = window.show();
      let _ = window.set_focus();
    }
    if let Some(window) = app.get_webview_window(&window_label) {
      let _ = window.close();
    }
    return Err("workspace already open in another window".to_string());
  }
  guard.insert(window_label.clone(), id.clone());
  drop(guard);
  let entry = WorkspaceEntry {
    id: id.clone(),
    name,
    path: current_path.clone(),
    last_opened_at: now,
  };

  let mut recent = load_recent_workspaces(&app)?;
  recent.retain(|item| item.id != entry.id && item.path != entry.path);
  recent.insert(0, entry.clone());
  // 控制最近列表大小，避免 UI 加载过慢。
  if recent.len() > 20 {
    recent.truncate(20);
  }
  save_recent_workspaces(&app, &recent)?;
  if !read_only {
    if let Err(err) = write_workspace_local_state(&workspace_path, now) {
      let message = format!("Failed to write .golutra/local.json: {err}");
      warning = Some(match warning {
        Some(existing) => format!("{existing}\n{message}"),
        None => message,
      });
    }
  }
  Ok(WorkspaceOpenResult {
    entry,
    read_only,
    warning,
  })
}

#[tauri::command]
/// 从运行时注册表中移除窗口与工作区的映射。
/// 约束：若 label 不存在则无副作用。
pub(crate) fn workspace_clear_window(
  state: State<'_, AppState>,
  window_label: String,
) -> Result<(), String> {
  let mut guard = state
    .workspace_windows
    .lock()
    .map_err(|_| "workspace registry lock poisoned".to_string())?;
  guard.remove(&window_label);
  Ok(())
}

#[tauri::command]
/// 读取应用数据目录中的 JSON 文件。
/// 错误：路径非法、读取或解析失败。
pub(crate) fn storage_read_app(app: AppHandle, relative_path: String) -> Result<Option<serde_json::Value>, String> {
  let storage = app.state::<StorageManager>();
  storage::read_app_json(storage.inner(), &relative_path)
}

#[tauri::command]
/// 写入应用数据目录中的 JSON 文件。
/// 错误：路径非法或写入失败。
pub(crate) fn storage_write_app(
  app: AppHandle,
  relative_path: String,
  payload: serde_json::Value,
) -> Result<(), String> {
  let storage = app.state::<StorageManager>();
  storage::write_app_json(storage.inner(), &relative_path, payload)
}

#[tauri::command]
/// 读取应用缓存目录中的 JSON 文件。
/// 错误：路径非法、读取或解析失败。
pub(crate) fn storage_read_cache(app: AppHandle, relative_path: String) -> Result<Option<serde_json::Value>, String> {
  let storage = app.state::<StorageManager>();
  storage::read_cache_json(storage.inner(), &relative_path)
}

#[tauri::command]
/// 写入应用缓存目录中的 JSON 文件。
/// 错误：路径非法或写入失败。
pub(crate) fn storage_write_cache(
  app: AppHandle,
  relative_path: String,
  payload: serde_json::Value,
) -> Result<(), String> {
  let storage = app.state::<StorageManager>();
  storage::write_cache_json(storage.inner(), &relative_path, payload)
}

#[tauri::command]
/// 写入应用缓存目录中的文本文件并返回绝对路径。
/// 错误：路径非法或写入失败。
pub(crate) fn storage_write_cache_text(
  app: AppHandle,
  relative_path: String,
  contents: String,
) -> Result<String, String> {
  let storage = app.state::<StorageManager>();
  storage::write_cache_text(storage.inner(), &relative_path, &contents)
}

#[tauri::command]
/// 读取工作区内的 JSON 文件（路径受限于工作区根目录）。
/// 错误：路径非法、读取或解析失败。
pub(crate) fn storage_read_workspace(
  workspace_path: String,
  relative_path: String,
) -> Result<Option<serde_json::Value>, String> {
  let path = resolve_workspace_path(&workspace_path, &relative_path)?;
  read_json_file(&path)
}

#[tauri::command]
/// 写入工作区内的 JSON 文件（路径受限于工作区根目录）。
/// 错误：路径非法或写入失败。
pub(crate) fn storage_write_workspace(
  workspace_path: String,
  relative_path: String,
  payload: serde_json::Value,
) -> Result<(), String> {
  let path = resolve_workspace_path(&workspace_path, &relative_path)?;
  write_json_file(&path, payload)
}

#[tauri::command]
/// 列出头像库条目。
/// 约束：会清理磁盘上不存在的记录以保持一致性。
pub(crate) fn avatar_list(app: AppHandle) -> Result<Vec<AvatarAsset>, String> {
  let mut entries = read_avatar_library(&app)?;
  let original_len = entries.len();
  entries.retain(|entry| {
    resolve_avatar_file_path(&app, &entry.filename)
      .map(|path| path.exists())
      .unwrap_or(false)
  });
  if entries.len() != original_len {
    write_avatar_library(&app, &entries)?;
  }
  Ok(entries)
}

#[tauri::command]
/// 存储头像并返回元数据。
/// 错误：内容为空、超过大小限制或写入失败。
pub(crate) fn avatar_store(
  app: AppHandle,
  bytes: Vec<u8>,
  extension: Option<String>,
) -> Result<AvatarAsset, String> {
  if bytes.is_empty() {
    return Err("avatar bytes are empty".to_string());
  }
  if bytes.len() > MAX_AVATAR_BYTES {
    return Err("avatar exceeds max size".to_string());
  }
  let ext = sanitize_extension(extension.as_deref().unwrap_or("png"));
  let id = hash_bytes(&bytes);
  let filename = format!("{id}.{ext}");
  let path = resolve_avatar_file_path(&app, &filename)?;
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|err| format!("failed to create avatar dir: {err}"))?;
  }
  fs::write(&path, &bytes).map_err(|err| format!("failed to write avatar file: {err}"))?;

  let mut entries = read_avatar_library(&app)?;
  let mut created_at = None;
  for entry in entries.iter_mut() {
    if entry.id == id {
      entry.filename = filename.clone();
      created_at = Some(entry.created_at);
      break;
    }
  }

  let created_at = created_at.unwrap_or(now_millis()?);
  if !entries.iter().any(|entry| entry.id == id) {
    entries.insert(
      0,
      AvatarAsset {
        id: id.clone(),
        filename: filename.clone(),
        created_at,
      },
    );
  }

  write_avatar_library(&app, &entries)?;
  entries
    .into_iter()
    .find(|entry| entry.id == id)
    .ok_or_else(|| "failed to store avatar metadata".to_string())
}

#[tauri::command]
/// 删除头像条目与文件。
/// 返回：是否实际删除。
pub(crate) fn avatar_delete(app: AppHandle, id: String) -> Result<bool, String> {
  let mut entries = read_avatar_library(&app)?;
  if let Some(index) = entries.iter().position(|entry| entry.id == id) {
    let entry = entries.remove(index);
    let path = resolve_avatar_file_path(&app, &entry.filename)?;
    if path.exists() {
      fs::remove_file(&path).map_err(|err| format!("failed to remove avatar file: {err}"))?;
    }
    write_avatar_library(&app, &entries)?;
    return Ok(true);
  }
  Ok(false)
}

#[tauri::command]
/// 将头像 ID 解析为本地路径（供前端加载）。
/// 错误：头像不存在或文件缺失。
pub(crate) fn avatar_resolve_path(app: AppHandle, id: String) -> Result<String, String> {
  let entries = read_avatar_library(&app)?;
  let entry = entries
    .into_iter()
    .find(|entry| entry.id == id)
    .ok_or_else(|| "avatar not found".to_string())?;
  let path = resolve_avatar_file_path(&app, &entry.filename)?;
  if !path.exists() {
    return Err("avatar file missing".to_string());
  }
  Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
/// 读取头像内容与 MIME。
/// 错误：头像不存在或读取失败。
pub(crate) fn avatar_read(app: AppHandle, id: String) -> Result<AvatarContent, String> {
  let entries = read_avatar_library(&app)?;
  let entry = entries
    .into_iter()
    .find(|entry| entry.id == id)
    .ok_or_else(|| "avatar not found".to_string())?;
  let path = resolve_avatar_file_path(&app, &entry.filename)?;
  if !path.exists() {
    return Err("avatar file missing".to_string());
  }
  let bytes = fs::read(&path).map_err(|err| format!("failed to read avatar file: {err}"))?;
  let mime = Path::new(&entry.filename)
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|ext| match ext.to_ascii_lowercase().as_str() {
      "png" => "image/png",
      "jpg" | "jpeg" => "image/jpeg",
      "webp" => "image/webp",
      "gif" => "image/gif",
      _ => "application/octet-stream",
    })
    .unwrap_or("application/octet-stream")
    .to_string();
  Ok(AvatarContent { bytes, mime })
}

