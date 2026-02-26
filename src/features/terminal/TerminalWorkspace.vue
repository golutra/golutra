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

    <div
      ref="tabBarRef"
      class="relative flex items-center gap-2 px-6 py-3 border-b border-white/5 bg-surface/30 overflow-x-auto"
    >
      <TransitionGroup name="terminal-tab" tag="div" class="flex items-center gap-2">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          :data-tab-id="tab.id"
          @pointerdown="onPointerDown(tab.id, $event)"
          @click="handleTabClick(tab.id, $event)"
          :class="[
            'group flex items-center gap-2 px-3 py-1.5 rounded-lg border transition whitespace-nowrap cursor-default',
            tab.id === activeId
              ? 'bg-white/10 border-white/30 text-white'
              : 'bg-white/5 border-white/10 text-white/60 hover:text-white hover:border-white/20',
            isDragging && dragId === tab.id ? 'terminal-tab--placeholder' : '',
            dragOverId === tab.id && dragId !== tab.id ? 'ring-1 ring-primary/60' : ''
          ]"
        >
          <span class="material-symbols-outlined text-[16px]">terminal</span>
          <span class="text-xs font-semibold">{{ tab.title }}</span>
          <span
            v-if="tab.hasActivity || tab.isBlinking"
            :class="[
              'ml-1 w-2 h-2 rounded-full bg-primary shadow-glow',
              tab.isBlinking ? 'terminal-tab__activity--blink' : ''
            ]"
          ></span>
          <span
            class="material-symbols-outlined text-[14px] text-white/40 hover:text-white"
            @pointerdown.stop
            @click.stop="handleCloseTab(tab.id)"
          >
            close
          </span>
        </button>
      </TransitionGroup>
      <div
        v-if="isDragging && dragGhostTab"
        class="terminal-tab-ghost group flex items-center gap-2 px-3 py-1.5 rounded-lg border whitespace-nowrap"
        :class="[
          dragGhostTab.id === activeId
            ? 'bg-white/10 border-white/30 text-white'
            : 'bg-white/5 border-white/10 text-white/60'
        ]"
        :style="dragGhostStyle"
      >
        <span class="material-symbols-outlined text-[16px]">terminal</span>
        <span class="text-xs font-semibold">{{ dragGhostTab.title }}</span>
        <span
          v-if="dragGhostTab.hasActivity || dragGhostTab.isBlinking"
          :class="[
            'ml-1 w-2 h-2 rounded-full bg-primary shadow-glow',
            dragGhostTab.isBlinking ? 'terminal-tab__activity--blink' : ''
          ]"
        ></span>
        <span class="material-symbols-outlined text-[14px] text-white/40">close</span>
      </div>
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
          :member-id="tab.memberId"
          :terminal-type="tab.terminalType"
          :active="tab.id === activeId"
          v-show="tab.id === activeId"
        />
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { emit, listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { storeToRefs } from 'pinia';
import TerminalPane from './TerminalPane.vue';
import { onActivity, onStatusChange, trackSession, untrackSession } from './terminalBridge';
import { useTerminalStore } from './terminalStore';
import {
  TERMINAL_OPEN_TAB_EVENT,
  TERMINAL_WINDOW_READY_EVENT,
  TERMINAL_WINDOW_READY_REQUEST_EVENT,
  type TerminalOpenTabPayload
} from './terminalEvents';
import { useToastStore } from '@/stores/toastStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';

const { t } = useI18n();
const toastStore = useToastStore();
const { pushToast } = toastStore;
const terminalStore = useTerminalStore();
const { tabs, activeId } = storeToRefs(terminalStore);
const { createTab, setActive, closeTab, moveTabToIndex, markActivity, clearActivity, openTab } = terminalStore;
const projectStore = useProjectStore();
const { members } = storeToRefs(projectStore);
const workspaceStore = useWorkspaceStore();
const { currentWorkspace } = storeToRefs(workspaceStore);

const tabBarRef = ref<HTMLDivElement | null>(null);
const dragId = ref<string | null>(null);
const dragOverId = ref<string | null>(null);
const dragPointerId = ref<number | null>(null);
const isDragging = ref(false);
const isCreating = ref(false);
const suppressClick = ref(false);
const dragInsertIndex = ref<number | null>(null);
const dragGhostLeft = ref(0);
const dragGhostTop = ref(0);
const dragGhostWidth = ref(0);
const dragGhostHeight = ref(0);
const dragGrabOffsetX = ref(0);
const lastPointerX = ref<number | null>(null);
const dragCaptureTarget = ref<HTMLElement | null>(null);
const pointerMoveThreshold = 4;
const snapThreshold = 10;
const autoScrollEdge = 28;
const autoScrollMaxSpeed = 16;
let autoScrollRaf: number | null = null;
let autoScrollVelocity = 0;
let dragStartX = 0;
let dragStartY = 0;
const statusBySession = new Map<string, string>();

type WorkspaceWindowContext = { id?: string; name?: string; path?: string };

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

const resolveWorkspaceContext = () => {
  const workspace = currentWorkspace.value;
  if (workspace?.path) {
    return { cwd: workspace.path, workspaceId: workspace.id };
  }
  if (typeof window === 'undefined') {
    return { cwd: undefined, workspaceId: undefined };
  }
  const meta = (window as typeof window & { __NEXUS_WORKSPACE__?: WorkspaceWindowContext }).__NEXUS_WORKSPACE__;
  const path = typeof meta?.path === 'string' ? meta.path.trim() : '';
  const id = typeof meta?.id === 'string' ? meta.id.trim() : '';
  return { cwd: path || undefined, workspaceId: id || undefined };
};

const handleNewTab = async () => {
  if (isCreating.value) {
    return;
  }
  isCreating.value = true;
  try {
    const { cwd, workspaceId } = resolveWorkspaceContext();
    const sessionId = await createTab({ cwd, workspaceId });
    trackSession(sessionId);
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

const cleanupDragListeners = () => {
  window.removeEventListener('pointermove', onPointerMove);
  window.removeEventListener('pointerup', onPointerUp);
  window.removeEventListener('pointercancel', onPointerUp);
};

const cleanupDragState = () => {
  dragOverId.value = null;
  dragId.value = null;
  dragPointerId.value = null;
  isDragging.value = false;
  dragInsertIndex.value = null;
  dragGhostLeft.value = 0;
  dragGhostTop.value = 0;
  dragGhostWidth.value = 0;
  dragGhostHeight.value = 0;
  dragGrabOffsetX.value = 0;
  lastPointerX.value = null;
  dragCaptureTarget.value = null;
};

const stopAutoScroll = () => {
  if (autoScrollRaf !== null) {
    window.cancelAnimationFrame(autoScrollRaf);
    autoScrollRaf = null;
  }
  autoScrollVelocity = 0;
};

const startAutoScroll = () => {
  if (autoScrollRaf !== null) {
    return;
  }
  const step = () => {
    if (!isDragging.value || autoScrollVelocity === 0) {
      autoScrollRaf = null;
      return;
    }
    const container = tabBarRef.value;
    if (container) {
      const prevScroll = container.scrollLeft;
      container.scrollLeft += autoScrollVelocity;
      if (container.scrollLeft !== prevScroll && lastPointerX.value !== null) {
        updateGhostPosition(lastPointerX.value);
        updateDragTarget(lastPointerX.value);
      }
    }
    autoScrollRaf = window.requestAnimationFrame(step);
  };
  autoScrollRaf = window.requestAnimationFrame(step);
};

const onPointerDown = (id: string, event: PointerEvent) => {
  if (event.button !== 0) {
    return;
  }
  dragId.value = id;
  dragOverId.value = null;
  dragInsertIndex.value = null;
  dragPointerId.value = event.pointerId;
  dragStartX = event.clientX;
  dragStartY = event.clientY;
  lastPointerX.value = event.clientX;
  isDragging.value = false;
  suppressClick.value = false;
  const target = event.currentTarget as HTMLElement | null;
  const rect = target?.getBoundingClientRect();
  const barRect = tabBarRef.value?.getBoundingClientRect();
  if (rect && barRect) {
    const scrollLeft = tabBarRef.value?.scrollLeft ?? 0;
    dragGhostLeft.value = rect.left - barRect.left + scrollLeft;
    dragGhostTop.value = rect.top - barRect.top;
    dragGhostWidth.value = rect.width;
    dragGhostHeight.value = rect.height;
    dragGrabOffsetX.value = event.clientX - rect.left;
  } else {
    dragGhostLeft.value = 0;
    dragGhostTop.value = 0;
    dragGhostWidth.value = 0;
    dragGhostHeight.value = 0;
    dragGrabOffsetX.value = 0;
  }
  dragCaptureTarget.value = target;
  if (target && target.setPointerCapture) {
    target.setPointerCapture(event.pointerId);
  }
  updateGhostPosition(event.clientX);
  event.preventDefault();
  window.addEventListener('pointermove', onPointerMove);
  window.addEventListener('pointerup', onPointerUp);
  window.addEventListener('pointercancel', onPointerUp);
};

const resolveInsertTarget = (clientX: number) => {
  const container = tabBarRef.value;
  if (!container || !dragId.value) {
    return null;
  }
  const elements = Array.from(container.querySelectorAll<HTMLElement>('[data-tab-id]')).filter(
    (element) => element.dataset.tabId && element.dataset.tabId !== dragId.value
  );
  if (elements.length === 0) {
    return { insertIndex: 0, overId: null, midpoint: null };
  }
  for (let index = 0; index < elements.length; index += 1) {
    const rect = elements[index].getBoundingClientRect();
    const midpoint = rect.left + rect.width / 2;
    if (clientX < midpoint) {
      return { insertIndex: index, overId: elements[index].dataset.tabId ?? null, midpoint };
    }
  }
  const last = elements[elements.length - 1];
  const lastRect = last.getBoundingClientRect();
  const lastMidpoint = lastRect.left + lastRect.width / 2;
  return { insertIndex: elements.length, overId: last?.dataset.tabId ?? null, midpoint: lastMidpoint };
};

const updateAutoScroll = (clientX: number) => {
  const container = tabBarRef.value;
  if (!container) {
    stopAutoScroll();
    return;
  }
  const rect = container.getBoundingClientRect();
  const distanceLeft = clientX - rect.left;
  const distanceRight = rect.right - clientX;
  let nextVelocity = 0;
  if (distanceLeft < autoScrollEdge) {
    const clamped = Math.max(0, distanceLeft);
    nextVelocity = -autoScrollMaxSpeed * (1 - clamped / autoScrollEdge);
  } else if (distanceRight < autoScrollEdge) {
    const clamped = Math.max(0, distanceRight);
    nextVelocity = autoScrollMaxSpeed * (1 - clamped / autoScrollEdge);
  }
  autoScrollVelocity = nextVelocity;
  if (autoScrollVelocity !== 0) {
    startAutoScroll();
  } else {
    stopAutoScroll();
  }
};

const updateGhostPosition = (clientX: number) => {
  const container = tabBarRef.value;
  const barRect = container?.getBoundingClientRect();
  if (!barRect) {
    return;
  }
  const scrollLeft = container?.scrollLeft ?? 0;
  dragGhostLeft.value = clientX - barRect.left - dragGrabOffsetX.value + scrollLeft;
};

const updateDragTarget = (clientX: number) => {
  if (!dragId.value) {
    return;
  }
  const target = resolveInsertTarget(clientX);
  let nextIndex = target?.insertIndex ?? null;
  dragOverId.value = target?.overId ?? null;
  if (target?.midpoint !== null && dragInsertIndex.value !== null && nextIndex !== null) {
    const distance = Math.abs(clientX - target.midpoint);
    if (distance <= snapThreshold && Math.abs(nextIndex - dragInsertIndex.value) <= 1) {
      nextIndex = dragInsertIndex.value;
    }
  }
  if (nextIndex !== null && nextIndex !== dragInsertIndex.value) {
    dragInsertIndex.value = nextIndex;
    moveTabToIndex(dragId.value, nextIndex);
  } else if (nextIndex !== null) {
    dragInsertIndex.value = nextIndex;
  }
};

const onPointerMove = (event: PointerEvent) => {
  if (!dragId.value || dragPointerId.value !== event.pointerId) {
    return;
  }
  if (!isDragging.value) {
    const distance = Math.max(Math.abs(event.clientX - dragStartX), Math.abs(event.clientY - dragStartY));
    if (distance < pointerMoveThreshold) {
      return;
    }
    isDragging.value = true;
  }

  event.preventDefault();
  updateGhostPosition(event.clientX);
  lastPointerX.value = event.clientX;
  updateAutoScroll(event.clientX);
  updateDragTarget(event.clientX);
};

const onPointerUp = (event: PointerEvent) => {
  if (dragPointerId.value !== null && event.pointerId !== dragPointerId.value) {
    return;
  }
  if (dragCaptureTarget.value?.hasPointerCapture(event.pointerId)) {
    dragCaptureTarget.value.releasePointerCapture(event.pointerId);
  }
  if (isDragging.value) {
    suppressClick.value = true;
    window.setTimeout(() => {
      suppressClick.value = false;
    }, 0);
  }
  stopAutoScroll();
  cleanupDragListeners();
  cleanupDragState();
};

const dragGhostTab = computed(() => {
  if (!dragId.value) {
    return null;
  }
  return tabs.value.find((tab) => tab.id === dragId.value) ?? null;
});

const dragGhostStyle = computed(() => {
  if (!isDragging.value || !dragGhostTab.value) {
    return undefined;
  }
  return {
    left: `${dragGhostLeft.value}px`,
    top: `${dragGhostTop.value}px`,
    width: `${dragGhostWidth.value}px`,
    height: `${dragGhostHeight.value}px`
  };
});

const handleTabClick = (id: string, event: MouseEvent) => {
  if (suppressClick.value) {
    suppressClick.value = false;
    event.preventDefault();
    event.stopPropagation();
    return;
  }
  setActive(id);
};

const handleCloseTab = async (sessionId: string) => {
  untrackSession(sessionId);
  await closeTab(sessionId);
};

const stopActivity = onActivity((sessionId) => {
  markActivity(sessionId);
});
const stopStatus = onStatusChange((payload) => {
  const previous = statusBySession.get(payload.sessionId);
  statusBySession.set(payload.sessionId, payload.status);
  if (payload.status === 'working') {
    const tab = tabs.value.find((item) => item.id === payload.sessionId);
    if (tab) {
      tab.isBlinking = false;
    }
    return;
  }
  if (payload.status === 'online' && previous === 'working') {
    if (activeId.value === payload.sessionId) {
      return;
    }
    const tab = tabs.value.find((item) => item.id === payload.sessionId);
    if (!tab) {
      return;
    }
    tab.hasActivity = true;
    tab.isBlinking = true;
  }
});
let stopOpenTab: (() => void) | null = null;
let stopReadyRequest: (() => void) | null = null;
let autoTabTimer: number | null = null;

const resolveCurrentWindow = () => {
  try {
    return getCurrentWebviewWindow();
  } catch {
    return null;
  }
};

const emitWindowReady = () => {
  const currentWindow = resolveCurrentWindow();
  if (!currentWindow) {
    return;
  }
  void emit(TERMINAL_WINDOW_READY_EVENT, { windowLabel: currentWindow.label });
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
    const currentWindow = resolveCurrentWindow();
    try {
      stopOpenTab = currentWindow
        ? await currentWindow.listen<TerminalOpenTabPayload>(TERMINAL_OPEN_TAB_EVENT, (event) => {
            const { sessionId, title, memberId, terminalType, keepAlive } = event.payload;
            const resolvedTitle = resolveTabTitle(memberId, title);
            trackSession(sessionId);
            openTab(sessionId, { title: resolvedTitle, memberId, terminalType, keepAlive });
            if (autoTabTimer !== null) {
              window.clearTimeout(autoTabTimer);
              autoTabTimer = null;
            }
          })
        : await listen<TerminalOpenTabPayload>(TERMINAL_OPEN_TAB_EVENT, (event) => {
            const { sessionId, title, memberId, terminalType, keepAlive } = event.payload;
            const resolvedTitle = resolveTabTitle(memberId, title);
            trackSession(sessionId);
            openTab(sessionId, { title: resolvedTitle, memberId, terminalType, keepAlive });
            if (autoTabTimer !== null) {
              window.clearTimeout(autoTabTimer);
              autoTabTimer = null;
            }
          });
    } catch (error) {
      console.error('Failed to listen for terminal tabs.', error);
    }

    try {
      stopReadyRequest = currentWindow
        ? await currentWindow.listen(TERMINAL_WINDOW_READY_REQUEST_EVENT, () => {
            emitWindowReady();
          })
        : await listen(TERMINAL_WINDOW_READY_REQUEST_EVENT, () => {
            emitWindowReady();
          });
    } catch (error) {
      console.error('Failed to listen for terminal ready requests.', error);
    }

    emitWindowReady();

    for (const tab of tabs.value) {
      trackSession(tab.id);
    }

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
  stopStatus();
  cleanupDragListeners();
  cleanupDragState();
  stopAutoScroll();
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

<style scoped>
.terminal-tab-move {
  transition: transform 220ms cubic-bezier(0.22, 0.61, 0.36, 1);
}

.terminal-tab--placeholder {
  opacity: 0;
  pointer-events: none;
}

.terminal-tab-ghost {
  position: absolute;
  z-index: 20;
  pointer-events: none;
  box-shadow: 0 10px 22px rgb(0 0 0 / 0.25);
  transform: scale(1.02);
  transition: none;
  will-change: transform;
}

@keyframes terminal-tab-blink {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.25;
  }
}

.terminal-tab__activity--blink {
  animation: terminal-tab-blink 0.9s ease-in-out infinite;
}
</style>
