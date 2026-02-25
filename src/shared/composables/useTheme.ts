import { computed, ref } from 'vue';

export type AppTheme = 'dark' | 'light' | 'system';

export const THEME_STORAGE_KEY = 'nexus-theme';

export type ThemeOption = {
  id: AppTheme;
  labelKey: string;
  descriptionKey: string;
};

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

const getStoredTheme = (): AppTheme => {
  if (typeof window === 'undefined') return 'dark';
  const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
  if (stored === 'dark' || stored === 'light' || stored === 'system') {
    return stored;
  }
  return 'dark';
};

const theme = ref<AppTheme>(getStoredTheme());

const getSystemTheme = (): 'dark' | 'light' => {
  if (typeof window === 'undefined' || !window.matchMedia) {
    return 'dark';
  }
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
};

const applyTheme = (value: AppTheme) => {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  root.dataset.theme = value;
  const resolved = value === 'system' ? getSystemTheme() : value;
  root.dataset.resolvedTheme = resolved;
  root.classList.toggle('dark', resolved === 'dark');
};

let mediaQuery: MediaQueryList | null = null;
let mediaHandler: ((event: MediaQueryListEvent) => void) | null = null;

const startSystemListener = () => {
  if (typeof window === 'undefined' || !window.matchMedia || mediaQuery) return;
  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  mediaHandler = () => applyTheme('system');
  mediaQuery.addEventListener('change', mediaHandler);
};

const stopSystemListener = () => {
  if (!mediaQuery || !mediaHandler) return;
  mediaQuery.removeEventListener('change', mediaHandler);
  mediaQuery = null;
  mediaHandler = null;
};

export const initializeTheme = () => {
  applyTheme(theme.value);
  if (theme.value === 'system') {
    startSystemListener();
  }
};

export const useTheme = () => {
  const setTheme = (next: AppTheme) => {
    theme.value = next;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(THEME_STORAGE_KEY, next);
    }
    applyTheme(next);
    if (next === 'system') {
      startSystemListener();
    } else {
      stopSystemListener();
    }
  };

  return {
    theme: computed(() => theme.value),
    setTheme,
    themeOptions
  };
};
