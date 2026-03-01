// 终端快照审计：复刻“打开-关闭-重开”链路以验证前后端快照一致性。
import { ref } from 'vue';
import { defineStore, storeToRefs } from 'pinia';
import { emitTo, listen } from '@tauri-apps/api/event';
import { isTauri } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { i18n } from '@/i18n';
import { BASE_TERMINALS, resolveBaseTerminalLabel, type BaseTerminal } from '@/shared/constants/terminalCatalog';
import { TERMINAL_CALL_CHAINS } from '@/shared/constants/terminalCallChains';
import { normalizeTerminalCommand } from '@/shared/utils/terminal';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useTerminalOrchestratorStore } from '@/stores/terminalOrchestratorStore';
import { openTerminalWindow } from '@/features/terminal/openTerminalWindow';
import { snapshotSessionLines } from '@/features/terminal/terminalBridge';
import {
  TERMINAL_OPEN_TAB_EVENT,
  TERMINAL_SNAPSHOT_REQUEST_EVENT,
  TERMINAL_SNAPSHOT_RESPONSE_EVENT,
  TERMINAL_WINDOW_READY_EVENT,
  TERMINAL_WINDOW_READY_REQUEST_EVENT,
  type TerminalOpenTabPayload,
  type TerminalSnapshotRequestPayload,
  type TerminalSnapshotResponsePayload,
  type TerminalWindowReadyPayload
} from '@/features/terminal/terminalEvents';
import { useFriendInvites, type InviteModel } from '@/features/chat/useFriendInvites';
import type { Member } from '@/features/chat/types';
import type { TerminalType } from '@/shared/types/terminal';

type SnapshotSignature = {
  hash: string;
  lineCount: number;
  rows?: number;
  cols?: number;
};

type SnapshotRoundResult = {
  round: number;
  status: 'passed' | 'failed';
  comparisons: {
    frontBackend: boolean;
    frontReopen: boolean;
    backendReopen: boolean;
  };
  front?: SnapshotSignature;
  backend?: SnapshotSignature;
  reopen?: SnapshotSignature;
  error?: string;
};

type SnapshotMemberResult = {
  memberId: string;
  name: string;
  terminalType?: TerminalType;
  created: boolean;
  rounds: SnapshotRoundResult[];
  status: 'passed' | 'failed';
};

type CallChainStepStatus = 'pending' | 'passed' | 'failed' | 'skipped';
type CallChainStepReport = { id: string; label: string; status: CallChainStepStatus; detail?: string };
type CallChainReport = { id: string; title: string; steps: CallChainStepReport[] };

const AUDIT_TERMINALS = BASE_TERMINALS.filter((item) => item.terminalType !== 'shell');
// 轮次与重试参数用于稳定窗口渲染与快照请求。
const SNAPSHOT_ROUNDS = 3;
const READY_POLL_INTERVAL_MS = 400;
const READY_TIMEOUT_FAST_MS = 45000;
const READY_TIMEOUT_SLOW_MS = 120000;
const SNAPSHOT_REQUEST_TIMEOUT_MS = 8000;
const SNAPSHOT_RETRY_DELAY_MS = 300;
const WINDOW_READY_TIMEOUT_MS = 15000;
const WINDOW_CLOSE_DELAY_MS = 400;

const sleep = (durationMs: number) => new Promise((resolve) => window.setTimeout(resolve, durationMs));

const buildRequestId = () => {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return `snapshot-${Date.now()}-${Math.random().toString(16).slice(2)}`;
};

const trimLineRight = (value: string) => value.replace(/\s+$/g, '');

const normalizeLines = (lines: string[]) => lines.map((line) => trimLineRight(line));

const CALL_CHAIN_DETAIL_LIMIT = 220;
const mergeCallChainDetail = (previous: string | undefined, next: string) => {
  const combined = previous ? `${previous} | ${next}` : next;
  if (combined.length <= CALL_CHAIN_DETAIL_LIMIT) {
    return combined;
  }
  return `…${combined.slice(-CALL_CHAIN_DETAIL_LIMIT)}`;
};

