// 终端协调层：集中处理跨功能模块的终端调用与错误兜底。
import { defineStore, storeToRefs } from 'pinia';
import { i18n } from '@/i18n';
import { useToastStore } from '@/stores/toastStore';
import { useTerminalMemberStore } from '@/features/terminal/terminalMemberStore';
import { onTerminalStreamMessage as onTerminalStreamMessageBridge } from '@/features/terminal/terminalBridge';
import { useProjectStore } from '@/features/workspace/projectStore';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';
import type { Conversation, Member, MessageMentionsPayload } from '@/features/chat/types';
import type { TerminalDispatchRequest } from '@/shared/types/terminalDispatch';
import type { TerminalChatPayload } from '@/features/terminal/terminalBridge';

const formatError = (error: unknown) => (error instanceof Error ? error.message : String(error));
const uniqueMemberIds = (ids: string[]) => Array.from(new Set(ids.filter((id) => id)));

export const useTerminalOrchestratorStore = defineStore('terminal-orchestrator', () => {
  const terminalMemberStore = useTerminalMemberStore();
  const toastStore = useToastStore();
  const projectStore = useProjectStore();
  const { members } = storeToRefs(projectStore);
  const { pushToast } = toastStore;
  const notifyTerminalResourceLimit = (message: string) => {
    if (!message.includes('terminal buffer limit reached')) {
      return false;
    }
    pushToast(i18n.global.t('terminal.resourceLimit'), { tone: 'error' });
    return true;
  };

  /**
   * 启动成员终端会话。
   * 输入：成员信息与打开标签控制。
   * 输出：会话条目或 null。
   */
  const ensureMemberSession = async (member: Member, options?: { openTab?: boolean }) => {
    if (!hasTerminalConfig(member.terminalType, member.terminalCommand)) {
      return null;
    }
    try {
      return await terminalMemberStore.ensureMemberSession(member, options);
    } catch (error) {
      notifyTerminalResourceLimit(formatError(error));
      console.error('Failed to start terminal session.', error);
      return null;
    }
  };

  /**
   * 打开成员终端标签页。
   * 输入：成员信息与可选窗口标签。
   * 输出：会话条目或 null。
   */
  const openMemberTerminal = async (member: Member, options?: { windowLabel?: string }) => {
    if (!hasTerminalConfig(member.terminalType, member.terminalCommand)) {
      return null;
    }
    try {
      return await terminalMemberStore.openMemberTerminal(member, options);
    } catch (error) {
      notifyTerminalResourceLimit(formatError(error));
      console.error('Failed to open member terminal.', error);
      return null;
    }
  };

  const resetTerminalWindowReady = (windowLabel: string) => {
    terminalMemberStore.resetWindowReady(windowLabel);
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
    try {
      await terminalMemberStore.stopMemberSession(memberId, options);
    } catch (error) {
      console.error('Failed to stop terminal session.', error);
    }
  };

  /**
   * 串行派发终端指令。
   * 输入：派发请求。
   * 输出：无。
   */
  const enqueueTerminalDispatch = async (request: TerminalDispatchRequest) => {
    try {
      void logDiagnosticsEvent('enqueue-dispatch', {
        memberId: request.memberId,
        conversationId: request.conversationId,
        conversationType: request.conversationType,
        senderId: request.senderId,
        senderName: request.senderName,
        text: request.text
      });
      await terminalMemberStore.enqueueTerminalDispatch(request);
    } catch (error) {
      console.error('Failed to dispatch terminal message.', error);
    }
  };

  const handleMentionAllTerminalDispatch = () => {};

  const resolveTerminalTargets = (
    conversation: Conversation,
    mentions: MessageMentionsPayload | undefined,
    senderId: string
  ) => {
    const memberMap = new Map(members.value.map((member) => [member.id, member]));
    const isTerminalMember = (memberId: string) => {
      if (memberId === senderId) {
        return false;
      }
      const member = memberMap.get(memberId);
      if (!member) {
        return false;
      }
      return hasTerminalConfig(member.terminalType, member.terminalCommand);
    };

    let targets: string[] = [];
    if (conversation.type === 'dm') {
      const targetId =
        conversation.targetId ?? conversation.memberIds.find((id) => id !== senderId) ?? '';
      if (!targetId || !isTerminalMember(targetId)) {
        targets = [];
      } else {
        targets = [targetId];
      }
      void logDiagnosticsEvent('parse-targets', {
        conversationId: conversation.id,
        conversationType: conversation.type,
        mentionIds: mentions?.mentionIds ?? [],
        mentionAll: mentions?.mentionAll ?? false,
        targets
      });
      return targets;
    }

    if (!mentions) {
      targets = [];
      void logDiagnosticsEvent('parse-targets', {
        conversationId: conversation.id,
        conversationType: conversation.type,
        mentionIds: [],
        mentionAll: false,
        targets
      });
      return targets;
    }
    const mentionIds = uniqueMemberIds(mentions.mentionIds ?? []);
    if (!mentions.mentionAll && mentionIds.length === 0) {
      targets = [];
      void logDiagnosticsEvent('parse-targets', {
        conversationId: conversation.id,
        conversationType: conversation.type,
        mentionIds,
        mentionAll: false,
        targets
      });
      return targets;
    }
    const memberSet = new Set(conversation.memberIds ?? []);
    const sourceIds = mentions.mentionAll ? conversation.memberIds : mentionIds;
    if (mentions.mentionAll) {
      // [TODO/seekskyworld, 2026-01-27] @all 规则：仅提醒管理员但不触发终端事件（单人模式）。
      handleMentionAllTerminalDispatch(conversation.id, senderId);
    }
    targets = uniqueMemberIds(sourceIds).filter((id) => memberSet.has(id)).filter(isTerminalMember);
    void logDiagnosticsEvent('parse-targets', {
      conversationId: conversation.id,
      conversationType: conversation.type,
      mentionIds,
      mentionAll: mentions.mentionAll,
      targets
    });
    return targets;
  };

  const dispatchConversationToTerminals = async (payload: {
    conversation: Conversation;
    text: string;
    mentions?: MessageMentionsPayload;
    senderId: string;
    senderName: string;
  }) => {
    const targets = resolveTerminalTargets(payload.conversation, payload.mentions, payload.senderId);
    if (targets.length === 0) {
      return;
    }
    for (const memberId of targets) {
      const request: TerminalDispatchRequest = {
        memberId,
        conversationId: payload.conversation.id,
        conversationType: payload.conversation.type,
        senderId: payload.senderId,
        senderName: payload.senderName,
        text: payload.text
      };
      void enqueueTerminalDispatch(request);
    }
  };

  /**
   * 订阅终端流式输出事件。
   * 输入：处理函数。
   * 输出：取消订阅函数。
   */
  const onTerminalStreamMessage = (handler: (payload: TerminalChatPayload) => void) =>
    onTerminalStreamMessageBridge(handler);

  return {
    ensureMemberSession,
    openMemberTerminal,
    stopMemberSession,
    enqueueTerminalDispatch,
    dispatchConversationToTerminals,
    resetTerminalWindowReady,
    onTerminalStreamMessage
  };
});

