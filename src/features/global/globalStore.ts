import { computed, ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';

import { readAppData, writeAppData } from '@/shared/tauri/storage';
import { listSkillFolders } from '@/features/skills/skillsBridge';

import {
  type GlobalData,
  type ImportedSkillFolder,
  buildDefaultGlobalData,
  normalizeGlobalData,
  reconcileImportedSkillFolders,
  sameImportedSkillFolders
} from './globalData';

const GLOBAL_DATA_PATH = 'global-data.json';

const formatError = (error: unknown) => (error instanceof Error ? error.message : String(error));

const sameNumberArray = (left: number[], right: number[]) =>
  left.length === right.length && left.every((value, index) => value === right[index]);

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
      const normalized = normalizeGlobalData(stored ?? undefined);
      let actualSkillFolders = null;
      try {
        actualSkillFolders = await listSkillFolders();
      } catch (error) {
        console.error('Failed to list skill folders for reconciliation.', error);
      }
      const reconciledImportedSkillFolders = reconcileImportedSkillFolders(
        normalized.importedSkillFolders,
        actualSkillFolders
      );
      const reconciled: GlobalData = {
        ...normalized,
        importedSkillFolders: reconciledImportedSkillFolders
      };
      globalData.value = reconciled;
      const shouldPersist =
        !stored ||
        !Array.isArray(stored.installedSkills) ||
        !Array.isArray(stored.installedPlugins) ||
        !Array.isArray(stored.importedSkillFolders) ||
        !sameNumberArray(normalized.installedSkills, reconciled.installedSkills) ||
        !sameNumberArray(normalized.installedPlugins, reconciled.installedPlugins) ||
        !sameImportedSkillFolders(normalized.importedSkillFolders, reconciled.importedSkillFolders);
      if (shouldPersist) {
        await writeAppData(GLOBAL_DATA_PATH, reconciled);
      }
      loadedGlobal.value = true;
    } catch (error) {
      globalError.value = formatError(error);
      console.error('Failed to load global data.', error);
    } finally {
      loadingGlobal.value = false;
    }
  };

  const refreshGlobalData = async () => {
    loadedGlobal.value = false;
    await hydrate();
  };

  const persistGlobalData = async () => {
    try {
      await writeAppData(GLOBAL_DATA_PATH, normalizeGlobalData(globalData.value));
    } catch (error) {
      globalError.value = formatError(error);
      console.error('Failed to persist global data.', error);
    }
  };

  const installedSkillIds = computed(() => globalData.value.installedSkills);
  const installedPluginIds = computed(() => globalData.value.installedPlugins);
  const importedSkillFolders = computed(() => globalData.value.importedSkillFolders);

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

  const addImportedSkillFolder = async (payload: { name: string; path: string }) => {
    const path = payload.path.trim();
    if (!path) {
      return;
    }
    if (globalData.value.importedSkillFolders.some((folder) => folder.path === path)) {
      return;
    }
    const name = payload.name.trim() || path.split(/[\\/]/).filter(Boolean).pop() || path;
    const entry: ImportedSkillFolder = {
      id: path,
      name,
      path,
      addedAt: Date.now()
    };
    globalData.value = {
      ...globalData.value,
      importedSkillFolders: [...globalData.value.importedSkillFolders, entry]
    };
    await persistGlobalData();
  };

  const removeImportedSkillFolder = async (id: string) => {
    globalData.value = {
      ...globalData.value,
      importedSkillFolders: globalData.value.importedSkillFolders.filter((item) => item.id !== id)
    };
    await persistGlobalData();
  };

  return {
    installedSkillIds,
    installedPluginIds,
    importedSkillFolders,
    loadingGlobal,
    globalError,
    hydrate,
    refreshGlobalData,
    installSkill,
    removeSkill,
    installPlugin,
    removePlugin,
    addImportedSkillFolder,
    removeImportedSkillFolder
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useGlobalStore, import.meta.hot));
}
