import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

export const getCurrentWindowLabel = () => {
  try {
    return getCurrentWebviewWindow().label || null;
  } catch {
    return null;
  }
};

export const openWorkspaceSelectionWindow = async () => {
  if (typeof window === 'undefined') {
    return null;
  }
  return invoke<string>('workspace_selection_open_window');
};

export const clearWorkspaceWindow = async () => {
  const label = getCurrentWindowLabel();
  if (!label) {
    return null;
  }
  return invoke<void>('workspace_clear_window', { windowLabel: label });
};
