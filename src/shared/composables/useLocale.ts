import { computed } from 'vue';
import { i18n, LOCALE_STORAGE_KEY, setDocumentLang, type AppLocale } from '@/i18n';

export type LocaleOption = {
  id: AppLocale;
  labelKey: string;
  flag: string;
};

export const localeOptions: LocaleOption[] = [
  { id: 'en-US', labelKey: 'language.enUS', flag: '🇺🇸' },
  { id: 'zh-CN', labelKey: 'language.zhCN', flag: '🇨🇳' }
];

export const useLocale = () => {
  const locale = computed(() => i18n.global.locale.value as AppLocale);

  const setLocale = (next: AppLocale) => {
    i18n.global.locale.value = next;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(LOCALE_STORAGE_KEY, next);
    }
    setDocumentLang(next);
  };

  return {
    locale,
    setLocale,
    localeOptions
  };
};
