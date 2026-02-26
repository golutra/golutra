import { ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';

export type AppTabId = 'workspaces' | 'chat' | 'friends' | 'store' | 'plugins' | 'settings';
export type SkillStoreTab = 'store' | 'installed';

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
