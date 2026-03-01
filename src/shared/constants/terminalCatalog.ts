// 终端成员的内置目录与默认视觉配置，作为邀请与展示的来源。
export type BaseTerminal = {
  id: string;
  nameKey?: string;
  label?: string;
  command: string;
  terminalType: 'codex' | 'gemini' | 'claude' | 'shell';
  icon: string;
  gradient: string;
};

// 目录顺序用于 UI 默认排序与索引映射，不应随意重排。
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
    id: 'opencode',
    nameKey: 'settings.memberOptions.opencode',
    command: 'opencode',
    terminalType: 'opencode',
    icon: 'code',
    gradient: 'from-indigo-500 to-blue-400'
  },
  {
    id: 'qwen-code',
    nameKey: 'settings.memberOptions.qwen',
    command: 'qwen',
    terminalType: 'qwen',
    icon: 'model_training',
    gradient: 'from-teal-500 to-emerald-400'
  },
  {
    id: 'terminal',
    label: 'Terminal',
    // 空命令表示使用系统默认 shell。
    command: '',
    terminalType: 'shell',
    icon: 'terminal',
    gradient: 'from-slate-500 to-slate-300'
  }
];

// 自定义终端使用统一图标与渐变，避免与内置模型视觉混淆。
export const CUSTOM_TERMINAL_ICON = 'settings_suggest';
export const CUSTOM_TERMINAL_GRADIENT = 'from-slate-500 to-slate-300';

export const resolveBaseTerminalLabel = (terminal: BaseTerminal, translate: (key: string) => string) => {
  if (terminal.label) {
    return terminal.label;
  }
  if (terminal.nameKey) {
    return translate(terminal.nameKey);
  }
  return terminal.id;
};
