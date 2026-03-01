//! 通知状态管理：维护未读 badge 聚合，并驱动托盘与悬浮窗更新。

use std::{
  collections::HashMap,
  sync::Mutex,
  time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::{
  AppHandle,
  Emitter,
  LogicalSize,
  Manager,
  PhysicalPosition,
  Position,
  Rect,
  Size,
  State,
  WebviewUrl,
  WebviewWindow,
  WebviewWindowBuilder,
};
use tauri::async_runtime::JoinHandle;
use tauri::image::Image;
use tokio::time::sleep;

use super::app::{list_workspace_registry_ids, MAIN_WINDOW_LABEL};
use crate::runtime::state::AppState;
use crate::message_service::chat_db::{chat_mark_workspace_read_latest, ChatDbManager};

pub const TRAY_ICON_ID: &str = "main-tray";
pub const PREVIEW_WINDOW_LABEL: &str = "notification-preview";
const PREVIEW_EVENT_NAME: &str = "notification-preview-updated";
const BLINK_INTERVAL_MS: u64 = 500;
const PREVIEW_HIDE_DELAY_MS: u64 = 240;
const PREVIEW_WIDTH: f64 = 320.0;
const PREVIEW_MIN_HEIGHT: f64 = 180.0;
const PREVIEW_MAX_HEIGHT: f64 = 720.0;
const PREVIEW_MARGIN: f64 = 8.0;
const PREVIEW_MAX_ITEMS: usize = 6;
const PREVIEW_ITEM_HEIGHT: f64 = 80.0;
const PREVIEW_ITEM_GAP: f64 = 8.0;
const PREVIEW_HEADER_HEIGHT: f64 = 22.0;
const PREVIEW_FOOTER_HEIGHT: f64 = 28.0;
const PREVIEW_SECTION_GAP: f64 = 12.0;
const PREVIEW_PADDING_Y: f64 = 28.0;
const PREVIEW_CARD_BORDER: f64 = 2.0;
// 预留边距，避免 DPI 取整导致底部被裁切。
const PREVIEW_SAFE_PADDING: f64 = 8.0;
const UNREAD_ICON_BYTES: &[u8] = include_bytes!("../../icons/icon-unread.png");
const TRANSPARENT_ICON_BYTES: &[u8] = include_bytes!("../../icons/Transparency.png");

fn is_main_window_label(label: &str) -> bool {
  label == MAIN_WINDOW_LABEL || label.starts_with("main-")
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreviewState {
  title: String,
  total_unread: u32,
  items: Vec<NotificationPreviewItem>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreviewItem {
  workspace_id: Option<String>,
  workspace_name: Option<String>,
  workspace_path: Option<String>,
  conversation_id: Option<String>,
  conversation_name: Option<String>,
  conversation_unread: u32,
  conversation_type: Option<String>,
  member_count: Option<u32>,
  sender_id: Option<String>,
  sender_name: Option<String>,
  sender_avatar: Option<String>,
  sender_can_open_terminal: bool,
  preview: String,
  last_message_at: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationUpdatePayload {
  window_label: String,
  unread_count: i64,
  title: String,
  total_unread: Option<i64>,
  items: Vec<NotificationPreviewItemPayload>,
  avatar_png: Option<Vec<u8>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreviewItemPayload {
  workspace_id: Option<String>,
  workspace_name: Option<String>,
  workspace_path: Option<String>,
  conversation_id: Option<String>,
  conversation_name: Option<String>,
  conversation_unread: i64,
  conversation_type: Option<String>,
  member_count: Option<i64>,
  sender_id: Option<String>,
  sender_name: Option<String>,
  sender_avatar: Option<String>,
  sender_can_open_terminal: Option<bool>,
  preview: String,
  last_message_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotificationOpenTerminalPayload {
  pub workspace_id: String,
  pub conversation_id: String,
  pub sender_id: String,
}

impl Default for NotificationPreviewState {
  fn default() -> Self {
    Self {
      title: String::new(),
      total_unread: 0,
      items: Vec::new(),
    }
  }
}

#[derive(Clone)]
struct NotificationWindowPreview {
  items: Vec<NotificationPreviewItem>,
  avatar_key: Option<String>,
  avatar_png: Option<Vec<u8>>,
}

struct NotificationIcons {
  default_icon: Option<Image<'static>>,
  fallback_unread_icon: Option<Image<'static>>,
  transparent_icon: Option<Image<'static>>,
}

struct NotificationBadgeStateInner {
  window_counts: HashMap<String, u32>,
  window_previews: HashMap<String, NotificationWindowPreview>,
  last_total: u32,
  last_title: String,
  blink_task: Option<JoinHandle<()>>,
  hide_task: Option<JoinHandle<()>>,
  tray_hovered: bool,
  preview_hovered: bool,
  last_preview_window_label: Option<String>,
  last_tray_rect: Option<Rect>,
  last_preview_size: Option<(f64, f64)>,
  preview: NotificationPreviewState,
  active_avatar_key: Option<String>,
  active_avatar_window: Option<String>,
  avatar_icon_base: Option<Image<'static>>,
  avatar_icon_pulse: Option<Image<'static>>,
}

/// 通知 badge 状态：按窗口聚合未读数，并驱动托盘与悬浮窗展示。
pub struct NotificationBadgeState {
  inner: Mutex<NotificationBadgeStateInner>,
  icons: NotificationIcons,
}

impl NotificationBadgeState {
  pub fn new(app: &AppHandle) -> Self {
    let default_icon = app.default_window_icon().map(|icon| icon.clone().to_owned());
    let fallback_unread_icon = load_unread_icon();
    let transparent_icon = load_transparent_icon();
    Self {
      inner: Mutex::new(NotificationBadgeStateInner {
        window_counts: HashMap::new(),
        window_previews: HashMap::new(),
        last_total: 0,
        last_title: String::new(),
        blink_task: None,
        hide_task: None,
        tray_hovered: false,
        preview_hovered: false,
        last_preview_window_label: None,
        last_tray_rect: None,
        last_preview_size: None,
        preview: NotificationPreviewState::default(),
        active_avatar_key: None,
        active_avatar_window: None,
        avatar_icon_base: None,
        avatar_icon_pulse: None,
      }),
      icons: NotificationIcons {
        default_icon,
        fallback_unread_icon,
        transparent_icon,
      },
    }
  }

  pub fn clear_window(&self, app: &AppHandle, window_label: &str) {
    if window_label.trim().is_empty() {
      return;
    }
    let avatar_changed = match self.remove_window_preview(window_label) {
      Ok(changed) => changed,
      Err(error) => {
        log::warn!("notification badge clear failed: {error}");
        return;
      }
    };
    match self.set_window_count(window_label, 0) {
      Ok((total, changed)) => {
        if changed || avatar_changed {
          self.apply_badge(app, total);
        }
      }
      Err(error) => log::warn!("notification badge clear failed: {error}"),
    }
    self.sync_preview_window(app);
  }

  pub fn clear_all(&self, app: &AppHandle) {
    let Ok(mut guard) = self.inner.lock() else {
      return;
    };
    guard.window_counts.clear();
    guard.window_previews.clear();
    guard.last_total = 0;
    guard.last_title.clear();
    guard.last_preview_window_label = None;
    guard.preview = NotificationPreviewState::default();
    guard.active_avatar_key = None;
    guard.active_avatar_window = None;
    guard.avatar_icon_base = None;
    guard.avatar_icon_pulse = None;
    guard.tray_hovered = false;
    guard.preview_hovered = false;
    if let Some(task) = guard.hide_task.take() {
      task.abort();
    }
    drop(guard);
    self.apply_badge(app, 0);
    self.sync_preview_window(app);
  }

  pub fn update_state(
    &self,
    app: &AppHandle,
    window_label: &str,
    count: u32,
    preview: NotificationPreviewState,
    avatar_png: Option<Vec<u8>>,
  ) -> Result<(), String> {
    let (total, changed) = self.set_window_count(window_label, count)?;
    let avatar_changed = self.set_preview_state(window_label, preview, avatar_png)?;
    if avatar_changed {
      let _ = self.stop_blink(app);
    }
    if changed || avatar_changed {
      self.apply_badge(app, total);
    }
    self.sync_preview_window(app);
    Ok(())
  }

  pub fn preview_state(&self) -> NotificationPreviewState {
    self
      .inner
      .lock()
      .map(|guard| guard.preview.clone())
      .unwrap_or_default()
  }

  pub fn preview_window_label(&self) -> Option<String> {
    self
      .inner
      .lock()
      .ok()
      .and_then(|guard| guard.last_preview_window_label.clone())
  }

  pub fn preview_window_labels(&self) -> Vec<String> {
    self
      .inner
      .lock()
      .map(|guard| guard.window_previews.keys().cloned().collect())
      .unwrap_or_default()
  }

  pub fn set_tray_hovered(&self, app: &AppHandle, hovered: bool, rect: Option<Rect>) {
    if let Ok(mut guard) = self.inner.lock() {
      guard.tray_hovered = hovered;
      if rect.is_some() {
        guard.last_tray_rect = rect;
      }
      if !hovered {
        self.schedule_preview_hide(app.clone(), &mut guard);
        return;
      }
      if let Some(task) = guard.hide_task.take() {
        task.abort();
      }
    }
    self.sync_preview_window(app);
  }

  pub fn set_preview_hovered(&self, app: &AppHandle, hovered: bool) {
    if let Ok(mut guard) = self.inner.lock() {
      guard.preview_hovered = hovered;
      if !hovered {
        // [HACK/OPTIMIZE, 2026-01-24] 原因: 托盘 hover 事件可能丢失，导致预览无法隐藏; 替代方案: 查询鼠标位置并判断是否仍在托盘; 移除条件: 托盘 hover 事件稳定或引入精确鼠标判断。
        guard.tray_hovered = false;
        self.schedule_preview_hide(app.clone(), &mut guard);
        return;
      }
      if let Some(task) = guard.hide_task.take() {
        task.abort();
      }
    }
    self.sync_preview_window(app);
  }

  pub fn force_hide_preview(&self, app: &AppHandle) {
    if let Ok(mut guard) = self.inner.lock() {
      guard.tray_hovered = false;
      guard.preview_hovered = false;
      if let Some(task) = guard.hide_task.take() {
        task.abort();
      }
    }
    hide_preview_window(app);
  }

  fn schedule_preview_hide(&self, app: AppHandle, guard: &mut NotificationBadgeStateInner) {
    if guard.hide_task.is_some() {
      return;
    }
    guard.hide_task = Some(tauri::async_runtime::spawn(async move {
      sleep(Duration::from_millis(PREVIEW_HIDE_DELAY_MS)).await;
      let badge_state = app.state::<NotificationBadgeState>();
      let Ok(mut state) = badge_state.inner.lock() else {
        return;
      };
      if state.tray_hovered || state.preview_hovered {
        state.hide_task = None;
        return;
      }
      state.tray_hovered = false;
      state.preview_hovered = false;
      state.hide_task = None;
      drop(state);
      hide_preview_window(&app);
    }));
  }

  fn set_preview_state(
    &self,
    window_label: &str,
    preview: NotificationPreviewState,
    avatar_png: Option<Vec<u8>>,
  ) -> Result<bool, String> {
    let mut guard = self
      .inner
      .lock()
      .map_err(|_| "notification badge lock poisoned".to_string())?;
    guard.last_preview_window_label = Some(window_label.to_string());
    let avatar_key = preview.items.first().and_then(|item| item.sender_avatar.clone());
    guard.window_previews.insert(
      window_label.to_string(),
      NotificationWindowPreview {
        items: preview.items.clone(),
        avatar_key,
        avatar_png,
      },
    );
    if !preview.title.is_empty() {
      guard.last_title = preview.title;
    }
    Ok(self.refresh_preview_snapshot(&mut guard))
  }

  fn set_window_count(&self, window_label: &str, count: u32) -> Result<(u32, bool), String> {
    let mut guard = self
      .inner
      .lock()
      .map_err(|_| "notification badge lock poisoned".to_string())?;
    if count == 0 {
      guard.window_counts.remove(window_label);
    } else {
      guard.window_counts.insert(window_label.to_string(), count);
    }
    let mut total = 0u32;
    for value in guard.window_counts.values() {
      total = total.saturating_add(*value);
    }
    let changed = total != guard.last_total;
    if changed {
      guard.last_total = total;
    }
    Ok((total, changed))
  }

  fn remove_window_preview(&self, window_label: &str) -> Result<bool, String> {
    let mut guard = self
      .inner
      .lock()
      .map_err(|_| "notification badge lock poisoned".to_string())?;
    guard.window_previews.remove(window_label);
    if guard
      .last_preview_window_label
      .as_deref()
      .is_some_and(|label| label == window_label)
    {
      guard.last_preview_window_label = None;
    }
    Ok(self.refresh_preview_snapshot(&mut guard))
  }

  fn refresh_preview_snapshot(&self, guard: &mut NotificationBadgeStateInner) -> bool {
    let previous_key = guard.active_avatar_key.clone();
    let previous_has_avatar = guard.avatar_icon_base.is_some();

    let mut merged: Vec<(NotificationPreviewItem, String)> = Vec::new();
    for (label, preview) in guard.window_previews.iter() {
      for item in &preview.items {
        merged.push((item.clone(), label.clone()));
      }
    }
    merged.sort_by_key(|(item, _)| std::cmp::Reverse(item.last_message_at.unwrap_or(0)));
    let active_window = merged.first().map(|(_, label)| label.clone());
    let items: Vec<NotificationPreviewItem> = merged
      .into_iter()
      .map(|(item, _)| item)
      .take(PREVIEW_MAX_ITEMS)
      .collect();

    guard.preview = NotificationPreviewState {
      title: guard.last_title.clone(),
      total_unread: guard.last_total,
      items,
    };
    guard.active_avatar_window = active_window.clone();
    let (next_key, next_png) = active_window
      .as_ref()
      .and_then(|label| guard.window_previews.get(label))
      .map(|preview| (preview.avatar_key.clone(), preview.avatar_png.clone()))
      .unwrap_or((None, None));
    guard.active_avatar_key = next_key.clone();

    let avatar_presence_changed = previous_has_avatar != next_png.is_some();
    let avatar_key_changed = previous_key != next_key;
    if avatar_key_changed || avatar_presence_changed {
      if let Some(bytes) = next_png {
        match Image::from_bytes(&bytes) {
          Ok(icon) => {
            let base_icon = icon.clone().to_owned();
            let pulse_icon = build_pulse_avatar_icon(&icon).unwrap_or_else(|| base_icon.clone());
            guard.avatar_icon_base = Some(base_icon);
            guard.avatar_icon_pulse = Some(pulse_icon);
          }
          Err(error) => {
            log::warn!("Failed to decode tray avatar: {error}");
            guard.avatar_icon_base = None;
            guard.avatar_icon_pulse = None;
          }
        }
      } else {
        guard.avatar_icon_base = None;
        guard.avatar_icon_pulse = None;
      }
    }
    avatar_key_changed || avatar_presence_changed
  }

  fn apply_badge(&self, app: &AppHandle, total: u32) {
    let has_unread = total > 0;
    if has_unread {
      self.set_tray_icon(app, true);
      if let Err(error) = self.start_blink(app.clone()) {
        log::warn!("notification badge blink start failed: {error}");
      }
    } else {
      if let Err(error) = self.stop_blink(app) {
        log::warn!("notification badge blink stop failed: {error}");
      }
      self.set_tray_icon(app, false);
    }
    self.set_tray_tooltip(app, total);
    self.set_taskbar_badge(app, total);
  }

  fn resolved_base_icon(&self, guard: &NotificationBadgeStateInner) -> Option<Image<'static>> {
    guard.avatar_icon_base.clone().or_else(|| self.icons.default_icon.clone())
  }

  fn resolved_pulse_icon(&self, guard: &NotificationBadgeStateInner) -> Option<Image<'static>> {
    guard
      .avatar_icon_pulse
      .clone()
      .or_else(|| self.icons.fallback_unread_icon.clone())
  }

  fn set_tray_icon(&self, app: &AppHandle, use_unread: bool) {
    let Some(tray) = app.tray_by_id(TRAY_ICON_ID) else {
      return;
    };
    let icon = if use_unread {
      self
        .inner
        .lock()
        .ok()
        .and_then(|guard| self.resolved_pulse_icon(&guard))
        .or_else(|| self.icons.default_icon.clone())
    } else {
      self
        .inner
        .lock()
        .ok()
        .and_then(|guard| self.resolved_base_icon(&guard))
        .or_else(|| self.icons.default_icon.clone())
    };
    let _ = tray.set_icon(icon);
    let _ = tray.set_visible(true);
  }

  fn set_tray_tooltip(&self, app: &AppHandle, total: u32) {
    let Some(tray) = app.tray_by_id(TRAY_ICON_ID) else {
      return;
    };
    let base = app.package_info().name.clone();
    let tooltip = if total > 0 {
      format!("{base} ({total})")
    } else {
      base
    };
    let _ = tray.set_tooltip(Some(tooltip));
  }

  fn set_taskbar_badge(&self, app: &AppHandle, total: u32) {
    let target = resolve_taskbar_window_label(app);
    let Some(window) = target.and_then(|label| app.get_webview_window(label.as_str())) else {
      return;
    };
    let count = if total == 0 { None } else { Some(total as i64) };
    let _ = window.set_badge_count(count);
  }

  fn start_blink(&self, app: AppHandle) -> Result<(), String> {
    let mut guard = self
      .inner
      .lock()
      .map_err(|_| "notification badge lock poisoned".to_string())?;
    if guard.blink_task.is_some() {
      return Ok(());
    }
    let Some(base_icon) = guard
      .avatar_icon_base
      .clone()
      .or_else(|| self.icons.fallback_unread_icon.clone())
      .or_else(|| self.icons.default_icon.clone())
    else {
      return Ok(());
    };
    let transparent_icon = self.icons.transparent_icon.clone().unwrap_or_else(|| base_icon.clone());
    guard.blink_task = Some(tauri::async_runtime::spawn(async move {
      // 通过原图/透明图切换保持托盘占位，避免闪烁时被系统回收位置。
      let mut show_transparent = true;
      loop {
        sleep(Duration::from_millis(BLINK_INTERVAL_MS)).await;
        let Some(tray) = app.tray_by_id(TRAY_ICON_ID) else {
          return;
        };
        let icon = if show_transparent {
          transparent_icon.clone()
        } else {
          base_icon.clone()
        };
        show_transparent = !show_transparent;
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_visible(true);
      }
    }));
    Ok(())
  }

  fn stop_blink(&self, app: &AppHandle) -> Result<(), String> {
    let mut guard = self
      .inner
      .lock()
      .map_err(|_| "notification badge lock poisoned".to_string())?;
    if let Some(task) = guard.blink_task.take() {
      task.abort();
    }
    if let Some(tray) = app.tray_by_id(TRAY_ICON_ID) {
      let _ = tray.set_visible(true);
    }
    Ok(())
  }

  fn sync_preview_window(&self, app: &AppHandle) {
    let (should_show, rect, preview) = match self.inner.lock() {
      Ok(guard) => {
        let has_unread = guard.preview.total_unread > 0;
        let hovered = guard.tray_hovered || guard.preview_hovered;
        (has_unread && hovered, guard.last_tray_rect, guard.preview.clone())
      }
      Err(_) => return,
    };
    if !should_show {
      hide_preview_window(app);
      return;
    }
    let window = match ensure_preview_window(app) {
      Ok(window) => window,
      Err(error) => {
        log::warn!("notification preview window failed: {error}");
        return;
      }
    };
    if let Ok(mut guard) = self.inner.lock() {
      update_preview_window_size(&window, &mut guard);
    }
    if let Some(rect) = rect {
      position_preview_window(app, &window, rect);
    }
    let _ = window.show();
    let _ = app.emit_to(PREVIEW_WINDOW_LABEL, PREVIEW_EVENT_NAME, preview);
  }
}

fn preview_height_for_items(items: &[NotificationPreviewItem]) -> f64 {
  if items.is_empty() {
    return PREVIEW_MIN_HEIGHT;
  }
  let visible = items.len().min(PREVIEW_MAX_ITEMS) as f64;
  let gaps = (visible - 1.0).max(0.0) * PREVIEW_ITEM_GAP;
  let items_height = visible * PREVIEW_ITEM_HEIGHT + gaps;
  let base = PREVIEW_PADDING_Y
    + PREVIEW_HEADER_HEIGHT
    + PREVIEW_FOOTER_HEIGHT
    + PREVIEW_SECTION_GAP * 2.0
    + PREVIEW_CARD_BORDER;
  (base + items_height + PREVIEW_SAFE_PADDING).clamp(PREVIEW_MIN_HEIGHT, PREVIEW_MAX_HEIGHT)
}

fn update_preview_window_size(window: &WebviewWindow, guard: &mut NotificationBadgeStateInner) {
  let target_height = preview_height_for_items(&guard.preview.items);
  let target_size = (PREVIEW_WIDTH, target_height);
  if guard.last_preview_size == Some(target_size) {
    return;
  }
  guard.last_preview_size = Some(target_size);
  let _ = window.set_min_size(Some(Size::Logical(LogicalSize::new(
    PREVIEW_WIDTH,
    PREVIEW_MIN_HEIGHT,
  ))));
  let _ = window.set_max_size(Some(Size::Logical(LogicalSize::new(
    PREVIEW_WIDTH,
    PREVIEW_MAX_HEIGHT,
  ))));
  let _ = window.set_size(Size::Logical(LogicalSize::new(PREVIEW_WIDTH, target_height)));
}

fn load_unread_icon() -> Option<Image<'static>> {
  match Image::from_bytes(UNREAD_ICON_BYTES) {
    Ok(icon) => Some(icon.to_owned()),
    Err(error) => {
      log::warn!("Failed to load unread tray icon: {error}");
      None
    }
  }
}

fn load_transparent_icon() -> Option<Image<'static>> {
  match Image::from_bytes(TRANSPARENT_ICON_BYTES) {
    Ok(icon) => Some(icon.to_owned()),
    Err(error) => {
      log::warn!("Failed to load transparent tray icon: {error}");
      None
    }
  }
}


fn build_pulse_avatar_icon(icon: &Image<'_>) -> Option<Image<'static>> {
  let width = icon.width();
  let height = icon.height();
  if width == 0 || height == 0 {
    return None;
  }
  let pixel_count = (width as usize).checked_mul(height as usize)?;
  let expected_len = pixel_count.checked_mul(4)?;
  let source = icon.rgba();
  if source.len() < expected_len {
    return None;
  }
  let mut rgba = source.to_vec();
  for chunk in rgba.chunks_exact_mut(4) {
    if chunk[3] == 0 {
      continue;
    }
    let r = (chunk[0] as f32 * 1.12 + 14.0).min(255.0) as u8;
    let g = (chunk[1] as f32 * 1.08 + 10.0).min(255.0) as u8;
    let b = (chunk[2] as f32 * 1.08 + 10.0).min(255.0) as u8;
    chunk[0] = r;
    chunk[1] = g;
    chunk[2] = b;
  }

  // 轻微高亮外圈，制造“闪动”对比感。
  let size = width.min(height) as f32;
  let cx = (width as f32 - 1.0) / 2.0;
  let cy = (height as f32 - 1.0) / 2.0;
  let radius = size * 0.42;
  let thickness = (size * 0.06).max(1.0);
  let inner = (radius - thickness).max(0.0);
  let inner2 = inner * inner;
  let outer2 = (radius + thickness) * (radius + thickness);
  for y in 0..height {
    for x in 0..width {
      let dx = x as f32 - cx;
      let dy = y as f32 - cy;
      let dist2 = dx * dx + dy * dy;
      if dist2 < inner2 || dist2 > outer2 {
        continue;
      }
      let idx = ((y * width + x) * 4) as usize;
      if rgba[idx + 3] == 0 {
        continue;
      }
      rgba[idx] = rgba[idx].saturating_add(28);
      rgba[idx + 1] = rgba[idx + 1].saturating_add(12);
      rgba[idx + 2] = rgba[idx + 2].saturating_add(8);
    }
  }
  Some(Image::new_owned(rgba, width, height))
}

fn ensure_preview_window(app: &AppHandle) -> Result<WebviewWindow, String> {
  if let Some(window) = app.get_webview_window(PREVIEW_WINDOW_LABEL) {
    return Ok(window);
  }
  let init_script = "window.__GOLUTRA_VIEW__ = 'notification-preview';";
  let window = WebviewWindowBuilder::new(app, PREVIEW_WINDOW_LABEL, WebviewUrl::App("index.html".into()))
    .initialization_script(init_script)
    .title("golutra")
    .inner_size(PREVIEW_WIDTH, PREVIEW_MIN_HEIGHT)
    .min_inner_size(PREVIEW_WIDTH, PREVIEW_MIN_HEIGHT)
    .max_inner_size(PREVIEW_WIDTH, PREVIEW_MAX_HEIGHT)
    .resizable(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .visible(false)
    .build()
    .map_err(|err| format!("failed to create notification preview window: {err}"))?;
  Ok(window)
}

fn position_preview_window(app: &AppHandle, window: &WebviewWindow, rect: Rect) {
  let (preview_width, preview_height) = window
    .inner_size()
    .map(|size| (size.width as f64, size.height as f64))
    .unwrap_or((PREVIEW_WIDTH, PREVIEW_MIN_HEIGHT));
  let (origin_x, origin_y, width, height) = rect_metrics(rect);
  let mut x = origin_x + width - preview_width;
  let mut y = origin_y - preview_height - PREVIEW_MARGIN;
  if y < 0.0 {
    y = origin_y + height + PREVIEW_MARGIN;
  }
  if x < 0.0 {
    x = origin_x.max(0.0);
  }
  let monitor = app
    .monitor_from_point(origin_x + width * 0.5, origin_y + height * 0.5)
    .ok()
    .flatten()
    .or_else(|| app.primary_monitor().ok().flatten());
  if let Some(monitor) = monitor {
    let work_area = monitor.work_area();
    let min_x = work_area.position.x as f64;
    let min_y = work_area.position.y as f64;
    let max_x = (min_x + work_area.size.width as f64 - preview_width).max(min_x);
    let max_y = (min_y + work_area.size.height as f64 - preview_height).max(min_y);
    x = x.clamp(min_x, max_x);
    y = y.clamp(min_y, max_y);
  }
  let _ = window.set_position(Position::Physical(PhysicalPosition::new(x as i32, y as i32)));
}

fn rect_metrics(rect: Rect) -> (f64, f64, f64, f64) {
  let (x, y) = match rect.position {
    Position::Physical(position) => (position.x as f64, position.y as f64),
    Position::Logical(position) => (position.x, position.y),
  };
  let (width, height) = match rect.size {
    Size::Physical(size) => (size.width as f64, size.height as f64),
    Size::Logical(size) => (size.width, size.height),
  };
  (x, y, width, height)
}

fn resolve_taskbar_window_label(app: &AppHandle) -> Option<String> {
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

fn hide_preview_window(app: &AppHandle) {
  if let Some(window) = app.get_webview_window(PREVIEW_WINDOW_LABEL) {
    let _ = window.hide();
  }
}

#[tauri::command]
pub fn notification_update_state(
  app: AppHandle,
  state: State<'_, NotificationBadgeState>,
  window_state: State<'_, AppState>,
  payload: NotificationUpdatePayload,
) -> Result<(), String> {
  let label = payload.window_label.trim();
  if label.is_empty() {
    return Ok(());
  }
  if !is_main_window_label(label) {
    return Ok(());
  }
  {
    let mut guard = window_state
      .active_main_window
      .lock()
      .map_err(|_| "active main window lock poisoned".to_string())?;
    if guard.is_none() {
      *guard = Some(label.to_string());
    }
  }
  let clamp_u32 = |value: i64| {
    if value <= 0 {
      0
    } else if value > u32::MAX as i64 {
      u32::MAX
    } else {
      value as u32
    }
  };
  let window_count = clamp_u32(payload.unread_count);
  let preview_total = clamp_u32(payload.total_unread.unwrap_or(payload.unread_count));
  let preview_items = payload
    .items
    .into_iter()
    .map(|item| NotificationPreviewItem {
      workspace_id: item.workspace_id,
      workspace_name: item.workspace_name,
      workspace_path: item.workspace_path,
      conversation_id: item.conversation_id,
      conversation_name: item.conversation_name,
      conversation_unread: clamp_u32(item.conversation_unread),
      conversation_type: item.conversation_type,
      member_count: item.member_count.and_then(|value| {
        if value <= 0 {
          None
        } else {
          Some(clamp_u32(value))
        }
      }),
      sender_id: item.sender_id,
      sender_name: item.sender_name,
      sender_avatar: item.sender_avatar,
      sender_can_open_terminal: item.sender_can_open_terminal.unwrap_or(false),
      preview: item.preview,
      last_message_at: item.last_message_at.and_then(|value| {
        if value <= 0 {
          None
        } else {
          Some(value as u64)
        }
      }),
    })
    .collect();
  let preview_state = NotificationPreviewState {
    title: payload.title,
    total_unread: preview_total,
    items: preview_items,
  };
  state.update_state(&app, label, window_count, preview_state, payload.avatar_png)?;
  Ok(())
}

#[tauri::command]
pub fn notification_set_active_window(
  app: AppHandle,
  window_state: State<'_, AppState>,
  window_label: String,
) -> Result<bool, String> {
  let label = window_label.trim();
  if label.is_empty() {
    return Ok(false);
  }
  if !is_main_window_label(label) {
    return Ok(false);
  }
  let mut guard = window_state
    .active_main_window
    .lock()
    .map_err(|_| "active main window lock poisoned".to_string())?;
  let changed = guard.as_deref() != Some(label);
  *guard = Some(label.to_string());
  if changed {
    let badge_state = app.state::<NotificationBadgeState>();
    badge_state.set_tray_tooltip(&app, badge_state.preview_state().total_unread);
  }
  Ok(true)
}

#[tauri::command]
pub fn notification_get_state(state: State<'_, NotificationBadgeState>) -> NotificationPreviewState {
  state.preview_state()
}

#[tauri::command]
pub fn notification_preview_hover(
  app: AppHandle,
  state: State<'_, NotificationBadgeState>,
  hovered: bool,
) -> Result<(), String> {
  state.set_preview_hovered(&app, hovered);
  Ok(())
}

#[tauri::command]
pub fn notification_preview_hide(
  app: AppHandle,
  state: State<'_, NotificationBadgeState>,
) -> Result<(), String> {
  state.force_hide_preview(&app);
  Ok(())
}

#[tauri::command]
pub fn notification_request_ignore_all(
  app: AppHandle,
  state: State<'_, NotificationBadgeState>,
  chat_state: State<'_, ChatDbManager>,
  user_id: String,
) -> Result<(), String> {
  let user_id = user_id.trim();
  if user_id.is_empty() {
    return Ok(());
  }
  let user_id = user_id.to_string();
  let targets = state.preview_window_labels();
  let fallback = state
    .preview_window_label()
    .unwrap_or_else(|| MAIN_WINDOW_LABEL.to_string());
  let workspace_ids = list_workspace_registry_ids(&app)?;
  for workspace_id in workspace_ids {
    if let Err(error) =
      chat_mark_workspace_read_latest(&app, &chat_state, workspace_id, user_id.clone())
    {
      log::warn!("notification ignore all failed to mark read: {error}");
    }
  }
  state.clear_all(&app);
  if targets.is_empty() {
    let _ = app.emit_to(fallback.as_str(), "notification-ignore-all", ());
    return Ok(());
  }
  for label in targets {
    let _ = app.emit_to(label.as_str(), "notification-ignore-all", ());
  }
  Ok(())
}
