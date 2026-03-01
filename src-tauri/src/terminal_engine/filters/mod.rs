//! 终端输出过滤入口：按 CLI 类型加载规则，供语义快照与未来流式复用。

pub(crate) mod profiles;
pub(crate) mod registry;
pub(crate) mod rules;
pub(crate) mod state;
pub(crate) mod types;

pub(crate) use state::FilterRuntime;
pub(crate) use types::{FilterContext, FilterDecision, FilterMode, FilterSource};
