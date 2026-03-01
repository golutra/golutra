// Tauri 终端入口：提供设置页等轻量查询能力。
import { invoke } from '@tauri-apps/api/core';
import type { TerminalEnvironmentOption } from '@/shared/types/terminal';

/**
 * 获取当前系统可用的终端环境列表。
 * 输出：终端环境数组（id/label/path）。
 */
export const listTerminalEnvironments = async (): Promise<TerminalEnvironmentOption[]> =>
  invoke<TerminalEnvironmentOption[]>('terminal_list_environments');
