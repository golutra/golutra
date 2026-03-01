//! 终端会话映射适配：通过 UI 层将 member_id 与远端 session_id 落库。

use tauri::{AppHandle, Manager};

use crate::message_service::chat_db::{
    terminal_session_delete_by_member_id, terminal_session_get_by_member_id,
    terminal_session_upsert, ChatDbManager,
};
use crate::ports::terminal_session::TerminalSessionRepository;

pub(crate) struct UiTerminalSessionRepository {
    app: AppHandle,
}

impl UiTerminalSessionRepository {
    pub(crate) fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl TerminalSessionRepository for UiTerminalSessionRepository {
    fn upsert_terminal_session(
        &self,
        workspace_id: &str,
        member_id: &str,
        session_id: &str,
    ) -> Result<(), String> {
        let state = self.app.state::<ChatDbManager>();
        terminal_session_upsert(state.inner(), workspace_id, member_id, session_id).map(|_| ())
    }

    fn delete_terminal_session(&self, workspace_id: &str, member_id: &str) -> Result<(), String> {
        let state = self.app.state::<ChatDbManager>();
        terminal_session_delete_by_member_id(state.inner(), workspace_id, member_id)
    }

    fn get_terminal_session(
        &self,
        workspace_id: &str,
        member_id: &str,
    ) -> Result<Option<String>, String> {
        let state = self.app.state::<ChatDbManager>();
        terminal_session_get_by_member_id(state.inner(), workspace_id, member_id)
    }
}
