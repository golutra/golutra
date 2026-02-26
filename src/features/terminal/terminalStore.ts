import { ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';
import { i18n } from '@/i18n';
import { closeSession, createSession } from './terminalBridge';
import type { TerminalType } from '@/shared/types/terminal';

export type TerminalTab = {
  id: string;
  title: string;
  hasActivity: boolean;
  isBlinking: boolean;
  memberId?: string;
  terminalType?: TerminalType;
  keepAlive?: boolean;
};

export const useTerminalStore = defineStore('terminal', () => {
  const tabs = ref<TerminalTab[]>([]);
  const activeId = ref<string | null>(null);
  let tabCounter = 1;

  const createTab = async (options?: { cwd?: string; workspaceId?: string }) => {
    const sessionId = await createSession({ cwd: options?.cwd, workspaceId: options?.workspaceId });
    const title = `${i18n.global.t('terminal.title')} ${tabCounter}`;
    tabCounter += 1;
    tabs.value.push({ id: sessionId, title, hasActivity: false, isBlinking: false, keepAlive: false });
    activeId.value = sessionId;
    return sessionId;
  };

  const openTab = (
    sessionId: string,
    options: { title: string; memberId?: string; terminalType?: TerminalType; keepAlive?: boolean }
  ) => {
    const existing = tabs.value.find((item) => item.id === sessionId);
    if (existing) {
      existing.title = options.title;
      existing.memberId = options.memberId;
      existing.terminalType = options.terminalType;
      existing.keepAlive = options.keepAlive ?? existing.keepAlive;
      existing.isBlinking = false;
      activeId.value = sessionId;
      existing.hasActivity = false;
      return;
    }
    tabs.value.push({
      id: sessionId,
      title: options.title,
      hasActivity: false,
      isBlinking: false,
      memberId: options.memberId,
      terminalType: options.terminalType,
      keepAlive: options.keepAlive ?? false
    });
    activeId.value = sessionId;
  };

  const setActive = (sessionId: string) => {
    activeId.value = sessionId;
    const tab = tabs.value.find((item) => item.id === sessionId);
    if (tab) {
      tab.hasActivity = false;
      tab.isBlinking = false;
    }
  };

  const closeTab = async (sessionId: string) => {
    const index = tabs.value.findIndex((item) => item.id === sessionId);
    if (index === -1) {
      return;
    }
    const tab = tabs.value[index];
    if (!tab.keepAlive) {
      await closeSession(sessionId);
    }
    const wasActive = activeId.value === sessionId;
    tabs.value.splice(index, 1);
    if (!wasActive) {
      return;
    }
    const fallback = tabs.value[index - 1] ?? tabs.value[index] ?? null;
    activeId.value = fallback?.id ?? null;
    if (fallback) {
      fallback.hasActivity = false;
    }
  };

  const moveTab = (fromId: string, toId: string) => {
    const fromIndex = tabs.value.findIndex((item) => item.id === fromId);
    const toIndex = tabs.value.findIndex((item) => item.id === toId);
    if (fromIndex === -1 || toIndex === -1 || fromIndex === toIndex) {
      return;
    }
    const [item] = tabs.value.splice(fromIndex, 1);
    tabs.value.splice(toIndex, 0, item);
  };

  const moveTabToIndex = (fromId: string, insertIndex: number) => {
    const fromIndex = tabs.value.findIndex((item) => item.id === fromId);
    if (fromIndex === -1) {
      return;
    }
    const [item] = tabs.value.splice(fromIndex, 1);
    const safeIndex = Math.max(0, Math.min(tabs.value.length, insertIndex));
    tabs.value.splice(safeIndex, 0, item);
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
    tabs.value = ordered;
  };

  const markActivity = (sessionId: string) => {
    const tab = tabs.value.find((item) => item.id === sessionId);
    if (tab && activeId.value !== sessionId) {
      tab.hasActivity = true;
      tab.isBlinking = false;
    }
  };

  const clearActivity = (sessionId: string) => {
    const tab = tabs.value.find((item) => item.id === sessionId);
    if (tab) {
      tab.hasActivity = false;
      tab.isBlinking = false;
    }
  };

  return {
    tabs,
    activeId,
    createTab,
    openTab,
    setActive,
    closeTab,
    moveTab,
    moveTabToIndex,
    setTabOrder,
    markActivity,
    clearActivity
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useTerminalStore, import.meta.hot));
}
