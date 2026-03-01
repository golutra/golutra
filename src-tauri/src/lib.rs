//! 应用主进程入口：注册 Tauri 命令、窗口与本地持久化路径解析。
//! 边界：不处理 UI 逻辑，只提供系统级服务与存储能力。

use std::sync::Arc;

use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_log::{Target, TargetKind};

mod application;
mod contracts;
mod message_service;
mod orchestration;
mod platform;
mod ports;
mod runtime;
mod terminal_engine;
mod ui_gateway;

pub use runtime::command_ipc::COMMAND_IPC_NAME;
pub(crate) use ui_gateway::app::now_millis;

use platform::{resolve_log_dir, ActivationState, UpdaterState};
use runtime::spawn_command_ipc_server;
use runtime::state::AppState;
use runtime::{CommandCenter, StorageManager};
use orchestration::chat_dispatch_batcher::ChatDispatchBatcher;
use orchestration::chat_outbox::spawn_chat_outbox_worker;
use terminal_engine::TerminalManager;
use ui_gateway::message_pipeline::{
    UiMessageRepository, UiMessageTransport, UiTerminalMessagePipeline,
};
use ui_gateway::terminal_events::UiTerminalEventPort;
use ui_gateway::terminal_session_repository::UiTerminalSessionRepository;
use ui_gateway::{
    apply_main_window_size, apply_windows_rounding, cleanup_ephemeral_sessions_for_window, export_commands,
    has_active_sessions, schedule_main_window_frame_refresh, setup_tray, show_main_window, shutdown_sessions,
    spawn_snapshot_dumper, spawn_status_poller, ChatDbManager, DiagnosticsState, NotificationBadgeState,
    MAIN_WINDOW_LABEL,
};

fn is_main_window_label(label: &str) -> bool {
    label == MAIN_WINDOW_LABEL || label.starts_with("main-")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// 启动 Tauri 应用并注册所有命令与后台服务。
/// 约束：错误会在构建阶段直接 panic，以便尽早暴露配置问题。
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .manage(TerminalManager::default())
        .manage(ChatDbManager::default())
        .manage(Arc::new(CommandCenter::new()))
        .manage(DiagnosticsState::new())
        .manage(UpdaterState::new())
        .manage(ActivationState::new())
        .setup(|app| {
            let app_handle = app.handle();
            app.manage(NotificationBadgeState::new(&app_handle));
            let transport = Arc::new(UiMessageTransport::new(app_handle.clone()));
            let repository = Arc::new(UiMessageRepository::new(app_handle.clone()));
            let pipeline = Arc::new(UiTerminalMessagePipeline::new(transport, repository));
            app.state::<TerminalManager>()
                .set_message_pipeline(pipeline);
            let app_data_dir = app.path().app_data_dir()?;
            let app_cache_dir = app.path().app_cache_dir()?;
            let storage_manager = StorageManager::new(app_data_dir.clone(), app_cache_dir);
            app.manage(storage_manager.clone());
            let dispatch_batcher = Arc::new(ChatDispatchBatcher::new());
            app.manage(dispatch_batcher.clone());
            app.state::<TerminalManager>()
                .set_dispatch_gate(dispatch_batcher.clone());
            let settings_service = Arc::new(crate::runtime::settings::SettingsService::new(
                storage_manager,
            ));
            app.state::<TerminalManager>()
                .set_settings_service(settings_service);
            app.state::<ChatDbManager>().set_base_dir(app_data_dir);
            let event_port = Arc::new(UiTerminalEventPort::new(app_handle.clone()));
            app.state::<TerminalManager>().set_event_port(event_port);
            let session_repository = Arc::new(UiTerminalSessionRepository::new(app_handle.clone()));
            app.state::<TerminalManager>()
                .set_session_repository(session_repository);
            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                apply_windows_rounding(&window);
                apply_main_window_size(&app_handle, &window);
            }
            if cfg!(debug_assertions) {
                // 仅在调试态输出详细日志，避免生产环境额外 IO。
                let log_dir = resolve_log_dir();
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .targets([
                            Target::new(TargetKind::Stdout),
                            Target::new(TargetKind::Folder {
                                path: log_dir,
                                file_name: Some("sidecar-debug".to_string()),
                            }),
                        ])
                        .build(),
                )?;
            }
            let manager = &app.state::<TerminalManager>();
            spawn_status_poller(app.handle().clone(), manager);
            spawn_snapshot_dumper(manager, resolve_log_dir());
            spawn_chat_outbox_worker(app.handle().clone());
            let command_center = app.state::<Arc<CommandCenter>>();
            if let Err(err) =
                spawn_command_ipc_server(app.handle().clone(), Arc::clone(command_center.inner()))
            {
                log::warn!("command ipc server failed: {err}");
            }
            #[cfg(desktop)]
            {
                setup_tray(app)?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_main_window(app);
            let _ = app.emit_to(MAIN_WINDOW_LABEL, "app-open-workspace-selection", ());
        }))
        .invoke_handler(export_commands())
        .on_window_event(|window, event| {
            #[cfg(target_os = "windows")]
            if matches!(
                event,
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }
            ) {
                if let Some(webview_window) = window.app_handle().get_webview_window(window.label())
                {
                    apply_windows_rounding(&webview_window);
                }
            }
            if matches!(event, WindowEvent::Focused(true)) {
                let label = window.label();
                if is_main_window_label(label) {
                    if let Ok(mut guard) = window
                        .app_handle()
                        .state::<AppState>()
                        .active_main_window
                        .lock()
                    {
                        *guard = Some(label.to_string());
                    }
                    #[cfg(target_os = "windows")]
                    if let Some(webview_window) = window
                        .app_handle()
                        .get_webview_window(label)
                    {
                        schedule_main_window_frame_refresh(&webview_window);
                    }
                }
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                if cfg!(target_os = "windows") && window.label() == MAIN_WINDOW_LABEL {
                    // Windows 关闭按钮走托盘最小化，避免误退出。
                    api.prevent_close();
                    let _ = window.hide();
                    return;
                }
            }
            if matches!(
                event,
                WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed
            ) {
                let app = window.app_handle();
                if matches!(event, WindowEvent::Destroyed) {
                    // 窗口真正销毁后清理临时会话，避免资源泄露。
                    let state = &app.state::<TerminalManager>();
                    let _ = cleanup_ephemeral_sessions_for_window(state, window.label());
                    let notification_state = app.state::<NotificationBadgeState>();
                    notification_state.clear_window(&app, window.label());
                    if is_main_window_label(window.label()) {
                        if let Ok(mut guard) = app.state::<AppState>().active_main_window.lock() {
                            if guard.as_deref() == Some(window.label()) {
                                *guard = None;
                            }
                        }
                    }
                }
                if let Ok(mut guard) = app.state::<AppState>().workspace_windows.lock() {
                    guard.remove(window.label());
                }
                if app.webview_windows().len() <= 1 {
                    // 最后一个窗口退出时，仅在无活跃会话时终止终端后台。
                    let state = &app.state::<TerminalManager>();
                    if !has_active_sessions(state) {
                        let _ = shutdown_sessions(state);
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
                // 退出时主动关闭会话，避免子进程残留。
                let state = &app.state::<TerminalManager>();
                let _ = shutdown_sessions(state);
            }
        });
}
