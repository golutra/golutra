// 终端标签页右键菜单规则：根据标签页上下文提供关闭与固定操作。
import { storeToRefs } from 'pinia';
import { registerContextMenuRule, unregisterContextMenuRule } from '@/shared/context-menu/registry';
import type { ContextMenuContext, ContextMenuEntry } from '@/shared/context-menu/types';
import { untrackSession } from './terminalBridge';
import { useTerminalStore } from './terminalStore';

const RULE_ID = 'terminal-tab-context-menu';

const resolveTabId = (context: ContextMenuContext) => {
  const element = context.targetElement?.closest('[data-tab-id]') as HTMLElement | null;
  return element?.dataset.tabId ?? null;
};

const closeTabWithCleanup = async (terminalId: string) => {
  const terminalStore = useTerminalStore();
  untrackSession(terminalId);
  await terminalStore.closeTab(terminalId);
};

const closeTabsWithCleanup = async (terminalIds: string[]) => {
  for (const terminalId of terminalIds) {
    await closeTabWithCleanup(terminalId);
  }
};

export const registerTerminalTabContextMenu = () => {
  const terminalStore = useTerminalStore();
  const { layoutMode } = storeToRefs(terminalStore);

  registerContextMenuRule({
    id: RULE_ID,
    order: 40,
    mode: 'override',
    matches: (context) => context.scopeChain.includes('terminal-tab') && Boolean(resolveTabId(context)),
    items: (context): ContextMenuEntry[] => {
      const tabId = resolveTabId(context);
      if (!tabId) {
        return [];
      }
      const tabs = terminalStore.tabs;
      const tabIndex = tabs.findIndex((tab) => tab.id === tabId);
      if (tabIndex === -1) {
        return [];
      }
      // 关闭其它/右侧时保留置顶标签，避免误关常驻会话。
      const closableOthers = tabs.filter((tab) => tab.id !== tabId && !tab.pinned).map((tab) => tab.id);
      const closableRight = tabs
        .filter((tab, index) => index > tabIndex && !tab.pinned)
        .map((tab) => tab.id);
      const targetTab = tabs[tabIndex];
      const pinLabelKey = targetTab?.pinned ? 'terminal.tabMenu.unpin' : 'terminal.tabMenu.pin';

      return [
        {
          kind: 'item',
          id: 'terminal-tab-close',
          labelKey: 'terminal.tabMenu.close',
          icon: 'close',
          enabled: true,
          action: async () => {
            await closeTabWithCleanup(tabId);
          }
        },
        {
          kind: 'item',
          id: 'terminal-tab-close-others',
          labelKey: 'terminal.tabMenu.closeOthers',
          icon: 'close',
          enabled: closableOthers.length > 0,
          action: async () => {
            if (closableOthers.length === 0) {
              return;
            }
            terminalStore.setActive(tabId);
            await closeTabsWithCleanup(closableOthers);
          }
        },
        {
          kind: 'item',
          id: 'terminal-tab-close-right',
          labelKey: 'terminal.tabMenu.closeRight',
          icon: 'close',
          enabled: closableRight.length > 0,
          action: async () => {
            if (closableRight.length === 0) {
              return;
            }
            terminalStore.setActive(tabId);
            await closeTabsWithCleanup(closableRight);
          }
        },
        {
          kind: 'item',
          id: 'terminal-tab-pin',
          labelKey: pinLabelKey,
          icon: 'push_pin',
          enabled: true,
          action: () => {
            terminalStore.togglePin(tabId);
          }
        },
        { kind: 'separator' },
        {
          kind: 'item',
          id: 'terminal-tab-layout-single',
          labelKey: 'terminal.tabMenu.layoutSingle',
          icon: 'crop_square',
          enabled: layoutMode.value !== 'single',
          action: () => {
            terminalStore.setLayoutMode('single', { preferTerminalId: tabId });
          }
        },
        {
          kind: 'item',
          id: 'terminal-tab-layout-split-vertical',
          labelKey: 'terminal.tabMenu.layoutSplitVertical',
          icon: 'view_column',
          enabled: layoutMode.value !== 'split-vertical',
          action: () => {
            terminalStore.setLayoutMode('split-vertical', { preferTerminalId: tabId });
          }
        },
        {
          kind: 'item',
          id: 'terminal-tab-layout-split-horizontal',
          labelKey: 'terminal.tabMenu.layoutSplitHorizontal',
          icon: 'view_stream',
          enabled: layoutMode.value !== 'split-horizontal',
          action: () => {
            terminalStore.setLayoutMode('split-horizontal', { preferTerminalId: tabId });
          }
        },
        {
          kind: 'item',
          id: 'terminal-tab-layout-grid',
          labelKey: 'terminal.tabMenu.layoutGrid',
          icon: 'grid_view',
          enabled: layoutMode.value !== 'grid-2x2',
          action: () => {
            terminalStore.setLayoutMode('grid-2x2', { preferTerminalId: tabId });
          }
        }
      ];
    }
  });

  return () => unregisterContextMenuRule(RULE_ID);
};
