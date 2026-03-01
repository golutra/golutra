// 终端成员会话管理：处理成员终端创建、窗口通信与派发串行化。
import { ref, watch } from 'vue';
import { defineStore, storeToRefs } from 'pinia';
import { emitTo, listen } from '@tauri-apps/api/event';
import { isTauri } from '@tauri-apps/api/core';
import type { Member } from '@/features/chat/types';
import type { TerminalDispatchRequest } from '@/shared/types/terminalDispatch';
import { useSettingsStore } from '@/features/global/settingsStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { openTerminalWindow } from './openTerminalWindow';
import {
  closeSession,
  createSession,
  dispatchSession,
  emitSessionStatus,
  listSessionStatuses,
  onStatusChange,
  setMemberStatus
} from './terminalBridge';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import type { TerminalConnectionStatus, TerminalFriendInviteMeta, TerminalPostReadyMode, TerminalType } from '@/shared/types/terminal';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';
import { createFrontLogger } from '@/shared/monitoring/passiveMonitor';
import {
  TERMINAL_OPEN_TAB_EVENT,
  TERMINAL_WINDOW_READY_EVENT,
  TERMINAL_WINDOW_READY_REQUEST_EVENT,
  type TerminalOpenTabPayload,
  type TerminalWindowReadyPayload
} from './terminalEvents';

type MemberTerminalSession = {
  memberId: string;
  terminalId: string;
  title: string;
  workspaceId: string;
  terminalStatus: TerminalConnectionStatus;
  terminalStatusRaw?: TerminalConnectionStatus;
  terminalType?: TerminalType;
  allowConnecting?: boolean;
};

// 首段只发送原始文本，避免在某些 CLI 输入模式下提前换行。
const buildCommandInput = (command: string) => command;
// 等待 CLI 稳定后再补发回车提交，降低输入模式误判的概率。
const COMMAND_CONFIRM_DELAY_MS = 100;
// 延迟补发的提交回车。
const COMMAND_CONFIRM_SUFFIX = '\r';
const resolveTitle = (value?: string, fallback?: string) => {
  const trimmed = value?.trim();
  if (trimmed) {
    return trimmed;
  }
  const fallbackTrimmed = fallback?.trim();
  return fallbackTrimmed ?? '';
};

// 记录窗口就绪状态，避免在窗口未初始化时丢失 tab 打开事件。
const readyWindowLabels = new Set<string>();
const pendingTabs = new Map<string, TerminalOpenTabPayload[]>();
let readyListenerInitialized = false;
let statusListenerInitialized = false;
let statusSyncInitialized = false;
let statusSnapshotInFlight: { workspaceId: string; task: Promise<void> } | null = null;
let lastSnapshotWorkspaceId: string | null = null;
let lastSnapshotAt = 0;
const STATUS_SNAPSHOT_THROTTLE_MS = 800; // 限制快照频率，避免短时间内重复拉取。

// 避免重复同步状态导致的无效 IPC。
const lastSyncedMemberStatus = new Map<string, string>();

