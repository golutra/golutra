<template>
  <NotificationPreview v-if="isNotificationPreview" />
  <div
    v-else
    class="window-frame relative"
    :class="{
      'window-frame--max': isMaximized,
      'window-frame--inactive': !isFocused,
      'window-frame--hover-stale': !isPointerReady
    }"
  >
    <header
      class="titlebar"
      :class="{
        'titlebar--mac': isMacOS,
        'titlebar--workspace': showWorkspaceSelection || resolvedView === 'workspace-selection'
      }"
      data-tauri-drag-region
      @dblclick="handleToggleMaximize"
    >
      <div class="titlebar__left" data-tauri-drag-region>
        <span class="titlebar__title">{{ windowTitle }}</span>
      </div>
      <div v-if="showWindowControls" class="titlebar__controls" data-tauri-drag-region="false" @dblclick.stop>
        <button
          type="button"
          class="titlebar__btn"
          :aria-label="t('app.windowControls.minimize')"
          :title="t('app.windowControls.minimize')"
          data-tauri-drag-region="false"
          @click="handleMinimize"
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
        </button>
        <button
          type="button"
          class="titlebar__btn"
          :aria-label="isMaximized ? t('app.windowControls.restore') : t('app.windowControls.maximize')"
          :title="isMaximized ? t('app.windowControls.restore') : t('app.windowControls.maximize')"
          data-tauri-drag-region="false"
          @click="handleToggleMaximize"
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <rect
              v-if="!isMaximized"
              x="2"
              y="2"
              width="6"
              height="6"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
              rx="0.6"
            />
            <g v-else fill="none" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round">
              <rect x="3" y="1.5" width="5.5" height="5.5" rx="0.6" />
              <rect x="1.5" y="3" width="5.5" height="5.5" rx="0.6" />
            </g>
          </svg>
        </button>
        <button
          type="button"
          class="titlebar__btn titlebar__btn--close"
          :aria-label="t('app.windowControls.close')"
          :title="t('app.windowControls.close')"
          data-tauri-drag-region="false"
          @click="handleClose"
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <line x1="2" y1="2" x2="8" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            <line x1="8" y1="2" x2="2" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
        </button>
      </div>
    </header>

    <div class="window-body">
      <div v-if="isTerminalView" class="flex h-full w-full bg-background app-shell relative overflow-hidden">
        <TerminalWorkspace />
      </div>
      <div
        v-else-if="showWorkspaceSelection"
        class="flex h-full w-full bg-background app-shell relative overflow-y-auto overflow-x-hidden"
      >
        <WorkspaceSelection />
      </div>
      <div
        v-else-if="!appReady"
        class="flex h-full w-full bg-background app-shell relative overflow-hidden items-center justify-center"
      >
        <div class="flex flex-col items-center gap-3 text-white/70 text-sm">
          <div class="h-10 w-10 rounded-full border border-white/20 border-t-transparent animate-spin"></div>
          <span>Loading workspace...</span>
        </div>
      </div>
      <div
        v-else
        class="flex h-full w-full bg-transparent app-shell font-sans relative overflow-hidden"
      >
        <SidebarNav :active-tab="activeTab" @change="setActiveTab($event)" />
        <main class="flex-1 h-full overflow-hidden relative flex flex-col pb-16 md:pb-0 bg-panel/50 glass-panel border-0">
          <FriendsView v-if="activeTab === 'friends'" />
          <SkillStore v-else-if="activeTab === 'store'" />
          <PluginMarketplace v-else-if="activeTab === 'plugins'" />
          <Settings
            v-else-if="activeTab === 'settings'"
            @logout="setActiveTab('workspaces')"
          />
          <ChatInterface v-else />
        </main>
      </div>
    </div>
    <ToastStack />
    <ContextMenuHost />
    <div v-if="showResizeHandles" class="absolute inset-0 pointer-events-none z-50">
      <div
        class="absolute top-0 left-4 right-4 h-1 pointer-events-auto cursor-ns-resize"
        @pointerdown.prevent="handleResizeStart('North')"
      ></div>
      <div
        class="absolute bottom-0 left-4 right-4 h-1 pointer-events-auto cursor-ns-resize"
        @pointerdown.prevent="handleResizeStart('South')"
      ></div>
      <div
        class="absolute left-0 top-4 bottom-4 w-1 pointer-events-auto cursor-ew-resize"
        @pointerdown.prevent="handleResizeStart('West')"
      ></div>
      <div
        class="absolute right-0 top-4 bottom-4 w-1 pointer-events-auto cursor-ew-resize"
        @pointerdown.prevent="handleResizeStart('East')"
      ></div>
      <div
        class="absolute top-0 left-0 w-3 h-3 pointer-events-auto cursor-nwse-resize"
        @pointerdown.prevent="handleResizeStart('NorthWest')"
      ></div>
      <div
        class="absolute top-0 right-0 w-3 h-3 pointer-events-auto cursor-nesw-resize"
        @pointerdown.prevent="handleResizeStart('NorthEast')"
      ></div>
      <div
        class="absolute bottom-0 left-0 w-3 h-3 pointer-events-auto cursor-nesw-resize"
        @pointerdown.prevent="handleResizeStart('SouthWest')"
      ></div>
      <div
        class="absolute bottom-0 right-0 w-3 h-3 pointer-events-auto cursor-nwse-resize"
        @pointerdown.prevent="handleResizeStart('SouthEast')"
      ></div>
    </div>
  </div>
