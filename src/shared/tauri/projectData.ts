// 项目数据 IPC 封装：统一通过后端业务服务层读写项目数据。
import { invoke } from '@tauri-apps/api/core';
import type { TerminalType } from '@/shared/types/terminal';

export type ProjectDataSource = 'workspace' | 'app' | 'none';
export type ProjectDataStorage = 'workspace' | 'app';

export type ProjectDataReadResult<T> = {
  data: T | null;
  source: ProjectDataSource;
  warning?: string | null;
};

export type ProjectDataWriteResult = {
  storage: ProjectDataStorage;
  warning?: string | null;
};

export type ProjectInviteMembersRequest = {
  roleType: 'assistant' | 'member';
  command?: string;
  terminalType?: TerminalType;
  instanceCount: number;
  unlimitedAccess: boolean;
  sandboxed: boolean;
};

export type ProjectInviteMembersResult<T> = {
  members: T[];
  createdMembers: T[];
  storage: ProjectDataStorage;
  warning?: string | null;
};

export type ProjectPurgeTerminalMembersScope = 'current';

export type ProjectPurgeTerminalMembersResult = {
  scope: ProjectPurgeTerminalMembersScope;
  totalRemoved: number;
  workspaceCount: number;
  warnings: string[];
};

/**
 * 读取项目数据。
 * 输入：工作区 ID 与路径。
 * 输出：项目数据读取结果。
 */
export const readProjectData = async <T>(
  workspaceId: string,
  workspacePath: string
): Promise<ProjectDataReadResult<T>> =>
  invoke<ProjectDataReadResult<T>>('project_data_read', { workspaceId, workspacePath });

/**
 * 写入项目数据。
 * 输入：工作区 ID 与路径、只读标记与 payload。
 * 输出：项目数据写入结果。
 */
export const writeProjectData = async (
  workspaceId: string,
  workspacePath: string,
  readOnly: boolean,
  payload: unknown
): Promise<ProjectDataWriteResult> =>
  invoke<ProjectDataWriteResult>('project_data_write', {
    workspaceId,
    workspacePath,
    readOnly,
    payload
  });

/**
 * 邀请并创建项目成员（后端统一生成）。
 * 输入：workspaceId 与成员创建参数。
 * 输出：完整成员列表与本次创建成员。
 */
export const inviteProjectMembers = async <T>(
  workspaceId: string,
  payload: ProjectInviteMembersRequest
): Promise<ProjectInviteMembersResult<T>> =>
  invoke<ProjectInviteMembersResult<T>>('project_members_invite', {
    workspaceId,
    payload
  });

/**
 * 清理终端好友并重置序号。
 * 输入：workspaceId 与清理范围。
 * 输出：清理统计结果。
 */
export const purgeProjectTerminalMembers = async (
  workspaceId: string,
  scope: ProjectPurgeTerminalMembersScope
): Promise<ProjectPurgeTerminalMembersResult> =>
  invoke<ProjectPurgeTerminalMembersResult>('project_members_purge_terminal', {
    workspaceId,
    payload: { scope }
  });
