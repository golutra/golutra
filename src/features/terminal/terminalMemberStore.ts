import { ref, watch } from 'vue';
import { defineStore, storeToRefs } from 'pinia';
import { emitTo, listen } from '@tauri-apps/api/event';
import { isTauri } from '@tauri-apps/api/core';
import type { ConversationType, Member } from '@/features/chat/types';
import { useSettingsStore } from '@/features/global/settingsStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { openTerminalWindow } from './openTerminalWindow';
import { closeSession, createSession, dispatchSession, onStatusChange, setMemberStatus } from './terminalBridge';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import type { TerminalConnectionStatus, TerminalType } from '@/shared/types/terminal';
import {
  TERMINAL_OPEN_TAB_EVENT,
  TERMINAL_WINDOW_READY_EVENT,
  TERMINAL_WINDOW_READY_REQUEST_EVENT,
  type TerminalOpenTabPayload,
  type TerminalWindowReadyPayload
} from './terminalEvents';

type MemberTerminalSession = {
  memberId: string;
  sessionId: string;
  title: string;
  workspaceId: string;
  terminalStatus: TerminalConnectionStatus;
  terminalType?: TerminalType;
};

export type TerminalDispatchRequest = {
  memberId: string;
  conversationId: string;
  conversationType: ConversationType;
  senderId: string;
  senderName: string;
  text: string;
};


const buildCommandInput = (command: string) => `${command}\r`;
const resolveTitle = (value?: string, fallback?: string) => {
  const trimmed = value?.trim();
  if (trimmed) {
    return trimmed;
  }
  const fallbackTrimmed = fallback?.trim();
  return fallbackTrimmed ?? '';
};

const readyWindowLabels = new Set<string>();
const pendingTabs = new Map<string, TerminalOpenTabPayload[]>();
let readyListenerInitialized = false;
let statusListenerInitialized = false;
let statusSyncInitialized = false;

const lastSyncedMemberStatus = new Map<string, string>();

const dispatchChains = new Map<string, Promise<void>>();
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
  console.info('[terminal-member]', ...args);
};

const resolveTerminalStatus = (status: string): TerminalConnectionStatus | null => {
  if (status === 'online') return 'connected';
  if (status === 'working') return 'working';
  if (status === 'offline') return 'disconnected';
  if (status === 'connecting' || status === 'connected' || status === 'disconnected') {
    return status;
  }
  return null;
};

const queuePendingTab = (windowLabel: string, payload: TerminalOpenTabPayload) => {
  const list = pendingTabs.get(windowLabel) ?? [];
  list.push(payload);
  pendingTabs.set(windowLabel, list);
};

const flushPendingTabs = async (windowLabel: string) => {
  const list = pendingTabs.get(windowLabel);
  if (!list || list.length === 0) {
    return;
  }
  pendingTabs.delete(windowLabel);
  for (const payload of list) {
    await emitTo(windowLabel, TERMINAL_OPEN_TAB_EVENT, payload);
  }
};

