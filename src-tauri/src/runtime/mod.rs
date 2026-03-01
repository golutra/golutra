//! 运行时/基础设施层入口：承载 IPC、PTY 与持久化等系统能力。

pub(crate) mod command_center;
pub(crate) mod command_ipc;
pub(crate) mod pty;
pub(crate) mod settings;
pub(crate) mod state;
pub(crate) mod storage;

pub(crate) use command_center::CommandCenter;
pub(crate) use command_ipc::spawn_command_ipc_server;
pub(crate) use pty::{
    list_terminal_environments, lookup_binary, resize_pty, spawn_command, spawn_shell,
    TerminalEnvironmentOption, TerminalHandle,
};
pub(crate) use storage::{read_app_json, StorageManager};
