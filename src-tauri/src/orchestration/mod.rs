//! 编排层入口：承载工作流与跨模块自动化编排逻辑。

pub(crate) mod terminal_friend_invite;
pub(crate) mod dispatch;
pub(crate) mod chat_dispatch_batcher;
pub(crate) mod chat_outbox;