// 同成员命令串行派发，避免终端输入交错。
const dispatchChains = new Map<string, Promise<void>>();
const debugLog = createFrontLogger('terminal-member');
const resolveTerminalStatus = (status: string): TerminalConnectionStatus | null => {
  if (status === 'online') return 'connected';
  if (status === 'working') return 'working';
  if (status === 'offline') return 'disconnected';
  if (status === 'pending' || status === 'connecting' || status === 'connected' || status === 'disconnected') {
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

const resetWindowReady = (windowLabel: string) => {
  if (!windowLabel) {
    return;
  }
  // 诊断窗口关闭后同 label 重建，必须清理就绪缓存避免 open-tab 事件丢失。
  readyWindowLabels.delete(windowLabel);
  pendingTabs.delete(windowLabel);
};

/**
 * 终端成员会话存储。
 * 输入：成员与工作区上下文。
 * 输出：会话操作方法与查询入口。
 */
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
  // UI 展示仅基于后端状态，避免引入额外流程干预。
  const resolveDisplayTerminalStatus = (entry: MemberTerminalSession): TerminalConnectionStatus => {
    const resolved = entry.terminalStatusRaw ?? entry.terminalStatus ?? 'pending';
    if (resolved === 'connecting' && !entry.allowConnecting) {
      return 'pending';
    }
    return resolved;
  };
  const applyDisplayTerminalStatus = (entry: MemberTerminalSession) => {
    const display = resolveDisplayTerminalStatus(entry);
    const memberStatus = members.value.find((member) => member.id === entry.memberId)?.terminalStatus ?? null;
    if (display === entry.terminalStatus && memberStatus === display) {
      return;
    }
    entry.terminalStatus = display;
    const workspaceId = currentWorkspace.value?.id;
    if (workspaceId && entry.workspaceId !== workspaceId) {
      return;
    }
    if (memberStatus === display) {
      return;
    }
    void updateMember(entry.memberId, { terminalStatus: display }, { persist: false });
  };

  const buildMemberKey = (memberId: string, workspaceId?: string) =>
    workspaceId ? `${workspaceId}:${memberId}` : memberId;
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
      const entry = Object.values(memberSessions.value).find((item) => item.terminalId === payload.terminalId);
      if (entry) {
        entry.terminalStatusRaw = terminalStatus;
        applyDisplayTerminalStatus(entry);
      } else {
        const member = members.value.find((candidate) => candidate.id === payload.memberId);
        if (!hasTerminalConfig(member?.terminalType, member?.terminalCommand)) {
          return;
        }
        void updateMember(payload.memberId, { terminalStatus }, { persist: false });
      }
    });
  };

  const syncMemberStatuses = (nextMembers: Member[]) => {
    const nextMap = new Map<string, string>();
    for (const member of nextMembers) {
      if (!hasTerminalConfig(member.terminalType, member.terminalCommand)) {
        continue;
      }
      const manualStatus = member.manualStatus ?? member.status ?? 'online';
      const syncStatus = manualStatus === 'dnd' ? 'dnd' : '';
      nextMap.set(member.id, syncStatus);
      const last = lastSyncedMemberStatus.get(member.id);
      if (last !== syncStatus) {
        lastSyncedMemberStatus.set(member.id, syncStatus);
        void setMemberStatus(member.id, syncStatus).catch(() => {});
      }
    }
    for (const memberId of Array.from(lastSyncedMemberStatus.keys())) {
      if (nextMap.has(memberId)) {
        continue;
      }
      lastSyncedMemberStatus.delete(memberId);
      void setMemberStatus(memberId, '').catch(() => {});
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

  /**
   * 获取指定成员的终端会话。
   * 输入：成员 id 与可选工作区 id。
   * 输出：会话对象或 null。
   */
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

  /**
   * 拉取后端会话状态快照并同步到成员状态。
   * 输入：无（使用当前工作区）。
   * 输出：无。
   */
  const syncWorkspaceStatuses = async (options?: { force?: boolean; reason?: string }) => {
    if (!isTauri()) {
      return;
    }
    const workspace = currentWorkspace.value;
    if (!workspace) {
      return;
    }
    const now = Date.now();
    if (
      !options?.force &&
      lastSnapshotWorkspaceId === workspace.id &&
      now - lastSnapshotAt < STATUS_SNAPSHOT_THROTTLE_MS
    ) {
      return;
    }
    if (statusSnapshotInFlight?.workspaceId === workspace.id) {
      await statusSnapshotInFlight.task;
      return;
    }
    const task = (async () => {
      ensureStatusListener();
      let payloads: Array<{ terminalId: string; status: string; memberId?: string; workspaceId?: string }> = [];
      try {
        payloads = await listSessionStatuses(workspace.id);
      } catch (error) {
        console.error('Failed to sync terminal statuses.', error);
        return;
      }
      if (currentWorkspace.value?.id !== workspace.id) {
        return;
      }
      lastSnapshotWorkspaceId = workspace.id;
      lastSnapshotAt = Date.now();
      const statusPriority: Record<TerminalConnectionStatus, number> = {
        working: 3,
        connected: 2,
        connecting: 1,
        disconnected: 0,
        pending: -1
      };
      const memberById = new Map(members.value.map((member) => [member.id, member]));
      const ignoredPayloads: Array<{ memberId: string; reason: string }> = [];
      const applyResults: Array<{ memberId: string; status: TerminalConnectionStatus; terminalId?: string; result: string }> =
        [];
      const payloadMemberIds = new Set<string>();
      const statusByMember = new Map<string, { status: TerminalConnectionStatus; terminalId: string }>();
      for (const payload of payloads) {
        if (!payload.memberId) {
          continue;
        }
        payloadMemberIds.add(payload.memberId);
        if (payload.workspaceId && payload.workspaceId !== workspace.id) {
          continue;
        }
        const nextStatus = resolveTerminalStatus(payload.status);
        if (!nextStatus) {
          continue;
        }
        const current = statusByMember.get(payload.memberId);
        if (!current || statusPriority[nextStatus] > statusPriority[current.status]) {
          statusByMember.set(payload.memberId, { status: nextStatus, terminalId: payload.terminalId });
        }
      }
      for (const memberId of payloadMemberIds) {
        const member = memberById.get(memberId);
        if (!member) {
          ignoredPayloads.push({ memberId, reason: 'member-missing' });
          continue;
        }
        if (!hasTerminalConfig(member.terminalType, member.terminalCommand)) {
          ignoredPayloads.push({ memberId, reason: 'terminal-config-missing' });
        }
      }
      for (const member of members.value) {
        if (!hasTerminalConfig(member.terminalType, member.terminalCommand)) {
          continue;
        }
        const match = statusByMember.get(member.id);
        const fallbackStatus = member.terminalStatus === 'disconnected' ? 'disconnected' : 'pending';
        const desiredStatus = match?.status ?? fallbackStatus;
        const memberKey = buildMemberKey(member.id, workspace.id);
        const entry = memberSessions.value[memberKey];
        if (match) {
          if (entry) {
            if (entry.terminalId !== match.terminalId) {
              entry.terminalId = match.terminalId;
            }
            entry.terminalStatusRaw = desiredStatus;
            applyDisplayTerminalStatus(entry);
            applyResults.push({
              memberId: member.id,
              status: desiredStatus,
              terminalId: match.terminalId,
              result: 'updated-entry'
            });
          } else {
            const resolvedTitle = resolveTitle(member.name, member.id);
            const nextEntry: MemberTerminalSession = {
              memberId: member.id,
              terminalId: match.terminalId,
              title: resolvedTitle,
              workspaceId: workspace.id,
              terminalStatus: desiredStatus,
              terminalStatusRaw: desiredStatus,
              terminalType: member.terminalType,
              allowConnecting: desiredStatus === 'connecting'
            };
            memberSessions.value[memberKey] = nextEntry;
            applyDisplayTerminalStatus(nextEntry);
            applyResults.push({
              memberId: member.id,
              status: desiredStatus,
              terminalId: match.terminalId,
              result: 'created-entry'
            });
          }
          continue;
        }
        if (entry) {
          entry.terminalStatusRaw = desiredStatus;
          applyDisplayTerminalStatus(entry);
          applyResults.push({
            memberId: member.id,
            status: desiredStatus,
            terminalId: entry.terminalId,
            result: 'marked-pending'
          });
          continue;
        }
        if (member.terminalStatus !== desiredStatus) {
          void updateMember(member.id, { terminalStatus: desiredStatus }, { persist: false });
          applyResults.push({
            memberId: member.id,
            status: desiredStatus,
            result: 'updated-member'
          });
        }
      }
    })();
    statusSnapshotInFlight = { workspaceId: workspace.id, task };
    try {
      await task;
    } finally {
      if (statusSnapshotInFlight?.task === task) {
        statusSnapshotInFlight = null;
      }
    }
  };

  /**
   * 在终端窗口中打开成员会话标签页。
   * 输入：会话条目、标题覆盖与可选窗口标签。
   * 输出：无。
   */
  const openMemberTab = async (
    entry: MemberTerminalSession,
    titleOverride?: string,
    options?: { windowLabel?: string }
  ) => {
    const windowLabel = options?.windowLabel ?? (await ensureTerminalWindow());
    if (!windowLabel) {
      return;
    }
    const terminalType = entry.terminalType ?? members.value.find((member) => member.id === entry.memberId)?.terminalType;
    const payload: TerminalOpenTabPayload = {
      terminalId: entry.terminalId,
      title: resolveTitle(titleOverride, entry.title),
      memberId: entry.memberId,
      terminalType,
      keepAlive: true
    };
    void logDiagnosticsEvent('terminal-open-tab', {
      terminalId: entry.terminalId,
      memberId: entry.memberId,
      terminalType,
      windowLabel,
      title: payload.title
    });
    ensureReadyListener();
    if (!readyWindowLabels.has(windowLabel)) {
      queuePendingTab(windowLabel, payload);
      void emitTo(windowLabel, TERMINAL_WINDOW_READY_REQUEST_EVENT, {}).catch(() => {});
    }
    await emitTo(windowLabel, TERMINAL_OPEN_TAB_EVENT, payload).catch(() => {});
  };

  /**
   * 获取或创建终端窗口。
   * 输入：无。
   * 输出：窗口 label 或 null。
   */
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
      void emitTo(result.label, TERMINAL_WINDOW_READY_REQUEST_EVENT, {}).catch(() => {});
    }
    return result.label;
  };

  /**
   * 启动成员终端会话。
   * 输入：成员信息与是否打开标签页。
   * 输出：会话条目或 null。
   */
  const startMemberSession = async (
    member: Member,
    options?: { openTab?: boolean; windowLabel?: string; inviteMeta?: TerminalFriendInviteMeta }
  ) => {
    const workspace = currentWorkspace.value;
    if (!workspace) {
      return null;
    }
    const command = member.terminalCommand?.trim();
    const terminalCommand = command ? command : undefined;
    if (!hasTerminalConfig(member.terminalType, terminalCommand)) {
      return null;
    }
    const postReadyMode: TerminalPostReadyMode = options?.inviteMeta ? 'invite' : 'none';
    const resolvedTitle = resolveTitle(member.name, member.id);
    const terminalPath = resolveTerminalPath(member);
    ensureStatusListener();
    const shouldShowConnecting = Boolean(options?.inviteMeta);
    const initialTerminalStatus: TerminalConnectionStatus = shouldShowConnecting ? 'connecting' : 'pending';
    if (member.terminalType && member.terminalType !== 'shell' && shouldShowConnecting) {
      void updateMember(member.id, { terminalStatus: initialTerminalStatus }, { persist: false });
    }
    let terminalId = '';
    try {
      terminalId = await createSession({
        cwd: workspace.path,
        memberId: member.id,
        memberName: member.name,
        workspaceId: workspace.id,
        keepAlive: true,
        terminalType: member.terminalType,
        terminalCommand,
        terminalPath,
        postReadyMode,
        inviteMeta: options?.inviteMeta
      });
    } catch (error) {
      throw error;
    }
    void logDiagnosticsEvent('start-member-session', {
      terminalId,
      memberId: member.id,
      workspaceId: workspace.id,
      terminalType: member.terminalType,
      terminalCommand,
      terminalPath,
      keepAlive: true
    });
    const entry: MemberTerminalSession = {
      memberId: member.id,
      terminalId,
      title: resolvedTitle,
      workspaceId: workspace.id,
      terminalStatus: initialTerminalStatus,
      terminalStatusRaw: initialTerminalStatus,
      terminalType: member.terminalType,
      allowConnecting: shouldShowConnecting
    };
    memberSessions.value[buildMemberKey(member.id, workspace.id)] = entry;
    if (member.terminalType && member.terminalType !== 'shell') {
      void updateMember(member.id, { terminalStatus: initialTerminalStatus }, { persist: false });
    }
    void emitSessionStatus(terminalId).catch(() => {});
    if (options?.openTab ?? true) {
      await openMemberTab(entry, resolvedTitle, { windowLabel: options?.windowLabel });
    }
    return entry;
  };

  /**
   * 确保成员会话存在，必要时重建。
   * 输入：成员信息与是否打开标签页。
   * 输出：会话条目或 null。
   */
  const ensureMemberSession = async (
    member: Member,
    options?: { openTab?: boolean; windowLabel?: string; inviteMeta?: TerminalFriendInviteMeta }
  ) => {
    ensureStatusListener();
    const workspaceId = currentWorkspace.value?.id;
    const memberKey = buildMemberKey(member.id, workspaceId);
    const shouldOpenTab = options?.openTab ?? true;
    let existing = getSession(member.id, workspaceId);
    if (!existing || existing.terminalStatus === 'disconnected') {
      await syncWorkspaceStatuses({ reason: 'ensure-member-session' });
      existing = getSession(member.id, workspaceId);
    }
    if (existing && existing.terminalStatus !== 'disconnected') {
      if (shouldOpenTab) {
        await openMemberTab(existing, member.name, { windowLabel: options?.windowLabel });
      }
      void emitSessionStatus(existing.terminalId).catch(() => {});
      void logDiagnosticsEvent('ensure-member-session', {
        memberId: member.id,
        workspaceId,
        terminalId: existing.terminalId,
        terminalStatus: existing.terminalStatus,
        openTab: shouldOpenTab,
        windowLabel: options?.windowLabel,
        reused: true
      });
      return existing;
    }
    const inflight = inFlightSessions.get(memberKey);
    if (inflight) {
      const entry = await inflight;
      if (entry && shouldOpenTab) {
        await openMemberTab(entry, member.name, { windowLabel: options?.windowLabel });
      }
      void logDiagnosticsEvent('ensure-member-session', {
        memberId: member.id,
        workspaceId,
        terminalId: entry?.terminalId ?? null,
        terminalStatus: entry?.terminalStatus ?? null,
        openTab: shouldOpenTab,
        windowLabel: options?.windowLabel,
        inflight: true
      });
      return entry;
    }
    const task = (async () => {
      if (existing) {
        try {
          await closeSession(existing.terminalId, { preserve: false });
        } catch {
          // 忽略重启会话时的清理失败，避免阻断重连流程。
        }
        delete memberSessions.value[memberKey];
      }
      return startMemberSession(member, options);
    })();
    inFlightSessions.set(memberKey, task);
    try {
      const entry = await task;
      void logDiagnosticsEvent('ensure-member-session', {
        memberId: member.id,
        workspaceId,
        terminalId: entry?.terminalId ?? null,
        terminalStatus: entry?.terminalStatus ?? null,
        openTab: shouldOpenTab,
        windowLabel: options?.windowLabel,
        created: true
      });
      return entry;
    } finally {
      if (inFlightSessions.get(memberKey) === task) {
        inFlightSessions.delete(memberKey);
      }
    }
  };

  /**
   * 发送终端指令并附带聊天上下文。
   * 输入：派发请求与成员信息。
   * 输出：无。
   */
  const dispatchTerminalMessage = async (request: TerminalDispatchRequest, member: Member) => {
    const entry = await ensureMemberSession(member, { openTab: false });
    if (!entry) {
      return;
    }
    await dispatchSession(entry.terminalId, buildCommandInput(request.text), {
      conversationId: request.conversationId,
      conversationType: request.conversationType,
      senderId: request.senderId,
      senderName: request.senderName
    });
    await new Promise<void>((resolve) => {
      window.setTimeout(resolve, COMMAND_CONFIRM_DELAY_MS);
    });
    await dispatchSession(entry.terminalId, COMMAND_CONFIRM_SUFFIX, {
      conversationId: request.conversationId,
      conversationType: request.conversationType,
      senderId: request.senderId,
      senderName: request.senderName
    });
  };

  /**
   * 串行派发终端指令，避免同成员命令交错。
   * 输入：派发请求。
   * 输出：无。
   */
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

  /**
   * 打开成员终端并确保自动启动标记为开启。
   * 输入：成员信息与可选窗口标签。
   * 输出：会话条目或 null。
   */
  const openMemberTerminal = async (member: Member, options?: { windowLabel?: string }) => {
    ensureStatusListener();
    debugLog('open member terminal', {
      memberId: member.id,
      terminalType: member.terminalType,
      terminalCommand: member.terminalCommand
    });
    void logDiagnosticsEvent('open-member-terminal', {
      memberId: member.id,
      terminalType: member.terminalType,
      terminalCommand: member.terminalCommand,
      windowLabel: options?.windowLabel
    });
    const entry = await ensureMemberSession(member, { openTab: true, windowLabel: options?.windowLabel });
    if (!entry) {
      return null;
    }
    if (
      hasTerminalConfig(member.terminalType, member.terminalCommand) &&
      (member.autoStartTerminal === false || member.manualStatus === 'offline')
    ) {
      void updateMember(member.id, { autoStartTerminal: true, manualStatus: 'online' });
    }
    debugLog('open member terminal ready', { memberId: member.id, terminalId: entry.terminalId });
    return entry;
  };

  /**
   * 停止成员终端会话。
   * 输入：成员 id 与关闭选项。
   * 输出：无。
   */
  const stopMemberSession = async (
    memberId: string,
    options?: { preserve?: boolean; fireAndForget?: boolean; deleteSessionMap?: boolean }
  ) => {
    const workspaceId = currentWorkspace.value?.id;
    const entry = getSession(memberId, workspaceId);
    if (!entry) {
      return;
    }
    applyDisplayTerminalStatus(entry);
    const closePromise = closeSession(entry.terminalId, {
      preserve: options?.preserve ?? true,
      deleteSessionMap: options?.deleteSessionMap
    });
    if (options?.fireAndForget) {
      void closePromise.catch(() => {});
    } else {
      await closePromise;
    }
    if (options?.preserve ?? true) {
      entry.terminalStatusRaw = 'pending';
      applyDisplayTerminalStatus(entry);
      dispatchChains.delete(buildMemberKey(memberId, workspaceId));
      return;
    }
    delete memberSessions.value[buildMemberKey(memberId, workspaceId)];
    dispatchChains.delete(buildMemberKey(memberId, workspaceId));
  };

  ensureStatusSync();

  return {
    syncWorkspaceStatuses,
    ensureMemberSession,
    enqueueTerminalDispatch,
    openMemberTerminal,
    stopMemberSession,
    getSession,
    resetWindowReady
  };
});

