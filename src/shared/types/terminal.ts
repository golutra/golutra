export type TerminalType = 'shell' | 'codex' | 'gemini' | 'claude';
export type TerminalConnectionStatus = 'connecting' | 'connected' | 'working' | 'disconnected';

export const isTerminalType = (value: unknown): value is TerminalType =>
  value === 'shell' || value === 'codex' || value === 'gemini' || value === 'claude';

export const isTerminalConnectionStatus = (value: unknown): value is TerminalConnectionStatus =>
  value === 'connecting' || value === 'connected' || value === 'working' || value === 'disconnected';

