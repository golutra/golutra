import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { TerminalType } from '@/shared/types/terminal';

type OutputPayload = { sessionId: string; data: string; seq: number };
type SnapshotPayload = { sessionId: string; data: string; seq: number; history?: string };
type ExitPayload = { sessionId: string; code?: number | null; signal?: string | null };
type StatusPayload = { sessionId: string; status: string; memberId?: string; workspaceId?: string };
type ErrorPayload = { sessionId: string; error: string; fatal?: boolean };
export type TerminalChatPayload = {
  sessionId: string;
  memberId?: string;
  workspaceId?: string;
  conversationId?: string;
  conversationType?: string;
  senderId?: string;
  senderName?: string;
  seq: number;
  timestamp: number;
  content: string;
  type: 'info' | 'error' | 'progress' | 'system' | 'user_input';
  source: 'pty' | 'chat' | 'system' | 'ai';
  mode: 'snapshot' | 'delta';
  spanId?: string;
  meta?: {
    command?: string;
    lineCount?: number;
    cursor?: { row: number; col: number };
    startRow?: number;
    endRow?: number;
  };
};

export type TerminalDispatchContext = {
  conversationId: string;
  conversationType: string;
  senderId: string;
  senderName: string;
};

type OutputListener = (payload: { data: string; seq: number }) => void;
type ExitListener = (payload: ExitPayload) => void;
type ActivityListener = (sessionId: string) => void;
type StatusListener = (payload: StatusPayload) => void;
type ChatListener = (payload: TerminalChatPayload) => void;
type ErrorListener = (payload: ErrorPayload) => void;

const outputListeners = new Map<string, Set<OutputListener>>();
const exitListeners = new Map<string, Set<ExitListener>>();
const activityListeners = new Set<ActivityListener>();
const statusListeners = new Set<StatusListener>();
const chatListeners = new Set<ChatListener>();
const errorListeners = new Map<string, Set<ErrorListener>>();
const buffers = new Map<string, OutputPayload[]>();
const trackedSessions = new Set<string>();
const loggedOutputSessions = new Set<string>();
const ackEncoder = new TextEncoder();
const ackBuffers = new Map<string, { pending: number; timer: number | null }>();
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
  console.info('[terminal-bridge]', ...args);
};

const BUFFER_LIMIT = 2000;
const ACK_BATCH_SIZE = 5000;
const ACK_FLUSH_MS = 50;
const SIZE_STORAGE_KEY = 'terminal-last-size';
let initPromise: Promise<void> | null = null;

type TerminalSize = { cols: number; rows: number };

const loadStoredSize = (): TerminalSize | null => {
  if (typeof window === 'undefined') {
    return null;
  }
  const raw = window.localStorage.getItem(SIZE_STORAGE_KEY);
  if (!raw) {
    return null;
  }
  try {
    const parsed = JSON.parse(raw) as Partial<TerminalSize>;
    const cols = Number(parsed.cols);
    const rows = Number(parsed.rows);
    if (Number.isFinite(cols) && Number.isFinite(rows) && cols > 0 && rows > 0) {
      return { cols, rows };
    }
  } catch {
    // Ignore malformed stored size payloads.
  }
  return null;
};

let lastKnownSize: TerminalSize | null = loadStoredSize();

const persistSize = (size: TerminalSize) => {
  if (typeof window === 'undefined') {
    return;
  }
  window.localStorage.setItem(SIZE_STORAGE_KEY, JSON.stringify(size));
};

const updateLastKnownSize = (cols: number, rows: number) => {
  if (!Number.isFinite(cols) || !Number.isFinite(rows) || cols <= 0 || rows <= 0) {
    return;
  }
  const next = { cols, rows };
  if (!lastKnownSize || lastKnownSize.cols !== cols || lastKnownSize.rows !== rows) {
    lastKnownSize = next;
    persistSize(next);
  }
};

const pushBuffer = (sessionId: string, payload: OutputPayload) => {
  const queue = buffers.get(sessionId) ?? [];
  queue.push(payload);
  if (queue.length > BUFFER_LIMIT) {
    queue.splice(0, queue.length - BUFFER_LIMIT);
  }
  buffers.set(sessionId, queue);
};

const flushBuffer = (sessionId: string, handler: OutputListener) => {
  const queue = buffers.get(sessionId);
  if (!queue || queue.length === 0) {
    return;
  }
  queue.sort((left, right) => left.seq - right.seq);
  for (const payload of queue) {
    handler({ data: payload.data, seq: payload.seq });
  }
  buffers.delete(sessionId);
};

