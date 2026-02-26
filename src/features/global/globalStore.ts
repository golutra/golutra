import { computed, ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';
import { readAppData, writeAppData } from '@/shared/tauri/storage';

type GlobalData = {
  version: number;
  installedSkills: number[];
  installedPlugins: number[];
};

const GLOBAL_DATA_PATH = 'global-data.json';

const buildDefaultGlobalData = (): GlobalData => ({
  version: 1,
  installedSkills: [],
  installedPlugins: []
});

const formatError = (error: unknown) => (error instanceof Error ? error.message : String(error));

export const useGlobalStore = defineStore('global', () => {
  const globalData = ref<GlobalData>(buildDefaultGlobalData());
  const loadingGlobal = ref(false);
  const loadedGlobal = ref(false);
  const globalError = ref<string | null>(null);

  const hydrate = async () => {
    if (loadingGlobal.value || loadedGlobal.value) return;
    loadingGlobal.value = true;
    globalError.value = null;
    try {
      const stored = await readAppData<GlobalData>(GLOBAL_DATA_PATH);
      if (stored && Array.isArray(stored.installedSkills) && Array.isArray(stored.installedPlugins)) {
        globalData.value = stored;
      } else {
        const defaults = buildDefaultGlobalData();
        globalData.value = defaults;
        await writeAppData(GLOBAL_DATA_PATH, defaults);
      }
      loadedGlobal.value = true;
    } catch (error) {
      globalError.value = formatError(error);
      console.error('Failed to load global data.', error);
    } finally {
      loadingGlobal.value = false;
    }
  };

  const persistGlobalData = async () => {
    try {
      await writeAppData(GLOBAL_DATA_PATH, globalData.value);
    } catch (error) {
      globalError.value = formatError(error);
      console.error('Failed to persist global data.', error);
    }
  };

  const installedSkillIds = computed(() => globalData.value.installedSkills);
  const installedPluginIds = computed(() => globalData.value.installedPlugins);

  const installSkill = async (id: number) => {
    if (globalData.value.installedSkills.includes(id)) return;
    globalData.value = {
      ...globalData.value,
      installedSkills: [...globalData.value.installedSkills, id]
    };
    await persistGlobalData();
  };

  const removeSkill = async (id: number) => {
    globalData.value = {
      ...globalData.value,
      installedSkills: globalData.value.installedSkills.filter((item) => item !== id)
    };
    await persistGlobalData();
  };

  const installPlugin = async (id: number) => {
    if (globalData.value.installedPlugins.includes(id)) return;
    globalData.value = {
      ...globalData.value,
      installedPlugins: [...globalData.value.installedPlugins, id]
    };
    await persistGlobalData();
  };

  const removePlugin = async (id: number) => {
    globalData.value = {
      ...globalData.value,
      installedPlugins: globalData.value.installedPlugins.filter((item) => item !== id)
    };
    await persistGlobalData();
  };

  return {
    installedSkillIds,
    installedPluginIds,
    loadingGlobal,
    globalError,
    hydrate,
    installSkill,
    removeSkill,
    installPlugin,
    removePlugin
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useGlobalStore, import.meta.hot));
}
