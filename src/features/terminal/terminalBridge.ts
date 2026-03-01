// 终端桥接层：封装 Tauri IPC 与前端订阅，统一缓冲与 ACK 逻辑。
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { TerminalFriendInviteMeta, TerminalPostReadyMode, TerminalType } from '@/shared/types/terminal';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';
import { createFrontLogger } from '@/shared/monitoring/passiveMonitor';

type OutputPayload = { terminalId: string; data: string; seq: number };
type SnapshotPayload = {
  terminalId: string;
  data: string;
  seq: number;
  history?: string;
  rows?: number;
  cols?: number;
  cursorRow?: number;
  cursorCol?: number;
};
type SnapshotLinesPayload = { terminalId: string; seq: number; lines: string[] };
type ExitPayload = { terminalId: string; code?: number | null; signal?: string | null };
type StatusPayload = { terminalId: string; status: string; memberId?: string; workspaceId?: string };
type ErrorPayload = { terminalId: string; error: string; fatal?: boolean };
export type TerminalChatPayload = {
  terminalId: string;
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
  mode: 'snapshot' | 'delta' | 'stream' | 'final';
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
  messageId?: string;
  clientTraceId?: string;
  timestamp?: number;
};

type OutputListener = (payload: { data: string; seq: number }) => void;
type ExitListener = (payload: ExitPayload) => void;
type ActivityListener = (terminalId: string) => void;
type StatusListener = (payload: StatusPayload) => void;
type ChatListener = (payload: TerminalChatPayload) => void;
type StreamListener = (payload: TerminalChatPayload) => void;
type ErrorListener = (payload: ErrorPayload) => void;

const outputListeners = new Map<string, Set<OutputListener>>();
const exitListeners = new Map<string, Set<ExitListener>>();
const activityListeners = new Set<ActivityListener>();
const statusListeners = new Set<StatusListener>();
const chatListeners = new Set<ChatListener>();
const streamListeners = new Set<StreamListener>();
const errorListeners = new Map<string, Set<ErrorListener>>();
const buffers = new Map<string, OutputPayload[]>();
const trackedSessions = new Set<string>();
const loggedOutputSessions = new Set<string>();
const ackEncoder = new TextEncoder();
const ackBuffers = new Map<string, { pending: number; timer: number | null }>();
const traceLog = createFrontLogger('terminal-trace');
const debugLog = createFrontLogger('terminal-bridge');

// 缓冲与 ACK 参数用于控制输出洪峰与 IPC 频率。
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
    // 忽略格式异常的尺寸缓存，避免影响后续初始化。
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

const pushBuffer = (terminalId: string, payload: OutputPayload) => {
  const queue = buffers.get(terminalId) ?? [];
  queue.push(payload);
  if (queue.length > BUFFER_LIMIT) {
    queue.splice(0, queue.length - BUFFER_LIMIT);
  }
  buffers.set(terminalId, queue);
};

const flushBuffer = (terminalId: string, handler: OutputListener) => {
  const queue = buffers.get(terminalId);
  if (!queue || queue.length === 0) {
    return;
  }
  queue.sort((left, right) => left.seq - right.seq);
  for (const payload of queue) {
    handler({ data: payload.data, seq: payload.seq });
  }
  buffers.delete(terminalId);
};

const queueAck = (terminalId: string, data: string) => {
  if (!data) {
    return;
  }
  const bytes = ackEncoder.encode(data).length;
  if (bytes <= 0) {
    return;
  }
  const entry = ackBuffers.get(terminalId) ?? { pending: 0, timer: null };
  entry.pending += bytes;
  if (entry.pending >= ACK_BATCH_SIZE) {
    const count = entry.pending;
    entry.pending = 0;
    if (entry.timer !== null) {
      window.clearTimeout(entry.timer);
      entry.timer = null;
    }
    traceLog('ack send', { terminalId, count, reason: 'batch' });
    void invoke('terminal_ack', { terminalId, count }).catch(() => {});
    ackBuffers.set(terminalId, entry);
    return;
  }
  if (entry.timer === null) {
    entry.timer = window.setTimeout(() => {
      entry.timer = null;
      if (entry.pending > 0) {
        const count = entry.pending;
        entry.pending = 0;
        traceLog('ack send', { terminalId, count, reason: 'timer' });
        void invoke('terminal_ack', { terminalId, count }).catch(() => {});
      }
      ackBuffers.set(terminalId, entry);
    }, ACK_FLUSH_MS);
  }
  ackBuffers.set(terminalId, entry);
};

