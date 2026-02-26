<template>
  <section class="flex h-full w-full flex-col overflow-hidden">
    <header class="flex items-center justify-between px-6 py-4 border-b border-white/5 bg-panel/60 backdrop-blur">
      <div>
        <h1 class="text-xl font-semibold text-white">{{ t('terminal.title') }}</h1>
        <p class="text-xs text-white/40">{{ t('terminal.subtitle') }}</p>
      </div>
      <div class="flex items-center gap-3">
        <button
          type="button"
          class="inline-flex items-center gap-2 px-3 py-2 rounded-lg text-xs font-semibold uppercase tracking-wide text-white/80 border border-white/10 bg-white/5 hover:bg-white/10 hover:text-white transition"
          @click="handleNewTab"
        >
          <span class="material-symbols-outlined text-[18px]">add</span>
          {{ t('terminal.newTab') }}
        </button>
      </div>
    </header>

    <div class="flex items-center gap-2 px-6 py-3 border-b border-white/5 bg-surface/30 overflow-x-auto">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        type="button"
        draggable="true"
        @dragstart="onDragStart(tab.id, $event)"
        @dragover="onDragOver(tab.id, $event)"
        @drop="onDrop(tab.id)"
        @dragend="onDragEnd"
        @click="setActive(tab.id)"
        :class="[
          'group flex items-center gap-2 px-3 py-1.5 rounded-lg border transition whitespace-nowrap',
          tab.id === activeId
            ? 'bg-white/10 border-white/30 text-white'
            : 'bg-white/5 border-white/10 text-white/60 hover:text-white hover:border-white/20',
          dragOverId === tab.id && dragId !== tab.id ? 'ring-1 ring-primary/60' : ''
        ]"
      >
        <span class="material-symbols-outlined text-[16px]">terminal</span>
        <span class="text-xs font-semibold">{{ tab.title }}</span>
        <span v-if="tab.hasActivity" class="ml-1 w-2 h-2 rounded-full bg-primary shadow-glow"></span>
        <span
          class="material-symbols-outlined text-[14px] text-white/40 hover:text-white"
          @click.stop="closeTab(tab.id)"
        >
          close
        </span>
      </button>
      <span v-if="tabs.length === 0" class="text-xs text-white/40">
        {{ t('terminal.emptyTabs') }}
      </span>
    </div>

    <div class="flex-1 min-h-0 relative">
      <div v-if="tabs.length === 0" class="h-full flex flex-col items-center justify-center text-center text-white/50">
        <div class="w-14 h-14 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center mb-4">
          <span class="material-symbols-outlined text-[28px]">terminal</span>
        </div>
        <p class="text-sm font-semibold text-white/70">{{ t('terminal.emptyTitle') }}</p>
        <p class="text-xs text-white/40 mt-1">{{ t('terminal.emptySubtitle') }}</p>
        <button
          type="button"
          class="mt-4 inline-flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-semibold uppercase tracking-wide text-white bg-primary hover:bg-primary-hover shadow-glow transition"
          @click="handleNewTab"
        >
          <span class="material-symbols-outlined text-[18px]">add</span>
          {{ t('terminal.newTab') }}
        </button>
      </div>
      <div v-else class="h-full w-full">
        <TerminalPane
          v-for="tab in tabs"
          :key="tab.id"
          :session-id="tab.id"
          :active="tab.id === activeId"
          v-show="tab.id === activeId"
        />
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { emit, listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { storeToRefs } from 'pinia';
import TerminalPane from './TerminalPane.vue';
import { onActivity } from './terminalBridge';
import { useTerminalStore } from './terminalStore';
import {
  TERMINAL_OPEN_TAB_EVENT,
  TERMINAL_WINDOW_READY_EVENT,
  TERMINAL_WINDOW_READY_REQUEST_EVENT,
  type TerminalOpenTabPayload
} from './terminalEvents';
import { useToastStore } from '@/stores/toastStore';
import { useProjectStore } from '@/features/workspace/projectStore';

const { t } = useI18n();
const toastStore = useToastStore();
const { pushToast } = toastStore;
const terminalStore = useTerminalStore();
const { tabs, activeId } = storeToRefs(terminalStore);
const { createTab, setActive, closeTab, moveTab, markActivity, clearActivity, openTab } = terminalStore;
const projectStore = useProjectStore();
const { members } = storeToRefs(projectStore);

const dragId = ref<string | null>(null);
const dragOverId = ref<string | null>(null);
const isCreating = ref(false);

const resolveTabTitle = (memberId: string | undefined, payloadTitle: string) => {
  if (memberId) {
    const memberName = members.value.find((member) => member.id === memberId)?.name?.trim();
    if (memberName) {
      return memberName;
    }
  }
  const fallbackTitle = payloadTitle.trim();
  return fallbackTitle || payloadTitle;
};

const handleNewTab = async () => {
  if (isCreating.value) {
    return;
  }
  isCreating.value = true;
  try {
    await createTab();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (message.includes('terminal buffer limit reached')) {
      pushToast(t('terminal.resourceLimit'), { tone: 'error' });
    } else {
      console.error('Failed to start terminal.', error);
    }
  } finally {
    isCreating.value = false;
  }
};

const onDragStart = (id: string, event: DragEvent) => {
  dragId.value = id;
  dragOverId.value = null;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', id);
  }
};