export const useTerminalMemberStore = defineStore('terminal-member', () => {
  const workspaceStore = useWorkspaceStore();
  const { currentWorkspace } = storeToRefs(workspaceStore);
  const projectStore = useProjectStore();
  const { updateMember } = projectStore;
  const { members } = storeToRefs(projectStore);
  const settingsStore = useSettingsStore();
  const { settings } = storeToRefs(settingsStore);
  const memberSessions = ref<Record<string, MemberTerminalSession>>({});
  const inFlightSessions = new Map<string, Promise<MemberTerminalSession | null>>();

  const buildMemberKey = (memberId: string, workspaceId?: string) =>
    workspaceId ? `${workspaceId}:${memberId}` : memberId;
  const buildMemberSessionId = (workspaceId: string, memberId: string) => `member-${workspaceId}-${memberId}`;
  const resolveTerminalPath = (member: Member) => {
    const trimmed = member.terminalPath?.trim();
    if (trimmed) {
      return trimmed;
    }
    const type = member.terminalType;
    if (!type) {
      return undefined;
    }
    return settings.value.members.terminalPaths?.[type];
  };

  const ensureReadyListener = () => {
    if (readyListenerInitialized) {
      return;
    }
    readyListenerInitialized = true;
    void listen<TerminalWindowReadyPayload>(TERMINAL_WINDOW_READY_EVENT, (event) => {
      const label = event.payload.windowLabel;
      if (!label) {
        return;
      }
      readyWindowLabels.add(label);
      void flushPendingTabs(label);
    });
  };

  const ensureStatusListener = () => {
    if (statusListenerInitialized) {
      return;
    }
    statusListenerInitialized = true;
    onStatusChange((payload) => {
      const workspaceId = currentWorkspace.value?.id;
      if (!payload.memberId || !payload.workspaceId || payload.workspaceId !== workspaceId) {
        return;
      }
      const terminalStatus = resolveTerminalStatus(payload.status);
      if (!terminalStatus) {
        return;
      }
      const entry = Object.values(memberSessions.value).find((item) => item.sessionId === payload.sessionId);
      if (entry) {
        entry.terminalStatus = terminalStatus;
      }
      const member = members.value.find((candidate) => candidate.id === payload.memberId);
      if (!hasTerminalConfig(member?.terminalType, member?.terminalCommand)) {
        return;
      }
      void updateMember(payload.memberId, { terminalStatus }, { persist: false });
    });
  };

  const syncMemberStatuses = (nextMembers: Member[]) => {
    const nextMap = new Map<string, string>();
    for (const member of nextMembers) {
      if (!hasTerminalConfig(member.terminalType, member.terminalCommand)) {
        continue;
      }
      const status = member.status ?? 'online';
      nextMap.set(member.id, status);
      const last = lastSyncedMemberStatus.get(member.id);
      if (last !== status) {
        lastSyncedMemberStatus.set(member.id, status);
        void setMemberStatus(member.id, status).catch(() => { });
      }
    }
    for (const memberId of Array.from(lastSyncedMemberStatus.keys())) {
      if (nextMap.has(memberId)) {
        continue;
      }
      lastSyncedMemberStatus.delete(memberId);
      void setMemberStatus(memberId, '').catch(() => { });
    }
  };

  const ensureStatusSync = () => {
    if (statusSyncInitialized) {
      return;
    }
    statusSyncInitialized = true;
    if (!isTauri()) {
      return;
    }
    watch(
      members,
      (next) => {
        syncMemberStatuses(next);
      },
      { immediate: true }
    );
  };

  const getSession = (memberId: string, workspaceId?: string) => {
    const entry = memberSessions.value[buildMemberKey(memberId, workspaceId)];
    if (!entry) {
      return null;
    }
    if (workspaceId && entry.workspaceId !== workspaceId) {
      return null;
    }
    return entry;
  };

  const openMemberTab = async (entry: MemberTerminalSession, titleOverride?: string) => {
    const windowLabel = await ensureTerminalWindow();
    if (!windowLabel) {
      return;
    }
    const terminalType = entry.terminalType ?? members.value.find((member) => member.id === entry.memberId)?.terminalType;
    const payload: TerminalOpenTabPayload = {
      sessionId: entry.sessionId,
      title: resolveTitle(titleOverride, entry.title),
      memberId: entry.memberId,
      terminalType,
      keepAlive: true
    };
    ensureReadyListener();
    if (!readyWindowLabels.has(windowLabel)) {
      queuePendingTab(windowLabel, payload);
    }
    await emitTo(windowLabel, TERMINAL_OPEN_TAB_EVENT, payload).catch(() => { });
  };

  const ensureTerminalWindow = async () => {
    ensureReadyListener();
    const workspace = currentWorkspace.value;
    if (!workspace) {
      return null;
    }
    const result = await openTerminalWindow({
      workspaceId: workspace.id,
      workspaceName: workspace.name,
      workspacePath: workspace.path
    });
    if (!result) {
      return null;
    }
    if (!result.reused) {
      readyWindowLabels.delete(result.label);
    }
    if (!readyWindowLabels.has(result.label)) {
      void emitTo(result.label, TERMINAL_WINDOW_READY_REQUEST_EVENT, {}).catch(() => { });
    }
    return result.label;
  };

  const startMemberSession = async (member: Member, options?: { openTab?: boolean }) => {
    const workspace = currentWorkspace.value;
    if (!workspace) {
      return null;
    }
    const command = member.terminalCommand?.trim();
    const terminalCommand = command ? command : undefined;
    if (!hasTerminalConfig(member.terminalType, terminalCommand)) {
      return null;
    }
    const resolvedTitle = resolveTitle(member.name, member.id);
    const requestedSessionId = buildMemberSessionId(workspace.id, member.id);
    const memberKey = buildMemberKey(member.id, workspace.id);
    ensureStatusListener();
    const entry: MemberTerminalSession = {
      memberId: member.id,
      sessionId: requestedSessionId,
      title: resolvedTitle,
      workspaceId: workspace.id,
      terminalStatus: 'connecting',
      terminalType: member.terminalType
    };
    memberSessions.value[memberKey] = entry;
    void updateMember(member.id, { terminalStatus: 'connecting' }, { persist: false });
    let sessionId: string;
    try {
      sessionId = await createSession({
        cwd: workspace.path,
        memberId: member.id,
        workspaceId: workspace.id,
        keepAlive: true,
        sessionId: requestedSessionId,
        terminalType: member.terminalType,
        terminalCommand,
        terminalPath: resolveTerminalPath(member)
      });
    } catch (error) {
      delete memberSessions.value[memberKey];
      void updateMember(member.id, { terminalStatus: 'disconnected' }, { persist: false });
      throw error;
    }
    if (sessionId !== requestedSessionId) {
      entry.sessionId = sessionId;
    }
    if (entry.terminalStatus === 'connecting') {
      entry.terminalStatus = 'connected';
      void updateMember(member.id, { terminalStatus: 'connected' }, { persist: false });
    }
    if (options?.openTab ?? true) {
      await openMemberTab(entry, resolvedTitle);
    }
    return entry;
  };

  const ensureMemberSession = async (member: Member, options?: { openTab?: boolean }) => {
    ensureStatusListener();
    const workspaceId = currentWorkspace.value?.id;
    const memberKey = buildMemberKey(member.id, workspaceId);
    const shouldOpenTab = options?.openTab ?? true;
    const existing = getSession(member.id, workspaceId);
    if (existing && existing.terminalStatus !== 'disconnected') {
      if (shouldOpenTab) {
        await openMemberTab(existing, member.name);
      }
      return existing;
    }
    const inflight = inFlightSessions.get(memberKey);
    if (inflight) {
      const entry = await inflight;
      if (entry && shouldOpenTab) {
        await openMemberTab(entry, member.name);
      }
      return entry;
    }
    const task = (async () => {
      if (existing) {
        try {
          await closeSession(existing.sessionId, { preserve: false });
        } catch {
          // Ignore cleanup errors when restarting a session.
        }
        delete memberSessions.value[memberKey];
      }
      return startMemberSession(member, options);
    })();
    inFlightSessions.set(memberKey, task);
    try {
      return await task;
    } finally {
      if (inFlightSessions.get(memberKey) === task) {
        inFlightSessions.delete(memberKey);
      }
    }
  };

  const dispatchTerminalMessage = async (request: TerminalDispatchRequest, member: Member) => {
    const entry = await ensureMemberSession(member, { openTab: false });
    if (!entry) {
      return;
    }
    await dispatchSession(entry.sessionId, buildCommandInput(request.text), {
      conversationId: request.conversationId,
      conversationType: request.conversationType,
      senderId: request.senderId,
      senderName: request.senderName
    });
  };

  const enqueueTerminalDispatch = async (request: TerminalDispatchRequest) => {
    const workspace = currentWorkspace.value;
    if (!workspace) {
      return;
    }
    const member = members.value.find((candidate) => candidate.id === request.memberId);
    if (!hasTerminalConfig(member?.terminalType, member?.terminalCommand)) {
      return;
    }
    const memberKey = buildMemberKey(request.memberId, workspace.id);
    const chain = dispatchChains.get(memberKey) ?? Promise.resolve();
    const task = chain.then(
      () => dispatchTerminalMessage(request, member),
      () => dispatchTerminalMessage(request, member)
    );
    dispatchChains.set(memberKey, task);
    await task;
  };

  const openMemberTerminal = async (member: Member) => {
    ensureStatusListener();
    debugLog('open member terminal', {
      memberId: member.id,
      terminalType: member.terminalType,
      terminalCommand: member.terminalCommand
    });
    const entry = await ensureMemberSession(member, { openTab: true });
    if (!entry) {
      return null;
    }
    if (
      hasTerminalConfig(member.terminalType, member.terminalCommand) &&
      (member.autoStartTerminal === false || member.manualStatus === 'offline')
    ) {
      void updateMember(member.id, { autoStartTerminal: true, manualStatus: 'online', status: 'online' });
    }
    debugLog('open member terminal ready', { memberId: member.id, sessionId: entry.sessionId });
    return entry;
  };

  const stopMemberSession = async (
    memberId: string,
    options?: { preserve?: boolean; fireAndForget?: boolean }
  ) => {
    const workspaceId = currentWorkspace.value?.id;
    const entry = getSession(memberId, workspaceId);
    if (!entry) {
      return;
    }
    const closePromise = closeSession(entry.sessionId, { preserve: options?.preserve ?? true });
    if (options?.fireAndForget) {
      void closePromise.catch(() => { });
    } else {
      await closePromise;
    }
    if (options?.preserve ?? true) {
      entry.terminalStatus = 'disconnected';
      void updateMember(memberId, { terminalStatus: 'disconnected' }, { persist: false });
      dispatchChains.delete(buildMemberKey(memberId, workspaceId));
      return;
    }
    delete memberSessions.value[buildMemberKey(memberId, workspaceId)];
    void updateMember(memberId, { terminalStatus: 'disconnected' }, { persist: false });
    dispatchChains.delete(buildMemberKey(memberId, workspaceId));
  };

  ensureStatusSync();

  return {
    ensureMemberSession,
    enqueueTerminalDispatch,
    openMemberTerminal,
    stopMemberSession,
    getSession
  };
});
