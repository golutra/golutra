import { computed, ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';

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
let storageHandler: ((event: StorageEvent) => void) | null = null;

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

const syncTheme = (value: AppTheme) => {
  applyTheme(value);
  if (value === 'system') {
    startSystemListener();
  } else {
    stopSystemListener();
  }
};

const parseStoredTheme = (value: string | null): AppTheme | null => {
  if (value === 'dark' || value === 'light' || value === 'system') {
    return value;
  }
  return null;
};

const startStorageListener = (onTheme: (value: AppTheme) => void) => {
  if (typeof window === 'undefined' || storageHandler) return;
  storageHandler = (event) => {
    if (event.storageArea !== window.localStorage) return;
    if (event.key && event.key !== THEME_STORAGE_KEY) return;
    const next = parseStoredTheme(event.newValue) ?? getStoredTheme();
    onTheme(next);
  };
  window.addEventListener('storage', storageHandler);
};

export const applyInitialTheme = () => {
  syncTheme(getStoredTheme());
};

export const useThemeStore = defineStore('theme', () => {
  const themeRef = ref<AppTheme>(getStoredTheme());

  const setTheme = (next: AppTheme) => {
    themeRef.value = next;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(THEME_STORAGE_KEY, next);
    }
    syncTheme(next);
  };

  const initializeTheme = () => {
    syncTheme(themeRef.value);
    startStorageListener((next) => {
      themeRef.value = next;
      syncTheme(next);
    });
  };

  return {
    theme: computed(() => themeRef.value),
    setTheme,
    initializeTheme,
    themeOptions
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useThemeStore, import.meta.hot));
}
