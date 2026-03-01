//! 应用层：承载跨入口复用的业务用例，避免 UI/CLI 分叉。

pub(crate) mod chat;
pub(crate) mod command;
pub(crate) mod project;