const onDragOver = (id: string, event: DragEvent) => {
  if (!dragId.value || dragId.value === id) {
    return;
  }
  event.preventDefault();
  dragOverId.value = id;
};

const onDrop = (id: string) => {
  if (!dragId.value || dragId.value === id) {
    dragOverId.value = null;
    dragId.value = null;
    return;
  }
  moveTab(dragId.value, id);
  dragOverId.value = null;
  dragId.value = null;
};

const onDragEnd = () => {
  dragOverId.value = null;
  dragId.value = null;
};

const stopActivity = onActivity((sessionId) => {
  markActivity(sessionId);
});
let stopOpenTab: (() => void) | null = null;
let stopReadyRequest: (() => void) | null = null;
let autoTabTimer: number | null = null;

const emitWindowReady = () => {
  try {
    const windowLabel = getCurrentWebviewWindow().label;
    void emit(TERMINAL_WINDOW_READY_EVENT, { windowLabel });
  } catch {
    // No-op when running outside a Tauri window.
  }
};

watch(activeId, (next) => {
  if (next) {
    clearActivity(next);
  }
});

watch(
  members,
  (nextMembers) => {
    const memberMap = new Map(nextMembers.map((member) => [member.id, member.name]));
    for (const tab of tabs.value) {
      if (!tab.memberId) {
        continue;
      }
      const memberName = memberMap.get(tab.memberId)?.trim();
      if (memberName && tab.title !== memberName) {
        tab.title = memberName;
      }
    }
  },
  { deep: true, immediate: true }
);

onMounted(async () => {
  const init = async () => {
    try {
      stopOpenTab = await listen<TerminalOpenTabPayload>(TERMINAL_OPEN_TAB_EVENT, (event) => {
        const { sessionId, title, memberId, keepAlive } = event.payload;
        const resolvedTitle = resolveTabTitle(memberId, title);
        openTab(sessionId, { title: resolvedTitle, memberId, keepAlive });
        if (autoTabTimer !== null) {
          window.clearTimeout(autoTabTimer);
          autoTabTimer = null;
        }
      });
    } catch (error) {
      console.error('Failed to listen for terminal tabs.', error);
    }

    try {
      stopReadyRequest = await listen(TERMINAL_WINDOW_READY_REQUEST_EVENT, () => {
        emitWindowReady();
      });
    } catch (error) {
      console.error('Failed to listen for terminal ready requests.', error);
    }

    emitWindowReady();

    if (tabs.value.length === 0) {
      autoTabTimer = window.setTimeout(() => {
        if (tabs.value.length === 0) {
          void handleNewTab();
        }
      }, 600);
    }
  };

  await init();
});

onBeforeUnmount(() => {
  stopActivity();
  stopOpenTab?.();
  stopOpenTab = null;
  stopReadyRequest?.();
  stopReadyRequest = null;
  if (autoTabTimer !== null) {
    window.clearTimeout(autoTabTimer);
    autoTabTimer = null;
  }
});
</script>
