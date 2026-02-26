<template>
  <div
    ref="rootRef"
    :class="['h-full w-full bg-[#0b0f14] relative', attachPhase !== 'idle' ? 'opacity-95' : '']"
  >
    <div ref="terminalRef" class="h-full w-full"></div>
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
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import '@xterm/xterm/css/xterm.css';
import {
  ackSession,
  attachSession,
  resizeSession,
  setSessionActive,
  subscribeError,
  subscribeExit,
  subscribeOutput,
  writeSession
} from './terminalBridge';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useTerminalMemberStore } from '@/features/terminal/terminalMemberStore';
import { useToastStore } from '@/stores/toastStore';
import type { TerminalType } from '@/shared/types/terminal';

type OutputChunk = { data: string; seq: number };

const props = defineProps<{ sessionId: string; active: boolean; memberId?: string; terminalType?: TerminalType }>();
const sessionId = props.sessionId;
const memberId = props.memberId;

const projectStore = useProjectStore();
const terminalMemberStore = useTerminalMemberStore();
const toastStore = useToastStore();
const { pushToast } = toastStore;
const { members } = storeToRefs(projectStore);

const terminalRef = ref<HTMLDivElement | null>(null);
const rootRef = ref<HTMLDivElement | null>(null);

const fitAddon = new FitAddon();
let webglAddon: WebglAddon | null = null;
let terminal: Terminal | null = null;
let resizeObserver: ResizeObserver | null = null;
let unsubscribeOutput: (() => void) | null = null;
let unsubscribeExit: (() => void) | null = null;
let unsubscribeError: (() => void) | null = null;
let mouseUpHandler: ((event: MouseEvent) => void) | null = null;
let refreshRaf: number | null = null;
let refreshRafTail: number | null = null;
let snapshotReady = false;
let lastAppliedSeq = 0;
let pendingOutput: OutputChunk[] = [];
const attachPhase = ref<'idle' | 'attaching' | 'reconnecting'>('idle');
let reconnectAttempted = false;
const fatalError = ref<string | null>(null);
const textEncoder = new TextEncoder();
let pendingAckBytes = 0;
let ackTimer: number | null = null;
let pendingInput = '';
let inputFlushTimer: number | null = null;
const isAckIpcDisabled = () => {
  try {
    return window.localStorage.getItem('terminal-disable-ack') === '1';
  } catch {
    return false;
  }
};
const isTraceEnabled = () => {
  if (!import.meta.env.DEV) {
    return false;
  }
  if (import.meta.env.VITE_TERMINAL_TRACE === '1') {
    return true;
  }
  try {
    const stored = window.localStorage.getItem('terminal-trace');
    if (stored === '1') {
      return true;
    }
    if (stored === null) {
      window.localStorage.setItem('terminal-trace', '1');
      return true;
    }
    return false;
  } catch {
    return false;
  }
};
const traceLog = (...args: unknown[]) => {
  if (!isTraceEnabled()) {
    return;
  }
  console.info('[terminal-trace]', ...args);
};
const debugLog = (...args: unknown[]) => {
  if (!import.meta.env.DEV) {
    return;
  }
  try {
    if (window.localStorage.getItem('terminal-debug') !== '1') {
      return;
    }
  } catch {
    return;
  }
  console.info('[terminal]', ...args);
};

const PENDING_OUTPUT_LIMIT = 2000;
const ACK_BATCH_SIZE = 5000;
const ACK_FLUSH_MS = 50;
const INPUT_BATCH_SIZE = 1024;
const INPUT_FLUSH_MS = 8;

const isMac = navigator.platform.toLowerCase().includes('mac');
let lastCopiedSelection = '';

const copySelection = (options?: { force?: boolean; clear?: boolean }) => {
  if (!terminal || !terminal.hasSelection()) {
    return;
  }
  const selection = terminal.getSelection();
  if (!selection || selection === lastCopiedSelection) {
    if (options?.force && selection) {
      void navigator.clipboard.writeText(selection).catch(() => {});
      if (options?.clear) {
        terminal.clearSelection();
        terminal.focus();
        lastCopiedSelection = '';
      }
    }
    return;
  }
  lastCopiedSelection = selection;
  void navigator.clipboard.writeText(selection).catch(() => {});
  if (options?.clear) {
    terminal.clearSelection();
    terminal.focus();
    lastCopiedSelection = '';
  }
};

