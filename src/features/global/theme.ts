// 主题引擎：负责系统主题解析与 DOM 同步，并写入本地缓存供首屏读取。
export type AppTheme = 'dark' | 'light' | 'system';

export type ThemeOption = {
  id: AppTheme;
  labelKey: string;
  descriptionKey: string;
};

export const THEME_CHANGED_EVENT = 'app-theme-changed';
const THEME_STORAGE_KEY = 'golutra-theme';

export const themeOptions: ThemeOption[] = [
  {
    id: 'dark',
    labelKey: 'settings.themeOptions.dark.label',
    descriptionKey: 'settings.themeOptions.dark.desc'
  },
  {
    id: 'light',
    labelKey: 'settings.themeOptions.light.label',
    descriptionKey: 'settings.themeOptions.light.desc'
  },
  {
    id: 'system',
    labelKey: 'settings.themeOptions.system.label',
    descriptionKey: 'settings.themeOptions.system.desc'
  }
];

// 读取系统偏好主题，仅在浏览器环境有效。
const getSystemTheme = (): 'dark' | 'light' => {
  if (typeof window === 'undefined' || !window.matchMedia) {
    return 'dark';
  }
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
};

// 直接写入 DOM 数据集，供 CSS 主题变量切换使用。
const applyThemeToDom = (value: AppTheme) => {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  root.dataset.theme = value;
  const resolved = value === 'system' ? getSystemTheme() : value;
  root.dataset.resolvedTheme = resolved;
  root.classList.toggle('dark', resolved === 'dark');
};

let mediaQuery: MediaQueryList | null = null;
let mediaHandler: ((event: MediaQueryListEvent) => void) | null = null;

// 监听系统主题变化，仅在 theme=system 时启用。
const startSystemListener = () => {
  if (typeof window === 'undefined' || !window.matchMedia || mediaQuery) return;
  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  mediaHandler = () => applyThemeToDom('system');
  mediaQuery.addEventListener('change', mediaHandler);
};

// 取消系统主题监听，避免重复触发。
const stopSystemListener = () => {
  if (!mediaQuery || !mediaHandler) return;
  mediaQuery.removeEventListener('change', mediaHandler);
  mediaQuery = null;
  mediaHandler = null;
};

export const isAppTheme = (value: unknown): value is AppTheme =>
  value === 'dark' || value === 'light' || value === 'system';

// 同步主题并维护系统主题监听。
export const syncTheme = (value: AppTheme) => {
  applyThemeToDom(value);
  if (value === 'system') {
    startSystemListener();
  } else {
    stopSystemListener();
  }
  if (typeof window !== 'undefined') {
    try {
      window.localStorage.setItem(THEME_STORAGE_KEY, value);
    } catch {
      // 忽略本地存储失败，避免阻断主题切换流程。
    }
  }
};
