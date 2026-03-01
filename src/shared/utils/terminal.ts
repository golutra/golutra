// 终端类型与命令的归一化工具，集中处理默认规则与兼容判定。
import { isTerminalType, type TerminalType } from '@/shared/types/terminal';

// 内置命令映射仅在命令为单一二进制名时生效，避免误判带参数的自定义命令。
const BUILTIN_COMMAND_MAP: Record<string, TerminalType> = {
  codex: 'codex',
  gemini: 'gemini',
  claude: 'claude',
  opencode: 'opencode',
  qwen: 'qwen'
};

/**
 * 归一化终端命令字符串。
 * 输入：可选的命令字符串。
 * 输出：去空白后的命令或 undefined。
 * 边界：空字符串视为未配置。
 */
export const normalizeTerminalCommand = (value?: string | null) => {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
};

/**
 * 归一化终端可执行路径。
 * 输入：可选路径字符串。
 * 输出：去空白后的路径或 undefined。
 * 边界：空字符串视为未配置。
 */
export const normalizeTerminalPath = (value?: string | null) => {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
};

/**
 * 解析终端类型。
 * 输入：显式类型或命令字符串。
 * 输出：解析出的类型，无法解析时返回 undefined。
 * 边界：命令为空时不强行回退到 shell。
 */
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

/**
 * 判断成员是否具备终端配置能力。
 * 输入：类型或命令。
 * 输出：是否可启用终端。
 */
export const hasTerminalConfig = (terminalType: unknown, terminalCommand?: string | null) =>
  Boolean(resolveTerminalType(terminalType, terminalCommand));

/**
 * 转换终端类型到展示名称。
 * 输入：终端类型。
 * 输出：展示字符串；未知类型返回空字符串。
 */
export const resolveTerminalLabel = (terminalType?: TerminalType | null) => {
  if (terminalType === 'codex') return 'Codex';
  if (terminalType === 'gemini') return 'Gemini';
  if (terminalType === 'claude') return 'Claude';
  if (terminalType === 'opencode') return 'opencode';
  if (terminalType === 'qwen') return 'Qwen Code';
  if (terminalType === 'shell') return 'Shell';
  return '';
};

