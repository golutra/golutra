// 终端相关基础类型与守卫函数，作为跨模块契约的唯一来源。
export type TerminalType = 'shell' | 'codex' | 'gemini' | 'claude' | 'opencode' | 'qwen';
export type TerminalConnectionStatus = 'pending' | 'connecting' | 'connected' | 'working' | 'disconnected';
export type TerminalPostReadyMode = 'invite' | 'none';

// 终端环境条目：用于设置页展示可用 shell 列表。
export type TerminalEnvironmentOption = {
  id: string;
  label: string;
  path: string;
};

// 终端好友邀请元信息：用于后端接收邀请时的上下文记录。
export type TerminalFriendInviteMeta = {
  memberName: string;
  defaultCommand: string;
  instanceCount: number;
  unlimitedAccess: boolean;
  sandboxed: boolean;
};

/**
 * 运行时类型守卫：判定是否为合法终端类型。
 * 输入：未知值。
 * 输出：是否为 TerminalType。
 */
export const isTerminalType = (value: unknown): value is TerminalType =>
  value === 'shell' ||
  value === 'codex' ||
  value === 'gemini' ||
  value === 'claude' ||
  value === 'opencode' ||
  value === 'qwen';

/**
 * 运行时类型守卫：判定是否为合法连接状态。
 * 输入：未知值。
 * 输出：是否为 TerminalConnectionStatus。
 */
export const isTerminalConnectionStatus = (value: unknown): value is TerminalConnectionStatus =>
  value === 'pending' ||
  value === 'connecting' ||
  value === 'connected' ||
  value === 'working' ||
  value === 'disconnected';
