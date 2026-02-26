import { computed, ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';
import { ask, open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { clearWorkspaceWindow, getCurrentWindowLabel } from '@/shared/tauri/windows';
import { writeAppData } from '@/shared/tauri/storage';
import { i18n } from '@/i18n';

export type WorkspaceEntry = {
  id: string;
  name: string;
  path: string;
  lastOpenedAt: number;
};

type WorkspaceOpenResult = {
  entry: WorkspaceEntry;
  readOnly?: boolean;
  warning?: string | null;
};

type WorkspaceRegistryMismatch = {
  projectId: string;
  lastKnownPath: string;
  currentPath: string;
};

type WorkspaceOpenResolution = 'move' | 'copy';

const WORKSPACE_REGISTRY_MISMATCH_PREFIX = 'workspace_registry_mismatch:';

const slugify = (value: string) =>
  value
    .trim()
    .toLowerCase()
    .replace(/[/\\]+/g, '-')
    .replace(/[^a-z0-9\- ]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');

const formatError = (error: unknown) => (error instanceof Error ? error.message : String(error));

export const useWorkspaceStore = defineStore('workspace', () => {
  const currentWorkspace = ref<WorkspaceEntry | null>(null);
  const recentWorkspaces = ref<WorkspaceEntry[]>([]);
  const loadingRecent = ref(false);
  const workspaceReadOnly = ref(false);
  const workspaceWarning = ref<string | null>(null);
  const workspaceError = ref<string | null>(null);

  const parseRegistryMismatch = (error: unknown): WorkspaceRegistryMismatch | null => {
    const message = typeof error === 'string' ? error : error instanceof Error ? error.message : null;
    if (!message) {
      return null;
    }
    const prefixIndex = message.indexOf(WORKSPACE_REGISTRY_MISMATCH_PREFIX);
    if (prefixIndex === -1) {
      return null;
    }
    const payload = message.slice(prefixIndex + WORKSPACE_REGISTRY_MISMATCH_PREFIX.length).trim();
    const jsonStart = payload.indexOf('{');
    const jsonEnd = payload.lastIndexOf('}');
    const jsonPayload =
      jsonStart !== -1 && jsonEnd !== -1 && jsonEnd > jsonStart
        ? payload.slice(jsonStart, jsonEnd + 1)
        : payload;
    try {
      const parsed = JSON.parse(jsonPayload) as Partial<WorkspaceRegistryMismatch>;
      if (!parsed.projectId || !parsed.lastKnownPath || !parsed.currentPath) {
        return null;
      }
      return {
        projectId: parsed.projectId,
        lastKnownPath: parsed.lastKnownPath,
        currentPath: parsed.currentPath
      };
    } catch {
      return null;
    }
  };

  const formatWorkspacePathForInfo = (path: string) => {
    if (!path) return path;
    if (!path.startsWith('\\\\?\\')) return path;
    const trimmed = path.slice(4);
    if (trimmed.toLowerCase().startsWith('unc\\')) {
      return `\\\\${trimmed.slice(4)}`;
    }
    return trimmed;
  };

  const buildRegistryMismatchMessage = (info: WorkspaceRegistryMismatch) =>
    i18n.global.t('workspace.registryMismatchMessage', {
      oldPath: formatWorkspacePathForInfo(info.lastKnownPath),
      newPath: formatWorkspacePathForInfo(info.currentPath)
    });

  const promptRegistryMismatch = async (info: WorkspaceRegistryMismatch): Promise<WorkspaceOpenResolution> => {
    const moved = await ask(buildRegistryMismatchMessage(info), {
      title: i18n.global.t('workspace.registryMismatchTitle'),
      kind: 'warning',
      okLabel: i18n.global.t('workspace.registryMismatchMoved'),
      cancelLabel: i18n.global.t('workspace.registryMismatchCopied')
    });
    return moved ? 'move' : 'copy';
  };

  const recordWorkspaceInfo = async (workspace: WorkspaceEntry) => {
    try {
      await writeAppData(`${workspace.id}/info.json`, {
        id: workspace.id,
        name: workspace.name,
        path: workspace.path,
        displayPath: formatWorkspacePathForInfo(workspace.path),
        lastAccessedAt: Date.now()
      });
    } catch (error) {
      console.error('Failed to persist workspace info metadata.', error);
    }
  };

  const loadRecent = async () => {
    if (loadingRecent.value) {
      return;
    }
    loadingRecent.value = true;
    try {
      const entries = await invoke<WorkspaceEntry[]>('workspace_recent_list');
      recentWorkspaces.value = Array.isArray(entries) ? entries : [];
    } catch (error) {
      console.error('Failed to load recent workspaces.', error);
    } finally {
      loadingRecent.value = false;
    }
  };

  const openWorkspaceByPath = async (path: string) => {
    workspaceError.value = null;
    const windowLabel = getCurrentWindowLabel();
    const resolveOpen = async (resolution?: WorkspaceOpenResolution) => {
      const result = await invoke<WorkspaceOpenResult | WorkspaceEntry>('workspace_open', {
        path,
        windowLabel: windowLabel ?? undefined,
        resolution
      });
      const workspace = 'entry' in result ? result.entry : result;
      const readOnly = 'entry' in result ? Boolean(result.readOnly) : false;
      const warning = 'entry' in result ? result.warning ?? null : null;
      currentWorkspace.value = workspace;
      workspaceReadOnly.value = readOnly;
      workspaceWarning.value = warning;
      await recordWorkspaceInfo(workspace);
      await loadRecent();
      return workspace;
    };
    try {
      return await resolveOpen();
    } catch (error) {
      const mismatch = parseRegistryMismatch(error);
      if (mismatch) {
        try {
          const resolution = await promptRegistryMismatch(mismatch);
          return await resolveOpen(resolution);
        } catch (retryError) {
          console.error('Failed to resolve workspace registry mismatch.', retryError);
          workspaceError.value = formatError(retryError);
          return null;
        }
      }
      console.error('Failed to open workspace.', error);
      workspaceError.value = formatError(error);
      return null;
    }
  };

  const openWorkspaceDialog = async () => {
    try {
      workspaceError.value = null;
      const selection = await open({ directory: true, multiple: false });
      if (!selection || Array.isArray(selection)) {
        return null;
      }
      return openWorkspaceByPath(selection);
    } catch (error) {
      console.error('Failed to open workspace dialog.', error);
      return null;
    }
  };

  const closeWorkspace = () => {
    currentWorkspace.value = null;
    workspaceReadOnly.value = false;
    workspaceWarning.value = null;
    void clearWorkspaceWindow();
  };

  const defaultChannelId = computed(() => {
    const name = currentWorkspace.value?.name ?? 'workspace';
    const slug = slugify(name);
    return slug || 'workspace';
  });

  const defaultChannelName = computed(() => currentWorkspace.value?.name ?? 'workspace');

  const recentPrimary = computed(() => recentWorkspaces.value.slice(0, 3));
  const recentMore = computed(() => recentWorkspaces.value.slice(3));

  return {
    currentWorkspace,
    workspaceReadOnly,
    workspaceWarning,
    workspaceError,
    recentWorkspaces,
    recentPrimary,
    recentMore,
    defaultChannelId,
    defaultChannelName,
    loadRecent,
    openWorkspaceDialog,
    openWorkspaceByPath,
    closeWorkspace,
    clearWorkspaceError: () => {
      workspaceError.value = null;
    },
    clearWorkspaceWarning: () => {
      workspaceWarning.value = null;
    }
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useWorkspaceStore, import.meta.hot));
}
