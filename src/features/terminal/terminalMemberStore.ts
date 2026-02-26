import { ref } from 'vue';
import { defineStore, storeToRefs } from 'pinia';
import { emitTo, listen } from '@tauri-apps/api/event';
import type { Member, MemberStatus } from '@/features/chat/types';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { openTerminalWindow } from './openTerminalWindow';
import { closeSession, createSession, onStatusChange } from './terminalBridge';
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
  command: string;
  title: string;
  workspaceId: string;
  status: MemberStatus;
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
  const memberSessions = ref<Record<string, MemberTerminalSession>>({});

  const buildMemberKey = (memberId: string, workspaceId?: string) =>
    workspaceId ? `${workspaceId}:${memberId}` : memberId;

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
      const status = payload.status as MemberStatus;
      if (!['online', 'working', 'dnd', 'offline'].includes(status)) {
        return;
      }
      const entry = Object.values(memberSessions.value).find((item) => item.sessionId === payload.sessionId);
      if (entry) {
        entry.status = status;
      }
      void updateMember(payload.memberId, { status });
    });
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
    const payload: TerminalOpenTabPayload = {
      sessionId: entry.sessionId,
      title: resolveTitle(titleOverride, entry.title),
      memberId: entry.memberId,
      keepAlive: true
    };
    ensureReadyListener();
    if (!readyWindowLabels.has(windowLabel)) {
      queuePendingTab(windowLabel, payload);
    }
    await emitTo(windowLabel, TERMINAL_OPEN_TAB_EVENT, payload).catch(() => {});
  };

  const ensureTerminalWindow = async () => {
    ensureReadyListener();
    const workspace = currentWorkspace.value;
    if (!workspace) {
      return null;
    }
    const result = await openTerminalWindow({ workspaceId: workspace.id, workspaceName: workspace.name });
    if (!result) {
      return null;
    }
    if (!result.reused) {
      readyWindowLabels.delete(result.label);
    }
    if (!readyWindowLabels.has(result.label)) {
      void emitTo(result.label, TERMINAL_WINDOW_READY_REQUEST_EVENT, {}).catch(() => {});
    }
    return result.label;
  };

  const startMemberSession = async (member: Member, options?: { openTab?: boolean }) => {
    const workspace = currentWorkspace.value;
    if (!workspace) {
      return null;
    }
    const command = member.terminalCommand?.trim();
    if (!command) {
      return null;
    }
    const resolvedTitle = resolveTitle(member.name, member.id);
    const requestedSessionId = member.name.trim() || undefined;
    ensureStatusListener();
    const sessionId = await createSession({
      cwd: workspace.path,
      memberId: member.id,
      workspaceId: workspace.id,
      keepAlive: true,
      sessionId: requestedSessionId,
      initialData: buildCommandInput(command)
    });
    const entry: MemberTerminalSession = {
      memberId: member.id,
      sessionId,
      command,
      title: resolvedTitle,
      workspaceId: workspace.id,
      status: 'online'
    };
    memberSessions.value[buildMemberKey(member.id, workspace.id)] = entry;
    if (options?.openTab ?? true) {
      await openMemberTab(entry, resolvedTitle);
    }
    return entry;
  };

  const ensureMemberSession = async (member: Member, options?: { openTab?: boolean }) => {
    ensureStatusListener();
    const workspaceId = currentWorkspace.value?.id;
    const existing = getSession(member.id, workspaceId);
    if (existing && existing.status !== 'offline') {
      if (options?.openTab ?? true) {
        await openMemberTab(existing, member.name);
      }
      return existing;
    }
    if (existing) {
      try {
        await closeSession(existing.sessionId, { preserve: false });
      } catch {
        // Ignore cleanup errors when restarting a session.
      }
      delete memberSessions.value[buildMemberKey(member.id, workspaceId)];
    }
    return startMemberSession(member, options);
  };

  const openMemberTerminal = async (member: Member) => {
    ensureStatusListener();
    const workspaceId = currentWorkspace.value?.id;
    const entry = getSession(member.id, workspaceId);
    if (entry) {
      await openMemberTab(entry, member.name);
      return entry;
    }
    return startMemberSession(member, { openTab: true });
  };

  const stopMemberSession = async (memberId: string, options?: { preserve?: boolean }) => {
    const workspaceId = currentWorkspace.value?.id;
    const entry = getSession(memberId, workspaceId);
    if (!entry) {
      return;
    }
    await closeSession(entry.sessionId, { preserve: options?.preserve ?? true });
    if (options?.preserve ?? true) {
      entry.status = 'offline';
      return;
    }
    delete memberSessions.value[buildMemberKey(memberId, workspaceId)];
  };

  return {
    ensureMemberSession,
    openMemberTerminal,
    stopMemberSession,
    getSession
  };
});
