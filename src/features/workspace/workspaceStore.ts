// 工作区状态管理：负责打开/关闭、最近列表与注册表异常处理。
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

// 与后端约定的错误前缀，用于解析工作区移动/复制提示信息。
const WORKSPACE_REGISTRY_MISMATCH_PREFIX = 'workspace_registry_mismatch:';

// 为默认频道生成稳定、可用的 slug，避免特殊字符导致下游不一致。
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

/**
 * 工作区状态存储。
 * 输入：外部触发的打开/关闭动作。
 * 输出：当前工作区、最近列表与对外操作函数。
 */
export const useWorkspaceStore = defineStore('workspace', () => {
  const currentWorkspace = ref<WorkspaceEntry | null>(null);
  const recentWorkspaces = ref<WorkspaceEntry[]>([]);
  const loadingRecent = ref(false);
  const workspaceReadOnly = ref(false);
  const workspaceWarning = ref<string | null>(null);
  const workspaceError = ref<string | null>(null);

  // 解析后端抛出的注册表不一致错误，提取可展示的路径信息。
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

  // 兼容 Windows 扩展路径与 UNC 表示，保持提示信息可读。
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

  /**
   * 弹出注册表不一致提示并返回用户决策。
   * 输入：旧路径与新路径信息。
   * 输出：'move' 或 'copy'。
   */
  const promptRegistryMismatch = async (info: WorkspaceRegistryMismatch): Promise<WorkspaceOpenResolution> => {
    const moved = await ask(buildRegistryMismatchMessage(info), {
      title: i18n.global.t('workspace.registryMismatchTitle'),
      kind: 'warning',
      okLabel: i18n.global.t('workspace.registryMismatchMoved'),
      cancelLabel: i18n.global.t('workspace.registryMismatchCopied')
    });
    return moved ? 'move' : 'copy';
  };

  // 记录工作区信息用于后续快速打开，失败时不阻塞主流程。
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

  /**
   * 拉取最近工作区列表。
   * 输入：无。
   * 输出：更新 recentWorkspaces；失败时仅记录日志。
   */
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

  /**
   * 按路径打开工作区，并处理注册表不一致提示。
   * 输入：工作区路径。
   * 输出：WorkspaceEntry 或 null。
   * 错误语义：失败时写入 workspaceError，返回 null。
   */
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

  /**
   * 打开目录选择对话框并尝试打开工作区。
   * 输入：无。
   * 输出：WorkspaceEntry 或 null。
   */
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

  /**
   * 关闭当前工作区并清理窗口级绑定。
   * 输入：无。
   * 输出：无。
   */
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