</template>

<script setup lang="ts">
// 应用根组件：根据当前窗口上下文切换主视图与终端视图，并处理窗口控制状态。
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { storeToRefs } from 'pinia';
import SidebarNav from '@/shared/components/SidebarNav.vue';
import ToastStack from '@/shared/components/ToastStack.vue';
import SkillStore from '@/features/SkillStore.vue';
import PluginMarketplace from '@/features/PluginMarketplace.vue';
import TerminalWorkspace from '@/features/terminal/TerminalWorkspace.vue';
import Settings from '@/features/Settings.vue';
import WorkspaceSelection from '@/features/WorkspaceSelection.vue';
import ChatInterface from '@/features/chat/ChatInterface.vue';
import FriendsView from '@/features/chat/FriendsView.vue';
import NotificationPreview from '@/features/notifications/NotificationPreview.vue';
import ContextMenuHost from '@/shared/context-menu/ContextMenuHost.vue';
import { registerDefaultContextMenuRules } from '@/shared/context-menu/defaults';
import { useContextMenu } from '@/shared/context-menu/useContextMenu';
import { useKeyboard } from '@/shared/keyboard/useKeyboard';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useGlobalStore } from '@/features/global/globalStore';
import { useSettingsStore } from '@/features/global/settingsStore';
import { useNavigationStore, type AppTabId } from '@/stores/navigationStore';
import { useNotificationOrchestratorStore } from '@/stores/notificationOrchestratorStore';
import { useWorkspaceBootstrap } from './useWorkspaceBootstrap';
import { registerAppKeybinds } from './useAppKeybinds';
import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow, type ResizeDirection } from '@tauri-apps/api/window';
import { openWorkspaceSelectionWindow } from '@/shared/tauri/windows';
import { LOCALE_CHANGED_EVENT, type AppLocale } from '@/i18n';
import { THEME_CHANGED_EVENT, type AppTheme } from '@/features/global/theme';

const { t } = useI18n();
// 兼容 query 参数与 Tauri 初始化脚本，两者均可能决定当前窗口视图。
const resolvedView =
  typeof window !== 'undefined'
    ? new URLSearchParams(window.location.search).get('view') ??
      (window as typeof window & { __GOLUTRA_VIEW__?: string }).__GOLUTRA_VIEW__
    : null;
const isNotificationPreview = resolvedView === 'notification-preview';
const isTerminalView = resolvedView === 'terminal';
const navigationStore = isNotificationPreview ? null : useNavigationStore();
const activeTab = navigationStore ? storeToRefs(navigationStore).activeTab : ref<AppTabId>('chat');
const setActiveTab = navigationStore ? navigationStore.setActiveTab : () => {};
const workspaceStore = isNotificationPreview ? null : useWorkspaceStore();
const currentWorkspace = workspaceStore ? storeToRefs(workspaceStore).currentWorkspace : ref(null);
const settingsStore = useSettingsStore();
const showWorkspaceSelection = computed(
  () => !isNotificationPreview && (activeTab.value === 'workspaces' || !currentWorkspace.value)
);
const appReady = isNotificationPreview ? ref(true) : useWorkspaceBootstrap().appReady;

