// 诊断日志桥接：管理前端诊断上下文，并将链路数据缓冲后发送给后端落盘。
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { FrontendLogEntry } from './types';
import {
  diagnosticsEndRun,
  diagnosticsLogChatConsistency,
  diagnosticsLogFrontendBatch,
  diagnosticsLogFrontendEvent,
  diagnosticsLogSnapshotTriplet,
  diagnosticsRegisterConversation,
  diagnosticsRegisterMember,
  diagnosticsRegisterSession,
  diagnosticsRegisterWindow,
  diagnosticsStartRun
} from '../api';
import { isFrontendPassiveEnabled } from '../gates/frontendGate';
import { safeSerialize } from '../utils/safeSerialize';

export type DiagnosticsContext = {
  runId: string;
  round?: number;
  chain?: string;
  memberId?: string;
};

const MAX_BUFFER_SIZE = 5000;
const MAX_BATCH_SIZE = 50;
const FLUSH_INTERVAL_MS = 500;
const MAX_ENTRY_BYTES = 64 * 1024;

let activeContext: DiagnosticsContext | null = null;
let autoDiagnosticsRunId: string | null = null;
let autoDiagnosticsStarting = false;
let buffer: FrontendLogEntry[] = [];
let flushTimer: number | null = null;
let flushInProgress = false;
let lifecycleAttached = false;
let internalGuard = false;
let seqCounter = 0;
let droppedCount = 0;
let droppedRunId: string | null = null;

const isDiagnosticsEnabled = () => isFrontendPassiveEnabled();

const buildAutoRunId = () => `auto-${Date.now()}-${Math.random().toString(16).slice(2)}`;

const ensureAutoDiagnosticsRun = async () => {
  if (!isDiagnosticsEnabled() || autoDiagnosticsRunId || autoDiagnosticsStarting) {
    return;
  }
  autoDiagnosticsStarting = true;
  const runId = buildAutoRunId();
  await diagnosticsStartRun({ runId, label: 'terminal-auto-debug' });
  autoDiagnosticsRunId = runId;
  if (!activeContext) {
    setDiagnosticsContext({ runId });
  }
  autoDiagnosticsStarting = false;
};

const resolveDiagnosticsContext = async () => {
  if (activeContext) {
    return activeContext;
  }
  await ensureAutoDiagnosticsRun();
  if (autoDiagnosticsRunId) {
    return { runId: autoDiagnosticsRunId };
  }
  return null;
};

const computePayloadBytes = (payload: unknown) => {
  try {
    return new TextEncoder().encode(JSON.stringify(payload)).length;
  } catch {
    return 0;
  }
};

const buildSafePayload = (payload: Record<string, unknown>) => {
  const safePayload = safeSerialize(payload);
  const bytes = computePayloadBytes(safePayload);
  if (bytes > MAX_ENTRY_BYTES) {
    return {
      truncated: true,
      bytes,
      preview: JSON.stringify(safePayload).slice(0, 2000)
    };
  }
  return safePayload;
};

const enqueueDropEntry = () => {
  if (!droppedRunId || droppedCount <= 0) {
    return;
  }
  buffer.unshift({
    runId: droppedRunId,
    stepId: 'monitoring.buffer-drop',
    payload: { dropped: droppedCount },
    clientTs: Date.now(),
    seq: ++seqCounter
  });
  droppedCount = 0;
  droppedRunId = null;
};

const enqueueEntry = (entry: FrontendLogEntry) => {
  buffer.push(entry);
  if (buffer.length > MAX_BUFFER_SIZE) {
    const overflow = buffer.length - MAX_BUFFER_SIZE;
    buffer.splice(0, overflow);
    droppedCount += overflow;
    droppedRunId = droppedRunId ?? entry.runId;
  }
  if (buffer.length >= MAX_BATCH_SIZE) {
    void flushBuffer('batch');
    return;
  }
  scheduleFlush();
};

const scheduleFlush = () => {
  if (flushTimer !== null) {
    return;
  }
  flushTimer = window.setTimeout(() => {
    flushTimer = null;
    void flushBuffer('timer');
  }, FLUSH_INTERVAL_MS);
};

