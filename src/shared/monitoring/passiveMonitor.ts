// 前端被动监控入口：统一控制 console 日志与诊断发送开关。
const resolveFrontDebugFlag = () => {
  try {
    return import.meta.env.GOLUTRA_Front_DEBUG === '1';
  } catch {
    return false;
  }
};

export const isFrontDebugEnabled = () => resolveFrontDebugFlag();

export const createFrontLogger = (label: string) => {
  return (...args: unknown[]) => {
    if (!isFrontDebugEnabled()) {
      return;
    }
    console.info(`[${label}]`, ...args);
  };
};