if (!isNotificationPreview) {
  useNotificationOrchestratorStore();
  const globalStore = useGlobalStore();
  void globalStore.hydrate();
}
const isTauriEnv = isTauri();
const APP_NAME = 'golutra';
const isMacOS = computed(() => typeof navigator !== 'undefined' && /Mac/i.test(navigator.platform));
const showWindowControls = computed(() => isTauriEnv && !isMacOS.value);
const showResizeHandles = computed(() => isTauriEnv && !isNotificationPreview);
const isMainView = resolvedView === null;
type WorkspaceBootPayload = { id?: string; name?: string; path?: string };
const terminalBootWorkspaceName =
  typeof window !== 'undefined'
    ? ((window as typeof window & { __GOLUTRA_WORKSPACE__?: WorkspaceBootPayload }).__GOLUTRA_WORKSPACE__?.name ?? '').trim()
    : '';
const contextTitle = computed(() => {
  if (isNotificationPreview) {
    return APP_NAME;
  }
  if (isTerminalView) {
    // 终端窗口优先使用初始化注入的工作区名，避免主窗口逻辑覆盖标题。
    const name = currentWorkspace.value?.name?.trim() || terminalBootWorkspaceName;
    return name ? `${name} - Terminal` : 'Terminal';
  }
  if (showWorkspaceSelection.value) return 'Workspaces';
  if (!appReady.value) return 'Loading';
  const workspaceName = currentWorkspace.value?.name?.trim();
  return workspaceName || 'Home';
});
const windowTitle = computed(() => `${contextTitle.value} - ${APP_NAME}`);
const appWindow = isTauriEnv ? getCurrentWindow() : null;
const isMaximized = ref(false);
const isFocused = ref(true);
const isPointerReady = ref(true);
let unlistenResize: (() => void) | null = null;
let unlistenFocus: (() => void) | null = null;
let unlistenOpenWorkspaceSelection: (() => void) | null = null;
let unlistenLocaleChange: (() => void) | null = null;
let unlistenThemeChange: (() => void) | null = null;
let removeContextMenuListener: (() => void) | null = null;
let removeKeydownListener: (() => void) | null = null;
let removeAppKeybinds: (() => void) | null = null;
let pointerReadyListener: (() => void) | null = null;
const wasFocused = ref(true);
const { handleContextMenuEvent } = useContextMenu();
const { handleKeydownEvent } = useKeyboard();

const armPointerReadyReset = () => {
  if (typeof window === 'undefined') {
    return;
  }
  isPointerReady.value = false;
  if (pointerReadyListener) {
    window.removeEventListener('pointermove', pointerReadyListener);
  }
  pointerReadyListener = () => {
    isPointerReady.value = true;
    if (pointerReadyListener) {
      window.removeEventListener('pointermove', pointerReadyListener);
      pointerReadyListener = null;
    }
  };
  // 窗口恢复可见时先抑制 hover，等待首次鼠标移动再启用。
  window.addEventListener('pointermove', pointerReadyListener, { once: true });
};

const refreshMaximized = async () => {
  if (!appWindow) return;
  try {
    isMaximized.value = await appWindow.isMaximized();
  } catch {
    isMaximized.value = false;
  }
};

const handleMinimize = () => {
  if (!appWindow) return;
  void appWindow.minimize();
};

const handleToggleMaximize = () => {
  if (!appWindow) return;
  void appWindow.toggleMaximize();
};

const handleClose = () => {
  if (!appWindow) return;
  armPointerReadyReset();
  void appWindow.close();
};

const handleResizeStart = (direction: ResizeDirection) => {
  if (!appWindow) return;
  void appWindow.startResizeDragging(direction);
};

const resolveWorkspaceBoot = () => {
  if (typeof window === 'undefined') {
    return null;
  }
  const payload = (window as typeof window & { __GOLUTRA_WORKSPACE__?: WorkspaceBootPayload }).__GOLUTRA_WORKSPACE__;
  const path = typeof payload?.path === 'string' ? payload.path.trim() : '';
  if (!path) {
    return null;
  }
  return { path };
};

const consumeWorkspaceBoot = () => {
  if (!workspaceStore || !isMainView || isTerminalView || isNotificationPreview) {
    return;
  }
  if (currentWorkspace.value) {
    return;
  }
  const boot = resolveWorkspaceBoot();
  if (!boot) {
    return;
  }
  void workspaceStore.openWorkspaceByPath(boot.path);
  try {
    delete (window as typeof window & { __GOLUTRA_WORKSPACE__?: WorkspaceBootPayload }).__GOLUTRA_WORKSPACE__;
  } catch {
    // ignore
  }
};