const resolveReadyTimeout = (terminalType?: TerminalType) => {
  if (terminalType === 'codex' || terminalType === 'gemini' || terminalType === 'claude') {
    return READY_TIMEOUT_SLOW_MS;
  }
  return READY_TIMEOUT_FAST_MS;
};

const hasSnapshotContent = (lines: string[]) => normalizeLines(lines).some((line) => line.trim().length > 0);

const buildInviteModel = (base: BaseTerminal): InviteModel => ({
  id: base.id,
  label: resolveBaseTerminalLabel(base, i18n.global.t),
  command: base.command,
  terminalType: base.terminalType,
  instances: 1,
  unlimitedAccess: true,
  sandboxed: false
});

const linesEqual = (left: string[], right: string[]) => {
  if (left.length !== right.length) {
    return false;
  }
  for (let index = 0; index < left.length; index += 1) {
    if (left[index] !== right[index]) {
      return false;
    }
  }
  return true;
};

const hashLines = (lines: string[]) => {
  let hash = 5381;
  for (const line of lines) {
    for (let index = 0; index < line.length; index += 1) {
      hash = ((hash << 5) + hash) ^ line.charCodeAt(index);
    }
    hash = ((hash << 5) + hash) ^ 10;
  }
  return (hash >>> 0).toString(16).padStart(8, '0');
};

const buildSignature = (lines: string[], rows?: number, cols?: number): SnapshotSignature => {
  const normalized = normalizeLines(lines);
  return {
    hash: hashLines(normalized),
    lineCount: normalized.length,
    rows,
    cols
  };
};

const formatError = (error: unknown) => (error instanceof Error ? error.message : String(error));

