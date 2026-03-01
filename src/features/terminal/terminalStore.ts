// 终端标签页状态：管理 session 与 tab 的对应关系及活动提示。
import { ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';
import { i18n } from '@/i18n';
import { closeSession, createSession } from './terminalBridge';
import type { TerminalType } from '@/shared/types/terminal';

export type TerminalLayoutMode = 'single' | 'split-vertical' | 'split-horizontal' | 'grid-2x2';
export type TerminalPaneId =
  | 'primary'
  | 'left'
  | 'right'
  | 'top'
  | 'bottom'
  | 'top-left'
  | 'top-right'
  | 'bottom-left'
  | 'bottom-right';

export type TerminalTab = {
  id: string;
  title: string;
  hasActivity: boolean;
  isBlinking: boolean;
  pinned: boolean;
  memberId?: string;
  terminalType?: TerminalType;
  keepAlive?: boolean;
};

type PaneAssignments = Partial<Record<TerminalPaneId, string | null>>;

const LAYOUT_PANES: Record<TerminalLayoutMode, TerminalPaneId[]> = {
  single: ['primary'],
  'split-vertical': ['left', 'right'],
  'split-horizontal': ['top', 'bottom'],
  'grid-2x2': ['top-left', 'top-right', 'bottom-left', 'bottom-right']
};

/**
 * 终端标签页存储。
 * 输入：创建/打开/关闭/排序操作。
 * 输出：tabs 列表与活动 tab id。
 */
export const useTerminalStore = defineStore('terminal', () => {
  const tabs = ref<TerminalTab[]>([]);
  const activeId = ref<string | null>(null);
  const layoutMode = ref<TerminalLayoutMode>('single');
  const paneAssignments = ref<PaneAssignments>({ primary: null });
  const focusedPaneId = ref<TerminalPaneId>('primary');
  const paneIds = ref<TerminalPaneId[]>(LAYOUT_PANES.single);
  // 本地自增序号用于默认标题，避免重复命名。
  let tabCounter = 1;

  const resolvePaneIds = (mode: TerminalLayoutMode) => LAYOUT_PANES[mode] ?? LAYOUT_PANES.single;

  const updatePaneIds = (mode: TerminalLayoutMode) => {
    paneIds.value = resolvePaneIds(mode);
  };

  const isTabAlive = (terminalId: string) => tabs.value.some((tab) => tab.id === terminalId);

  const resolveFallbackPaneId = (paneId?: TerminalPaneId | null) => {
    if (paneId && paneIds.value.includes(paneId)) {
      return paneId;
    }
    return paneIds.value[0] ?? 'primary';
  };

  const findPaneByTerminalId = (terminalId: string) => {
    for (const paneId of paneIds.value) {
      if (paneAssignments.value[paneId] === terminalId) {
        return paneId;
      }
    }
    return null;
  };

  const sortTabsByPin = (list: TerminalTab[]) => {
    const pinned: TerminalTab[] = [];
    const unpinned: TerminalTab[] = [];
    for (const tab of list) {
      if (tab.pinned) {
        pinned.push(tab);
      } else {
        unpinned.push(tab);
      }
    }
    return pinned.concat(unpinned);
  };

  const normalizePinnedTabs = () => {
    const next = sortTabsByPin(tabs.value);
    const current = tabs.value;
    if (next.length !== current.length) {
      tabs.value = next;
      return;
    }
    for (let index = 0; index < next.length; index += 1) {
      if (next[index] !== current[index]) {
        tabs.value = next;
        return;
      }
    }
  };

  const syncPaneAssignments = () => {
    const nextAssignments: PaneAssignments = {};
    for (const paneId of paneIds.value) {
      const existing = paneAssignments.value[paneId];
      if (existing && isTabAlive(existing)) {
        nextAssignments[paneId] = existing;
      } else {
        nextAssignments[paneId] = null;
      }
    }
    paneAssignments.value = nextAssignments;
    if (!paneIds.value.includes(focusedPaneId.value)) {
      focusedPaneId.value = resolveFallbackPaneId(null);
    }
    const focusedTab = paneAssignments.value[focusedPaneId.value];
    if (focusedTab) {
      activeId.value = focusedTab;
      return;
    }
    const firstAssigned = paneIds.value.map((paneId) => paneAssignments.value[paneId]).find((id) => id);
    activeId.value = firstAssigned ?? null;
  };

  const buildLayoutCandidates = (preferredId?: string) => {
    const candidates: string[] = [];
    const pushCandidate = (id?: string | null) => {
      if (!id || !isTabAlive(id) || candidates.includes(id)) {
        return;
      }
      candidates.push(id);
    };
    pushCandidate(preferredId);
    for (const terminalId of Object.values(paneAssignments.value)) {
      pushCandidate(terminalId);
    }
    for (const tab of tabs.value) {
      pushCandidate(tab.id);
    }
    return candidates;
  };

  const setLayoutMode = (mode: TerminalLayoutMode, options?: { preferTerminalId?: string }) => {
    if (layoutMode.value === mode) {
      return;
    }
    updatePaneIds(mode);
    layoutMode.value = mode;
    const candidates = buildLayoutCandidates(options?.preferTerminalId);
    const nextAssignments: PaneAssignments = {};
    for (const paneId of paneIds.value) {
      nextAssignments[paneId] = candidates.shift() ?? null;
    }
    paneAssignments.value = nextAssignments;
    const preferred = options?.preferTerminalId;
    const preferredPane = preferred
      ? paneIds.value.find((paneId) => paneAssignments.value[paneId] === preferred)
      : null;
    focusedPaneId.value = preferredPane ?? resolveFallbackPaneId(focusedPaneId.value);
    syncPaneAssignments();
  };

  const unassignTab = (terminalId: string) => {
    let changed = false;
    const nextAssignments: PaneAssignments = { ...paneAssignments.value };
    for (const paneId of paneIds.value) {
      if (nextAssignments[paneId] === terminalId) {
        nextAssignments[paneId] = null;
        changed = true;
      }
    }
    if (!changed) {
      return;
    }
    paneAssignments.value = nextAssignments;
    syncPaneAssignments();
  };

  const assignTabToPane = (
    terminalId: string,
    paneId: TerminalPaneId,
    options?: { focus?: boolean; activate?: boolean }
  ) => {
    if (!isTabAlive(terminalId)) {
      return;
    }
    const resolvedPane = resolveFallbackPaneId(paneId);
    const nextAssignments: PaneAssignments = { ...paneAssignments.value };
    for (const existingPane of paneIds.value) {
      if (nextAssignments[existingPane] === terminalId) {
        nextAssignments[existingPane] = null;
      }
    }
    nextAssignments[resolvedPane] = terminalId;
    paneAssignments.value = nextAssignments;
    if (options?.activate) {
      activeId.value = terminalId;
    }
    if (options?.focus) {
      focusedPaneId.value = resolvedPane;
      activeId.value = terminalId;
    }
  };

  const assignTabOnOpen = (terminalId: string, shouldActivate: boolean) => {
    const emptyPane = paneIds.value.find((paneId) => !paneAssignments.value[paneId]);
    if (shouldActivate) {
      const targetPane = emptyPane ?? resolveFallbackPaneId(focusedPaneId.value);
      assignTabToPane(terminalId, targetPane, { focus: true, activate: true });
      return;
    }
    if (emptyPane) {
      assignTabToPane(terminalId, emptyPane, { activate: false });
    }
  };

  /**
   * 创建新的终端会话并加入标签列表。
   * 输入：可选 cwd/workspaceId。
   * 输出：新会话 id。
   */
  const createTab = async (options?: { cwd?: string; workspaceId?: string }) => {
    const terminalId = await createSession({
      cwd: options?.cwd,
      workspaceId: options?.workspaceId
    });
    const title = `${i18n.global.t('terminal.title')} ${tabCounter}`;
    tabCounter += 1;
    tabs.value.push({ id: terminalId, title, hasActivity: false, isBlinking: false, pinned: false, keepAlive: false });
    normalizePinnedTabs();
    assignTabOnOpen(terminalId, true);
    return terminalId;
  };

  const openTab = (
    terminalId: string,
    options: { title: string; memberId?: string; terminalType?: TerminalType; keepAlive?: boolean; activate?: boolean }
  ) => {
    const shouldActivate = options.activate !== false;
    const existing = tabs.value.find((item) => item.id === terminalId);
    if (existing) {
      existing.title = options.title;
      existing.memberId = options.memberId;
      existing.terminalType = options.terminalType;
      existing.keepAlive = options.keepAlive ?? existing.keepAlive;
      existing.isBlinking = false;
      if (shouldActivate) {
        activeId.value = terminalId;
        existing.hasActivity = false;
      }
      if (!findPaneByTerminalId(terminalId)) {
        assignTabOnOpen(terminalId, shouldActivate);
      }
      return;
    }
    tabs.value.push({
      id: terminalId,
      title: options.title,
      hasActivity: false,
      isBlinking: false,
      pinned: false,
      memberId: options.memberId,
      terminalType: options.terminalType,
      keepAlive: options.keepAlive ?? false
    });
    normalizePinnedTabs();
    assignTabOnOpen(terminalId, shouldActivate);
  };

  const setActive = (terminalId: string) => {
    activeId.value = terminalId;
    const tab = tabs.value.find((item) => item.id === terminalId);
    if (tab) {
      tab.hasActivity = false;
      tab.isBlinking = false;
    }
    const paneId = findPaneByTerminalId(terminalId);
    if (paneId) {
      focusedPaneId.value = paneId;
    }
  };

  /**
   * 关闭标签页，必要时终止后端会话。
   * 输入：terminalId。
   * 输出：无。
   */
  const closeTab = async (terminalId: string) => {
    const index = tabs.value.findIndex((item) => item.id === terminalId);
    if (index === -1) {
      return;
    }
    const tab = tabs.value[index];
    if (!tab.keepAlive) {
      await closeSession(terminalId);
    }
    tabs.value.splice(index, 1);
    unassignTab(terminalId);
  };

  const moveTab = (fromId: string, toId: string) => {
    const fromIndex = tabs.value.findIndex((item) => item.id === fromId);
    const toIndex = tabs.value.findIndex((item) => item.id === toId);
    if (fromIndex === -1 || toIndex === -1 || fromIndex === toIndex) {
      return;
    }
    const [item] = tabs.value.splice(fromIndex, 1);
    tabs.value.splice(toIndex, 0, item);
    normalizePinnedTabs();
  };

  const moveTabToIndex = (fromId: string, insertIndex: number) => {
    const fromIndex = tabs.value.findIndex((item) => item.id === fromId);
    if (fromIndex === -1) {
      return;
    }
    const [item] = tabs.value.splice(fromIndex, 1);
    const safeIndex = Math.max(0, Math.min(tabs.value.length, insertIndex));
    tabs.value.splice(safeIndex, 0, item);
    normalizePinnedTabs();
  };

  const swapTabOrder = (fromId: string, toId: string) => {
    if (fromId === toId) {
      return;
    }
    const fromIndex = tabs.value.findIndex((item) => item.id === fromId);
    const toIndex = tabs.value.findIndex((item) => item.id === toId);
    if (fromIndex === -1 || toIndex === -1 || fromIndex === toIndex) {
      return;
    }
    const fromTab = tabs.value[fromIndex];
    const toTab = tabs.value[toIndex];
    if (fromTab.pinned !== toTab.pinned) {
      return;
    }
    const next = tabs.value.slice();
    next[fromIndex] = toTab;
    next[toIndex] = fromTab;
    tabs.value = next;
    normalizePinnedTabs();
  };

  const setTabOrder = (orderedIds: string[]) => {
    if (orderedIds.length === 0) {
      return;
    }
    const remaining = new Map(tabs.value.map((tab) => [tab.id, tab]));
    const ordered: TerminalTab[] = [];
    for (const id of orderedIds) {
      const tab = remaining.get(id);
      if (tab) {
        ordered.push(tab);
        remaining.delete(id);
      }
    }
    for (const tab of tabs.value) {
      if (remaining.has(tab.id)) {
        ordered.push(tab);
      }
    }
    tabs.value = sortTabsByPin(ordered);
  };

  const markActivity = (terminalId: string) => {
    const tab = tabs.value.find((item) => item.id === terminalId);
    if (tab && activeId.value !== terminalId) {
      tab.hasActivity = true;
      tab.isBlinking = false;
    }
  };

  const clearActivity = (terminalId: string) => {
    const tab = tabs.value.find((item) => item.id === terminalId);
    if (tab) {
      tab.hasActivity = false;
      tab.isBlinking = false;
    }
  };

  const togglePin = (terminalId: string) => {
    const tab = tabs.value.find((item) => item.id === terminalId);
    if (tab) {
      tab.pinned = !tab.pinned;
      normalizePinnedTabs();
    }
  };

  return {
    tabs,
    activeId,
    layoutMode,
    paneAssignments,
    focusedPaneId,
    paneIds,
    createTab,
    openTab,
    setActive,
    closeTab,
    moveTab,
    moveTabToIndex,
    swapTabOrder,
    setTabOrder,
    markActivity,
    clearActivity,
    togglePin,
    setLayoutMode,
    assignTabToPane,
    unassignTab,
    syncPaneAssignments
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useTerminalStore, import.meta.hot));
}
