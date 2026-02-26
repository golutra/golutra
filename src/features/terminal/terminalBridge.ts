import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

type OutputPayload = { sessionId: string; data: string };
type ExitPayload = { sessionId: string; code?: number | null; signal?: string | null };
type StatusPayload = { sessionId: string; status: string; memberId?: string; workspaceId?: string };

type OutputListener = (data: string) => void;
type ExitListener = (payload: ExitPayload) => void;
type ActivityListener = (sessionId: string) => void;
type StatusListener = (payload: StatusPayload) => void;

const outputListeners = new Map<string, Set<OutputListener>>();
const exitListeners = new Map<string, Set<ExitListener>>();
const activityListeners = new Set<ActivityListener>();
const statusListeners = new Set<StatusListener>();
const buffers = new Map<string, string[]>();

const BUFFER_LIMIT = 2000;
let initialized = false;

const pushBuffer = (sessionId: string, data: string) => {
  const queue = buffers.get(sessionId) ?? [];
  queue.push(data);
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
  handler(queue.join(''));
  buffers.delete(sessionId);
};

const ensureListeners = async () => {
  if (initialized) {
    return;
  }
  initialized = true;
  await listen<OutputPayload>('terminal-output', (event) => {
    const { sessionId, data } = event.payload;
    const listeners = outputListeners.get(sessionId);
    if (listeners && listeners.size > 0) {
      for (const handler of listeners) {
        handler(data);
      }
    } else {
      pushBuffer(sessionId, data);
    }
    for (const handler of activityListeners) {
      handler(sessionId);
    }
  });

  await listen<ExitPayload>('terminal-exit', (event) => {
    const { sessionId } = event.payload;
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
};

export const createSession = async (options?: {
  cols?: number;
  rows?: number;
  cwd?: string;
  memberId?: string;
  workspaceId?: string;
  keepAlive?: boolean;
  sessionId?: string;
  initialData?: string;
}) => {
  await ensureListeners();
  return invoke<string>('terminal_create', {
    cols: options?.cols,
    rows: options?.rows,
    cwd: options?.cwd,
    memberId: options?.memberId,
    workspaceId: options?.workspaceId,
    keepAlive: options?.keepAlive,
    sessionId: options?.sessionId,
    initialData: options?.initialData
  });
};

export const writeSession = async (sessionId: string, data: string) => {
  await ensureListeners();
  return invoke('terminal_write', { sessionId, data });
};

export const resizeSession = async (sessionId: string, cols: number, rows: number) => {
  await ensureListeners();
  return invoke('terminal_resize', { sessionId, cols, rows });
};

export const closeSession = async (sessionId: string, options?: { preserve?: boolean }) => {
  await ensureListeners();
  buffers.delete(sessionId);
  outputListeners.delete(sessionId);
  exitListeners.delete(sessionId);
  return invoke('terminal_close', { sessionId, preserve: options?.preserve });
};

export const getSessionHistory = async (sessionId: string) => {
  await ensureListeners();
  buffers.delete(sessionId);
  return invoke<string>('get_session_history', { sessionId });
};

export const subscribeOutput = (sessionId: string, handler: OutputListener) => {
  void ensureListeners();
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
  void ensureListeners();
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

export const onActivity = (handler: ActivityListener) => {
  activityListeners.add(handler);
  return () => activityListeners.delete(handler);
};

export const onStatusChange = (handler: StatusListener) => {
  void ensureListeners();
  statusListeners.add(handler);
  return () => statusListeners.delete(handler);
};
