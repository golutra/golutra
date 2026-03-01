// 国际化入口：统一消息表、默认语言与文档语言设置。
import { createI18n } from 'vue-i18n';
import enUS from './locales/en-US';
import zhCN from './locales/zh-CN';

// 受支持的语言包集合。
export const messages = {
  'en-US': enUS,
  'zh-CN': zhCN
} as const;

export type AppLocale = keyof typeof messages;
export const LOCALE_CHANGED_EVENT = 'app-locale-changed';
const LOCALE_STORAGE_KEY = 'golutra-locale';

// 仅在浏览器环境读取文档语言，未设置时回退默认值。
const detectLocale = (): AppLocale => {
  if (typeof document !== 'undefined') {
    const lang = document.documentElement.lang;
    if (lang && lang in messages) {
      return lang as AppLocale;
    }
  }
  return 'en-US';
};

const initialLocale = detectLocale();

/**
 * 应用级 i18n 实例。
 * 输入：默认语言与消息表。
 * 输出：供应用注入的 i18n 对象。
 */
export const i18n = createI18n({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: 'en-US',
  messages
});

/**
 * 同步更新文档语言标签。
 * 输入：当前语言。
 * 输出：无。
 */
export const setDocumentLang = (locale: AppLocale) => {
  if (typeof document !== 'undefined') {
    document.documentElement.lang = locale;
  }
};

export const applyLocale = (locale: AppLocale) => {
  if (i18n.global.locale.value !== locale) {
    i18n.global.locale.value = locale;
  }
  setDocumentLang(locale);
  if (typeof window !== 'undefined') {
    try {
      window.localStorage.setItem(LOCALE_STORAGE_KEY, locale);
    } catch {
      // 忽略本地存储失败，避免阻断语言切换流程。
    }
  }
};

setDocumentLang(initialLocale);
