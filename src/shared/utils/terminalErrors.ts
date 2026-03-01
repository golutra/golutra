// 终端错误解析：解码后端错误码，供前端统一翻译与展示。
export type TerminalErrorDetail = {
  path?: string;
};

export type TerminalErrorPayload = {
  code?: string;
  message?: string;
  detail?: TerminalErrorDetail;
};

export const TERMINAL_ERROR_CODES = {
  shellBinaryNotFound: 'terminal.shell.binary_not_found',
  shellLaunchFailed: 'terminal.shell.launch_failed',
  terminalLaunchFailed: 'terminal.launch_failed'
} as const;

export type TerminalErrorCode = (typeof TERMINAL_ERROR_CODES)[keyof typeof TERMINAL_ERROR_CODES];

export const parseTerminalError = (error: unknown) => {
  const raw = error instanceof Error ? error.message : String(error);
  try {
    const parsed = JSON.parse(raw) as TerminalErrorPayload;
    if (parsed && typeof parsed.code === 'string') {
      return {
        code: parsed.code as TerminalErrorCode,
        message: parsed.message || raw,
        detail: parsed.detail
      };
    }
  } catch {
    // 忽略非结构化错误，回退到原始文本。
  }
  return { message: raw };
};

export const resolveTerminalErrorI18nKey = (code?: string) => {
  switch (code) {
    case TERMINAL_ERROR_CODES.shellBinaryNotFound:
      return 'settings.terminalTestErrors.shellBinaryNotFound';
    case TERMINAL_ERROR_CODES.shellLaunchFailed:
      return 'settings.terminalTestErrors.shellLaunchFailed';
    default:
      return null;
  }
};
