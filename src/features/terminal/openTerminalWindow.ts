// 终端窗口打开入口：封装 Tauri 调用与复用策略参数。
import { invoke } from '@tauri-apps/api/core';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';

export type TerminalWindowOptions = {
  reuse?: boolean;
  workspaceId?: string;
  workspaceName?: string;
  workspacePath?: string;
  autoTab?: boolean;
};

export type TerminalWindowResult = {
  label: string;
  reused: boolean;
};

/**
 * 打开或复用终端窗口。
 * 输入：窗口复用与工作区上下文。
 * 输出：窗口信息或 null（非浏览器环境）。
 */
export const openTerminalWindow = async (options?: TerminalWindowOptions) => {
  if (typeof window === 'undefined') {
    return null;
  }

  void logDiagnosticsEvent('open-terminal-window', {
    reuse: options?.reuse ?? true,
    workspaceId: options?.workspaceId,
    workspaceName: options?.workspaceName,
    workspacePath: options?.workspacePath,
    autoTab: options?.autoTab
  });
  return invoke<TerminalWindowResult>('terminal_open_window', {
    reuse: options?.reuse ?? true,
    workspaceId: options?.workspaceId,
    workspaceName: options?.workspaceName,
    workspacePath: options?.workspacePath,
    autoTab: options?.autoTab
  });
};

