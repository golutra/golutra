import { invoke } from '@tauri-apps/api/core';

export type TerminalWindowOptions = {
  reuse?: boolean;
  workspaceId?: string;
  workspaceName?: string;
  workspacePath?: string;
};

export type TerminalWindowResult = {
  label: string;
  reused: boolean;
};

export const openTerminalWindow = async (options?: TerminalWindowOptions) => {
  if (typeof window === 'undefined') {
    return null;
  }

  return invoke<TerminalWindowResult>('terminal_open_window', {
    reuse: options?.reuse ?? true,
    workspaceId: options?.workspaceId,
    workspaceName: options?.workspaceName,
    workspacePath: options?.workspacePath
  });
};
