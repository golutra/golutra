//! 终端设置端口：提供应用全局配置读取能力。

pub trait TerminalSettingsPort: Send + Sync {
    /// 获取当前语言设置（如 "zh", "en"）
    fn get_language(&self) -> Option<String>;

    /// 获取聊天流式输出开关（true=开启）
    fn get_chat_stream_enabled(&self) -> Option<bool>;
}