const attachClipboardHandlers = (root: HTMLElement) => {
  if (!terminal) {
    return;
  }
  terminal.attachCustomKeyEventHandler((event) => {
    const ctrlKey = isMac ? event.metaKey : event.ctrlKey;
    const key = event.key.toLowerCase();

    if (ctrlKey && key === 'c') {
      if (terminal.hasSelection()) {
        copySelection({ force: true, clear: true });
        return false;
      }
      return true;
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

const fitTerminal = () => {
  if (!terminal) {
    return;
  }
  fitAddon.fit();
  if (terminal.cols > 0 && terminal.rows > 0) {
    void resizeSession(sessionId, terminal.cols, terminal.rows).catch(() => {});
  }
};

const scheduleWebglRefresh = () => {
  if (!terminal || !webglAddon) {
    return;
  }
  if (refreshRaf !== null || refreshRafTail !== null) {
    return;
  }
  refreshRaf = window.requestAnimationFrame(() => {
    refreshRaf = null;
    refreshRafTail = window.requestAnimationFrame(() => {
      refreshRafTail = null;
      if (!terminal || !webglAddon) {
        return;
      }
      if (terminal.rows <= 0 || terminal.cols <= 0) {
        return;
      }
      try {
        terminal.clearTextureAtlas();
      } catch {
        // Ignore WebGL atlas reset failures.
      }
      terminal.refresh(0, terminal.rows - 1);
    });
  });
};

const pushPendingOutput = (chunk: OutputChunk) => {
  pendingOutput.push(chunk);
  if (pendingOutput.length > PENDING_OUTPUT_LIMIT) {
    pendingOutput.splice(0, pendingOutput.length - PENDING_OUTPUT_LIMIT);
  }
};

const queueAckBytes = (count: number) => {
  if (isAckIpcDisabled()) {
    return;
  }
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
    void ackSession(sessionId, toSend).catch(() => {});
    return;
  }
  if (ackTimer === null) {
    ackTimer = window.setTimeout(() => {
      ackTimer = null;
      if (pendingAckBytes > 0) {
        const toSend = pendingAckBytes;
        pendingAckBytes = 0;
        void ackSession(sessionId, toSend).catch(() => {});
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
      traceLog('output flush', { sessionId, seq: chunk.seq, len: chunk.data.length, t: receivedAt });
      terminal.write(chunk.data, () => {
        const writtenAt = performance.now();
        traceLog('output write', {
          sessionId,
          seq: chunk.seq,
          len: chunk.data.length,
          dt: Math.round(writtenAt - receivedAt)
        });
        ackWrittenData(chunk.data);
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
  traceLog('input flush', { sessionId, len: data.length, reason });
  void writeSession(sessionId, data).catch((error) => {
    terminal?.writeln(`\r\n[terminal error] ${String(error)}`);
  });
};

const attachOutput = () => {
  if (!terminal || unsubscribeOutput) {
    return;
  }
  unsubscribeOutput = subscribeOutput(sessionId, ({ data, seq }) => {
    if (!snapshotReady) {
      if (pendingOutput.length === 0) {
        debugLog('buffer output before snapshot', { sessionId, seq, len: data.length });
      }
      traceLog('output buffer pre-snapshot', { sessionId, seq, len: data.length });
      pushPendingOutput({ data, seq });
      return;
    }
    if (seq <= lastAppliedSeq) {
      return;
    }
    lastAppliedSeq = seq;
    if (data) {
      const receivedAt = performance.now();
      traceLog('output receive', { sessionId, seq, len: data.length, t: receivedAt });
      terminal?.write(data, () => {
        const writtenAt = performance.now();
        traceLog('output write', {
          sessionId,
          seq,
          len: data.length,
          dt: Math.round(writtenAt - receivedAt)
        });
        ackWrittenData(data);
      });
    }
  });
  unsubscribeExit = subscribeExit(sessionId, (payload) => {
    const reason = payload.signal ? `signal ${payload.signal}` : `code ${payload.code ?? 'unknown'}`;
    terminal?.writeln(`\r\n[process exited: ${reason}]`);
  });
  unsubscribeError = subscribeError(sessionId, (payload) => {
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

const applySnapshotPayload = (snapshot: { data: string; seq: number; history?: string }) => {
  if (!terminal) {
    return;
  }
  debugLog('apply snapshot', {
    sessionId,
    seq: snapshot.seq,
    dataLen: snapshot.data.length,
    historyLen: snapshot.history?.length ?? 0
  });
  terminal.reset();
  const applySnapshot = () => {
    if (!terminal) {
      return;
    }
    terminal.write(snapshot.data, () => {
      ackWrittenData(snapshot.data);
      snapshotReady = true;
      lastAppliedSeq = snapshot.seq;
      flushPendingOutput(snapshot.seq);
      applyActiveState(props.active);
      attachPhase.value = 'idle';
      reconnectAttempted = false;
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
  applyActiveState(props.active);
  attachPhase.value = 'idle';
};

const attemptAttach = async () => {
  debugLog('attach start', { sessionId });
  const snapshot = await attachSession(sessionId);
  applySnapshotPayload(snapshot);
  debugLog('attach done', { sessionId, seq: snapshot.seq });
};

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

const applyActiveState = (isActive: boolean) => {
  if (!terminal) {
    return;
  }
  if (fatalError.value) {
    terminal.options.disableStdin = true;
    terminal.blur();
    return;
  }
  terminal.options.disableStdin = !isActive;
  if (!snapshotReady) {
    if (isActive) {
      void nextTick(() => {
        fitTerminal();
        scheduleWebglRefresh();
        terminal?.focus();
      });
    } else {
      terminal.blur();
    }
    return;
  }
  if (isActive) {
    void setSessionActive(sessionId, true).catch(() => {});
    attachOutput();
    if (rootRef.value && resizeObserver) {
      resizeObserver.observe(rootRef.value);
    }
    void nextTick(() => {
      fitTerminal();
      scheduleWebglRefresh();
      terminal?.focus();
    });
  } else {
    if (pendingInput) {
      flushPendingInput('deactivate');
    }
    void setSessionActive(sessionId, false).catch(() => {});
    detachOutput();
    resizeObserver?.disconnect();
    terminal.blur();
  }
};

onMounted(async () => {
  if (!terminalRef.value || !rootRef.value) {
    return;
  }
  terminal = new Terminal({
    cursorBlink: true,
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
  terminal.open(terminalRef.value);
  await nextTick();
  if ('fonts' in document) {
    try {
      await document.fonts.ready;
    } catch {
      // Ignore font readiness failures.
    }
  }
  fitTerminal();
  try {
    webglAddon = new WebglAddon();
    webglAddon.onContextLoss(() => {
      webglAddon?.dispose();
      webglAddon = null;
    });
    terminal.loadAddon(webglAddon);
    window.setTimeout(() => {
      if (!terminal) {
        return;
      }
      try {
        terminal.clearTextureAtlas();
      } catch {
        // Ignore WebGL atlas reset failures.
      }
      fitTerminal();
    }, 50);
  } catch {
    webglAddon?.dispose();
    webglAddon = null;
  }
  attachClipboardHandlers(rootRef.value);

  terminal.onData((data) => {
    if (!props.active || fatalError.value) {
      return;
    }
    pendingInput += data;
    if (pendingInput.length >= INPUT_BATCH_SIZE || data.includes('\n') || data.includes('\r')) {
      flushPendingInput('threshold');
      return;
    }
    if (inputFlushTimer === null) {
      inputFlushTimer = window.setTimeout(() => {
        flushPendingInput('timer');
      }, INPUT_FLUSH_MS);
    }
  });

  resizeObserver = new ResizeObserver(() => {
    if (props.active) {
      fitTerminal();
    }
  });

  attachOutput();
  void setSessionActive(sessionId, props.active).catch(() => {});
  await attachWithRecovery();
});

watch(
  () => props.active,
  (isActive) => {
    applyActiveState(isActive);
  },
  { flush: 'post' }
);

onBeforeUnmount(() => {
  detachOutput();
  void setSessionActive(sessionId, false).catch(() => {});
  if (ackTimer !== null) {
    window.clearTimeout(ackTimer);
    ackTimer = null;
  }
  if (inputFlushTimer !== null) {
    window.clearTimeout(inputFlushTimer);
    inputFlushTimer = null;
  }
  if (pendingInput) {
    flushPendingInput('unmount');
  }
  if (pendingAckBytes > 0) {
    const toSend = pendingAckBytes;
    pendingAckBytes = 0;
    void ackSession(sessionId, toSend).catch(() => {});
  }
  if (rootRef.value && mouseUpHandler) {
    rootRef.value.removeEventListener('mouseup', mouseUpHandler);
  }
  if (refreshRaf !== null) {
    window.cancelAnimationFrame(refreshRaf);
    refreshRaf = null;
  }
  if (refreshRafTail !== null) {
    window.cancelAnimationFrame(refreshRafTail);
    refreshRafTail = null;
  }
  resizeObserver?.disconnect();
  webglAddon?.dispose();
  webglAddon = null;
  terminal?.dispose();
  terminal = null;
});
</script>
