// 监控 IPC 接口：隔离前端逻辑与后端命令调用。
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { FrontendLogEntry } from './diagnostics/types';

const safeInvoke = async (command: string, payload: Record<string, unknown>) => {
  if (!isTauri()) {
    return true;
  }
  try {
    await invoke(command, payload);
    return true;
  } catch {
    // IPC 失败不阻断业务逻辑。
    return false;
  }
};

export const diagnosticsStartRun = async (payload: {
  runId: string;
  workspaceId?: string;
  label?: string;
}) => {
  await safeInvoke('diagnostics_start_run', payload);
};

export const diagnosticsEndRun = async (payload: { runId: string; status?: string }) => {
  await safeInvoke('diagnostics_end_run', payload);
};

export const diagnosticsRegisterMember = async (payload: {
  runId: string;
  memberId: string;
  terminalType?: string | null;
  terminalCommand?: string | null;
  name?: string | null;
}) => {
  await safeInvoke('diagnostics_register_member', payload);
};

export const diagnosticsRegisterSession = async (payload: {
  runId: string;
  terminalId: string;
  memberId?: string;
}) => {
  await safeInvoke('diagnostics_register_session', payload);
};

export const diagnosticsRegisterConversation = async (payload: {
  runId: string;
  conversationId: string;
  memberId?: string;
}) => {
  await safeInvoke('diagnostics_register_conversation', payload);
};

export const diagnosticsRegisterWindow = async (payload: { runId: string; windowLabel: string }) => {
  await safeInvoke('diagnostics_register_window', payload);
};

export const diagnosticsLogFrontendEvent = async (payload: {
  runId: string;
  round?: number;
  memberId?: string;
  stepId: string;
  payload: unknown;
  clientTs?: number;
  seq?: number;
}) => {
  await safeInvoke('diagnostics_log_frontend_event', payload);
};

export const diagnosticsLogFrontendBatch = async (entries: FrontendLogEntry[]) => {
  if (entries.length === 0) {
    return true;
  }
  return await safeInvoke('diagnostics_log_frontend_batch', { entries });
};

export const diagnosticsLogSnapshotTriplet = async (payload: {
  runId: string;
  round: number;
  memberId: string;
  frontBefore: string[];
  backendStored: string[];
  frontReopen: string[];
}) => {
  await safeInvoke('diagnostics_log_snapshot_triplet', payload);
};

export const diagnosticsLogChatConsistency = async (payload: {
  runId: string;
  round: number;
  memberId: string;
  replyText?: string | null;
  replyMissing?: boolean;
  terminalLines: string[];
}) => {
  await safeInvoke('diagnostics_log_chat_consistency', payload);
};
