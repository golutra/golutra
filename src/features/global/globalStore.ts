// 全局数据存储：用于安装状态等跨工作区的应用级数据。
import { computed, ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';
import { readAppData, writeAppData } from '@/shared/tauri/storage';

type GlobalData = {
  version: number;
  installedSkills: number[];
  installedPlugins: number[];
  importedSkillFolders: ImportedSkillFolder[];
};

export type ImportedSkillFolder = {
  id: string;
  name: string;
  path: string;
  addedAt: number;
};

// 应用级数据文件名，与后端存储约定保持一致。
const GLOBAL_DATA_PATH = 'global-data.json';

const buildDefaultGlobalData = (): GlobalData => ({
  version: 1,
  installedSkills: [],
  installedPlugins: [],
  importedSkillFolders: []
});

const normalizeImportedSkillFolders = (candidate?: unknown): ImportedSkillFolder[] => {
  if (!Array.isArray(candidate)) {
    return [];
  }
  const folders: ImportedSkillFolder[] = [];
  const seenPaths = new Set<string>();
  for (const entry of candidate) {
    if (!entry || typeof entry !== 'object') {
      continue;
    }
    const record = entry as Partial<ImportedSkillFolder>;
    const path = typeof record.path === 'string' ? record.path.trim() : '';
    if (!path || seenPaths.has(path)) {
      continue;
    }
    const name =
      typeof record.name === 'string' && record.name.trim()
        ? record.name.trim()
        : path.split(/[\\/]/).filter(Boolean).pop() ?? path;
    const id = typeof record.id === 'string' && record.id.trim() ? record.id.trim() : path;
    const addedAt =
      typeof record.addedAt === 'number' && Number.isFinite(record.addedAt)
        ? record.addedAt
        : 0;
    folders.push({ id, name, path, addedAt });
    seenPaths.add(path);
  }
  return folders;
};

const normalizeGlobalData = (candidate?: Partial<GlobalData>): GlobalData => {
  const defaults = buildDefaultGlobalData();
  const version = Number(candidate?.version);
  const installedSkills = Array.isArray(candidate?.installedSkills) ? candidate?.installedSkills : defaults.installedSkills;
  const installedPlugins = Array.isArray(candidate?.installedPlugins) ? candidate?.installedPlugins : defaults.installedPlugins;
  const importedSkillFolders = normalizeImportedSkillFolders(candidate?.importedSkillFolders);
  return {
    version: Number.isFinite(version) && version > 0 ? version : defaults.version,
    installedSkills,
    installedPlugins,
    importedSkillFolders
  };
};

const formatError = (error: unknown) => (error instanceof Error ? error.message : String(error));

/**
 * 全局数据存储。
 * 输入：hydrate 初始化与安装/移除操作。
 * 输出：已安装列表与操作函数。
 */
export const useGlobalStore = defineStore('global', () => {
  const globalData = ref<GlobalData>(buildDefaultGlobalData());
  const loadingGlobal = ref(false);
  const loadedGlobal = ref(false);
  const globalError = ref<string | null>(null);

  /**
   * 读取并初始化全局数据，仅执行一次。
   * 输入：无。
   * 输出：更新 globalData；失败时记录错误信息。
   */
  const hydrate = async () => {
    if (loadingGlobal.value || loadedGlobal.value) return;
    loadingGlobal.value = true;
    globalError.value = null;
    try {
      const stored = await readAppData<GlobalData>(GLOBAL_DATA_PATH);
      const normalized = normalizeGlobalData(stored ?? undefined);
      globalData.value = normalized;
      const shouldPersist =
        !stored ||
        !Array.isArray(stored.installedSkills) ||
        !Array.isArray(stored.installedPlugins) ||
        !Array.isArray(stored.importedSkillFolders);
      if (shouldPersist) {
        await writeAppData(GLOBAL_DATA_PATH, normalized);
      }
      loadedGlobal.value = true;
    } catch (error) {
      globalError.value = formatError(error);
      console.error('Failed to load global data.', error);
    } finally {
      loadingGlobal.value = false;
    }
  };

  /**
   * 强制刷新全局数据，忽略已加载状态。
   * 输入：无。
   * 输出：更新 globalData；失败时记录错误信息。
   */
  const refreshGlobalData = async () => {
    loadedGlobal.value = false;
    await hydrate();
  };

  /**
   * 持久化全局数据。
   * 输入：无（使用当前 globalData）。
   * 输出：无；失败时记录错误。
   */
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