const ensureListeners = async () => {
  if (initPromise) {
    return initPromise;
  }
  initPromise = (async () => {
    await listen<OutputPayload>('terminal-output', (event) => {
      const { terminalId, data, seq } = event.payload;
      const receivedAt = performance.now();
      if (!loggedOutputSessions.has(terminalId)) {
        loggedOutputSessions.add(terminalId);
        debugLog('terminal output first', { terminalId, seq, len: data.length });
      }
      const listeners = outputListeners.get(terminalId);
      const hasListeners = Boolean(listeners && listeners.size > 0);
      const isTracked = trackedSessions.has(terminalId);
      traceLog('output event', {
        terminalId,
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
        queueAck(terminalId, data);
        if (isTracked) {
          pushBuffer(terminalId, { terminalId, data, seq });
        } else {
          return;
        }
      }
      if (hasListeners || isTracked) {
        for (const handler of activityListeners) {
          handler(terminalId);
        }
      }
    });

    await listen<ExitPayload>('terminal-exit', (event) => {
      const { terminalId } = event.payload;
      debugLog('terminal exit', event.payload);
      const listeners = exitListeners.get(terminalId);
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

    await listen<TerminalChatPayload>('terminal-message-stream', (event) => {
      for (const handler of streamListeners) {
        handler(event.payload);
      }
    });

    await listen<ErrorPayload>('terminal-error', (event) => {
      console.warn('[terminal-error]', event.payload);
      const listeners = errorListeners.get(event.payload.terminalId);
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

/**
 * 创建终端会话。
 * 输入：可选 cols/rows/cwd/成员与终端参数。
 * 输出：后端会话 id 字符串。
 * 错误语义：Tauri 调用失败会抛出异常。
 */
export const createSession = async (options?: {
  cols?: number;
  rows?: number;
  cwd?: string;
  memberId?: string;
  memberName?: string;
  workspaceId?: string;
  keepAlive?: boolean;
  terminalId?: string;
  terminalType?: TerminalType;
  terminalCommand?: string;
  terminalPath?: string;
  strictShell?: boolean;
  postReadyMode?: TerminalPostReadyMode;
  inviteMeta?: TerminalFriendInviteMeta;
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
    terminalPath: options?.terminalPath,
    strictShell: options?.strictShell
  });
  const inviteMeta = options?.inviteMeta;
  const memberName = options?.memberName ?? inviteMeta?.memberName;
  const postReadyMode = options?.postReadyMode ?? (inviteMeta ? 'invite' : 'none');
  const terminalId = await invoke<string>('terminal_create', {
    cols,
    rows,
    cwd: options?.cwd,
    memberId: options?.memberId,
    workspaceId: options?.workspaceId,
    keepAlive: options?.keepAlive,
    terminalId: options?.terminalId,
    terminalType: options?.terminalType,
    terminalCommand: options?.terminalCommand,
    terminalPath: options?.terminalPath,
    strictShell: options?.strictShell,
    memberName,
    defaultCommand: inviteMeta?.defaultCommand,
    inviteInstanceCount: inviteMeta?.instanceCount,
    inviteUnlimitedAccess: inviteMeta?.unlimitedAccess,
    inviteSandboxed: inviteMeta?.sandboxed,
    postReadyMode
  });
  void logDiagnosticsEvent('create-session', {
    terminalId,
    memberId: options?.memberId,
    workspaceId: options?.workspaceId,
    terminalType: options?.terminalType,
    terminalCommand: options?.terminalCommand,
    terminalPath: options?.terminalPath,
    strictShell: options?.strictShell,
    cols,
    rows,
    keepAlive: options?.keepAlive ?? false
  });
  return terminalId;
};

/**
 * 向会话写入用户输入。
 * 输入：terminalId 与数据字符串。
 * 输出：无。
 */
export const writeSession = async (terminalId: string, data: string) => {
  await ensureListeners();
  return invoke('terminal_write', { terminalId, data });
};

/**
 * 确认已消费的输出字节数，用于后端流控。
 * 输入：terminalId 与字节数。
 * 输出：无。
 */
export const ackSession = async (terminalId: string, count: number) => {
  traceLog('ack send', { terminalId, count, reason: 'explicit' });
  await ensureListeners();
  return invoke('terminal_ack', { terminalId, count });
};

/**
 * 标记会话是否处于 UI 激活状态。
 * 输入：terminalId 与布尔标记。
 * 输出：无。
 */
export const setSessionActive = async (terminalId: string, active: boolean) => {
  await ensureListeners();
  return invoke('terminal_set_active', { terminalId, active });
};

export const emitSessionStatus = async (terminalId: string) => {
  await ensureListeners();
  return invoke('terminal_emit_status', { terminalId });
};

/**
 * 同步成员状态到终端后端。
 * 输入：成员 id 与状态字符串。
 * 输出：无。
 */
export const setMemberStatus = async (memberId: string, status: string) => {
  await ensureListeners();
  return invoke('terminal_set_member_status', { memberId, status });
};

/**
 * 获取指定工作区的会话状态快照。
 * 输入：workspaceId。
 * 输出：状态列表。
 */
export const listSessionStatuses = async (workspaceId?: string) => {
  await ensureListeners();
  return invoke<StatusPayload[]>('terminal_list_statuses', { workspaceId });
};

/**
 * 向终端派发来自聊天的指令并附带上下文。
 * 输入：terminalId、命令文本与上下文信息。
 * 输出：无。
 */
export const dispatchSession = async (
  terminalId: string,
  data: string,
  context: TerminalDispatchContext
) => {
  await ensureListeners();
  void logDiagnosticsEvent('dispatch-session', {
    terminalId,
    conversationId: context.conversationId,
    conversationType: context.conversationType,
    senderId: context.senderId,
    senderName: context.senderName,
    data
  });
  return invoke('terminal_dispatch', { terminalId, data, context });
};

/**
 * 调整终端尺寸并更新本地缓存大小。
 * 输入：terminalId 与列/行数。
 * 输出：无。
 */
export const resizeSession = async (terminalId: string, cols: number, rows: number) => {
  await ensureListeners();
  updateLastKnownSize(cols, rows);
  return invoke('terminal_resize', { terminalId, cols, rows });
};

/**
 * 关闭会话并清理本地监听/缓冲。
 * 输入：terminalId 与可选 preserve 标记。
 * 输出：无。
 */
export const closeSession = async (
  terminalId: string,
  options?: { preserve?: boolean; deleteSessionMap?: boolean }
) => {
  await ensureListeners();
  buffers.delete(terminalId);
  outputListeners.delete(terminalId);
  exitListeners.delete(terminalId);
  trackedSessions.delete(terminalId);
  const ackEntry = ackBuffers.get(terminalId);
  if (ackEntry?.timer !== null && ackEntry?.timer !== undefined) {
    window.clearTimeout(ackEntry.timer);
  }
  ackBuffers.delete(terminalId);
  return invoke('terminal_close', {
    terminalId,
    preserve: options?.preserve,
    deleteSessionMap: options?.deleteSessionMap
  });
};

/**
 * 获取会话快照用于 UI attach。
 * 输入：terminalId。
 * 输出：快照 payload。
 */
export const attachSession = async (terminalId: string) => {
  await ensureListeners();
  return invoke<SnapshotPayload>('terminal_attach', { terminalId });
};

/**
 * 获取会话快照的文本行，便于前端校验屏幕一致性。
 * 输入：terminalId。
 * 输出：文本行快照与序列号。
 */
export const snapshotSessionLines = async (terminalId: string) => {
  await ensureListeners();
  return invoke<SnapshotLinesPayload>('terminal_snapshot_lines', { terminalId });
};

/**
 * 只读获取会话快照文本，避免改写输出窗口路由。
 * 输入：terminalId。
 * 输出：快照 payload。
 */
export const snapshotSessionText = async (terminalId: string) => {
  await ensureListeners();
  return invoke<SnapshotPayload>('terminal_snapshot_text', { terminalId });
};

/**
 * 标记会话为可缓冲输出的追踪对象。
 * 输入：terminalId。
 * 输出：无。
 */
export const trackSession = (terminalId: string) => {
  trackedSessions.add(terminalId);
};

/**
 * 取消会话追踪并清理缓冲。
 * 输入：terminalId。
 * 输出：无。
 */
export const untrackSession = (terminalId: string) => {
  trackedSessions.delete(terminalId);
  buffers.delete(terminalId);
  const ackEntry = ackBuffers.get(terminalId);
  if (ackEntry?.timer !== null && ackEntry?.timer !== undefined) {
    window.clearTimeout(ackEntry.timer);
  }
  ackBuffers.delete(terminalId);
};

/**
 * 订阅终端输出事件。
 * 输入：terminalId 与处理函数。
 * 输出：取消订阅函数。
 */
export const subscribeOutput = (terminalId: string, handler: OutputListener) => {
  void ensureListeners().catch(() => {});
  const listeners = outputListeners.get(terminalId) ?? new Set<OutputListener>();
  listeners.add(handler);
  outputListeners.set(terminalId, listeners);
  flushBuffer(terminalId, handler);
  return () => {
    const current = outputListeners.get(terminalId);
    if (!current) {
      return;
    }
    current.delete(handler);
    if (current.size === 0) {
      outputListeners.delete(terminalId);
    }
  };
};

/**
 * 订阅会话退出事件。
 * 输入：terminalId 与处理函数。
 * 输出：取消订阅函数。
 */
export const subscribeExit = (terminalId: string, handler: ExitListener) => {
  void ensureListeners().catch(() => {});
  const listeners = exitListeners.get(terminalId) ?? new Set<ExitListener>();
  listeners.add(handler);
  exitListeners.set(terminalId, listeners);
  return () => {
    const current = exitListeners.get(terminalId);
    if (!current) {
      return;
    }
    current.delete(handler);
    if (current.size === 0) {
      exitListeners.delete(terminalId);
    }
  };
};

/**
 * 订阅会话错误事件。
 * 输入：terminalId 与处理函数。
 * 输出：取消订阅函数。
 */
export const subscribeError = (terminalId: string, handler: ErrorListener) => {
  void ensureListeners().catch(() => {});
  const listeners = errorListeners.get(terminalId) ?? new Set<ErrorListener>();
  listeners.add(handler);
  errorListeners.set(terminalId, listeners);
  return () => {
    const current = errorListeners.get(terminalId);
    if (!current) {
      return;
    }
    current.delete(handler);
    if (current.size === 0) {
      errorListeners.delete(terminalId);
    }
  };
};

/**
 * 订阅任意会话活动事件。
 * 输入：处理函数。
 * 输出：取消订阅函数。
 */
export const onActivity = (handler: ActivityListener) => {
  activityListeners.add(handler);
  return () => activityListeners.delete(handler);
};

/**
 * 订阅会话状态变更事件。
 * 输入：处理函数。
 * 输出：取消订阅函数。
 */
export const onStatusChange = (handler: StatusListener) => {
  void ensureListeners().catch(() => {});
  statusListeners.add(handler);
  return () => statusListeners.delete(handler);
};

/**
 * 订阅终端聊天输出事件。
 * 输入：处理函数。
 * 输出：取消订阅函数。
 */
export const onChatMessage = (handler: ChatListener) => {
  void ensureListeners().catch(() => {});
  chatListeners.add(handler);
  return () => chatListeners.delete(handler);
};

/**
 * 订阅终端流式输出事件。
 * 输入：处理函数。
 * 输出：取消订阅函数。
 */
export const onTerminalStreamMessage = (handler: StreamListener) => {
  void ensureListeners().catch(() => {});
  streamListeners.add(handler);
  return () => streamListeners.delete(handler);
};

