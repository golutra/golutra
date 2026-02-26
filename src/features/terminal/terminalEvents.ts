import type { TerminalType } from '@/shared/types/terminal';

export const TERMINAL_OPEN_TAB_EVENT = 'terminal-open-tab';
export const TERMINAL_WINDOW_READY_EVENT = 'terminal-window-ready';
export const TERMINAL_WINDOW_READY_REQUEST_EVENT = 'terminal-window-ready-request';

export type TerminalOpenTabPayload = {
  sessionId: string;
  title: string;
  memberId?: string;
  terminalType?: TerminalType;
  keepAlive?: boolean;
};

export type TerminalWindowReadyPayload = {
  windowLabel: string;
};
