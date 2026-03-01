<template>
  <div
    ref="rootRef"
    :class="['h-full w-full bg-[#0b0f14] relative', attachPhase !== 'idle' ? 'opacity-95' : '']"
    data-context-scope="terminal-pane"
    :data-terminal-session-id="terminalId"
  >
    <div ref="terminalRef" class="h-full w-full"></div>
    <div
      v-if="findOpen"
      class="absolute left-3 right-3 top-3 z-30 flex flex-wrap items-center gap-2 rounded-xl bg-black/70 border border-white/10 px-3 py-2 shadow-2xl backdrop-blur"
      @keydown.stop
    >
      <span class="material-symbols-outlined text-[16px] text-white/60">search</span>
      <input
        ref="findInputRef"
        v-model="findQuery"
        type="text"
        class="min-w-[120px] flex-1 w-36 bg-transparent text-xs text-white/80 placeholder-white/40 outline-none"
        :placeholder="t('terminal.findPlaceholder')"
        @input="handleFindInput"
        @keydown="handleFindKeydown"
      />
      <div class="flex items-center gap-1">
        <button
          type="button"
          class="inline-flex h-6 items-center justify-center rounded-md px-1.5 text-[11px] font-semibold"
          :class="findCaseSensitive ? 'bg-white/15 text-white' : 'text-white/50 hover:text-white hover:bg-white/10'"
          :title="t('terminal.findCaseSensitive')"
          @click="toggleFindCase"
        >
          Aa
        </button>
        <button
          type="button"
          class="inline-flex h-6 items-center justify-center rounded-md px-1.5 text-[11px] font-semibold"
          :class="findWholeWord ? 'bg-white/15 text-white' : 'text-white/50 hover:text-white hover:bg-white/10'"
          :title="t('terminal.findWholeWord')"
          @click="toggleFindWholeWord"
        >
          ab
        </button>
        <button
          type="button"
          class="inline-flex h-6 items-center justify-center rounded-md px-1.5 text-[11px] font-semibold"
          :class="findRegex ? 'bg-white/15 text-white' : 'text-white/50 hover:text-white hover:bg-white/10'"
          :title="t('terminal.findRegex')"
          @click="toggleFindRegex"
        >
          .*
        </button>
      </div>
      <span class="min-w-[64px] text-right text-xs text-white/50">
        {{ findResultsTotal > 0 ? `${findResultsIndex}/${findResultsTotal}` : t('terminal.findNoResults') }}
      </span>
      <button
        type="button"
        class="inline-flex h-6 w-6 items-center justify-center rounded-md text-white/60 hover:text-white hover:bg-white/10"
        :disabled="findResultsTotal === 0"
        @click="handleFindPrevious"
      >
        <span class="material-symbols-outlined text-[16px]">keyboard_arrow_up</span>
      </button>
      <button
        type="button"
        class="inline-flex h-6 w-6 items-center justify-center rounded-md text-white/60 hover:text-white hover:bg-white/10"
        :disabled="findResultsTotal === 0"
        @click="handleFindNext"
      >
        <span class="material-symbols-outlined text-[16px]">keyboard_arrow_down</span>
      </button>
      <button
        type="button"
        class="ml-1 inline-flex h-6 w-6 items-center justify-center rounded-md text-white/60 hover:text-white hover:bg-white/10"
        @click="closeFind"
      >
        <span class="material-symbols-outlined text-[16px]">close</span>
      </button>
    </div>
    <div
      v-if="attachPhase !== 'idle'"
      class="absolute inset-0 z-10 flex items-center justify-center bg-black/10 backdrop-blur-[1px] pointer-events-none"
    >
      <div class="px-3 py-1 rounded-lg border border-white/10 bg-black/40 text-xs text-white/70 font-medium">
        {{ attachPhase === 'reconnecting' ? 'Reconnecting...' : 'Connecting...' }}
      </div>
    </div>
    <div
      v-if="fatalError"
      class="absolute inset-0 z-20 flex items-center justify-center bg-black/40 backdrop-blur-[1px] pointer-events-none"
    >
      <div class="px-4 py-2 rounded-lg border border-white/10 bg-black/60 text-xs text-white/80 font-medium">
        Terminal crashed. Please reopen.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// 单个终端面板：负责 xterm 渲染、输出节流与输入批处理。
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { CanvasAddon } from '@xterm/addon-canvas';
import { WebglAddon } from '@xterm/addon-webgl';
import { SearchAddon } from '@xterm/addon-search';
import type { ISearchOptions } from '@xterm/addon-search';
import '@xterm/xterm/css/xterm.css';
import { readText as readClipboardText, writeText as writeClipboardText } from '@tauri-apps/plugin-clipboard-manager';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { emit, listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import {
  ackSession,
  attachSession,
  resizeSession,
  setSessionActive,
  snapshotSessionLines,
  subscribeError,
  subscribeExit,
  subscribeOutput,
  writeSession
} from './terminalBridge';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useTerminalMemberStore } from '@/features/terminal/terminalMemberStore';
import { useTerminalStore } from '@/features/terminal/terminalStore';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';
import { isFrontDebugEnabled, createFrontLogger } from '@/shared/monitoring/passiveMonitor';
import { registerContextMenuRule, unregisterContextMenuRule } from '@/shared/context-menu/registry';
import type { ContextMenuEntry } from '@/shared/context-menu/types';
import {
  TERMINAL_SNAPSHOT_REQUEST_EVENT,
  TERMINAL_SNAPSHOT_RESPONSE_EVENT,
  type TerminalSnapshotRequestPayload,
  type TerminalSnapshotResponsePayload
} from '@/features/terminal/terminalEvents';
import { useToastStore } from '@/stores/toastStore';
import type { TerminalType } from '@/shared/types/terminal';

type OutputChunk = { data: string; seq: number };
type TerminalSize = { cols: number; rows: number };
type PassiveSnapshotEntry = {
  at: number;
  frontBefore: string[];
  backendStored: string[];
  backendSeq?: number | null;
  backendError?: string | null;
  rows: number;
  cols: number;
};

const props = defineProps<{
  terminalId: string;
  active: boolean;
  focused?: boolean;
  memberId?: string;
  terminalType?: TerminalType;
  useWebgl?: boolean;
}>();
const terminalId = props.terminalId;
const memberId = props.memberId;
const { t } = useI18n();
const isFocused = computed(() => props.focused ?? props.active);
const shouldUseWebgl = computed(() => props.useWebgl !== false);

const projectStore = useProjectStore();
const terminalMemberStore = useTerminalMemberStore();
const terminalStore = useTerminalStore();
const toastStore = useToastStore();
const { pushToast } = toastStore;
const { members } = storeToRefs(projectStore);

const terminalRef = ref<HTMLDivElement | null>(null);
const rootRef = ref<HTMLDivElement | null>(null);

const fitAddon = new FitAddon();
let webglAddon: WebglAddon | null = null;
let canvasAddon: CanvasAddon | null = null;
let searchAddon: SearchAddon | null = null;
let terminal: Terminal | null = null;
let resizeObserver: ResizeObserver | null = null;
let unsubscribeOutput: (() => void) | null = null;
let unsubscribeExit: (() => void) | null = null;
let unsubscribeError: (() => void) | null = null;
let mouseUpHandler: ((event: MouseEvent) => void) | null = null;
let stopTerminalContextMenu: (() => void) | null = null;
let searchResultDisposable: { dispose: () => void } | null = null;
let fitRaf: number | null = null;
let fitRafReason: string | null = null;
let refreshRaf: number | null = null;
let refreshRafTail: number | null = null;
let outputRefreshRaf: number | null = null;
let stopSnapshotRequest: (() => void) | null = null;
// snapshotReady 用于阻止 attach 前的乱序输出，lastAppliedSeq 防止重复渲染。
let snapshotReady = false;
let lastAppliedSeq = 0;
let pendingOutput: OutputChunk[] = [];
const attachPhase = ref<'idle' | 'attaching' | 'reconnecting'>('idle');
let reconnectAttempted = false;
let lastResizeCols = 0;
let lastResizeRows = 0;
let pendingResize: TerminalSize | null = null;
let resizeSyncTimer: number | null = null;
let reattachWindowUntil = 0;
let reattachAttempted = false;
let fitToken = 0;
const fatalError = ref<string | null>(null);
const textEncoder = new TextEncoder();
let pendingAckBytes = 0;
let ackTimer: number | null = null;
let pendingInput = '';
let inputFlushTimer: number | null = null;
let inputPasteBracketed = false;
let inputPasteHeuristic = false;
let pasteHoldTimer: number | null = null;
let pasteHoldDelayTimer: number | null = null;
let pasteHoldActive = false;
let viewportResizeHandler: (() => void) | null = null;
let windowResizeHandler: (() => void) | null = null;
let visibilityFitHandler: (() => void) | null = null;
const traceLog = createFrontLogger('terminal-trace');
const findOpen = ref(false);
const findQuery = ref('');
const findInputRef = ref<HTMLInputElement | null>(null);
const findCaseSensitive = ref(false);
const findWholeWord = ref(false);
const findRegex = ref(false);
const findResultsTotal = ref(0);
const findResultsIndex = ref(0);
const findInvalidRegex = ref(false);
const rendererMode = ref<'webgl' | 'canvas'>('webgl');
let webglCooldownUntil = 0;
let webglCooldownTimer: number | null = null;