const queueAck = (sessionId: string, data: string) => {
  if (isAckIpcDisabled()) {
    return;
  }
  if (!data) {
    return;
  }
  const bytes = ackEncoder.encode(data).length;
  if (bytes <= 0) {
    return;
  }
  const entry = ackBuffers.get(sessionId) ?? { pending: 0, timer: null };
  entry.pending += bytes;
  if (entry.pending >= ACK_BATCH_SIZE) {
    const count = entry.pending;
    entry.pending = 0;
    if (entry.timer !== null) {
      window.clearTimeout(entry.timer);
      entry.timer = null;
    }
    traceLog('ack send', { sessionId, count, reason: 'batch' });
    void invoke('terminal_ack', { sessionId, count }).catch(() => {});
    ackBuffers.set(sessionId, entry);
    return;
  }
  if (entry.timer === null) {
    entry.timer = window.setTimeout(() => {
      entry.timer = null;
      if (entry.pending > 0) {
        const count = entry.pending;
        entry.pending = 0;
        traceLog('ack send', { sessionId, count, reason: 'timer' });
        void invoke('terminal_ack', { sessionId, count }).catch(() => {});
      }
      ackBuffers.set(sessionId, entry);
    }, ACK_FLUSH_MS);
  }
  ackBuffers.set(sessionId, entry);
};

const ensureListeners = async () => {
  if (initPromise) {
    return initPromise;
  }
  initPromise = (async () => {
    await listen<OutputPayload>('terminal-output', (event) => {
      const { sessionId, data, seq } = event.payload;
      const receivedAt = performance.now();
      if (!loggedOutputSessions.has(sessionId)) {
        loggedOutputSessions.add(sessionId);
        debugLog('terminal output first', { sessionId, seq, len: data.length });
      }
      const listeners = outputListeners.get(sessionId);
      const hasListeners = Boolean(listeners && listeners.size > 0);
      const isTracked = trackedSessions.has(sessionId);
      traceLog('output event', {
        sessionId,
        seq,
        len: data.length,
        hasListeners,
        isTracked,
        t: receivedAt
      });
      if (hasListeners) {
        for (const handler of listeners) {
          handler({ data, seq });
        }
      } else {
        queueAck(sessionId, data);
        if (isTracked) {
          pushBuffer(sessionId, { sessionId, data, seq });
        } else {
          return;
        }
      }
      if (hasListeners || isTracked) {
        for (const handler of activityListeners) {
          handler(sessionId);
        }
      }
    });

    await listen<ExitPayload>('terminal-exit', (event) => {
      const { sessionId } = event.payload;
      debugLog('terminal exit', event.payload);
      const listeners = exitListeners.get(sessionId);
      if (!listeners || listeners.size === 0) {
        return;
      }
      for (const handler of listeners) {
        handler(event.payload);
      }
    });

    await listen<StatusPayload>('terminal-status-change', (event) => {
      for (const handler of statusListeners) {
        handler(event.payload);
      }
    });

    await listen<TerminalChatPayload>('terminal-chat-output', (event) => {
      for (const handler of chatListeners) {
        handler(event.payload);
      }
    });

    await listen<ErrorPayload>('terminal-error', (event) => {
      console.warn('[terminal-error]', event.payload);
      const listeners = errorListeners.get(event.payload.sessionId);
      if (listeners) {
        for (const handler of listeners) {
          handler(event.payload);
        }
      }
    });

  })();
  try {
    await initPromise;
  } catch (error) {
    initPromise = null;
    throw error;
  }
};

export const createSession = async (options?: {
  cols?: number;
  rows?: number;
  cwd?: string;
  memberId?: string;
  workspaceId?: string;
  keepAlive?: boolean;
  sessionId?: string;
  terminalType?: TerminalType;
  terminalCommand?: string;
  terminalPath?: string;
}) => {
  await ensureListeners();
  const cols = options?.cols ?? lastKnownSize?.cols;
  const rows = options?.rows ?? lastKnownSize?.rows;
  debugLog('create session', {
    cols,
    rows,
    memberId: options?.memberId,
    workspaceId: options?.workspaceId,
    terminalType: options?.terminalType,
    terminalCommand: options?.terminalCommand,
    terminalPath: options?.terminalPath
  });
  return invoke<string>('terminal_create', {
    cols,
    rows,
    cwd: options?.cwd,
    memberId: options?.memberId,
    workspaceId: options?.workspaceId,
    keepAlive: options?.keepAlive,
    sessionId: options?.sessionId,
    terminalType: options?.terminalType,
    terminalCommand: options?.terminalCommand,
    terminalPath: options?.terminalPath
  });
};

