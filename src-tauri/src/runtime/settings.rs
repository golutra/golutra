//! 全局设置服务：负责读取与缓存应用配置。

use super::storage::{read_app_json, StorageManager};
use serde::Deserialize;

const GLOBAL_SETTINGS_PATH: &str = "global-settings.json";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredGlobalSettings {
    locale: Option<String>,
    chat: Option<StoredChatSettings>,
    general: Option<StoredGeneralSettings>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredChatSettings {
    stream_output: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredGeneralSettings {
    language: Option<String>,
}

pub(crate) struct SettingsService {
    storage: StorageManager,
}

impl SettingsService {
    pub(crate) fn new(storage: StorageManager) -> Self {
        Self { storage }
    }

    fn read_settings(&self) -> Option<StoredGlobalSettings> {
        let raw = read_app_json(&self.storage, GLOBAL_SETTINGS_PATH)
            .ok()
            .flatten()?;
        serde_json::from_value(raw).ok()
    }

    pub(crate) fn get_language(&self) -> Option<String> {
        let parsed = self.read_settings()?;
        parsed.locale.or_else(|| parsed.general.and_then(|g| g.language))
    }

    pub(crate) fn get_chat_stream_enabled(&self) -> Option<bool> {
        let parsed = self.read_settings()?;
        parsed.chat.and_then(|chat| chat.stream_output)
    }
}

impl crate::ports::settings::TerminalSettingsPort for SettingsService {
    fn get_language(&self) -> Option<String> {
        self.get_language()
    }

    fn get_chat_stream_enabled(&self) -> Option<bool> {
        self.get_chat_stream_enabled()
    }
}