let beepTraceRunId: string | null = null;
let beepTraceWindowLabel: string | null = null;
let beepTraceRefCount = 0;
let beepTraceKeySeq = 0;

const isBeepTraceEnabled = () => {
  return isFrontDebugEnabled();
};

const isBeepTraceVerbose = () => {
  return isFrontDebugEnabled();
};

const safeInvokeBeepTrace = async (command: string, payload: Record<string, unknown>) => {
  if (!isTauri()) {
    return;
  }
  try {
    await invoke(command, payload);
  } catch {
    // 诊断日志失败不阻断终端交互。
  }
};

const resolveBeepTraceWindowLabel = () => {
  try {
    return getCurrentWebviewWindow().label ?? null;
  } catch {
    return null;
  }
};

const logBeepTrace = async (stepId: string, payload: Record<string, unknown>) => {
  if (!beepTraceRunId) {
    return;
  }
  await safeInvokeBeepTrace('diagnostics_log_frontend_event', {
    runId: beepTraceRunId,
    stepId,
    payload
  });
};

const ensureBeepTraceRun = async () => {
  if (!isBeepTraceEnabled() || beepTraceRunId) {
    return;
  }
  const runId = `beep-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  beepTraceRunId = runId;
  beepTraceWindowLabel = resolveBeepTraceWindowLabel();
  await safeInvokeBeepTrace('diagnostics_start_run', {
    runId,
    label: 'terminal-beep'
  });
  if (beepTraceWindowLabel) {
    await safeInvokeBeepTrace('diagnostics_register_window', {
      runId,
      windowLabel: beepTraceWindowLabel
    });
  }
  await logBeepTrace('beep-trace.start', {
    windowLabel: beepTraceWindowLabel
  });
};

const endBeepTraceRun = async (status: string) => {
  if (!beepTraceRunId) {
    return;
  }
  await safeInvokeBeepTrace('diagnostics_end_run', {
    runId: beepTraceRunId,
    status
  });
  beepTraceRunId = null;
  beepTraceWindowLabel = null;
};

const countSnapshotLines = (data: string) => {
  if (!data) {
    return 0;
  }
  return data.split('\r\n').length;
};

// 诊断请求只取当前视口文本，保留原始字符以便比对链路数据。
const collectViewportLines = () => {
  if (!terminal) {
    return null;
  }
  const buffer = terminal.buffer.active;
  const start = buffer.viewportY;
  const rows = terminal.rows;
  const cols = terminal.cols;
  const lines: string[] = [];
  for (let index = 0; index < rows; index += 1) {
    const line = buffer.getLine(start + index);
    const text = line ? line.translateToString(true) : '';
    lines.push(text);
  }
  return { lines, rows, cols };
};

const collectXtermMeta = () => {
  if (!terminal) {
    return null;
  }
  const buffer = terminal.buffer.active as typeof terminal.buffer.active & { cursorX?: number; cursorY?: number };
  return {
    cols: terminal.cols,
    rows: terminal.rows,
    scrollback: terminal.options.scrollback,
    viewportY: buffer.viewportY,
    baseY: buffer.baseY,
    bufferLength: buffer.length,
    cursorX: typeof buffer.cursorX === 'number' ? buffer.cursorX : null,
    cursorY: typeof buffer.cursorY === 'number' ? buffer.cursorY : null,
    fontFamily: terminal.options.fontFamily ?? null,
    fontSize: terminal.options.fontSize ?? null,
    webglEnabled: Boolean(webglAddon),
    devicePixelRatio: window.devicePixelRatio
  };
};

const summarizeLines = (lines: string[]) => {
  let nonEmptyCount = 0;
  let firstNonEmpty: number | null = null;
  let lastNonEmpty: number | null = null;
  let maxLineLength = 0;
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (line.length > maxLineLength) {
      maxLineLength = line.length;
    }
    if (line.trim().length > 0) {
      nonEmptyCount += 1;
      if (firstNonEmpty === null) {
        firstNonEmpty = index;
      }
      lastNonEmpty = index;
    }
  }
  return {
    lineCount: lines.length,
    nonEmptyCount,
    firstNonEmpty,
    lastNonEmpty,
    maxLineLength
  };
};

const summarizeViewportLines = () => {
  const viewport = collectViewportLines();
  if (!viewport) {
    return null;
  }
  return summarizeLines(viewport.lines);
};

const PASSIVE_SNAPSHOT_STORAGE_KEY = 'terminal-passive-snapshot-cache';
// 缓存仅用于一次性 reopen 比对，避免长期堆积占用。
const PASSIVE_SNAPSHOT_MAX_AGE_MS = 10 * 60 * 1000;

const isPassiveSnapshotEnabled = () => {
  return isFrontDebugEnabled();
};

const readPassiveSnapshotCache = () => {
  if (typeof window === 'undefined') {
    return {};
  }
  const raw = window.localStorage.getItem(PASSIVE_SNAPSHOT_STORAGE_KEY);
  if (!raw) {
    return {};
  }
  try {
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== 'object') {
      return {};
    }
    return parsed as Record<string, PassiveSnapshotEntry>;
  } catch {
    return {};
  }
};

const writePassiveSnapshotCache = (cache: Record<string, PassiveSnapshotEntry>) => {
  if (typeof window === 'undefined') {
    return;
  }
  try {
    window.localStorage.setItem(PASSIVE_SNAPSHOT_STORAGE_KEY, JSON.stringify(cache));
  } catch {
    // 忽略缓存写入失败，避免影响正常交互。
  }
};

const prunePassiveSnapshotCache = (cache: Record<string, PassiveSnapshotEntry>) => {
  const now = Date.now();
  let changed = false;
  for (const [key, entry] of Object.entries(cache)) {
    if (!entry || typeof entry.at !== 'number' || now - entry.at > PASSIVE_SNAPSHOT_MAX_AGE_MS) {
      delete cache[key];
      changed = true;
    }
  }
  return changed;
};

const resolvePassiveSnapshotEntry = (cache: Record<string, PassiveSnapshotEntry>, key: string) => {
  const entry = cache[key];
  if (!entry) {
    return null;
  }
  if (typeof entry.at !== 'number') {
    return null;
  }
  if (!Array.isArray(entry.frontBefore) || !Array.isArray(entry.backendStored)) {
    return null;
  }
  return entry;
};

const storePassiveSnapshot = async (reason: string) => {
  if (!isPassiveSnapshotEnabled() || !terminal || !snapshotReady) {
    return;
  }
  const viewport = collectViewportLines();
  if (!viewport) {
    return;
  }
  let backendStored: string[] = [];
  let backendSeq: number | null = null;
  let backendError: string | null = null;
  try {
    const snapshot = await snapshotSessionLines(terminalId);
    backendStored = snapshot.lines ?? [];
    backendSeq = typeof snapshot.seq === 'number' ? snapshot.seq : null;
  } catch (error) {
    backendError = error instanceof Error ? error.message : String(error);
  }
  const cache = readPassiveSnapshotCache();
  prunePassiveSnapshotCache(cache);
  cache[terminalId] = {
    at: Date.now(),
    frontBefore: viewport.lines,
    backendStored,
    backendSeq,
    backendError,
    rows: viewport.rows,
    cols: viewport.cols
  };
  writePassiveSnapshotCache(cache);
  void logDiagnosticsEvent('passive-snapshot.store', {
    terminalId,
    memberId,
    terminalType: props.terminalType ?? null,
    reason,
    frontSummary: summarizeLines(viewport.lines),
    backendSummary: summarizeLines(backendStored),
    backendSeq,
    backendError,
    rows: viewport.rows,
    cols: viewport.cols
  });
};

const maybeLogPassiveSnapshotTriplet = async (reason: string) => {
  if (!isPassiveSnapshotEnabled()) {
    return;
  }
  const viewport = collectViewportLines();
  if (!viewport) {
    return;
  }
  const cache = readPassiveSnapshotCache();
  if (prunePassiveSnapshotCache(cache)) {
    writePassiveSnapshotCache(cache);
  }
  const entry = resolvePassiveSnapshotEntry(cache, terminalId);
  if (!entry) {
    return;
  }
  const ageMs = Date.now() - entry.at;
  if (ageMs > PASSIVE_SNAPSHOT_MAX_AGE_MS) {
    delete cache[terminalId];
    writePassiveSnapshotCache(cache);
    return;
  }
  delete cache[terminalId];
  writePassiveSnapshotCache(cache);
  void logDiagnosticsEvent('passive-snapshot.triplet', {
    terminalId,
    memberId,
    terminalType: props.terminalType ?? null,
    reason,
    ageMs,
    frontBefore: entry.frontBefore,
    backendStored: entry.backendStored,
    frontReopen: viewport.lines,
    backendSeq: entry.backendSeq ?? null,
    backendError: entry.backendError ?? null,
    frontBeforeSummary: summarizeLines(entry.frontBefore),
    backendStoredSummary: summarizeLines(entry.backendStored),
    frontReopenSummary: summarizeLines(viewport.lines),
    rows: viewport.rows,
    cols: viewport.cols
  });
};

const emitSnapshotResponse = (payload: TerminalSnapshotResponsePayload) =>
  emit(TERMINAL_SNAPSHOT_RESPONSE_EVENT, payload).catch(() => {});

const respondSnapshotRequest = async (payload: TerminalSnapshotRequestPayload) => {
  if (payload.terminalId !== terminalId) {
    return;
  }
  if (!terminal) {
    emitSnapshotResponse({
      requestId: payload.requestId,
      terminalId,
      error: 'terminal not ready'
    });
    return;
  }
  if (!snapshotReady) {
    emitSnapshotResponse({
      requestId: payload.requestId,
      terminalId,
      error: 'snapshot not ready'
    });
    return;
  }
  const viewport = collectViewportLines();
  if (!viewport) {
    emitSnapshotResponse({
      requestId: payload.requestId,
      terminalId,
      error: 'snapshot unavailable'
    });
    return;
  }
  emitSnapshotResponse({
    requestId: payload.requestId,
    terminalId,
    lines: viewport.lines,
    rows: viewport.rows,
    cols: viewport.cols
  });
};

const startSnapshotListener = async () => {
  if (stopSnapshotRequest) {
    return;
  }
  stopSnapshotRequest = await listen<TerminalSnapshotRequestPayload>(TERMINAL_SNAPSHOT_REQUEST_EVENT, (event) => {
    void respondSnapshotRequest(event.payload);
  });
};
const debugLog = createFrontLogger('terminal');

// 输出/ACK/输入的批处理参数，平衡吞吐与交互延迟。
const PENDING_OUTPUT_LIMIT = 2000;
const ACK_BATCH_SIZE = 5000;
const ACK_FLUSH_MS = 50;
const INPUT_BATCH_SIZE = 1024;
const INPUT_FLUSH_MS = 8;
const INPUT_PASTE_THRESHOLD = 32;
const INPUT_IMMEDIATE_MAX_LEN = 4;
// 长按粘贴做节流，避免一次性刷屏或卡顿。
const PASTE_HOLD_DELAY_MS = 240;
const PASTE_HOLD_INTERVAL_MS = 120;
// 首次打开时需要等待字体/WebGL就绪的尺寸抖动收敛，避免 attach 后立刻 reflow。
const STABLE_FIT_TIMEOUT_MS = 400;
const STABLE_FIT_REPEAT_COUNT = 2;
const REATTACH_WINDOW_MS = 300;
// 拖拽窗口时避免高频 IPC，把尺寸同步到后端做节流。
const RESIZE_SYNC_DEBOUNCE_MS = 120;
// WebGL 失效后冷却一段时间再允许恢复，避免频繁抖动。
const WEBGL_COOLDOWN_MS = 15000;
// Canvas 渲染在超宽列下容易卡顿，限制最大列宽避免性能崩溃。
const MAX_CANVAS_COLS = 240;
// 清理类 ANSI 会改变屏幕/光标但不一定触发完整重绘，命中后强制刷新。
const CLEANUP_SEQUENCE_PATTERN = /\x1b\[[0-9;]*[JKLMPX]/;

const isMac = navigator.platform.toLowerCase().includes('mac');
let lastCopiedSelection = '';
let beepTraceAttached = false;
let beepTracePointerHandler: ((event: PointerEvent) => void) | null = null;
let beepTraceKeydownHandler: ((event: KeyboardEvent) => void) | null = null;
let beepTraceKeyupHandler: ((event: KeyboardEvent) => void) | null = null;
let beepTraceFocusInHandler: ((event: FocusEvent) => void) | null = null;
let beepTraceFocusOutHandler: ((event: FocusEvent) => void) | null = null;
let beepTraceWindowFocusHandler: (() => void) | null = null;
let beepTraceWindowBlurHandler: (() => void) | null = null;
let beepTraceVisibilityHandler: (() => void) | null = null;
let beepTraceTextareaFocusHandler: (() => void) | null = null;
let beepTraceTextareaBlurHandler: (() => void) | null = null;
let beepTraceCompositionStartHandler: ((event: CompositionEvent) => void) | null = null;
let beepTraceCompositionEndHandler: ((event: CompositionEvent) => void) | null = null;

const hasCleanupSequence = (data: string) => {
  if (!data || !data.includes('\x1b[')) {
    return false;
  }
  return CLEANUP_SEQUENCE_PATTERN.test(data);
};

const copySelection = (options?: { force?: boolean; clear?: boolean }) => {
  if (!terminal || !terminal.hasSelection()) {
    return;
  }
  const selection = terminal.getSelection();
  if (!selection || selection === lastCopiedSelection) {
    if (options?.force && selection) {
      void writeClipboardText(selection).catch(() => {});
      if (options?.clear) {
        terminal.clearSelection();
        terminal.focus();
        lastCopiedSelection = '';
      }
    }
    return;
  }
  lastCopiedSelection = selection;
  void writeClipboardText(selection).catch(() => {});
  if (options?.clear) {
    terminal.clearSelection();
    terminal.focus();
    lastCopiedSelection = '';
  }
};

const pasteClipboardText = async (options?: { force?: boolean; focus?: boolean }) => {
  if (!terminal || fatalError.value) {
    return;
  }
  if (!isFocused.value) {
    if (options?.focus) {
      terminalStore.setActive(terminalId);
      await nextTick();
    } else if (!options?.force) {
      return;
    }
  }
  try {
    const text = await readClipboardText();
    if (!text) {
      return;
    }
    if (pendingInput) {
      flushPendingInput('paste-buffer');
    }
    void writeSession(terminalId, text).catch((error) => {
      terminal?.writeln(`\r\n[terminal error] ${String(error)}`);
    });
    if (options?.focus) {
      terminal?.focus();
    }
  } catch {
    // 忽略读取剪贴板失败，避免在 UI 中产生噪声错误。
  }
};

const stopPasteHold = () => {
  pasteHoldActive = false;
  if (pasteHoldDelayTimer !== null) {
    window.clearTimeout(pasteHoldDelayTimer);
    pasteHoldDelayTimer = null;
  }
  if (pasteHoldTimer !== null) {
    window.clearInterval(pasteHoldTimer);
    pasteHoldTimer = null;
  }
};

const startPasteHold = () => {
  if (pasteHoldActive || !terminal || !isFocused.value || fatalError.value) {
    return;
  }
  pasteHoldActive = true;
  void pasteClipboardText();
  pasteHoldDelayTimer = window.setTimeout(() => {
    pasteHoldDelayTimer = null;
    if (!pasteHoldActive) {
      return;
    }
    pasteHoldTimer = window.setInterval(() => {
      if (!pasteHoldActive) {
        return;
      }
      void pasteClipboardText();
    }, PASTE_HOLD_INTERVAL_MS);
  }, PASTE_HOLD_DELAY_MS);
};

const attachClipboardHandlers = (root: HTMLElement) => {
  if (!terminal) {
    return;
  }
  terminal.attachCustomKeyEventHandler((event) => {
    const ctrlKey = isMac ? event.metaKey : event.ctrlKey;
    const key = event.key.toLowerCase();

    if (event.type === 'keyup' && (key === 'v' || key === 'control' || key === 'meta')) {
      stopPasteHold();
    }
    if (ctrlKey && key === 'c') {
      if (terminal.hasSelection()) {
        copySelection({ force: true, clear: true });
        return false;
      }
      return true;
    }
    if (ctrlKey && key === 'v') {
      if (event.type === 'keydown') {
        startPasteHold();
      }
      return false;
    }
    if (ctrlKey && key === 'f') {
      if (!isFocused.value || fatalError.value) {
        return false;
      }
      openFind();
      return false;
    }
    if (!isMac && event.shiftKey && key === 'insert') {
      void pasteClipboardText();
      return false;
    }
    return true;
  });

  mouseUpHandler = (event) => {
    if (event.button === 0) {
      copySelection();
    }
  };
  root.addEventListener('mouseup', mouseUpHandler);
};

const searchDecorations = {
  matchBackground: '#1d4ed8',
  matchBorder: '#38bdf8',
  matchOverviewRuler: '#1d4ed8',
  activeMatchBackground: '#f59e0b',
  activeMatchBorder: '#fbbf24',
  activeMatchColorOverviewRuler: '#f59e0b'
};

const clearFindHighlights = () => {
  searchAddon?.clearDecorations();
};

const refreshFindResults = () => {
  findInvalidRegex.value = false;
  const query = findQuery.value.trim();
  if (!query || !terminal) {
    findResultsTotal.value = 0;
    findResultsIndex.value = 0;
    clearFindHighlights();
    return;
  }
  if (findRegex.value) {
    try {
      new RegExp(query);
    } catch {
      findInvalidRegex.value = true;
      findResultsTotal.value = 0;
      findResultsIndex.value = 0;
      clearFindHighlights();
    }
  }
};

const runFind = (direction: 'next' | 'previous', incremental: boolean) => {
  const query = findQuery.value.trim();
  if (!searchAddon || !query || findInvalidRegex.value) {
    return false;
  }
  const options: ISearchOptions = {
    caseSensitive: findCaseSensitive.value,
    regex: findRegex.value,
    wholeWord: findWholeWord.value,
    incremental,
    decorations: searchDecorations
  };
  if (direction === 'previous') {
    return searchAddon.findPrevious(query, options);
  }
  return searchAddon.findNext(query, options);
};

const openFind = () => {
  if (!terminal) {
    return;
  }
  if (!findOpen.value) {
    const selection = terminal.hasSelection() ? terminal.getSelection() : '';
    if (selection) {
      findQuery.value = selection;
    }
  }
  findOpen.value = true;
  void nextTick(() => {
    findInputRef.value?.focus();
    findInputRef.value?.select();
  });
  refreshFindResults();
  if (findQuery.value.trim()) {
    runFind('next', true);
  }
};

const closeFind = () => {
  findOpen.value = false;
  clearFindHighlights();
  terminal?.focus();
};

const handleFindInput = () => {
  refreshFindResults();
  if (findQuery.value.trim()) {
    runFind('next', true);
  }
};

const isEnterKey = (event: KeyboardEvent) => event.key === 'Enter' || event.code === 'NumpadEnter';

const handleFindKeydown = (event: KeyboardEvent) => {
  if (isEnterKey(event)) {
    event.preventDefault();
    if (event.shiftKey) {
      handleFindPrevious();
      return;
    }
    handleFindNext();
    return;
  }
  if (event.key === 'Escape') {
    event.preventDefault();
    closeFind();
  }
};

const handleFindPrevious = () => {
  refreshFindResults();
  if (findResultsTotal.value <= 0) {
    return;
  }
  runFind('previous', false);
};

const handleFindNext = () => {
  refreshFindResults();
  if (findResultsTotal.value <= 0) {
    return;
  }
  runFind('next', false);
};

const toggleFindCase = () => {
  findCaseSensitive.value = !findCaseSensitive.value;
  refreshFindResults();
  runFind('next', true);
};

const toggleFindWholeWord = () => {
  findWholeWord.value = !findWholeWord.value;
  refreshFindResults();
  runFind('next', true);
};

const toggleFindRegex = () => {
  findRegex.value = !findRegex.value;
  refreshFindResults();
  runFind('next', true);
};

const registerTerminalContextMenu = () => {
  const ruleId = `terminal-pane-${terminalId}`;
  registerContextMenuRule({
    id: ruleId,
    order: 30,
    mode: 'override',
    matches: (context) => {
      if (!context.scopeChain.includes('terminal-pane')) {
        return false;
      }
      const target = context.targetElement?.closest(`[data-terminal-session-id="${terminalId}"]`);
      return Boolean(target);
    },
    items: (): ContextMenuEntry[] => {
      const hasSelection = Boolean(terminal?.hasSelection());
      return [
        {
          kind: 'item',
          id: 'terminal-copy',
          labelKey: 'contextMenu.copy',
          icon: 'content_copy',
          enabled: hasSelection,
          action: () => copySelection({ force: true })
        },
        {
          kind: 'item',
          id: 'terminal-paste',
          labelKey: 'contextMenu.paste',
          icon: 'content_paste',
          enabled: !fatalError.value,
          action: async () => {
            await pasteClipboardText({ force: true, focus: true });
          }
        },
        {
          kind: 'item',
          id: 'terminal-select-all',
          labelKey: 'contextMenu.selectAll',
          icon: 'select_all',
          enabled: Boolean(terminal),
          action: () => {
            terminal?.selectAll();
          }
        },
        { kind: 'separator' },
        {
          kind: 'item',
          id: 'terminal-find',
          labelKey: 'terminal.contextMenu.find',
          icon: 'search',
          enabled: Boolean(terminal),
          action: () => {
            openFind();
          }
        }
      ];
    }
  });
  return () => unregisterContextMenuRule(ruleId);
};

const describeElement = (element: Element | null) => {
  if (!element || !(element instanceof HTMLElement)) {
    return null;
  }
  const className = typeof element.className === 'string' ? element.className.trim() : '';
  const trimmedClassName = className.length > 120 ? `${className.slice(0, 120)}...` : className;
  return {
    tag: element.tagName,
    id: element.id || null,
    className: trimmedClassName || null,
    role: element.getAttribute('role') ?? null,
    ariaLabel: element.getAttribute('aria-label') ?? null,
    tabIndex: element.tabIndex
  };
};

const describeEventPath = (event: Event) => {
  if (!isBeepTraceVerbose() || typeof event.composedPath !== 'function') {
    return null;
  }
  const rawPath = event.composedPath();
  const elements = rawPath
    .filter((entry): entry is Element => entry instanceof Element)
    .slice(0, 6)
    .map((entry) => describeElement(entry))
    .filter((entry): entry is NonNullable<typeof entry> => Boolean(entry));
  return elements.length > 0 ? elements : null;
};

const describeTextarea = (textarea?: HTMLTextAreaElement) => {
  if (!textarea) {
    return null;
  }
  return {
    focused: document.activeElement === textarea,
    readOnly: textarea.readOnly,
    disabled: textarea.disabled,
    valueLen: textarea.value.length,
    tabIndex: textarea.tabIndex
  };
};

const collectFocusSnapshot = (extra?: Record<string, unknown>) => ({
  terminalId,
  memberId,
  terminalType: props.terminalType ?? null,
  active: props.active,
  focused: isFocused.value,
  attachPhase: attachPhase.value,
  snapshotReady,
  disableStdin: terminal?.options.disableStdin ?? null,
  windowHasFocus: typeof document.hasFocus === 'function' ? document.hasFocus() : null,
  visibility: document.visibilityState,
  activeElement: describeElement(document.activeElement),
  terminalTextArea: describeTextarea(terminal?.textarea),
  windowLabel: beepTraceWindowLabel,
  ...extra
});

const classifyKey = (key: string) => {
  if (key.length === 1) {
    if (/[a-zA-Z]/.test(key)) {
      return 'alpha';
    }
    if (/[0-9]/.test(key)) {
      return 'digit';
    }
    return 'symbol';
  }
  return 'control';
};

const classifyInputChunk = (data: string) => {
  if (!data) {
    return 'empty';
  }
  if (data.length === 1) {
    const code = data.charCodeAt(0);
    if (code === 8 || code === 127) {
      return 'backspace';
    }
    if (code === 9) {
      return 'tab';
    }
    if (code === 10 || code === 13) {
      return 'enter';
    }
    if (code === 27) {
      return 'escape';
    }
    if (code >= 32 && code <= 126) {
      return 'printable';
    }
    return 'non-ascii';
  }
  if (data.includes('\x1b')) {
    return 'escape-seq';
  }
  if (data.includes('\n') || data.includes('\r')) {
    return 'multi-line';
  }
  return 'bulk';
};

const shouldLogKeyEvent = (event: KeyboardEvent) => {
  if (isBeepTraceVerbose()) {
    return true;
  }
  const textarea = terminal?.textarea ?? null;
  const target = event.target instanceof Element ? event.target : null;
  const inTextarea = Boolean(textarea && target === textarea);
  const hasFocus = typeof document.hasFocus === 'function' ? document.hasFocus() : null;
  return !isFocused.value || terminal?.options.disableStdin || hasFocus === false || !inTextarea;
};

const attachBeepTraceListeners = () => {
  if (beepTraceAttached || !rootRef.value) {
    return;
  }
  beepTraceAttached = true;
  const rootNode = rootRef.value;
  beepTracePointerHandler = (event) => {
    if (!beepTraceRunId) {
      return;
    }
    const target = event.target instanceof Element ? event.target : null;
    const inTerminal = Boolean(target && rootNode.contains(target));
    void logBeepTrace(
      'beep.pointerdown',
      collectFocusSnapshot({
        pointerType: event.pointerType,
        button: event.button,
        inTerminal,
        target: describeElement(target),
        path: describeEventPath(event)
      })
    );
  };
  document.addEventListener('pointerdown', beepTracePointerHandler, true);
  beepTraceKeydownHandler = (event) => {
    if (!beepTraceRunId || !shouldLogKeyEvent(event)) {
      return;
    }
    const target = event.target instanceof Element ? event.target : null;
    const textarea = terminal?.textarea ?? null;
    const inTextarea = Boolean(textarea && target === textarea);
    const hasFocus = typeof document.hasFocus === 'function' ? document.hasFocus() : null;
    const seq = (beepTraceKeySeq += 1);
    void logBeepTrace(
      'beep.keydown',
      collectFocusSnapshot({
        seq,
        keyKind: classifyKey(event.key),
        keyLen: event.key.length,
        code: event.code,
        repeat: event.repeat,
        altKey: event.altKey,
        ctrlKey: event.ctrlKey,
        shiftKey: event.shiftKey,
        metaKey: event.metaKey,
        isComposing: event.isComposing,
        defaultPrevented: event.defaultPrevented,
        cancelable: event.cancelable,
        target: describeElement(target),
        path: describeEventPath(event),
        suspect: !isFocused.value || terminal?.options.disableStdin || hasFocus === false || !inTextarea
      })
    );
  };
  window.addEventListener('keydown', beepTraceKeydownHandler, true);
  beepTraceKeyupHandler = (event) => {
    if (!beepTraceRunId || !shouldLogKeyEvent(event)) {
      return;
    }
    const target = event.target instanceof Element ? event.target : null;
    void logBeepTrace(
      'beep.keyup',
      collectFocusSnapshot({
        keyKind: classifyKey(event.key),
        keyLen: event.key.length,
        code: event.code,
        repeat: event.repeat,
        altKey: event.altKey,
        ctrlKey: event.ctrlKey,
        shiftKey: event.shiftKey,
        metaKey: event.metaKey,
        isComposing: event.isComposing,
        defaultPrevented: event.defaultPrevented,
        cancelable: event.cancelable,
        target: describeElement(target),
        path: describeEventPath(event)
      })
    );
  };
  window.addEventListener('keyup', beepTraceKeyupHandler, true);
  beepTraceFocusInHandler = (event) => {
    if (!beepTraceRunId) {
      return;
    }
    void logBeepTrace(
      'beep.focusin',
      collectFocusSnapshot({
        target: describeElement(event.target instanceof Element ? event.target : null),
        path: describeEventPath(event)
      })
    );
  };
  document.addEventListener('focusin', beepTraceFocusInHandler, true);
  beepTraceFocusOutHandler = (event) => {
    if (!beepTraceRunId) {
      return;
    }
    void logBeepTrace(
      'beep.focusout',
      collectFocusSnapshot({
        target: describeElement(event.target instanceof Element ? event.target : null),
        path: describeEventPath(event)
      })
    );
  };
  document.addEventListener('focusout', beepTraceFocusOutHandler, true);
  beepTraceWindowFocusHandler = () => {
    if (!beepTraceRunId) {
      return;
    }
    void logBeepTrace('beep.window-focus', collectFocusSnapshot());
  };
  beepTraceWindowBlurHandler = () => {
    if (!beepTraceRunId) {
      return;
    }
    void logBeepTrace('beep.window-blur', collectFocusSnapshot());
  };
  beepTraceVisibilityHandler = () => {
    if (!beepTraceRunId) {
      return;
    }
    void logBeepTrace('beep.visibility', collectFocusSnapshot());
  };
  window.addEventListener('focus', beepTraceWindowFocusHandler);
  window.addEventListener('blur', beepTraceWindowBlurHandler);
  document.addEventListener('visibilitychange', beepTraceVisibilityHandler);
  beepTraceCompositionStartHandler = (event) => {
    if (!beepTraceRunId) {
      return;
    }
    void logBeepTrace(
      'beep.composition-start',
      collectFocusSnapshot({
        dataLen: event.data?.length ?? 0,
        target: describeElement(event.target instanceof Element ? event.target : null),
        path: describeEventPath(event)
      })
    );
  };
  beepTraceCompositionEndHandler = (event) => {
    if (!beepTraceRunId) {
      return;
    }
    void logBeepTrace(
      'beep.composition-end',
      collectFocusSnapshot({
        dataLen: event.data?.length ?? 0,
        target: describeElement(event.target instanceof Element ? event.target : null),
        path: describeEventPath(event)
      })
    );
  };
  window.addEventListener('compositionstart', beepTraceCompositionStartHandler, true);
  window.addEventListener('compositionend', beepTraceCompositionEndHandler, true);
  const textarea = terminal?.textarea;
  if (textarea) {
    beepTraceTextareaFocusHandler = () => {
      if (!beepTraceRunId) {
        return;
      }
      void logBeepTrace('beep.textarea-focus', collectFocusSnapshot());
    };
    beepTraceTextareaBlurHandler = () => {
      if (!beepTraceRunId) {
        return;
      }
      void logBeepTrace('beep.textarea-blur', collectFocusSnapshot());
    };
    textarea.addEventListener('focus', beepTraceTextareaFocusHandler, true);
    textarea.addEventListener('blur', beepTraceTextareaBlurHandler, true);
  }
};

const detachBeepTraceListeners = () => {
  if (!beepTraceAttached) {
    return;
  }
  beepTraceAttached = false;
  if (beepTracePointerHandler) {
    document.removeEventListener('pointerdown', beepTracePointerHandler, true);
  }
  if (beepTraceKeydownHandler) {
    window.removeEventListener('keydown', beepTraceKeydownHandler, true);
  }
  if (beepTraceKeyupHandler) {
    window.removeEventListener('keyup', beepTraceKeyupHandler, true);
  }
  if (beepTraceFocusInHandler) {
    document.removeEventListener('focusin', beepTraceFocusInHandler, true);
  }
  if (beepTraceFocusOutHandler) {
    document.removeEventListener('focusout', beepTraceFocusOutHandler, true);
  }
  if (beepTraceWindowFocusHandler) {
    window.removeEventListener('focus', beepTraceWindowFocusHandler);
  }
  if (beepTraceWindowBlurHandler) {
    window.removeEventListener('blur', beepTraceWindowBlurHandler);
  }
  if (beepTraceVisibilityHandler) {
    document.removeEventListener('visibilitychange', beepTraceVisibilityHandler);
  }
  if (beepTraceCompositionStartHandler) {
    window.removeEventListener('compositionstart', beepTraceCompositionStartHandler, true);
  }
  if (beepTraceCompositionEndHandler) {
    window.removeEventListener('compositionend', beepTraceCompositionEndHandler, true);
  }
  const textarea = terminal?.textarea;
  if (textarea && beepTraceTextareaFocusHandler) {
    textarea.removeEventListener('focus', beepTraceTextareaFocusHandler, true);
  }
  if (textarea && beepTraceTextareaBlurHandler) {
    textarea.removeEventListener('blur', beepTraceTextareaBlurHandler, true);
  }
  beepTracePointerHandler = null;
  beepTraceKeydownHandler = null;
  beepTraceKeyupHandler = null;
  beepTraceFocusInHandler = null;
  beepTraceFocusOutHandler = null;
  beepTraceWindowFocusHandler = null;
  beepTraceWindowBlurHandler = null;
  beepTraceVisibilityHandler = null;
  beepTraceTextareaFocusHandler = null;
  beepTraceTextareaBlurHandler = null;
  beepTraceCompositionStartHandler = null;
  beepTraceCompositionEndHandler = null;
};

// 终端本地即时适配，后端尺寸同步做节流，保证拖拽不卡顿。
const scheduleBackendResize = (reason?: string) => {
  if (!pendingResize) {
    return;
  }
  if (!snapshotReady || attachPhase.value !== 'idle') {
    return;
  }
  resizeSyncTimer = window.setTimeout(() => {
    resizeSyncTimer = null;
    if (!pendingResize) {
      return;
    }
    if (!snapshotReady || attachPhase.value !== 'idle') {
      return;
    }
    const { cols, rows } = pendingResize;
    pendingResize = null;
    if (cols <= 0 || rows <= 0) {
      return;
    }
    if (cols === lastResizeCols && rows === lastResizeRows) {
      return;
    }
    lastResizeCols = cols;
    lastResizeRows = rows;
    void resizeSession(terminalId, cols, rows).catch(() => {});
    void scheduleReattachAfterResize(reason, { cols, rows });
  }, RESIZE_SYNC_DEBOUNCE_MS);
};

// 合并多来源的 fit 触发，避免拖拽时过度调用。
const requestFit = (reason?: string) => {
  fitRafReason = reason ?? fitRafReason;
  if (fitRaf !== null) {
    return;
  }
  fitRaf = window.requestAnimationFrame(() => {
    fitRaf = null;
    const currentReason = fitRafReason ?? 'raf';
    fitRafReason = null;
    fitTerminal(currentReason);
  });
};

const fitTerminal = (reason?: string): TerminalSize | null => {
  if (!terminal) {
    return null;
  }
  const beforeMeta = collectXtermMeta();
  const beforeViewport = summarizeViewportLines();
  fitAddon.fit();
  let afterMeta = collectXtermMeta();
  let afterViewport = summarizeViewportLines();
  let canvasClamp: { beforeCols: number; afterCols: number } | null = null;
  if (afterMeta && rendererMode.value === 'canvas' && afterMeta.cols > MAX_CANVAS_COLS) {
    const clampedCols = MAX_CANVAS_COLS;
    terminal.resize(clampedCols, afterMeta.rows);
    canvasClamp = { beforeCols: afterMeta.cols, afterCols: clampedCols };
    afterMeta = collectXtermMeta();
    afterViewport = summarizeViewportLines();
  }
  if (
    beforeMeta &&
    afterMeta &&
    (beforeMeta.cols !== afterMeta.cols || beforeMeta.rows !== afterMeta.rows)
  ) {
    void logDiagnosticsEvent('terminal-resize-request', {
      terminalId,
      memberId,
      terminalType: props.terminalType,
      reason: reason ?? null,
      before: beforeMeta,
      after: afterMeta,
      canvasClamp,
      viewportBefore: beforeViewport,
      viewportAfter: afterViewport
    });
  }
  if (afterMeta && afterMeta.cols > 0 && afterMeta.rows > 0) {
    pendingResize = { cols: afterMeta.cols, rows: afterMeta.rows };
    scheduleBackendResize(reason);
  }
  return afterMeta ? { cols: afterMeta.cols, rows: afterMeta.rows } : null;
};

const waitForStableFit = async (reason: string) => {
  if (!terminal) {
    return null;
  }
  const token = ++fitToken;
  const deadline = performance.now() + STABLE_FIT_TIMEOUT_MS;
  let last = fitTerminal(reason);
  let stableCount = 0;
  while (performance.now() < deadline) {
    if (fitToken !== token) {
      return last;
    }
    await new Promise<void>((resolve) => window.requestAnimationFrame(() => resolve()));
    const current = fitTerminal(reason);
    if (!current || current.cols <= 0 || current.rows <= 0) {
      continue;
    }
    if (last && last.cols === current.cols && last.rows === current.rows) {
      stableCount += 1;
      if (stableCount >= STABLE_FIT_REPEAT_COUNT) {
        return current;
      }
    } else {
      last = current;
      stableCount = 0;
    }
  }
  return last;
};

const scheduleReattachAfterResize = async (reason: string | undefined, size: TerminalSize) => {
  if (!snapshotReady || reattachAttempted || fatalError.value) {
    return;
  }
  if (attachPhase.value !== 'idle') {
    return;
  }
  if (performance.now() > reattachWindowUntil) {
    return;
  }
  reattachAttempted = true;
  snapshotReady = false;
  pendingOutput = [];
  lastAppliedSeq = 0;
  void logDiagnosticsEvent('terminal-reattach', {
    terminalId,
    memberId,
    terminalType: props.terminalType,
    reason: reason ?? null,
    cols: size.cols,
    rows: size.rows,
    windowUntil: reattachWindowUntil
  });
  await waitForStableFit('reattach');
  await attachWithRecovery();
};

const scheduleRendererRefresh = (_reason: string) => {
  if (!terminal) {
    return;
  }
  if (refreshRaf !== null || refreshRafTail !== null) {
    return;
  }
  refreshRaf = window.requestAnimationFrame(() => {
    refreshRaf = null;
    refreshRafTail = window.requestAnimationFrame(() => {
      refreshRafTail = null;
      if (!terminal) {
        return;
      }
      if (terminal.rows <= 0 || terminal.cols <= 0) {
        return;
      }
      if (webglAddon) {
        try {
          terminal.clearTextureAtlas();
        } catch {
          // 忽略 WebGL 字形图集重置失败，避免影响主流程。
        }
      }
      terminal.refresh(0, terminal.rows - 1);
    });
  });
};

// 非焦点面板的输出偶发不触发重绘，按帧补一次轻量刷新保证可见更新。
const scheduleOutputRefresh = () => {
  if (!terminal || isFocused.value) {
    return;
  }
  if (outputRefreshRaf !== null) {
    return;
  }
  outputRefreshRaf = window.requestAnimationFrame(() => {
    outputRefreshRaf = null;
    if (!terminal || isFocused.value) {
      return;
    }
    scheduleRendererRefresh('output');
  });
};

const isWebglCooldownActive = () => webglCooldownUntil > Date.now();

const setWebglCooldown = (reason: string) => {
  webglCooldownUntil = Date.now() + WEBGL_COOLDOWN_MS;
  debugLog('webgl cooldown', { terminalId, reason, until: webglCooldownUntil });
  if (webglCooldownTimer !== null) {
    window.clearTimeout(webglCooldownTimer);
    webglCooldownTimer = null;
  }
  // 冷却结束后仅尝试一次恢复，避免持续抖动。
  webglCooldownTimer = window.setTimeout(() => {
    webglCooldownTimer = null;
    if (!terminal || !shouldUseWebgl.value) {
      return;
    }
    syncRendererPreference('cooldown');
  }, WEBGL_COOLDOWN_MS + 200);
};

const disableWebglAddon = () => {
  webglAddon?.dispose();
  webglAddon = null;
};

const disableCanvasAddon = () => {
  canvasAddon?.dispose();
  canvasAddon = null;
};

const enableCanvasAddon = (reason: string) => {
  if (!terminal || canvasAddon) {
    return true;
  }
  try {
    canvasAddon = new CanvasAddon();
    terminal.loadAddon(canvasAddon);
    debugLog('canvas addon enabled', { terminalId, reason });
    return true;
  } catch (error) {
    canvasAddon?.dispose();
    canvasAddon = null;
    console.warn('Failed to enable canvas renderer.', error);
    return false;
  }
};

// 分屏时 WebGL 多实例易掉帧或不刷新，按需启用避免渲染失效。
const enableWebglAddon = (reason: string) => {
  if (!terminal || webglAddon || !shouldUseWebgl.value || isWebglCooldownActive()) {
    return false;
  }
  try {
    webglAddon = new WebglAddon();
    webglAddon.onContextLoss(() => {
      disableWebglAddon();
      setWebglCooldown('context-loss');
      rendererMode.value = 'canvas';
      enableCanvasAddon('context-loss');
      requestFit('webgl-context-loss');
    });
    terminal.loadAddon(webglAddon);
    window.setTimeout(() => {
      if (!terminal) {
        return;
      }
      try {
        terminal.clearTextureAtlas();
      } catch {
        // 忽略 WebGL 字形图集重置失败，避免影响主流程。
      }
      fitTerminal(`webgl-${reason}`);
    }, 50);
    return true;
  } catch (error) {
    disableWebglAddon();
    console.warn('Failed to enable webgl renderer.', error);
    return false;
  }
};

const applyRendererMode = (mode: 'webgl' | 'canvas', reason: string) => {
  if (rendererMode.value === mode) {
    return;
  }
  if (mode === 'webgl') {
    disableCanvasAddon();
    if (enableWebglAddon(reason)) {
      rendererMode.value = 'webgl';
      requestFit(`renderer-${reason}`);
      return;
    }
    setWebglCooldown('enable-failed');
    rendererMode.value = 'canvas';
    enableCanvasAddon('webgl-fallback');
    requestFit('webgl-fallback');
    return;
  }
  disableWebglAddon();
  enableCanvasAddon(reason);
  rendererMode.value = 'canvas';
  requestFit(`renderer-${reason}`);
};

const syncRendererPreference = (reason: string) => {
  if (!terminal) {
    return;
  }
  if (shouldUseWebgl.value && !isWebglCooldownActive()) {
    applyRendererMode('webgl', reason);
    return;
  }
  applyRendererMode('canvas', reason);
};

const pushPendingOutput = (chunk: OutputChunk) => {
  pendingOutput.push(chunk);
  if (pendingOutput.length > PENDING_OUTPUT_LIMIT) {
    pendingOutput.splice(0, pendingOutput.length - PENDING_OUTPUT_LIMIT);
  }
};

const queueAckBytes = (count: number) => {
  if (!Number.isFinite(count) || count <= 0) {
    return;
  }
  pendingAckBytes += count;
  if (pendingAckBytes >= ACK_BATCH_SIZE) {
    const toSend = pendingAckBytes;
    pendingAckBytes = 0;
    if (ackTimer !== null) {
      window.clearTimeout(ackTimer);
      ackTimer = null;
    }
    void ackSession(terminalId, toSend).catch(() => {});
    return;
  }
  if (ackTimer === null) {
    ackTimer = window.setTimeout(() => {
      ackTimer = null;
      if (pendingAckBytes > 0) {
        const toSend = pendingAckBytes;
        pendingAckBytes = 0;
        void ackSession(terminalId, toSend).catch(() => {});
      }
    }, ACK_FLUSH_MS);
  }
};

const ackWrittenData = (data: string) => {
  if (!data) {
    return;
  }
  queueAckBytes(textEncoder.encode(data).length);
};

const flushPendingOutput = (minSeq: number) => {
  if (!terminal || pendingOutput.length === 0) {
    pendingOutput = [];
    return;
  }
  pendingOutput.sort((left, right) => left.seq - right.seq);
  for (const chunk of pendingOutput) {
    if (chunk.seq <= minSeq || chunk.seq <= lastAppliedSeq) {
      continue;
    }
    lastAppliedSeq = chunk.seq;
    if (chunk.data) {
      const receivedAt = performance.now();
      const needsCleanupRefresh = hasCleanupSequence(chunk.data);
      traceLog('output flush', { terminalId, seq: chunk.seq, len: chunk.data.length, t: receivedAt });
      terminal.write(chunk.data, () => {
        const writtenAt = performance.now();
        traceLog('output write', {
          terminalId,
          seq: chunk.seq,
          len: chunk.data.length,
          dt: Math.round(writtenAt - receivedAt)
        });
        ackWrittenData(chunk.data);
        if (needsCleanupRefresh) {
          scheduleRendererRefresh('output-cleanup');
        } else {
          scheduleOutputRefresh();
        }
      });
    }
  }
  pendingOutput = [];
};

const flushPendingInput = (reason: string) => {
  if (!pendingInput) {
    if (inputFlushTimer !== null) {
      window.clearTimeout(inputFlushTimer);
      inputFlushTimer = null;
    }
    return;
  }
  const data = pendingInput;
  pendingInput = '';
  if (inputFlushTimer !== null) {
    window.clearTimeout(inputFlushTimer);
    inputFlushTimer = null;
  }
  traceLog('input flush', { terminalId, len: data.length, reason });
  void writeSession(terminalId, data).catch((error) => {
    terminal?.writeln(`\r\n[terminal error] ${String(error)}`);
  });
};

const attachOutput = () => {
  if (!terminal || unsubscribeOutput) {
    return;
  }
  unsubscribeOutput = subscribeOutput(terminalId, ({ data, seq }) => {
    if (!snapshotReady) {
      if (pendingOutput.length === 0) {
        debugLog('buffer output before snapshot', { terminalId, seq, len: data.length });
      }
      traceLog('output buffer pre-snapshot', { terminalId, seq, len: data.length });
      pushPendingOutput({ data, seq });
      return;
    }
    if (seq <= lastAppliedSeq) {
      return;
    }
    lastAppliedSeq = seq;
    if (data) {
      const receivedAt = performance.now();
      const needsCleanupRefresh = hasCleanupSequence(data);
      traceLog('output receive', { terminalId, seq, len: data.length, t: receivedAt });
      terminal?.write(data, () => {
        const writtenAt = performance.now();
        traceLog('output write', {
          terminalId,
          seq,
          len: data.length,
          dt: Math.round(writtenAt - receivedAt)
        });
        ackWrittenData(data);
        if (needsCleanupRefresh) {
          scheduleRendererRefresh('output-cleanup');
        } else {
          scheduleOutputRefresh();
        }
      });
    }
  });
  unsubscribeExit = subscribeExit(terminalId, (payload) => {
    const reason = payload.signal ? `signal ${payload.signal}` : `code ${payload.code ?? 'unknown'}`;
    terminal?.writeln(`\r\n[process exited: ${reason}]`);
  });
  unsubscribeError = subscribeError(terminalId, (payload) => {
    terminal?.writeln(`\r\n[terminal error] ${payload.error}`);
    pushToast(payload.error, { tone: 'error' });
    if (payload.fatal) {
      fatalError.value = payload.error;
      if (terminal) {
        terminal.options.disableStdin = true;
      }
    }
  });
};

const detachOutput = () => {
  unsubscribeOutput?.();
  unsubscribeOutput = null;
  unsubscribeExit?.();
  unsubscribeExit = null;
  unsubscribeError?.();
  unsubscribeError = null;
};

const resolveMember = () => {
  if (!memberId) {
    return null;
  }
  return members.value.find((member) => member.id === memberId) ?? null;
};

const formatAttachError = (error: unknown) => (error instanceof Error ? error.message : String(error));

const resolveSnapshotCursor = (snapshot: { cursorRow?: number; cursorCol?: number }) => {
  if (!terminal) {
    return null;
  }
  const row = Number(snapshot.cursorRow);
  const col = Number(snapshot.cursorCol);
  if (!Number.isFinite(row) || !Number.isFinite(col)) {
    return null;
  }
  const normalizedRow = Math.round(row);
  const normalizedCol = Math.round(col);
  if (normalizedRow <= 0 || normalizedCol <= 0) {
    return null;
  }
  const maxRow = Math.max(1, terminal.rows);
  const maxCol = Math.max(1, terminal.cols);
  return {
    row: Math.min(normalizedRow, maxRow),
    col: Math.min(normalizedCol, maxCol)
  };
};

const applySnapshotCursor = (
  snapshot: { cursorRow?: number; cursorCol?: number },
  done: (cursorApplied: boolean) => void
) => {
  if (!terminal) {
    done(false);
    return;
  }
  const cursor = resolveSnapshotCursor(snapshot);
  if (!cursor) {
    done(false);
    return;
  }
  // 快照光标是后端真相，先写入再放行增量输出，避免错位覆盖。
  terminal.write(`\x1b[${cursor.row};${cursor.col}H`, () => {
    done(true);
  });
};

// 先写入历史再写入当前屏幕，确保快照与后续增量输出对齐。
const applySnapshotPayload = (snapshot: {
  data: string;
  seq: number;
  history?: string;
  cursorRow?: number;
  cursorCol?: number;
}) => {
  if (!terminal) {
    return;
  }
  debugLog('apply snapshot', {
    terminalId,
    seq: snapshot.seq,
    dataLen: snapshot.data.length,
    historyLen: snapshot.history?.length ?? 0,
    cursorRow: snapshot.cursorRow,
    cursorCol: snapshot.cursorCol
  });
  terminal.reset();
  const finalizeAttach = (cursorApplied: boolean) => {
    snapshotReady = true;
    requestFit('attach');
    lastAppliedSeq = snapshot.seq;
    flushPendingOutput(snapshot.seq);
    reattachWindowUntil = performance.now() + REATTACH_WINDOW_MS;
    applyActiveState(props.active, isFocused.value);
    attachPhase.value = 'idle';
    reconnectAttempted = false;
    scheduleBackendResize('attach');
    void logDiagnosticsEvent('terminal-attach-applied', {
      terminalId,
      memberId,
      terminalType: props.terminalType,
      seq: snapshot.seq,
      dataLen: snapshot.data.length,
      snapshotLineCount: countSnapshotLines(snapshot.data),
      cursorRow: snapshot.cursorRow ?? null,
      cursorCol: snapshot.cursorCol ?? null,
      cursorApplied,
      xterm: collectXtermMeta(),
      viewport: summarizeViewportLines()
    });
    void maybeLogPassiveSnapshotTriplet('attach');
  };
  const applySnapshot = () => {
    if (!terminal) {
      return;
    }
    terminal.write(snapshot.data, () => {
      ackWrittenData(snapshot.data);
      applySnapshotCursor(snapshot, finalizeAttach);
    });
  };
  if (snapshot.history) {
    terminal.write(snapshot.history, () => {
      ackWrittenData(snapshot.history);
      applySnapshot();
    });
  } else {
    applySnapshot();
  }
};

const finalizeAttachFailure = (error: unknown) => {
  const message = formatAttachError(error);
  console.error('Failed to attach terminal snapshot.', error);
  terminal?.writeln(`\r\n[terminal error] ${message}`);
  terminal?.writeln('\r\n[session disconnected, please retry]');
  snapshotReady = true;
  flushPendingOutput(0);
  applyActiveState(props.active, isFocused.value);
  attachPhase.value = 'idle';
};

const attemptAttach = async () => {
  debugLog('attach start', { terminalId });
  const snapshot = await attachSession(terminalId);
  if (terminal && typeof snapshot.cols === 'number' && typeof snapshot.rows === 'number') {
    if (snapshot.cols > 0 && snapshot.rows > 0) {
      if (terminal.cols !== snapshot.cols || terminal.rows !== snapshot.rows) {
        terminal.resize(snapshot.cols, snapshot.rows);
      }
      lastResizeCols = snapshot.cols;
      lastResizeRows = snapshot.rows;
    }
  }
  const snapshotLineCount = countSnapshotLines(snapshot.data);
  const xtermMeta = collectXtermMeta();
  const viewportSummary = summarizeViewportLines();
  void logDiagnosticsEvent('terminal-attach', {
    terminalId,
    memberId,
    terminalType: props.terminalType,
    seq: snapshot.seq,
    data: snapshot.data,
    history: snapshot.history ?? null,
    dataLen: snapshot.data.length,
    snapshotLineCount,
    cursorRow: snapshot.cursorRow ?? null,
    cursorCol: snapshot.cursorCol ?? null,
    xterm: xtermMeta,
    viewport: viewportSummary
  });
  applySnapshotPayload(snapshot);
  debugLog('attach done', { terminalId, seq: snapshot.seq });
};

// attach 失败时尝试重建会话并重连，避免 UI 停留在空白态。
const attachWithRecovery = async () => {
  attachPhase.value = 'attaching';
  try {
    await attemptAttach();
    return;
  } catch (error) {
    const member = resolveMember();
    if (!member) {
      finalizeAttachFailure(error);
      return;
    }
    if (reconnectAttempted) {
      finalizeAttachFailure(error);
      return;
    }
    reconnectAttempted = true;
    attachPhase.value = 'reconnecting';
    try {
      await terminalMemberStore.ensureMemberSession(member, { openTab: false });
      pendingOutput = [];
      lastAppliedSeq = 0;
      await attemptAttach();
      return;
    } catch (recoveryError) {
      finalizeAttachFailure(recoveryError);
      return;
    }
  }
};

const applyActiveState = (isVisible: boolean, nextFocused: boolean) => {
  if (!terminal) {
    return;
  }
  if (!nextFocused) {
    stopPasteHold();
  }
  if (fatalError.value) {
    terminal.options.disableStdin = true;
    terminal.blur();
    if (beepTraceRunId) {
      void logBeepTrace(
        'beep.active-state',
        collectFocusSnapshot({ reason: 'fatal-error', nextActive: false, visible: isVisible })
      );
    }
    return;
  }
  terminal.options.disableStdin = !nextFocused;
  if (beepTraceRunId) {
    void logBeepTrace(
      'beep.active-state',
      collectFocusSnapshot({ reason: 'set-active', nextActive: nextFocused, visible: isVisible })
    );
  }
  if (!snapshotReady) {
    if (nextFocused) {
      void nextTick(() => {
        requestFit('activate');
        scheduleRendererRefresh('activate');
        terminal?.focus();
        if (beepTraceRunId) {
          void logBeepTrace('beep.focus-call', collectFocusSnapshot({ reason: 'activate' }));
        }
      });
    } else {
      terminal.blur();
    }
    return;
  }
  if (isVisible) {
    void setSessionActive(terminalId, true).catch(() => {});
    attachOutput();
    if (terminalRef.value && resizeObserver) {
      resizeObserver.observe(terminalRef.value);
    }
    if (nextFocused) {
      void nextTick(() => {
        requestFit('activate');
        scheduleRendererRefresh('activate');
        terminal?.focus();
        if (beepTraceRunId) {
          void logBeepTrace('beep.focus-call', collectFocusSnapshot({ reason: 'activate' }));
        }
      });
    } else {
      terminal.blur();
    }
  } else {
    if (pendingInput) {
      flushPendingInput('deactivate');
    }
    inputPasteBracketed = false;
    inputPasteHeuristic = false;
    void setSessionActive(terminalId, false).catch(() => {});
    detachOutput();
    resizeObserver?.disconnect();
    terminal.blur();
  }
};



onMounted(async () => {
  if (!terminalRef.value || !rootRef.value) {
    return;
  }
  void startSnapshotListener();
  terminal = new Terminal({
    cursorBlink: true,
    allowProposedApi: true,
    fontFamily:
      "'JetBrains Mono', 'Fira Code', ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace",
    fontSize: 13,
    scrollback: 5000,
    theme: {
      background: '#0b0f14',
      foreground: '#e5e7eb',
      cursor: '#38bdf8'
    }
  });

  terminal.loadAddon(fitAddon);
  searchAddon = new SearchAddon();
  terminal.loadAddon(searchAddon);
  searchResultDisposable = searchAddon.onDidChangeResults(({ resultIndex, resultCount }) => {
    findResultsTotal.value = resultCount;
    findResultsIndex.value = resultIndex >= 0 ? resultIndex + 1 : 0;
  });
  terminal.open(terminalRef.value);
  await nextTick();
  if ('fonts' in document) {
    try {
      await document.fonts.ready;
    } catch {
      // 忽略字体就绪失败，避免阻断终端渲染。
    }
  }
  await waitForStableFit('mount');
  syncRendererPreference('mount');
  attachClipboardHandlers(rootRef.value);
  stopTerminalContextMenu = registerTerminalContextMenu();

  terminal.onBell(() => {
    if (!beepTraceRunId) {
      return;
    }
    void logBeepTrace('beep.bell', collectFocusSnapshot({ reason: 'xterm' }));
  });

  terminal.onKey(({ domEvent }) => {
    if (!beepTraceRunId || !isBeepTraceVerbose()) {
      return;
    }
    void logBeepTrace(
      'beep.xterm-key',
      collectFocusSnapshot({
        keyKind: classifyKey(domEvent.key),
        keyLen: domEvent.key.length,
        code: domEvent.code,
        repeat: domEvent.repeat,
        altKey: domEvent.altKey,
        ctrlKey: domEvent.ctrlKey,
        shiftKey: domEvent.shiftKey,
        metaKey: domEvent.metaKey,
        isComposing: domEvent.isComposing,
        defaultPrevented: domEvent.defaultPrevented,
        cancelable: domEvent.cancelable,
        target: describeElement(domEvent.target instanceof Element ? domEvent.target : null),
        path: describeEventPath(domEvent)
      })
    );
  });

  terminal.onData((data) => {
    if (!isFocused.value || fatalError.value) {
      return;
    }
    if (beepTraceRunId) {
      void logBeepTrace(
        'beep.input',
        collectFocusSnapshot({
          inputLen: data.length,
          inputKind: classifyInputChunk(data)
        })
      );
    }
    pendingInput += data;
    const sawPasteStart = data.includes('\x1b[200~');
    const sawPasteEnd = data.includes('\x1b[201~');
    if (sawPasteStart) {
      inputPasteBracketed = true;
    }
    if (data.length >= INPUT_PASTE_THRESHOLD) {
      inputPasteHeuristic = true;
    }
    if (sawPasteEnd) {
      inputPasteBracketed = false;
    }
    const pasteActive = inputPasteBracketed || inputPasteHeuristic;
    if (!pasteActive && data.length <= INPUT_IMMEDIATE_MAX_LEN) {
      flushPendingInput('type');
      return;
    }
    if (pasteActive) {
      flushPendingInput('paste');
      if (!inputPasteBracketed) {
        inputPasteHeuristic = false;
      }
      return;
    }
    if (pendingInput.length >= INPUT_BATCH_SIZE || data.includes('\n') || data.includes('\r')) {
      flushPendingInput('threshold');
      if (!inputPasteBracketed) {
        inputPasteHeuristic = false;
      }
      return;
    }
    if (inputFlushTimer === null) {
      inputFlushTimer = window.setTimeout(() => {
        flushPendingInput('timer');
        if (!inputPasteBracketed) {
          inputPasteHeuristic = false;
        }
      }, INPUT_FLUSH_MS);
    }
  });

  resizeObserver = new ResizeObserver(() => {
    if (props.active) {
      requestFit('resize-observer');
    }
  });

  attachOutput();
  void setSessionActive(terminalId, props.active).catch(() => {});
  await attachWithRecovery();

  if (isBeepTraceEnabled()) {
    await ensureBeepTraceRun();
    beepTraceRefCount += 1;
    attachBeepTraceListeners();
    void logBeepTrace('beep.init', collectFocusSnapshot({ reason: 'mount' }));
  }

  windowResizeHandler = () => {
    if (props.active) {
      requestFit('window-resize');
    }
  };
  window.addEventListener('resize', windowResizeHandler);
  if (window.visualViewport) {
    viewportResizeHandler = () => {
      if (props.active) {
        requestFit('viewport-resize');
      }
    };
    window.visualViewport.addEventListener('resize', viewportResizeHandler);
  }
  visibilityFitHandler = () => {
    if (document.visibilityState === 'visible' && props.active) {
      requestFit('visibility');
    }
  };
  document.addEventListener('visibilitychange', visibilityFitHandler);
});

watch(
  () => props.active,
  (isActive) => {
    applyActiveState(isActive, isFocused.value);
  },
  { flush: 'post' }
);

watch(
  () => isFocused.value,
  (nextFocused) => {
    applyActiveState(props.active, nextFocused);
  },
  { flush: 'post' }
);

watch(
  () => shouldUseWebgl.value,
  () => {
    syncRendererPreference('toggle');
  }
);

onBeforeUnmount(() => {
  void storePassiveSnapshot('unmount');
  stopPasteHold();
  detachOutput();
  stopSnapshotRequest?.();
  stopSnapshotRequest = null;
  void setSessionActive(terminalId, false).catch(() => {});
  if (ackTimer !== null) {
    window.clearTimeout(ackTimer);
    ackTimer = null;
  }
  if (fitRaf !== null) {
    window.cancelAnimationFrame(fitRaf);
    fitRaf = null;
  }
  fitRafReason = null;
  if (webglCooldownTimer !== null) {
    window.clearTimeout(webglCooldownTimer);
    webglCooldownTimer = null;
  }
  if (resizeSyncTimer !== null) {
    window.clearTimeout(resizeSyncTimer);
    resizeSyncTimer = null;
  }
  pendingResize = null;
  if (inputFlushTimer !== null) {
    window.clearTimeout(inputFlushTimer);
    inputFlushTimer = null;
  }
  inputPasteBracketed = false;
  inputPasteHeuristic = false;
  if (pendingInput) {
    flushPendingInput('unmount');
  }
  if (pendingAckBytes > 0) {
    const toSend = pendingAckBytes;
    pendingAckBytes = 0;
    void ackSession(terminalId, toSend).catch(() => {});
  }
  if (rootRef.value && mouseUpHandler) {
    rootRef.value.removeEventListener('mouseup', mouseUpHandler);
  }
  stopTerminalContextMenu?.();
  stopTerminalContextMenu = null;
  if (refreshRaf !== null) {
    window.cancelAnimationFrame(refreshRaf);
    refreshRaf = null;
  }
  if (refreshRafTail !== null) {
    window.cancelAnimationFrame(refreshRafTail);
    refreshRafTail = null;
  }
  if (outputRefreshRaf !== null) {
    window.cancelAnimationFrame(outputRefreshRaf);
    outputRefreshRaf = null;
  }
  if (beepTraceAttached) {
    detachBeepTraceListeners();
  }
  if (windowResizeHandler) {
    window.removeEventListener('resize', windowResizeHandler);
    windowResizeHandler = null;
  }
  if (viewportResizeHandler && window.visualViewport) {
    window.visualViewport.removeEventListener('resize', viewportResizeHandler);
    viewportResizeHandler = null;
  }
  if (visibilityFitHandler) {
    document.removeEventListener('visibilitychange', visibilityFitHandler);
    visibilityFitHandler = null;
  }
  resizeObserver?.disconnect();
  webglAddon?.dispose();
  webglAddon = null;
  canvasAddon?.dispose();
  canvasAddon = null;
  searchResultDisposable?.dispose();
  searchResultDisposable = null;
  searchAddon?.dispose();
  searchAddon = null;
  terminal?.dispose();
  terminal = null;
  if (beepTraceRunId) {
    beepTraceRefCount = Math.max(0, beepTraceRefCount - 1);
    if (beepTraceRefCount === 0) {
      void endBeepTraceRun('terminal-pane-unmount');
    }
  }
});
</script>