export const writeSession = async (sessionId: string, data: string) => {
  await ensureListeners();
  return invoke('terminal_write', { sessionId, data });
};

export const ackSession = async (sessionId: string, count: number) => {
  if (isAckIpcDisabled()) {
    return;
  }
  traceLog('ack send', { sessionId, count, reason: 'explicit' });
  await ensureListeners();
  return invoke('terminal_ack', { sessionId, count });
};

export const setSessionActive = async (sessionId: string, active: boolean) => {
  await ensureListeners();
  return invoke('terminal_set_active', { sessionId, active });
};

export const setMemberStatus = async (memberId: string, status: string) => {
  await ensureListeners();
  return invoke('terminal_set_member_status', { memberId, status });
};

export const dispatchSession = async (
  sessionId: string,
  data: string,
  context: TerminalDispatchContext
) => {
  await ensureListeners();
  return invoke('terminal_dispatch', { sessionId, data, context });
};

export const resizeSession = async (sessionId: string, cols: number, rows: number) => {
  await ensureListeners();
  updateLastKnownSize(cols, rows);
  return invoke('terminal_resize', { sessionId, cols, rows });
};

export const closeSession = async (sessionId: string, options?: { preserve?: boolean }) => {
  await ensureListeners();
  buffers.delete(sessionId);
  outputListeners.delete(sessionId);
  exitListeners.delete(sessionId);
  trackedSessions.delete(sessionId);
  const ackEntry = ackBuffers.get(sessionId);
  if (ackEntry?.timer !== null && ackEntry?.timer !== undefined) {
    window.clearTimeout(ackEntry.timer);
  }
  ackBuffers.delete(sessionId);
  return invoke('terminal_close', { sessionId, preserve: options?.preserve });
};

export const attachSession = async (sessionId: string) => {
  await ensureListeners();
  return invoke<SnapshotPayload>('terminal_attach', { sessionId });
};

export const trackSession = (sessionId: string) => {
  trackedSessions.add(sessionId);
};

export const untrackSession = (sessionId: string) => {
  trackedSessions.delete(sessionId);
  buffers.delete(sessionId);
  const ackEntry = ackBuffers.get(sessionId);
  if (ackEntry?.timer !== null && ackEntry?.timer !== undefined) {
    window.clearTimeout(ackEntry.timer);
  }
  ackBuffers.delete(sessionId);
};

export const subscribeOutput = (sessionId: string, handler: OutputListener) => {
  void ensureListeners().catch(() => {});
  const listeners = outputListeners.get(sessionId) ?? new Set<OutputListener>();
  listeners.add(handler);
  outputListeners.set(sessionId, listeners);
  flushBuffer(sessionId, handler);
  return () => {
    const current = outputListeners.get(sessionId);
    if (!current) {
      return;
    }
    current.delete(handler);
    if (current.size === 0) {
      outputListeners.delete(sessionId);
    }
  };
};

export const subscribeExit = (sessionId: string, handler: ExitListener) => {
  void ensureListeners().catch(() => {});
  const listeners = exitListeners.get(sessionId) ?? new Set<ExitListener>();
  listeners.add(handler);
  exitListeners.set(sessionId, listeners);
  return () => {
    const current = exitListeners.get(sessionId);
    if (!current) {
      return;
    }
    current.delete(handler);
    if (current.size === 0) {
      exitListeners.delete(sessionId);
    }
  };
};

export const subscribeError = (sessionId: string, handler: ErrorListener) => {
  void ensureListeners().catch(() => {});
  const listeners = errorListeners.get(sessionId) ?? new Set<ErrorListener>();
  listeners.add(handler);
  errorListeners.set(sessionId, listeners);
  return () => {
    const current = errorListeners.get(sessionId);
    if (!current) {
      return;
    }
    current.delete(handler);
    if (current.size === 0) {
      errorListeners.delete(sessionId);
    }
  };
};

export const onActivity = (handler: ActivityListener) => {
  activityListeners.add(handler);
  return () => activityListeners.delete(handler);
};

export const onStatusChange = (handler: StatusListener) => {
  void ensureListeners().catch(() => {});
  statusListeners.add(handler);
  return () => statusListeners.delete(handler);
};

export const onChatMessage = (handler: ChatListener) => {
  void ensureListeners().catch(() => {});
  chatListeners.add(handler);
  return () => chatListeners.delete(handler);
};
