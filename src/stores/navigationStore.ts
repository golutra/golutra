// 导航状态：负责记录当前主视图与技能商店子视图，避免跨组件重复传参。
import { ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';

export type AppTabId = 'workspaces' | 'chat' | 'friends' | 'store' | 'plugins' | 'settings';
export type SkillStoreTab = 'store' | 'installed';

/**
 * 全局导航状态存储。
 * 输入：切换目标标签。
 * 输出：当前标签与写入方法。
 * 边界：仅允许预定义标签值。
 */
export const useNavigationStore = defineStore('navigation', () => {
  const activeTab = ref<AppTabId>('workspaces');
  const skillStoreTab = ref<SkillStoreTab>('store');

  const setActiveTab = (tab: AppTabId) => {
    activeTab.value = tab;
  };

  const setSkillStoreTab = (tab: SkillStoreTab) => {
    skillStoreTab.value = tab;
  };

  return {
    activeTab,
    skillStoreTab,
    setActiveTab,
    setSkillStoreTab
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useNavigationStore, import.meta.hot));
}
