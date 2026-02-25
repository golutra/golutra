import { createI18n } from 'vue-i18n';
import enUS from './locales/en-US';
import zhCN from './locales/zh-CN';

export const LOCALE_STORAGE_KEY = 'nexus-locale';

export const messages = {
  'en-US': enUS,
  'zh-CN': zhCN
} as const;

export type AppLocale = keyof typeof messages;

const detectLocale = (): AppLocale => {
  if (typeof window === 'undefined') {
    return 'en-US';
  }

  const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY);
  if (stored && stored in messages) {
    return stored as AppLocale;
  }

  return 'en-US';
};

const initialLocale = detectLocale();

export const i18n = createI18n({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: 'en-US',
  messages
});

export const setDocumentLang = (locale: AppLocale) => {
  if (typeof document !== 'undefined') {
    document.documentElement.lang = locale;
  }
};

setDocumentLang(initialLocale);
