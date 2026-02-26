<template>
  <div
    class="window-frame"
    :class="{ 'window-frame--max': isMaximized, 'window-frame--inactive': !isFocused }"
  >
    <header
      class="titlebar"
      :class="{ 'titlebar--mac': isMacOS }"
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
          :aria-label="t('app.windowControls.maximize')"
          :title="t('app.windowControls.maximize')"
          data-tauri-drag-region="false"
          @click="handleToggleMaximize"
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <rect x="2" y="2" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1.2" rx="0.6" />
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
        class="flex h-full w-full bg-background app-shell font-sans relative overflow-hidden"
      >
        <SidebarNav :active-tab="activeTab" @change="setActiveTab($event)" />
        <main class="flex-1 h-full overflow-hidden relative flex flex-col pb-16 md:pb-0">
          <SkillStore v-if="activeTab === 'store'" />
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
  </div>
</template>

<script setup lang="ts">
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
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useGlobalStore } from '@/features/global/globalStore';
import { useNavigationStore } from '@/stores/navigationStore';
import { useWorkspaceBootstrap } from './useWorkspaceBootstrap';
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

const navigationStore = useNavigationStore();
const { activeTab } = storeToRefs(navigationStore);
const { setActiveTab } = navigationStore;
const workspaceStore = useWorkspaceStore();
const { currentWorkspace } = storeToRefs(workspaceStore);
const showWorkspaceSelection = computed(() => activeTab.value === 'workspaces' || !currentWorkspace.value);
const { appReady } = useWorkspaceBootstrap();
const globalStore = useGlobalStore();
void globalStore.hydrate();
const { t } = useI18n();
const resolvedView =
  typeof window !== 'undefined'
    ? new URLSearchParams(window.location.search).get('view') ??
      (window as typeof window & { __NEXUS_VIEW__?: string }).__NEXUS_VIEW__
    : null;
const isTerminalView = resolvedView === 'terminal';
const isTauriEnv = isTauri();
const APP_NAME = 'golutra';
const isMacOS = computed(() => typeof navigator !== 'undefined' && /Mac/i.test(navigator.platform));
const showWindowControls = computed(() => isTauriEnv && !isMacOS.value);
const contextTitle = computed(() => {
  if (isTerminalView) {
    const name = currentWorkspace.value?.name?.trim();
    return name ? `${name} Terminal` : 'Terminal';
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
let unlistenResize: (() => void) | null = null;
let unlistenFocus: (() => void) | null = null;

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
  void appWindow.close();
};

watch(
  () => currentWorkspace.value,
  (workspace) => {
    if (!workspace) {
      setActiveTab('workspaces');
      return;
    }
    if (activeTab.value === 'workspaces') {
      setActiveTab('chat');
    }
  },
  { immediate: true }
);

watch(
  windowTitle,
  (title) => {
    if (!appWindow) return;
    appWindow.setTitle(title).catch(() => {});
  },
  { immediate: true }
);

onMounted(() => {
  if (!appWindow) return;
  void refreshMaximized();
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
    })
    .then((unlisten) => {
      unlistenFocus = unlisten;
    })
    .catch(() => {});
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
});
</script>
