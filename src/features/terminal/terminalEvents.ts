// 终端窗口事件协议：统一事件名与 payload 结构。
import type { TerminalType } from '@/shared/types/terminal';

// 事件名需要与后端/窗口通信保持一致，避免跨窗口监听失效。
export const TERMINAL_OPEN_TAB_EVENT = 'terminal-open-tab';
export const TERMINAL_WINDOW_READY_EVENT = 'terminal-window-ready';
export const TERMINAL_WINDOW_READY_REQUEST_EVENT = 'terminal-window-ready-request';
export const TERMINAL_SNAPSHOT_REQUEST_EVENT = 'terminal-snapshot-request';
export const TERMINAL_SNAPSHOT_RESPONSE_EVENT = 'terminal-snapshot-response';
export const TERMINAL_TAB_OPENED_EVENT = 'terminal-tab-opened';

// 终端标签页打开事件载荷。
export type TerminalOpenTabPayload = {
  terminalId: string;
  title: string;
  memberId?: string;
  terminalType?: TerminalType;
  keepAlive?: boolean;
};

// 终端窗口就绪事件载荷。
export type TerminalWindowReadyPayload = {
  windowLabel: string;
};

// 前端快照请求载荷。
export type TerminalSnapshotRequestPayload = {
  requestId: string;
  terminalId: string;
};

// 前端快照响应载荷。
export type TerminalSnapshotResponsePayload = {
  requestId: string;
  terminalId: string;
  lines?: string[];
  rows?: number;
  cols?: number;
  error?: string;
};

// 终端标签页打开确认，用于诊断流程等待 UI 侧完成建页。
export type TerminalTabOpenedPayload = {
  windowLabel: string;
  terminalId: string;
  memberId?: string;
  terminalType?: TerminalType;
};