const flushBuffer = async (reason: string) => {
  if (flushInProgress) {
    return;
  }
  if (buffer.length === 0) {
    return;
  }
  flushInProgress = true;
  if (flushTimer !== null) {
    window.clearTimeout(flushTimer);
    flushTimer = null;
  }
  enqueueDropEntry();
  const batch = buffer.splice(0, buffer.length);
  for (let index = 0; index < batch.length; index += MAX_BATCH_SIZE) {
    const slice = batch.slice(index, index + MAX_BATCH_SIZE);
    const ok = await diagnosticsLogFrontendBatch(slice);
    if (!ok) {
      buffer = slice.concat(buffer);
      break;
    }
  }
  flushInProgress = false;
  if (buffer.length > 0) {
    scheduleFlush();
  }
  void reason;
};

const logInternalWarning = (message: string, error?: unknown) => {
  if (internalGuard) {
    return;
  }
  internalGuard = true;
  console.warn(message, error);
  internalGuard = false;
};

export const initializeMonitoring = () => {
  if (lifecycleAttached || typeof window === 'undefined') {
    return;
  }
  lifecycleAttached = true;
  window.addEventListener('beforeunload', () => {
    void flushBuffer('beforeunload');
  });
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'hidden') {
      void flushBuffer('visibility');
    }
  });
  try {
    void getCurrentWebviewWindow();
  } catch (error) {
    logInternalWarning('Failed to attach close-requested flush.', error);
  }
};

export const setDiagnosticsContext = (next: DiagnosticsContext | null) => {
  activeContext = next;
};

export const updateDiagnosticsContext = (patch: Partial<DiagnosticsContext>) => {
  if (!activeContext) {
    return;
  }
  activeContext = { ...activeContext, ...patch };
};

export const clearDiagnosticsContext = () => {
  activeContext = null;
};

export const startDiagnosticsRun = async (payload: {
  runId: string;
  workspaceId?: string;
  label?: string;
}) => {
  if (!isDiagnosticsEnabled()) {
    return;
  }
  await diagnosticsStartRun(payload);
  setDiagnosticsContext({ runId: payload.runId });
};

export const endDiagnosticsRun = async (status?: string) => {
  if (!isDiagnosticsEnabled()) {
    clearDiagnosticsContext();
    return;
  }
  if (!activeContext) {
    clearDiagnosticsContext();
    return;
  }
  await diagnosticsEndRun({ runId: activeContext.runId, status });
  clearDiagnosticsContext();
  if (isDiagnosticsEnabled()) {
    if (autoDiagnosticsRunId) {
      setDiagnosticsContext({ runId: autoDiagnosticsRunId });
    } else {
      await ensureAutoDiagnosticsRun();
    }
  }
};

export const logDiagnosticsEvent = async (
  stepId: string,
  payload: Record<string, unknown>,
  options?: { memberId?: string; round?: number; runId?: string; chain?: string }
) => {
  if (!isDiagnosticsEnabled() || internalGuard) {
    return;
  }
  const context = await resolveDiagnosticsContext();
  const runId = options?.runId ?? context?.runId;
  if (!runId) {
    return;
  }
  const round = options?.round ?? context?.round;
  const memberId =
    options?.memberId ??
    context?.memberId ??
    (typeof payload.memberId === 'string' ? payload.memberId : undefined);
  const chain = options?.chain ?? context?.chain;
  const step = chain ? `${chain}.${stepId}` : stepId;
  const entry: FrontendLogEntry = {
    runId,
    stepId: step,
    payload: buildSafePayload(payload),
    round,
    memberId,
    clientTs: Date.now(),
    seq: ++seqCounter
  };
  enqueueEntry(entry);
};

