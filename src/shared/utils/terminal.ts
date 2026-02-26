import { isTerminalType, type TerminalType } from '@/shared/types/terminal';

const BUILTIN_COMMAND_MAP: Record<string, TerminalType> = {
  codex: 'codex',
  gemini: 'gemini',
  claude: 'claude'
};

export const normalizeTerminalCommand = (value?: string | null) => {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
};

export const normalizeTerminalPath = (value?: string | null) => {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
};

export const resolveTerminalType = (terminalType: unknown, terminalCommand?: string | null) => {
  const command = normalizeTerminalCommand(terminalCommand);
  if (isTerminalType(terminalType)) {
    return terminalType;
  }
  if (!command) {
    return undefined;
  }
  const [binary, ...rest] = command.split(/\s+/);
  if (rest.length === 0) {
    const mapped = BUILTIN_COMMAND_MAP[binary.toLowerCase()];
    if (mapped) {
      return mapped;
    }
  }
  return 'shell';
};

export const hasTerminalConfig = (terminalType: unknown, terminalCommand?: string | null) =>
  Boolean(resolveTerminalType(terminalType, terminalCommand));

export const resolveTerminalLabel = (terminalType?: TerminalType | null) => {
  if (terminalType === 'codex') return 'Codex';
  if (terminalType === 'gemini') return 'Gemini';
  if (terminalType === 'claude') return 'Claude';
  if (terminalType === 'shell') return 'Shell';
  return '';
};

