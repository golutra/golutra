export type BaseTerminal = {
  id: string;
  nameKey: string;
  command: string;
  terminalType: 'codex' | 'gemini' | 'claude' | 'shell';
  icon: string;
  gradient: string;
};

export const BASE_TERMINALS: BaseTerminal[] = [
  {
    id: 'gemini-cli',
    nameKey: 'settings.memberOptions.gemini',
    command: 'gemini',
    terminalType: 'gemini',
    icon: 'token',
    gradient: 'from-sky-500 to-cyan-400'
  },
  {
    id: 'codex',
    nameKey: 'settings.memberOptions.codex',
    command: 'codex',
    terminalType: 'codex',
    icon: 'code',
    gradient: 'from-emerald-500 to-lime-400'
  },
  {
    id: 'claude-code',
    nameKey: 'settings.memberOptions.claude',
    command: 'claude',
    terminalType: 'claude',
    icon: 'psychology',
    gradient: 'from-amber-500 to-orange-400'
  },
  {
    id: 'terminal',
    nameKey: 'settings.memberOptions.terminal',
    command: '',
    terminalType: 'shell',
    icon: 'terminal',
    gradient: 'from-slate-500 to-slate-300'
  }
];

export const CUSTOM_TERMINAL_ICON = 'settings_suggest';
export const CUSTOM_TERMINAL_GRADIENT = 'from-slate-500 to-slate-300';
