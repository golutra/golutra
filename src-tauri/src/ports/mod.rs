//! 端口层入口：为跨层调用提供稳定接口。

pub(crate) mod message_service;
pub(crate) mod settings;
pub(crate) mod terminal_dispatch_gate;
pub(crate) mod terminal_event;
pub(crate) mod terminal_message;
pub(crate) mod terminal_session;
