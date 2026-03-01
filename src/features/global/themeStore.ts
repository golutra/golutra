// 主题状态管理：与全局设置同步，避免重复持久化。
import { acceptHMRUpdate, defineStore, storeToRefs } from 'pinia';
import { useSettingsStore } from '@/features/global/settingsStore';
import { syncTheme, themeOptions, type AppTheme } from '@/features/global/theme';

export { themeOptions, type AppTheme };

/**
 * 主题状态存储。
 * 输入：setTheme 修改主题。
 * 输出：当前主题与可用选项。
 */
export const useThemeStore = defineStore('theme', () => {
  const settingsStore = useSettingsStore();
  const { theme: themeRef } = storeToRefs(settingsStore);

  const setTheme = (next: AppTheme) => {
    settingsStore.setTheme(next);
  };

  const initializeTheme = () => {
    syncTheme(themeRef.value);
  };

  return {
    theme: themeRef,
    setTheme,
    initializeTheme,
    themeOptions
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useThemeStore, import.meta.hot));
}