export const registerDiagnosticsMember = async (payload: {
  runId?: string;
  memberId: string;
  terminalType?: string | null;
  terminalCommand?: string | null;
  name?: string | null;
}) => {
  if (!isDiagnosticsEnabled()) {
    return;
  }
  const context = await resolveDiagnosticsContext();
  const runId = payload.runId ?? context?.runId;
  if (!runId) {
    return;
  }
  await diagnosticsRegisterMember({
    runId,
    memberId: payload.memberId,
    terminalType: payload.terminalType ?? undefined,
    terminalCommand: payload.terminalCommand ?? undefined,
    name: payload.name ?? undefined
  });
};

export const registerDiagnosticsSession = async (payload: {
  runId?: string;
  terminalId: string;
  memberId?: string | null;
}) => {
  if (!isDiagnosticsEnabled()) {
    return;
  }
  const context = await resolveDiagnosticsContext();
  const runId = payload.runId ?? context?.runId;
  if (!runId) {
    return;
  }
  await diagnosticsRegisterSession({
    runId,
    terminalId: payload.terminalId,
    memberId: payload.memberId ?? undefined
  });
};

export const registerDiagnosticsConversation = async (payload: {
  runId?: string;
  conversationId: string;
  memberId?: string | null;
}) => {
  if (!isDiagnosticsEnabled()) {
    return;
  }
  const context = await resolveDiagnosticsContext();
  const runId = payload.runId ?? context?.runId;
  if (!runId) {
    return;
  }
  await diagnosticsRegisterConversation({
    runId,
    conversationId: payload.conversationId,
    memberId: payload.memberId ?? undefined
  });
};

export const registerDiagnosticsWindow = async (payload: { runId?: string; windowLabel: string }) => {
  if (!isDiagnosticsEnabled()) {
    return;
  }
  const context = await resolveDiagnosticsContext();
  const runId = payload.runId ?? context?.runId;
  if (!runId) {
    return;
  }
  await diagnosticsRegisterWindow({
    runId,
    windowLabel: payload.windowLabel
  });
};

export const logSnapshotTriplet = async (payload: {
  round: number;
  memberId: string;
  frontBefore: string[];
  backendStored: string[];
  frontReopen: string[];
}) => {
  if (!isDiagnosticsEnabled()) {
    return;
  }
  const context = await resolveDiagnosticsContext();
  if (!context?.runId) {
    return;
  }
  await diagnosticsLogSnapshotTriplet({
    runId: context.runId,
    round: payload.round,
    memberId: payload.memberId,
    frontBefore: payload.frontBefore,
    backendStored: payload.backendStored,
    frontReopen: payload.frontReopen
  });
};

export const logChatConsistency = async (payload: {
  round: number;
  memberId: string;
  replyText?: string | null;
  replyMissing?: boolean;
  terminalLines: string[];
}) => {
  if (!isDiagnosticsEnabled()) {
    return;
  }
  const context = await resolveDiagnosticsContext();
  if (!context?.runId) {
    return;
  }
  await diagnosticsLogChatConsistency({
    runId: context.runId,
    round: payload.round,
    memberId: payload.memberId,
    replyText: payload.replyText ?? null,
    replyMissing: payload.replyMissing ?? false,
    terminalLines: payload.terminalLines
  });
};

// 供模块内调试使用的单条发送（保持兼容）。
export const logDiagnosticsEventNow = async (
  stepId: string,
  payload: Record<string, unknown>,
  options?: { memberId?: string; round?: number; runId?: string; chain?: string }
) => {
  if (!isDiagnosticsEnabled()) {
    return;
  }
  const context = await resolveDiagnosticsContext();
  const runId = options?.runId ?? context?.runId;
  if (!runId) {
    return;
  }
  const round = options?.round ?? context?.round;
  const memberId =
    options?.memberId ??
    context?.memberId ??
    (typeof payload.memberId === 'string' ? payload.memberId : undefined);
  const chain = options?.chain ?? context?.chain;
  const step = chain ? `${chain}.${stepId}` : stepId;
  await diagnosticsLogFrontendEvent({
    runId,
    round,
    memberId,
    stepId: step,
    payload: buildSafePayload(payload),
    clientTs: Date.now(),
    seq: ++seqCounter
  });
};
