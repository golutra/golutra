// Tauri 窗口封装：隔离 UI 与平台 API 交互细节，统一错误回退策略。
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

/**
 * 获取当前窗口 label。
 * 输入：无。
 * 输出：label 或 null。
 * 错误语义：在非 Tauri 环境或调用失败时返回 null。
 */
export const getCurrentWindowLabel = () => {
  try {
    return getCurrentWebviewWindow().label || null;
  } catch {
    return null;
  }
};

/**
 * 打开工作区选择窗口。
 * 输入：无。
 * 输出：新窗口 label 或 null。
 * 边界：在非浏览器上下文下直接返回 null。
 */
export const openWorkspaceSelectionWindow = async () => {
  if (typeof window === 'undefined') {
    return null;
  }
  return invoke<string>('workspace_selection_open_window');
};

/**
 * 清理当前窗口关联的工作区状态。
 * 输入：无（内部读取当前窗口 label）。
 * 输出：Tauri 命令返回值或 null。
 */
export const clearWorkspaceWindow = async () => {
  const label = getCurrentWindowLabel();
  if (!label) {
    return null;
  }
  return invoke<void>('workspace_clear_window', { windowLabel: label });
};
