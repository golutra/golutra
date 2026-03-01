// 应用级快捷键：处理全局导航与搜索聚焦等通用动作。
import { nextTick, type Ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useSettingsStore } from '@/features/global/settingsStore';
import { registerKeybindRule, unregisterKeybindRule } from '@/shared/keyboard/registry';
import { resolveKeybindKeys, type KeybindActionId } from '@/shared/keyboard/profiles';
import type { KeybindContext, KeybindItem } from '@/shared/keyboard/types';
import type { AppTabId } from '@/stores/navigationStore';

type AppKeybindOptions = {
  activeTab: Ref<AppTabId>;
  setActiveTab: (tab: AppTabId) => void;
};

type ScopeRoot = Element | Document;

const APP_KEYBIND_RULE_ID = 'app-keybinds';
const GUARD_KEYBIND_RULE_ID = 'app-keybinds-guard';

const isVisibleElement = (element: HTMLElement) => {
  const style = window.getComputedStyle(element);
  if (style.display === 'none' || style.visibility === 'hidden' || Number(style.opacity) === 0) {
    return false;
  }
  const rect = element.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0;
};

const findVisibleTarget = (selector: string, root?: ScopeRoot | null) => {
  if (typeof document === 'undefined') {
    return null;
  }
  const scope = root ?? document;
  const candidates = Array.from(scope.querySelectorAll<HTMLElement>(selector));
  return candidates.find((element) => isVisibleElement(element)) ?? null;
};

const waitForNextFrame = () =>
  new Promise<void>((resolve) => {
    if (typeof window === 'undefined') {
      resolve();
      return;
    }
    window.requestAnimationFrame(() => resolve());
  });

// 以就近 scope 限定查找范围，避免误命中其它视图。
const resolveScopeRoot = (context: KeybindContext) => {
  const anchor =
    context.targetElement ??
    (context.activeElement instanceof HTMLElement ? context.activeElement : null);
  return anchor?.closest('[data-key-scope]') as HTMLElement | null;
};

const focusByRole = async (role: string, scopeRoot?: HTMLElement | null) => {
  const selector = `[data-key-role="${role}"]`;
  const scopedTarget = scopeRoot ? findVisibleTarget(selector, scopeRoot) : null;
  const target = scopedTarget ?? findVisibleTarget(selector);
  if (!target) {
    return false;
  }
  if (
    (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) &&
    target.disabled
  ) {
    return false;
  }
  target.focus();
  if (role === 'search' && (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement)) {
    target.select?.();
  }
  return true;
};

export const registerAppKeybinds = (options: AppKeybindOptions) => {
  const settingsStore = useSettingsStore();
  const { settings } = storeToRefs(settingsStore);

  const buildItems = (): KeybindItem[] => {
    if (!settings.value.keybinds.enabled) {
      return [];
    }
    const profile = settings.value.keybinds.profile;
    const items: KeybindItem[] = [];
    const addItem = (actionId: KeybindActionId, item: Omit<KeybindItem, 'id' | 'keys'>) => {
      const keys = resolveKeybindKeys(profile, actionId);
      if (keys.length === 0) {
        return;
      }
      items.push({ id: `app-${actionId}`, keys, ...item });
    };

    addItem('open-settings', {
      allowInEditable: true,
      action: () => options.setActiveTab('settings')
    });

    addItem('focus-search', {
      allowInEditable: true,
      action: async (context) => {
        const scopeRoot = resolveScopeRoot(context);
        await nextTick();
        await waitForNextFrame();
        await focusByRole('search', scopeRoot);
      }
    });

    addItem('new-message', {
      action: async () => {
        if (options.activeTab.value !== 'chat') {
          options.setActiveTab('chat');
        }
        await nextTick();
        await waitForNextFrame();
        await focusByRole('chat-input');
      }
    });

    return items;
  };

  registerKeybindRule({
    id: APP_KEYBIND_RULE_ID,
    order: 0,
    mode: 'merge',
    stop: false,
    matches: (context) => context.view !== 'terminal' && context.view !== 'notification-preview',
    items: () => buildItems()
  });

  // 浏览器快捷键与功能键拦截：阻止 WebView 默认行为。
  registerKeybindRule({
    id: GUARD_KEYBIND_RULE_ID,
    order: 1000,
    mode: 'merge',
    stop: false,
    matches: () => true,
    items: () => {
      // [TODO/golutra, 2026-01-25] 调试完成后恢复 F12 守护。
      const functionKeys = Array.from({ length: 11 }, (_, index) => `F${index + 1}`);
      const browserShortcuts = [
        'CmdOrCtrl + R',
        'CmdOrCtrl + Shift + R',
        'CmdOrCtrl + L',
        'CmdOrCtrl + K',
        'CmdOrCtrl + P',
        'CmdOrCtrl + N',
        'CmdOrCtrl + Shift + N',
        'CmdOrCtrl + T',
        'CmdOrCtrl + Shift + T',
        'CmdOrCtrl + W',
        'CmdOrCtrl + Shift + W',
        'CmdOrCtrl + Shift + I',
        'CmdOrCtrl + Shift + C',
        'CmdOrCtrl + Shift + J',
        'CmdOrCtrl + U',
        'CmdOrCtrl + S',
        'CmdOrCtrl + F',
        'CmdOrCtrl + D',
        'CmdOrCtrl + =',
        'CmdOrCtrl + -',
        'CmdOrCtrl + 0',
        'Alt + ArrowLeft',
        'Alt + ArrowRight'
      ];
      const keys = functionKeys.concat(browserShortcuts);
      return keys.map((combo) => ({
        id: `guard-${combo}`,
        keys: combo,
        allowInEditable: true,
        allowWhenComposing: true,
        allowRepeat: true,
        stop: false,
        action: () => {}
      }));
    }
  });

  return () => {
    unregisterKeybindRule(APP_KEYBIND_RULE_ID);
    unregisterKeybindRule(GUARD_KEYBIND_RULE_ID);
  };
};