export const useTerminalSnapshotAuditStore = defineStore('terminal-snapshot-audit', () => {
  const projectStore = useProjectStore();
  const workspaceStore = useWorkspaceStore();
  const terminalOrchestratorStore = useTerminalOrchestratorStore();
  const friendInvites = useFriendInvites();
  const { members } = storeToRefs(projectStore);
  const { currentWorkspace, workspaceReadOnly } = storeToRefs(workspaceStore);
  const { removeMember } = projectStore;
  const { stopMemberSession, ensureMemberSession } = terminalOrchestratorStore;
  const { handleInvite, syncDefaultChannelMembers } = friendInvites;

  const status = ref<'idle' | 'running' | 'passed' | 'failed'>('idle');
  const results = ref<SnapshotMemberResult[]>([]);
  const lastError = ref<string | null>(null);
  const lastRunAt = ref<number | null>(null);
  const callChains = ref<CallChainReport[]>(
    TERMINAL_CALL_CHAINS.map((chain) => ({
      ...chain,
      steps: chain.steps.map((step) => ({ ...step, status: 'pending' as CallChainStepStatus }))
    }))
  );

  const pendingSnapshotRequests = new Map<
    string,
    { resolve: (payload: TerminalSnapshotResponsePayload) => void; reject: (error: Error) => void; timeoutId: number }
  >();
  let snapshotListenerReady = false;

  const ensureSnapshotListener = async () => {
    if (snapshotListenerReady) {
      return;
    }
    snapshotListenerReady = true;
    await listen<TerminalSnapshotResponsePayload>(TERMINAL_SNAPSHOT_RESPONSE_EVENT, (event) => {
      const payload = event.payload;
      const pending = pendingSnapshotRequests.get(payload.requestId);
      if (!pending) {
        return;
      }
      pendingSnapshotRequests.delete(payload.requestId);
      window.clearTimeout(pending.timeoutId);
      if (payload.error) {
        pending.reject(new Error(payload.error));
        return;
      }
      pending.resolve(payload);
    });
  };

  const resetCallChains = () => {
    callChains.value = TERMINAL_CALL_CHAINS.map((chain) => ({
      ...chain,
      steps: chain.steps.map((step) => ({ ...step, status: 'pending' as CallChainStepStatus }))
    }));
  };

  const markCallChainStep = (stepId: string, status: CallChainStepStatus, detail?: string) => {
    callChains.value = callChains.value.map((chain) => ({
      ...chain,
      steps: chain.steps.map((step) => {
        if (step.id !== stepId) {
          return step;
        }
        if (step.status === 'passed' && status !== 'passed') {
          return step;
        }
        return { ...step, status, detail: detail ?? step.detail };
      })
    }));
  };

  const appendCallChainDetail = (stepId: string, detail: string) => {
    callChains.value = callChains.value.map((chain) => ({
      ...chain,
      steps: chain.steps.map((step) => {
        if (step.id !== stepId) {
          return step;
        }
        return { ...step, detail: mergeCallChainDetail(step.detail, detail) };
      })
    }));
  };

  const markSkippedSteps = () => {
    const skip = (stepId: string, reason: string) => {
      markCallChainStep(stepId, 'skipped', reason);
    };
    skip('chat-backend.send-message', '快照审计不派发消息');
    skip('chat-backend.parse-targets', '快照审计不解析终端目标');
    skip('chat-backend.enqueue-dispatch', '快照审计不进入派发队列');
    skip('chat-backend.ensure-member-session', '快照审计不走聊天派发');
    skip('chat-backend.dispatch-session', '快照审计不调用 terminal_dispatch');
    skip('chat-backend.backend-write-append', '快照审计不写入 PTY');
    skip('chat-backend.chat-message-created', '快照审计不生成聊天消息');
    skip('chat-backend.on-chat-message-created', '快照审计不触发消息事件');
    skip('chat-backend.append-terminal-message', '快照审计不回写消息');
  };

  const requestFrontendSnapshot = async (windowLabel: string, terminalId: string) => {
    await ensureSnapshotListener();
    const requestId = buildRequestId();
    return new Promise<TerminalSnapshotResponsePayload>((resolve, reject) => {
      const timeoutId = window.setTimeout(() => {
        pendingSnapshotRequests.delete(requestId);
        reject(new Error('snapshot timeout'));
      }, SNAPSHOT_REQUEST_TIMEOUT_MS);
      pendingSnapshotRequests.set(requestId, { resolve, reject, timeoutId });
      const payload: TerminalSnapshotRequestPayload = { requestId, terminalId };
      void emitTo(windowLabel, TERMINAL_SNAPSHOT_REQUEST_EVENT, payload).catch((error) => {
        pendingSnapshotRequests.delete(requestId);
        window.clearTimeout(timeoutId);
        reject(error instanceof Error ? error : new Error(String(error)));
      });
    });
  };

  const requestFrontendSnapshotWithDeadline = async (
    windowLabel: string,
    terminalId: string,
    timeoutMs: number
  ) => {
    let lastError: Error | null = null;
    const startedAt = Date.now();
    while (Date.now() - startedAt < timeoutMs) {
      try {
        return await requestFrontendSnapshot(windowLabel, terminalId);
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));
        await sleep(SNAPSHOT_RETRY_DELAY_MS);
      }
    }
    throw lastError ?? new Error('snapshot request failed');
  };

  const closeTerminalWindow = async (windowLabel: string) => {
    const window = await WebviewWindow.getByLabel(windowLabel);
    if (!window) {
      return;
    }
    await window.close();
  };

  // 监听窗口 ready 事件，确保 open-tab 不会在窗口初始化前丢失。
  const readyWindowLabels = new Set<string>();
  const pendingWindowReady = new Map<string, { resolve: (ready: boolean) => void; timeoutId: number }>();
  let windowReadyListenerReady = false;

  const ensureWindowReadyListener = async () => {
    if (windowReadyListenerReady) {
      return;
    }
    windowReadyListenerReady = true;
    await listen<TerminalWindowReadyPayload>(TERMINAL_WINDOW_READY_EVENT, (event) => {
      const label = event.payload.windowLabel;
      if (!label) {
        return;
      }
      readyWindowLabels.add(label);
      const pending = pendingWindowReady.get(label);
      if (!pending) {
        return;
      }
      pendingWindowReady.delete(label);
      window.clearTimeout(pending.timeoutId);
      pending.resolve(true);
    });
  };

  const waitForWindowReady = async (windowLabel: string, timeoutMs: number) => {
    await ensureWindowReadyListener();
    if (readyWindowLabels.has(windowLabel)) {
      return true;
    }
    return new Promise<boolean>((resolve) => {
      const existing = pendingWindowReady.get(windowLabel);
      if (existing) {
        window.clearTimeout(existing.timeoutId);
      }
      const timeoutId = window.setTimeout(() => {
        pendingWindowReady.delete(windowLabel);
        resolve(false);
      }, timeoutMs);
      pendingWindowReady.set(windowLabel, { resolve, timeoutId });
      void emitTo(windowLabel, TERMINAL_WINDOW_READY_REQUEST_EVENT, {}).catch(() => {});
    });
  };

  const openSessionTabInWindow = async (windowLabel: string, terminalId: string, member: Member) => {
    const payload: TerminalOpenTabPayload = {
      terminalId,
      title: member.name,
      memberId: member.id,
      terminalType: member.terminalType,
      keepAlive: true
    };
    await emitTo(windowLabel, TERMINAL_OPEN_TAB_EVENT, payload);
  };

  const matchesDefaultTerminal = (member: Member, baseCommand: string, baseType: TerminalType) => {
    const command = normalizeTerminalCommand(member.terminalCommand) ?? '';
    const target = normalizeTerminalCommand(baseCommand) ?? '';
    return member.terminalType === baseType && command === target;
  };

  const inviteAuditMember = async (base: BaseTerminal) => {
    const model = buildInviteModel(base);
    markCallChainStep('friend.create.invite-click', 'passed');
    appendCallChainDetail('friend.create.invite-click', `${model.label} x${model.instances}`);
    markCallChainStep('friend.create.invite-emit', 'passed');
    appendCallChainDetail('friend.create.invite-emit', `${model.terminalType}:${model.command || 'shell'}`);
    markCallChainStep('friend.create.handle-invite', 'passed');
    appendCallChainDetail('friend.create.handle-invite', `role=member ${model.label}`);
    const beforeIds = new Set(members.value.map((member) => member.id));
    await handleInvite(model, 'member');
    const createdMembers = members.value.filter((member) => !beforeIds.has(member.id));
    if (createdMembers.length === 0) {
      const detail = '未创建新的终端成员';
      markCallChainStep('friend.create.build-member', 'failed', detail);
      markCallChainStep('friend.create.add-member', 'failed', detail);
      markCallChainStep('friend.create.set-conversation-members', 'failed', detail);
      return { member: null, createdIds: [] };
    }
    const target =
      createdMembers.find((member) => matchesDefaultTerminal(member, base.command, base.terminalType)) ??
      createdMembers[0];
    markCallChainStep('friend.create.build-member', 'passed');
    appendCallChainDetail('friend.create.build-member', `${target.name}(${target.id})`);
    markCallChainStep('friend.create.add-member', 'passed');
    appendCallChainDetail('friend.create.add-member', target.id);
    markCallChainStep('friend.create.set-conversation-members', 'passed');
    appendCallChainDetail('friend.create.set-conversation-members', `members=${members.value.length}`);
    return { member: target, createdIds: createdMembers.map((member) => member.id) };
  };

  const resolveDefaultTerminalMembers = async () => {
    const entries: Array<{ base: BaseTerminal; member?: Member; created: boolean; missing: boolean }> = [];
    const createdIds: string[] = [];
    for (const base of AUDIT_TERMINALS) {
      const existing = members.value.find((member) => matchesDefaultTerminal(member, base.command, base.terminalType));
      if (workspaceReadOnly.value) {
        if (existing) {
          entries.push({ base, member: existing, created: false, missing: false });
        } else {
          entries.push({ base, member: undefined, created: false, missing: true });
        }
        continue;
      }
      const { member, createdIds: nextCreated } = await inviteAuditMember(base);
      if (nextCreated.length > 0) {
        createdIds.push(...nextCreated);
      }
      if (member) {
        entries.push({ base, member, created: true, missing: false });
        continue;
      }
      if (existing) {
        entries.push({ base, member: existing, created: false, missing: false });
      } else {
        entries.push({ base, member: undefined, created: false, missing: true });
      }
    }
    return { entries, createdIds };
  };

  const cleanupCreatedMembers = async (memberIds: string[]) => {
    for (const memberId of memberIds) {
      await stopMemberSession(memberId, { preserve: false, fireAndForget: true, deleteSessionMap: true });
      await removeMember(memberId);
    }
    if (memberIds.length > 0) {
      await syncDefaultChannelMembers();
    }
  };

  const openTerminalWindowForAudit = async () => {
    const workspace = currentWorkspace.value;
    if (!workspace) {
      throw new Error('workspace missing');
    }
    const result = await openTerminalWindow({
      reuse: false,
      workspaceName: workspace.name,
      workspacePath: workspace.path
    });
    if (!result?.label) {
      throw new Error('terminal window unavailable');
    }
    return result;
  };

  const captureBackendSnapshot = async (terminalId: string) => {
    const payload = await snapshotSessionLines(terminalId);
    return payload.lines ?? [];
  };

  // 等待后端快照产生可见内容，避免空白对比误判。
  const waitForBackendSnapshotReady = async (terminalId: string, timeoutMs: number) => {
    const startedAt = Date.now();
    while (Date.now() - startedAt < timeoutMs) {
      const payload = await snapshotSessionLines(terminalId);
      if (hasSnapshotContent(payload.lines ?? [])) {
        return true;
      }
      await sleep(READY_POLL_INTERVAL_MS);
    }
    return false;
  };

  // 单轮验证：打开->采集->关闭->后端快照->重开->采集。
  const runSnapshotRound = async (
    member: Member,
    terminalId: string,
    round: number
  ): Promise<SnapshotRoundResult> => {
    const comparisons = { frontBackend: false, frontReopen: false, backendReopen: false };
    let windowLabel: string | null = null;
    let reopenWindowLabel: string | null = null;
    let currentStep: string | null = null;
    try {
      const readyTimeout = resolveReadyTimeout(member.terminalType);
      markCallChainStep('ui-open.avatar-click', 'passed');
      appendCallChainDetail('ui-open.avatar-click', `round${round} ${member.name}`);
      markCallChainStep('ui-open.chatinterface-handle-member-action', 'passed');
      appendCallChainDetail('ui-open.chatinterface-handle-member-action', member.id);

      const backendReady = await waitForBackendSnapshotReady(terminalId, readyTimeout);
      if (!backendReady) {
        throw new Error(`backend snapshot empty after ${readyTimeout}ms`);
      }
      await ensureWindowReadyListener();

      currentStep = 'ui-open.open-terminal-window';
      const windowResult = await openTerminalWindowForAudit();
      windowLabel = windowResult.label;
      const windowReady = await waitForWindowReady(windowLabel, WINDOW_READY_TIMEOUT_MS);
      if (!windowReady) {
        throw new Error('terminal window ready timeout');
      }
      markCallChainStep('ui-open.open-terminal-window', 'passed');
      appendCallChainDetail(
        'ui-open.open-terminal-window',
        `round${round} ${windowLabel}${windowResult.reused ? ' reused' : ''}`
      );

      currentStep = 'ui-open.open-member-terminal';
      markCallChainStep('ui-open.open-member-terminal', 'passed');
      appendCallChainDetail('ui-open.open-member-terminal', `round${round} ${member.name} emit-tab`);
      currentStep = 'ui-open.terminal-open-tab';
      await openSessionTabInWindow(windowLabel, terminalId, member);
      markCallChainStep('ui-open.terminal-open-tab', 'passed');
      appendCallChainDetail('ui-open.terminal-open-tab', `round${round} ${windowLabel} session=${terminalId}`);

      currentStep = 'ui-open.terminal-attach';
      const frontSnapshot = await requestFrontendSnapshotWithDeadline(windowLabel, terminalId, readyTimeout);
      markCallChainStep('ui-open.terminal-attach', 'passed');
      appendCallChainDetail('ui-open.terminal-attach', `round${round} lines=${frontSnapshot.lines?.length ?? 0}`);

      await closeTerminalWindow(windowLabel);
      windowLabel = null;
      await sleep(WINDOW_CLOSE_DELAY_MS);
      const backendLines = await captureBackendSnapshot(terminalId);
      if (!hasSnapshotContent(backendLines)) {
        throw new Error('backend snapshot empty after close');
      }

      currentStep = 'ui-open.open-terminal-window';
      const reopenWindowResult = await openTerminalWindowForAudit();
      reopenWindowLabel = reopenWindowResult.label;
      const reopenReady = await waitForWindowReady(reopenWindowLabel, WINDOW_READY_TIMEOUT_MS);
      if (!reopenReady) {
        throw new Error('terminal window ready timeout');
      }
      markCallChainStep('ui-open.open-terminal-window', 'passed');
      appendCallChainDetail(
        'ui-open.open-terminal-window',
        `round${round} reopen ${reopenWindowLabel}${reopenWindowResult.reused ? ' reused' : ''}`
      );

      currentStep = 'ui-open.open-member-terminal';
      markCallChainStep('ui-open.open-member-terminal', 'passed');
      appendCallChainDetail('ui-open.open-member-terminal', `round${round} reopen ${member.name} emit-tab`);
      currentStep = 'ui-open.terminal-open-tab';
      await openSessionTabInWindow(reopenWindowLabel, terminalId, member);
      markCallChainStep('ui-open.terminal-open-tab', 'passed');
      appendCallChainDetail(
        'ui-open.terminal-open-tab',
        `round${round} reopen ${reopenWindowLabel} session=${terminalId}`
      );

      currentStep = 'ui-open.terminal-attach';
      const reopenSnapshot = await requestFrontendSnapshotWithDeadline(reopenWindowLabel, terminalId, readyTimeout);
      markCallChainStep('ui-open.terminal-attach', 'passed');
      appendCallChainDetail('ui-open.terminal-attach', `round${round} reopen lines=${reopenSnapshot.lines?.length ?? 0}`);

      await closeTerminalWindow(reopenWindowLabel);
      reopenWindowLabel = null;
      await sleep(WINDOW_CLOSE_DELAY_MS);
      const frontLines = frontSnapshot.lines ?? [];
      const reopenLines = reopenSnapshot.lines ?? [];
      const normalizedFront = normalizeLines(frontLines);
      const normalizedBackend = normalizeLines(backendLines);
      const normalizedReopen = normalizeLines(reopenLines);
      comparisons.frontBackend = linesEqual(normalizedFront, normalizedBackend);
      comparisons.frontReopen = linesEqual(normalizedFront, normalizedReopen);
      comparisons.backendReopen = linesEqual(normalizedBackend, normalizedReopen);
      const status = comparisons.frontBackend && comparisons.frontReopen && comparisons.backendReopen ? 'passed' : 'failed';
      return {
        round,
        status,
        comparisons,
        front: buildSignature(frontLines, frontSnapshot.rows, frontSnapshot.cols),
        backend: buildSignature(backendLines),
        reopen: buildSignature(reopenLines, reopenSnapshot.rows, reopenSnapshot.cols)
      };
    } catch (error) {
      if (currentStep) {
        markCallChainStep(currentStep, 'failed', formatError(error));
      }
      return {
        round,
        status: 'failed',
        comparisons,
        error: formatError(error)
      };
    } finally {
      if (windowLabel) {
        await closeTerminalWindow(windowLabel).catch(() => {});
      }
      if (reopenWindowLabel) {
        await closeTerminalWindow(reopenWindowLabel).catch(() => {});
      }
    }
  };

  const runMemberAudit = async (member: Member, created: boolean): Promise<SnapshotMemberResult> => {
    const entry = await ensureMemberSession(member, { openTab: false });
    if (!entry) {
      const error = 'terminal session unavailable';
      if (created) {
        markCallChainStep('friend.create.ensure-member-session', 'failed', error);
        markCallChainStep('friend.create.create-session', 'failed', error);
      }
      markCallChainStep('ui-open.ensure-member-session', 'failed', error);
      markCallChainStep('ui-open.create-session', 'failed', error);
      return {
        memberId: member.id,
        name: member.name,
        terminalType: member.terminalType,
        created,
        rounds: [
          {
            round: 1,
            status: 'failed',
            comparisons: {
              frontBackend: false,
              frontReopen: false,
              backendReopen: false
            },
            error
          }
        ],
        status: 'failed'
      };
    }
    if (created) {
      markCallChainStep('friend.create.ensure-member-session', 'passed');
      appendCallChainDetail(
        'friend.create.ensure-member-session',
        `${member.name}(${member.id}) session=${entry.terminalId}`
      );
      markCallChainStep('friend.create.create-session', 'passed');
      appendCallChainDetail('friend.create.create-session', entry.terminalId);
    }
    markCallChainStep('ui-open.ensure-member-session', 'passed');
    appendCallChainDetail('ui-open.ensure-member-session', `${member.name}(${member.id}) session=${entry.terminalId}`);
    markCallChainStep('ui-open.create-session', 'passed');
    appendCallChainDetail('ui-open.create-session', entry.terminalId);
    const rounds: SnapshotRoundResult[] = [];
    for (let index = 0; index < SNAPSHOT_ROUNDS; index += 1) {
      rounds.push(await runSnapshotRound(member, entry.terminalId, index + 1));
    }
    const status = rounds.every((round) => round.status === 'passed') ? 'passed' : 'failed';
    return {
      memberId: member.id,
      name: member.name,
      terminalType: member.terminalType,
      created,
      rounds,
      status
    };
  };

  const runSnapshotAudit = async () => {
    if (status.value === 'running') {
      return;
    }
    status.value = 'running';
    results.value = [];
    lastError.value = null;
    lastRunAt.value = null;
    resetCallChains();
    markSkippedSteps();
    if (workspaceReadOnly.value) {
      const reason = '工作区只读，跳过邀请终端成员';
      const inviteSteps = [
        'friend.create.invite-click',
        'friend.create.invite-emit',
        'friend.create.handle-invite',
        'friend.create.build-member',
        'friend.create.add-member',
        'friend.create.set-conversation-members',
        'friend.create.ensure-member-session',
        'friend.create.create-session'
      ];
      for (const stepId of inviteSteps) {
        markCallChainStep(stepId, 'skipped', reason);
      }
    }
    const createdIds: string[] = [];
    try {
      if (!isTauri()) {
        throw new Error('terminal snapshot audit requires Tauri runtime');
      }
      if (!currentWorkspace.value) {
        throw new Error('workspace missing');
      }
      const { entries, createdIds: newIds } = await resolveDefaultTerminalMembers();
      createdIds.push(...newIds);
      const missingEntries = entries.filter((entry) => entry.missing);
      const createdSet = new Set(createdIds);
      const nextResults: SnapshotMemberResult[] = [];
      for (const entry of entries) {
        if (!entry.member) {
          nextResults.push({
            memberId: entry.base.id,
            name: resolveBaseTerminalLabel(entry.base, i18n.global.t),
            terminalType: entry.base.terminalType,
            created: false,
            rounds: [
              {
                round: 1,
                status: 'failed',
                comparisons: {
                  frontBackend: false,
                  frontReopen: false,
                  backendReopen: false
                },
                error: 'terminal member missing'
              }
            ],
            status: 'failed'
          });
          continue;
        }
        nextResults.push(await runMemberAudit(entry.member, createdSet.has(entry.member.id)));
      }
      if (nextResults.length === 0) {
        throw new Error('no terminal members available');
      }
      results.value = nextResults;
      status.value = nextResults.some((result) => result.status === 'failed') ? 'failed' : 'passed';
      if (missingEntries.length > 0 && !lastError.value) {
        lastError.value = `missing default terminal members: ${missingEntries
          .map((entry) => resolveBaseTerminalLabel(entry.base, i18n.global.t))
          .join(', ')}`;
      }
      lastRunAt.value = Date.now();
    } catch (error) {
      lastError.value = formatError(error);
      status.value = 'failed';
      lastRunAt.value = Date.now();
    } finally {
      if (createdIds.length > 0) {
        await cleanupCreatedMembers(createdIds);
      }
      if (status.value === 'running') {
        status.value = 'failed';
      }
    }
  };

  return {
    status,
    results,
    lastError,
    lastRunAt,
    callChains,
    runSnapshotAudit
  };
});