watch(
  () => currentWorkspace.value,
  (workspace) => {
    if (isNotificationPreview) {
      return;
    }
    if (!workspace) {
      setActiveTab('workspaces');
      return;
    }
    // 首次进入工作区时自动切到聊天视图，避免停留在工作区选择页。
    if (activeTab.value === 'workspaces') {
      setActiveTab('chat');
    }
  },
  { immediate: true }
);

watch(
  windowTitle,
  (title) => {
    if (!appWindow || isNotificationPreview) return;
    appWindow.setTitle(title).catch(() => {});
  },
  { immediate: true }
);

onMounted(() => {
  if (!appWindow || isNotificationPreview) return;
  void refreshMaximized();
  consumeWorkspaceBoot();
  if (isTauriEnv && isMainView) {
    listen(
      'app-open-workspace-selection',
      () => {
        void openWorkspaceSelectionWindow();
      },
      { target: 'main' }
    )
      .then((unlisten) => {
        unlistenOpenWorkspaceSelection = unlisten;
      })
      .catch(() => {});
  }
  if (isTauriEnv) {
    listen<{ locale: AppLocale }>(LOCALE_CHANGED_EVENT, (event) => {
      const next = event.payload?.locale;
      if (next) {
        settingsStore.applyExternalLocale(next);
      }
    })
      .then((unlisten) => {
        unlistenLocaleChange = unlisten;
      })
      .catch(() => {});
    listen<{ theme: AppTheme }>(THEME_CHANGED_EVENT, (event) => {
      const next = event.payload?.theme;
      if (next) {
        settingsStore.applyExternalTheme(next);
      }
    })
      .then((unlisten) => {
        unlistenThemeChange = unlisten;
      })
      .catch(() => {});
  }
  appWindow
    .onResized(() => {
      void refreshMaximized();
    })
    .then((unlisten) => {
      unlistenResize = unlisten;
    })
    .catch(() => {});
  appWindow
    .onFocusChanged((event) => {
      isFocused.value = event.payload;
      if (event.payload && !wasFocused.value) {
        armPointerReadyReset();
      }
      wasFocused.value = event.payload;
    })
    .then((unlisten) => {
      unlistenFocus = unlisten;
    })
    .catch(() => {});
});

onMounted(() => {
  registerDefaultContextMenuRules();
  if (typeof window === 'undefined') {
    return;
  }
  const listener = (event: MouseEvent) => handleContextMenuEvent(event);
  window.addEventListener('contextmenu', listener);
  removeContextMenuListener = () => {
    window.removeEventListener('contextmenu', listener);
  };
});

onMounted(() => {
  if (isNotificationPreview) {
    return;
  }
  removeAppKeybinds = registerAppKeybinds({ activeTab, setActiveTab });
  if (typeof window === 'undefined') {
    return;
  }
  const listener = (event: KeyboardEvent) => handleKeydownEvent(event);
  window.addEventListener('keydown', listener, { capture: true });
  removeKeydownListener = () => {
    window.removeEventListener('keydown', listener, { capture: true });
  };
});

onBeforeUnmount(() => {
  if (unlistenResize) {
    unlistenResize();
    unlistenResize = null;
  }
  if (unlistenFocus) {
    unlistenFocus();
    unlistenFocus = null;
  }
  if (unlistenOpenWorkspaceSelection) {
    unlistenOpenWorkspaceSelection();
    unlistenOpenWorkspaceSelection = null;
  }
  if (unlistenLocaleChange) {
    unlistenLocaleChange();
    unlistenLocaleChange = null;
  }
  if (unlistenThemeChange) {
    unlistenThemeChange();
    unlistenThemeChange = null;
  }
  if (removeContextMenuListener) {
    removeContextMenuListener();
    removeContextMenuListener = null;
  }
  if (removeKeydownListener) {
    removeKeydownListener();
    removeKeydownListener = null;
  }
  if (removeAppKeybinds) {
    removeAppKeybinds();
    removeAppKeybinds = null;
  }
  if (pointerReadyListener) {
    window.removeEventListener('pointermove', pointerReadyListener);
    pointerReadyListener = null;
  }
});
</script>
