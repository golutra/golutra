//! 终端会话映射端口：隔离终端引擎对 member_id ↔ session_id 存储的直接依赖。

use std::sync::Arc;

pub(crate) trait TerminalSessionRepository: Send + Sync {
    fn upsert_terminal_session(
        &self,
        workspace_id: &str,
        member_id: &str,
        session_id: &str,
    ) -> Result<(), String>;

    fn delete_terminal_session(&self, workspace_id: &str, member_id: &str) -> Result<(), String>;

    /// 查询 member_id 对应的 session_id。
    fn get_terminal_session(
        &self,
        workspace_id: &str,
        member_id: &str,
    ) -> Result<Option<String>, String>;
}

struct NoopTerminalSessionRepository;

impl TerminalSessionRepository for NoopTerminalSessionRepository {
    fn upsert_terminal_session(
        &self,
        _workspace_id: &str,
        _member_id: &str,
        _session_id: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    fn delete_terminal_session(&self, _workspace_id: &str, _member_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn get_terminal_session(
        &self,
        _workspace_id: &str,
        _member_id: &str,
    ) -> Result<Option<String>, String> {
        Ok(None)
    }
}

pub(crate) fn default_terminal_session_repository() -> Arc<dyn TerminalSessionRepository> {
    Arc::new(NoopTerminalSessionRepository)
}
