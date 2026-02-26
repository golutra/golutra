<template>
  <div ref="rootRef" class="h-full w-full bg-[#0b0f14]">
    <div ref="terminalRef" class="h-full w-full"></div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import '@xterm/xterm/css/xterm.css';
import { getSessionHistory, resizeSession, subscribeExit, subscribeOutput, writeSession } from './terminalBridge';

const props = defineProps<{ sessionId: string; active: boolean }>();
const sessionId = props.sessionId;

const terminalRef = ref<HTMLDivElement | null>(null);
const rootRef = ref<HTMLDivElement | null>(null);

const fitAddon = new FitAddon();
let webglAddon: WebglAddon | null = null;
let terminal: Terminal | null = null;
let resizeObserver: ResizeObserver | null = null;
let unsubscribeOutput: (() => void) | null = null;
let unsubscribeExit: (() => void) | null = null;
let mouseUpHandler: ((event: MouseEvent) => void) | null = null;
let refreshRaf: number | null = null;
let refreshRafTail: number | null = null;
let historyReady = false;

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
      } catch {}
      terminal.refresh(0, terminal.rows - 1);
    });
  });
};

const attachOutput = () => {
  if (!terminal || unsubscribeOutput) {
    return;
  }
  unsubscribeOutput = subscribeOutput(sessionId, (data) => {
    terminal?.write(data);
  });
  unsubscribeExit = subscribeExit(sessionId, (payload) => {
    const reason = payload.signal ? `signal ${payload.signal}` : `code ${payload.code ?? 'unknown'}`;
    terminal?.writeln(`\r\n[process exited: ${reason}]`);
  });
};

const detachOutput = () => {
  unsubscribeOutput?.();
  unsubscribeOutput = null;
  unsubscribeExit?.();
  unsubscribeExit = null;
};

const applyActiveState = (isActive: boolean) => {
  if (!terminal) {
    return;
  }
  terminal.options.disableStdin = !isActive;
  if (!historyReady) {
    if (!isActive) {
      detachOutput();
      resizeObserver?.disconnect();
      terminal.blur();
    }
    return;
  }
  if (isActive) {
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
    } catch {}
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
      } catch {}
      fitTerminal();
    }, 50);
  } catch {
    webglAddon?.dispose();
    webglAddon = null;
  }
  attachClipboardHandlers(rootRef.value);

  terminal.onData((data) => {
    if (!props.active) {
      return;
    }
    void writeSession(sessionId, data).catch((error) => {
      terminal?.writeln(`\r\n[terminal error] ${String(error)}`);
    });
  });

  resizeObserver = new ResizeObserver(() => {
    if (props.active) {
      fitTerminal();
    }
  });

  try {
    const history = await getSessionHistory(sessionId);
    if (history) {
      terminal?.write(history);
    }
  } catch (error) {
    console.error('Failed to load terminal history.', error);
  }
  historyReady = true;
  applyActiveState(props.active);
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
