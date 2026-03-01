// 聊天状态管理：负责会话列表、消息流与终端联动。
import { computed, ref, watch } from 'vue';
import { acceptHMRUpdate, defineStore, storeToRefs } from 'pinia';
import { CURRENT_USER_ID } from './data';
import { DEFAULT_AVATAR } from '@/shared/constants/avatars';
import { buildSeededAvatar, ensureAvatar } from '@/shared/utils/avatar';
import { buildGroupConversationTitle } from './utils';
import { i18n } from '@/i18n';
import { useSettingsStore } from '@/features/global/settingsStore';
import type { Conversation, Member, Message, MessageContent, MessageMentionsPayload } from './types';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';
import { isTauri } from '@tauri-apps/api/core';
import type {
  ChatMessageCreatedPayload,
  ChatMessageStatusPayload,
  ChatDispatchMentions,
  ChatUnreadSyncPayload,
  ConversationDto,
  MessageDto
} from './chatBridge';
import {
  clearConversation as clearConversationRemote,
  createGroupConversation as createGroupConversationRemote,
  deleteConversation as deleteConversationRemote,
  ensureDirectConversation,
  generateUlid,
  getConversationMessages,
  listConversations,
  markConversationRead as markConversationReadRemote,
  onChatMessageStatus,
  onChatUnreadSync,
  renameConversation as renameConversationRemote,
  sendConversationMessageAndDispatch,
  setConversationMembers as setConversationMembersRemote,
  setConversationSettings as setConversationSettingsRemote
} from './chatBridge';

// 延迟与分页参数用于控制 UI 节奏与一次加载上限。
const MAX_MESSAGE_LENGTH = 1200;
const MESSAGES_PAGE_LIMIT = 200;

const uniqueMemberIds = (ids: string[]) => Array.from(new Set(ids.filter((id) => id)));

const normalizeConversationMemberIds = (ids: string[], members: Member[]) => {
  const knownIds = new Set(members.map((member) => member.id));
  return uniqueMemberIds(ids.filter((id) => knownIds.has(id)));
};

const resolveMessageAvatar = (members: Member[], senderId?: string, fallbackName?: string) => {
  if (senderId) {
    const member = members.find((candidate) => candidate.id === senderId);
    if (member?.avatar) {
      return member.avatar;
    }
    if (member?.name) {
      return buildSeededAvatar(member.name);
    }
  }
  if (fallbackName) {
    return buildSeededAvatar(fallbackName);
  }
  return DEFAULT_AVATAR;
};

const resolvePreviewText = (content: MessageContent) => {
  if (content.type === 'text') {
    return content.text;
  }
  return content.key;
};

const formatError = (error: unknown) => (error instanceof Error ? error.message : String(error));

// 防止并发加载覆盖状态的序号标记。
let loadSequence = 0;
let unreadSyncListenerBound = false;
let messageStatusListenerBound = false;

// readThrough：前台阅读时触发已读同步，未读以服务端为准。
type TerminalMessageOptions = { readThrough?: boolean };
type PendingTerminalMessage = { payload: ChatMessageCreatedPayload; readThrough: boolean };
type TerminalStreamPayload = {
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
  type?: string;
  source?: string;
  mode?: string;
  spanId?: string;
};

/**
 * 聊天状态存储。
 * 输入：会话加载、消息发送与会话操作。
 * 输出：会话列表与操作方法。
 */
export const useChatStore = defineStore('chat', () => {
  const workspaceStore = useWorkspaceStore();
  const projectStore = useProjectStore();
  const settingsStore = useSettingsStore();
  const { currentWorkspace } = storeToRefs(workspaceStore);
  const { members } = storeToRefs(projectStore);
  const { settings } = storeToRefs(settingsStore);
  const streamOutputEnabled = computed(() => settings.value.chat.streamOutput);

  const defaultState = () => ({
    conversations: [] as Conversation[],
    isReady: false,
    chatError: null as string | null,
    defaultChannelId: null as string | null,
    totalUnreadCount: 0
  });

  const conversations = ref<Conversation[]>(defaultState().conversations);
  const isReady = ref(defaultState().isReady);
  const chatError = ref<string | null>(defaultState().chatError);
  const defaultChannelId = ref<string | null>(defaultState().defaultChannelId);
  const totalUnreadCount = ref<number>(defaultState().totalUnreadCount);
  const pendingTerminalMessages = ref<PendingTerminalMessage[]>([]);
  const streamMessageIds = new Map<string, string>();
  const loadedMessages = new Set<string>();
  const loadingMessages = new Set<string>();
  const conversationPaging = new Map<string, { hasMore: boolean; loading: boolean }>();

  const currentUser = computed(
    () => members.value.find((member) => member.id === CURRENT_USER_ID) ?? members.value[0]
  );
  const currentUserId = computed(() => currentUser.value?.id ?? CURRENT_USER_ID);
  const accountAvatar = computed(() => ensureAvatar(settings.value.account.avatar));
  const resolveSenderName = () => {
    const displayName = settings.value.account.displayName.trim();
    if (displayName) {
      return displayName;
    }
    return currentUser.value?.name ?? 'Owner';
  };

  const resolveConversationTitle = (conversation: Conversation) => {
    if (conversation.type === 'dm') {
      const targetId = conversation.targetId ?? '';
      const target = members.value.find((member) => member.id === targetId);
      return target?.name ?? 'Member';
    }
    const workspaceLabel = currentWorkspace.value?.name?.trim();
    if ((conversation.isDefault || conversation.id === defaultChannelId.value) && workspaceLabel) {
      return workspaceLabel;
    }
    if (conversation.customName) {
      return conversation.customName;
    }
    if (conversation.nameKey) {
      return i18n.global.t(conversation.nameKey);
    }
    const groupTitle = buildGroupConversationTitle(
      conversation.memberIds,
      members.value,
      currentUserId.value,
      25
    );
    return groupTitle || conversation.id;
  };

  const normalizeConversation = (dto: ConversationDto): Conversation => {
    const memberIds = normalizeConversationMemberIds(dto.memberIds ?? [], members.value);
    let targetId: string | undefined = dto.targetId;
    if (dto.type === 'dm' && !targetId) {
      targetId = memberIds.find((id) => id !== currentUserId.value);
    }
    return {
      id: dto.id,
      type: dto.type,
      targetId,
      memberIds,
      nameKey: undefined,
      customName: dto.customName ?? undefined,
      descriptionKey: undefined,
      pinned: Boolean(dto.pinned),
      muted: Boolean(dto.muted),
      lastMessageAt: dto.lastMessageAt ?? undefined,
      lastMessagePreview: dto.lastMessagePreview ?? undefined,
      lastMessageSenderId: undefined,
      lastMessageSenderName: undefined,
      lastMessageSenderAvatar: undefined,
      lastMessageAttachment: undefined,
      isDefault: dto.isDefault ?? false,
      unreadCount: dto.unreadCount ?? 0,
      messages: []
    };
  };

  const normalizeMessage = (dto: MessageDto): Message => {
    const senderId = dto.senderId;
    const member = senderId ? members.value.find((candidate) => candidate.id === senderId) : undefined;
    const userKey = !member && dto.isAi ? 'members.roles.aiAssistant' : undefined;
    const user = member?.name ?? (dto.isAi ? '' : '');
    const avatar =
      senderId === currentUserId.value
        ? accountAvatar.value
        : member?.avatar ?? resolveMessageAvatar(members.value, senderId, user || undefined);

    return {
      id: dto.id,
      senderId,
      user,
      userKey,
      userArgs: undefined,
      avatar,
      content: dto.content,
      createdAt: dto.createdAt,
      isAi: dto.isAi,
      attachment: dto.attachment,
      status: dto.status
    };
  };

  const sortConversations = (items: Conversation[]) =>
    [...items].sort((a, b) => {
      if (a.pinned !== b.pinned) {
        return a.pinned ? -1 : 1;
      }
      const timeA = a.lastMessageAt ?? 0;
      const timeB = b.lastMessageAt ?? 0;
      if (timeA !== timeB) {
        return timeB - timeA;
      }
      const nameA = a.customName ?? a.id;
      const nameB = b.customName ?? b.id;
      return nameA.localeCompare(nameB);
    });

  const updateConversation = (conversationId: string, updater: (conversation: Conversation) => Conversation) => {
    conversations.value = conversations.value.map((conversation) =>
      conversation.id === conversationId ? updater(conversation) : conversation
    );
  };

  const updateConversationOrder = () => {
    conversations.value = sortConversations(conversations.value);
  };

  // 以服务端未读为真相，收到同步事件后覆盖本地状态。
  const applyUnreadSync = (payload: ChatUnreadSyncPayload) => {
    if (!payload) {
      return;
    }
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId || payload.workspaceId !== workspaceId) {
      return;
    }
    if (typeof payload.totalUnreadCount === 'number') {
      totalUnreadCount.value = payload.totalUnreadCount;
    }
    if (payload.resetAll) {
      conversations.value = conversations.value.map((conversation) => ({
        ...conversation,
        unreadCount: 0
      }));
      return;
    }
    const conversationId = payload.conversationId?.trim();
    if (!conversationId) {
      return;
    }
    if (typeof payload.conversationUnreadCount !== 'number') {
      return;
    }
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      unreadCount: payload.conversationUnreadCount ?? 0
    }));
  };

  const applyMessageStatus = (payload: ChatMessageStatusPayload) => {
    if (!payload) {
      return;
    }
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId || payload.workspaceId !== workspaceId) {
      return;
    }
    const conversationId = payload.conversationId?.trim();
    if (!conversationId) {
      return;
    }
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      messages: conversation.messages.map((message) =>
        message.id === payload.messageId ? { ...message, status: payload.status } : message
      )
    }));
  };

  const registerUnreadSyncListener = () => {
    if (!isTauri() || unreadSyncListenerBound) {
      return;
    }
    unreadSyncListenerBound = true;
    onChatUnreadSync((payload) => {
      applyUnreadSync(payload);
    });
  };

  const registerMessageStatusListener = () => {
    if (!isTauri() || messageStatusListenerBound) {
      return;
    }
    messageStatusListenerBound = true;
    onChatMessageStatus((payload) => {
      applyMessageStatus(payload);
    });
  };

  const applyMessageToConversation = (conversationId: string, message: Message) => {
    updateConversation(conversationId, (conversation) => {
      const messages = [...conversation.messages, message];
      const senderName =
        message.user ||
        (message.senderId
          ? members.value.find((candidate) => candidate.id === message.senderId)?.name
          : undefined) ||
        (message.isAi ? 'Assistant' : 'Member');
      return {
        ...conversation,
        messages,
        lastMessageAt: message.createdAt,
        lastMessagePreview: resolvePreviewText(message.content),
        lastMessageSenderId: message.senderId,
        lastMessageSenderName: senderName,
        lastMessageSenderAvatar: message.avatar,
        lastMessageAttachment: message.attachment,
        unreadCount: conversation.unreadCount
      };
    });
    updateConversationOrder();
  };

  const buildStreamMessageId = (spanId: string) => `terminal-stream:${spanId}`;

  const applyTerminalStreamMessage = (payload: TerminalStreamPayload) => {
    if (!payload || payload.mode === 'final') {
      return;
    }
    if (!streamOutputEnabled.value) {
      return;
    }
    const conversationId = payload.conversationId?.trim();
    const spanId = payload.spanId?.trim();
    if (!conversationId || !spanId) {
      return;
    }
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId || (payload.workspaceId && payload.workspaceId !== workspaceId)) {
      return;
    }
    if (!loadedMessages.has(conversationId)) {
      return;
    }
    const rawText = payload.content ?? '';
    const text = payload.mode === 'delta' ? rawText : rawText.trimEnd();
    if (!text) {
      return;
    }
    const messageId = streamMessageIds.get(spanId) ?? buildStreamMessageId(spanId);
    streamMessageIds.set(spanId, messageId);
    const dto: MessageDto = {
      id: messageId,
      senderId: payload.memberId,
      content: { type: 'text', text },
      createdAt: payload.timestamp || Date.now(),
      isAi: false,
      status: 'sending'
    };
    const message = normalizeMessage(dto);
    updateConversation(conversationId, (conversation) => {
      const messages = [...conversation.messages];
      const index = messages.findIndex((item) => item.id === messageId);
      if (index >= 0) {
        const current = messages[index];
        const currentText = current.content.type === 'text' ? current.content.text : '';
        const nextText = payload.mode === 'delta' ? `${currentText}${text}` : text;
        messages[index] = {
          ...current,
          content: { type: 'text', text: nextText }
        };
      } else {
        messages.push(message);
      }
      return {
        ...conversation,
        messages
      };
    });
  };

  const setConversationMessages = (conversationId: string, messages: Message[]) => {
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      messages
    }));
  };

  /**
   * 获取会话分页状态，必要时初始化默认值。
   * 输入：conversationId。
   * 输出：分页状态对象。
   */
  const getPagingState = (conversationId: string) => {
    const state = conversationPaging.get(conversationId);
    if (state) {
      return state;
    }
    const next = { hasMore: true, loading: false };
    conversationPaging.set(conversationId, next);
    return next;
  };

  const updatePagingState = (conversationId: string, updater: (state: { hasMore: boolean; loading: boolean }) => void) => {
    const state = getPagingState(conversationId);
    updater(state);
    conversationPaging.set(conversationId, state);
  };

  /**
   * 加载会话列表与首页统计。
   * 输入：无（依赖当前工作区）。
   * 输出：更新 conversations/isReady 等状态。
   */
  const loadSession = async () => {
    const workspace = currentWorkspace.value;
    if (!workspace) {
      conversations.value = [];
      defaultChannelId.value = null;
      totalUnreadCount.value = 0;
      isReady.value = false;
      return;
    }

    const requestId = ++loadSequence;
    isReady.value = false;
    chatError.value = null;

    try {
      const memberIds = members.value.map((member) => member.id);
      const feed = await listConversations(
        workspace.id,
        currentUserId.value,
        workspace.name,
        memberIds
      );

      if (requestId !== loadSequence || workspace.id !== currentWorkspace.value?.id) {
        return;
      }

      const merged = [...(feed.pinned ?? []), ...(feed.timeline ?? [])];
      const seen = new Set<string>();
      const normalized: Conversation[] = [];
      for (const dto of merged) {
        if (!dto || !dto.id || seen.has(dto.id)) {
          continue;
        }
        seen.add(dto.id);
        normalized.push(normalizeConversation(dto));
      }

      conversations.value = sortConversations(normalized);
      defaultChannelId.value = feed.defaultChannelId ?? null;
      totalUnreadCount.value = feed.totalUnreadCount ?? 0;
      isReady.value = true;
    } catch (error) {
      chatError.value = formatError(error);
      console.error('Failed to load chat data.', error);
    }
  };

  /**
   * 加载会话消息首屏。
   * 输入：conversationId 与可选强制刷新标记。
   * 输出：无；失败时记录错误。
   */
  const loadConversationMessages = async (conversationId: string, force = false) => {
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId || !conversationId) return;
    if (loadingMessages.has(conversationId)) return;
    if (!force && loadedMessages.has(conversationId)) return;
    loadingMessages.add(conversationId);
    updatePagingState(conversationId, (state) => {
      state.loading = true;
    });
    try {
      const dtos = await getConversationMessages(workspaceId, conversationId, MESSAGES_PAGE_LIMIT);
      const messages = dtos.map((dto) => normalizeMessage(dto));
      setConversationMessages(conversationId, messages);
      loadedMessages.add(conversationId);
      updatePagingState(conversationId, (state) => {
        state.hasMore = dtos.length >= MESSAGES_PAGE_LIMIT;
      });
    } catch (error) {
      chatError.value = formatError(error);
      console.error('Failed to load conversation messages.', error);
    } finally {
      loadingMessages.delete(conversationId);
      updatePagingState(conversationId, (state) => {
        state.loading = false;
      });
    }
  };

  /**
   * 仅补齐会话的最新消息元信息，不改写消息列表。
   * 输入：conversationId。
   * 输出：最新消息或 null。
   */
  const ensureConversationLatestMessage = async (conversationId: string) => {
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId || !conversationId) return null;
    const conversation = conversations.value.find((item) => item.id === conversationId);
    if (!conversation) return null;
    if (conversation.lastMessageSenderId || conversation.lastMessageSenderName || conversation.lastMessageSenderAvatar) {
      return null;
    }
    try {
      const dtos = await getConversationMessages(workspaceId, conversationId, 1);
      if (!dtos.length) return null;
      const message = normalizeMessage(dtos[0]);
      const senderName =
        message.user ||
        (message.senderId
          ? members.value.find((candidate) => candidate.id === message.senderId)?.name
          : undefined) ||
        (message.isAi ? 'Assistant' : 'Member');
      updateConversation(conversationId, (item) => ({
        ...item,
        lastMessageAt: message.createdAt,
        lastMessagePreview: resolvePreviewText(message.content),
        lastMessageSenderId: message.senderId,
        lastMessageSenderName: senderName,
        lastMessageSenderAvatar: message.avatar,
        lastMessageAttachment: message.attachment
      }));
      return message;
    } catch (error) {
      console.error('Failed to load latest conversation message.', error);
      return null;
    }
  };

  /**
   * 加载更早的会话消息。
   * 输入：conversationId。
   * 输出：无；无更多数据时更新 hasMore。
   */
  const loadOlderMessages = async (conversationId: string) => {
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId || !conversationId) return;
    const paging = getPagingState(conversationId);
    if (paging.loading || !paging.hasMore) return;
    const conversation = conversations.value.find((item) => item.id === conversationId);
    const beforeId = conversation?.messages[0]?.id;
    updatePagingState(conversationId, (state) => {
      state.loading = true;
    });
    try {
      const dtos = await getConversationMessages(
        workspaceId,
        conversationId,
        MESSAGES_PAGE_LIMIT,
        beforeId
      );
      if (dtos.length === 0) {
        updatePagingState(conversationId, (state) => {
          state.hasMore = false;
        });
        return;
      }
      const messages = dtos.map((dto) => normalizeMessage(dto));
      updateConversation(conversationId, (conversation) => ({
        ...conversation,
        messages: [...messages, ...conversation.messages]
      }));
      updatePagingState(conversationId, (state) => {
        state.hasMore = dtos.length >= MESSAGES_PAGE_LIMIT;
      });
    } catch (error) {
      console.error('Failed to load older messages.', error);
    } finally {
      updatePagingState(conversationId, (state) => {
        state.loading = false;
      });
    }
  };

  /**
   * 发送消息并同步到会话列表。
   * 输入：文本、会话 id 与可选提及信息。
   * 输出：发送后的消息或 null。
   */
  const sendMessage = async (payload: { text: string; conversationId: string; mentions?: MessageMentionsPayload }) => {
    const trimmed = payload.text.trim();
    if (!trimmed) return null;
    const workspaceId = currentWorkspace.value?.id;
    const workspacePath = currentWorkspace.value?.path;
    if (!workspaceId) return null;
    if (!workspacePath) return null;
    const conversation = conversations.value.find((item) => item.id === payload.conversationId);
    if (!conversation) return null;

    const text = trimmed.slice(0, MAX_MESSAGE_LENGTH);
    try {
      const senderName = resolveSenderName();
      const messageId = isTauri() ? await generateUlid() : crypto.randomUUID();
      const clientTraceId = isTauri() ? await generateUlid() : crypto.randomUUID();
      const mentions: ChatDispatchMentions = payload.mentions ?? { mentionIds: [], mentionAll: false };
      const result = await sendConversationMessageAndDispatch({
        workspaceId,
        workspacePath,
        conversationId: payload.conversationId,
        conversationType: conversation.type,
        text,
        senderId: currentUserId.value,
        senderName,
        mentions,
        messageId,
        clientTraceId,
        timestamp: Date.now()
      });
      const message = normalizeMessage(result);
      applyMessageToConversation(payload.conversationId, message);
      void logDiagnosticsEvent('send-message', {
        conversationId: payload.conversationId,
        messageId: message.id,
        senderId: currentUserId.value,
        text,
        mentions: payload.mentions ?? null
      });

      return message;
    } catch (error) {
      console.error('Failed to send message.', error);
      return null;
    }
  };

  /**
   * 追加终端输出消息到会话。
   * 输入：终端消息 payload。
   * 输出：无；未就绪时进入队列缓冲。
   */
  const applyTerminalFinalMessage = (
    payload: ChatMessageCreatedPayload,
    options?: TerminalMessageOptions
  ) => {
    if (!payload || !payload.conversationId) {
      return;
    }
    const readThrough = Boolean(options?.readThrough);
    if (!isReady.value) {
      pendingTerminalMessages.value.push({ payload, readThrough });
      return;
    }
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId) return;
    if (payload.workspaceId && payload.workspaceId !== workspaceId) return;
    const conversationId = payload.conversationId;
    const spanId = payload.spanId?.trim();
    const streamMessageId = spanId ? streamMessageIds.get(spanId) : null;
    if (spanId) {
      streamMessageIds.delete(spanId);
    }
    const conversation = conversations.value.find((item) => item.id === conversationId);
    if (!conversation) return;
    const message = normalizeMessage(payload.message);
    if (streamMessageId) {
      updateConversation(conversationId, (current) => {
        const messages = [...current.messages];
        const index = messages.findIndex((item) => item.id === streamMessageId);
        if (index >= 0) {
          messages[index] = message;
        } else {
          messages.push(message);
        }
        const senderName =
          message.user ||
          (message.senderId
            ? members.value.find((candidate) => candidate.id === message.senderId)?.name
            : undefined) ||
          (message.isAi ? 'Assistant' : 'Member');
        return {
          ...current,
          messages,
          lastMessageAt: message.createdAt,
          lastMessagePreview: resolvePreviewText(message.content),
          lastMessageSenderId: message.senderId,
          lastMessageSenderName: senderName,
          lastMessageSenderAvatar: message.avatar,
          lastMessageAttachment: message.attachment,
          unreadCount: current.unreadCount
        };
      });
      updateConversationOrder();
    } else {
      applyMessageToConversation(conversationId, message);
    }
    void logDiagnosticsEvent('append-terminal-message', {
      conversationId,
      messageId: message.id,
      senderId: message.senderId,
      content: message.content
    });
    if (readThrough) {
      // 前台消息直接触发已读同步，未读以服务端广播为准。
      void markConversationRead(conversationId);
    }
  };

  const appendTerminalMessage = (payload: ChatMessageCreatedPayload, options?: TerminalMessageOptions) =>
    applyTerminalFinalMessage(payload, options);

  /**
   * 确保与指定成员的私聊会话存在。
   * 输入：成员 id。
   * 输出：会话 id 或 null。
   */
  const ensureDirectMessage = async (memberId: string) => {
    if (!memberId || memberId === currentUserId.value) return null;
    if (!members.value.some((member) => member.id === memberId)) return null;

    const existing = conversations.value.find(
      (conversation) => conversation.type === 'dm' && conversation.targetId === memberId
    );
    if (existing) {
      return existing.id;
    }

    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId) return null;

    try {
      const dto = await ensureDirectConversation(workspaceId, currentUserId.value, memberId);
      const conversation = normalizeConversation(dto);
      conversations.value = sortConversations([...conversations.value, conversation]);
      return conversation.id;
    } catch (error) {
      console.error('Failed to create direct conversation.', error);
      return null;
    }
  };

  /**
   * 创建群聊会话。
   * 输入：成员 id 列表与可选名称。
   * 输出：会话对象或 null。
   */
  const createGroupConversation = async (memberIds: string[], customName?: string | null) => {
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId) return null;
    const nextMembers = uniqueMemberIds([currentUserId.value, ...memberIds]);
    if (nextMembers.length < 2) return null;
    try {
      const dto = await createGroupConversationRemote(
        workspaceId,
        currentUserId.value,
        nextMembers,
        customName ?? undefined
      );
      const conversation = normalizeConversation(dto);
      conversations.value = sortConversations([...conversations.value, conversation]);
      return conversation;
    } catch (error) {
      console.error('Failed to create group conversation.', error);
      return null;
    }
  };

  /**
   * 更新会话成员列表。
   * 输入：conversationId 与成员 id 列表。
   * 输出：无。
   */
  const setConversationMembers = async (conversationId: string, memberIds: string[]) => {
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId) return;
    const nextMembers = uniqueMemberIds([currentUserId.value, ...memberIds]);
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      memberIds: nextMembers
    }));
    void logDiagnosticsEvent('set-conversation-members', {
      conversationId,
      memberIds: nextMembers
    });
    try {
      await setConversationMembersRemote(workspaceId, conversationId, nextMembers);
    } catch (error) {
      console.error('Failed to update conversation members.', error);
    }
  };

  /**
   * 切换会话置顶状态。
   * 输入：conversationId。
   * 输出：无。
   */
  const toggleConversationPin = async (conversationId: string) => {
    const conversation = conversations.value.find((item) => item.id === conversationId);
    if (!conversation) return;
    const nextPinned = !conversation.pinned;
    updateConversation(conversationId, (item) => ({ ...item, pinned: nextPinned }));
    updateConversationOrder();
    try {
      const workspaceId = currentWorkspace.value?.id;
      if (!workspaceId) return;
      await setConversationSettingsRemote(workspaceId, currentUserId.value, conversationId, nextPinned, undefined);
    } catch (error) {
      console.error('Failed to update pin state.', error);
    }
  };

  /**
   * 切换会话静音状态。
   * 输入：conversationId。
   * 输出：无。
   */
  const toggleConversationMute = async (conversationId: string) => {
    const conversation = conversations.value.find((item) => item.id === conversationId);
    if (!conversation) return;
    const nextMuted = !conversation.muted;
    updateConversation(conversationId, (item) => ({ ...item, muted: nextMuted }));
    try {
      const workspaceId = currentWorkspace.value?.id;
      if (!workspaceId) return;
      await setConversationSettingsRemote(workspaceId, currentUserId.value, conversationId, undefined, nextMuted);
    } catch (error) {
      console.error('Failed to update mute state.', error);
    }
  };

  /**
   * 重命名会话。
   * 输入：conversationId 与新名称。
   * 输出：无。
   */
  const renameConversation = async (conversationId: string, name: string) => {
    const trimmed = name.trim();
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      customName: trimmed ? trimmed : undefined
    }));
    updateConversationOrder();
    try {
      const workspaceId = currentWorkspace.value?.id;
      if (!workspaceId) return;
      await renameConversationRemote(workspaceId, conversationId, trimmed ? trimmed : null);
    } catch (error) {
      console.error('Failed to rename conversation.', error);
    }
  };

  /**
   * 清空会话消息。
   * 输入：conversationId。
   * 输出：无。
   */
  const clearConversationMessages = async (conversationId: string) => {
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      messages: [],
      lastMessageAt: undefined,
      lastMessagePreview: undefined
    }));
    updateConversationOrder();
    loadedMessages.add(conversationId);
    try {
      const workspaceId = currentWorkspace.value?.id;
      if (!workspaceId) return;
      await clearConversationRemote(workspaceId, conversationId);
    } catch (error) {
      console.error('Failed to clear conversation.', error);
    }
  };

  const deleteConversationLocal = (conversationId: string) => {
    conversations.value = conversations.value.filter((conversation) => conversation.id !== conversationId);
    loadedMessages.delete(conversationId);
    loadingMessages.delete(conversationId);
    conversationPaging.delete(conversationId);
  };

  /**
   * 删除会话并清理本地缓存。
   * 输入：conversationId。
   * 输出：无。
   */
  const deleteConversation = async (conversationId: string) => {
    deleteConversationLocal(conversationId);
    try {
      const workspaceId = currentWorkspace.value?.id;
      if (!workspaceId) return;
      await deleteConversationRemote(workspaceId, conversationId);
    } catch (error) {
      console.error('Failed to delete conversation.', error);
    }
  };

  /**
   * 标记会话已读并等待服务端同步未读数。
   * 输入：conversationId。
   * 输出：无。
   */
  const markConversationRead = async (conversationId: string) => {
    if (!conversationId) return;
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId) return;
    try {
      await markConversationReadRemote(workspaceId, currentUserId.value, conversationId);
    } catch (error) {
      console.error('Failed to mark conversation read.', error);
    }
  };

  /**
   * 批量标记所有会话已读。
   * 输入：无。
   * 输出：无。
   */
  const markAllConversationsRead = async () => {
    const targets = conversations.value
      .filter((conversation) => (conversation.unreadCount ?? 0) > 0)
      .map((conversation) => conversation.id);
    for (const conversationId of targets) {
      await markConversationRead(conversationId);
    }
  };

  /**
   * 删除与指定成员相关的私聊会话。
   * 输入：成员 id。
   * 输出：无。
   */
  const deleteMemberConversations = async (memberId: string) => {
    if (!memberId || memberId === currentUserId.value) return;
    const targets = conversations.value
      .filter((conversation) => {
        if (conversation.type !== 'dm') return false;
        const targetId =
          conversation.targetId ?? conversation.memberIds.find((id) => id !== currentUserId.value);
        return targetId === memberId || conversation.memberIds.includes(memberId);
      })
      .map((conversation) => conversation.id);
    for (const conversationId of targets) {
      await deleteConversation(conversationId);
    }
  };

  const refreshMessageAuthors = () => {
    const memberMap = new Map(members.value.map((member) => [member.id, member]));
    conversations.value = conversations.value.map((conversation) => ({
      ...conversation,
      messages: conversation.messages.map((message) => {
        if (!message.senderId) {
          return message;
        }
        const member = memberMap.get(message.senderId);
        if (!member) {
          return message;
        }
        const avatar = member.id === currentUserId.value ? accountAvatar.value : member.avatar;
        return {
          ...message,
          user: member.name,
          avatar,
          userKey: undefined,
          userArgs: undefined
        };
      })
    }));
  };

  const syncConversationMembers = async (memberIds: string[]) => {
    const memberSet = new Set(memberIds);
    const updates: Conversation[] = [];
    const remoteUpdates: Array<{ id: string; memberIds: string[] }> = [];

    const hasSameMembers = (a: string[], b: string[]) => {
      if (a.length !== b.length) return false;
      const setA = new Set(a);
      return b.every((id) => setA.has(id));
    };

    for (const conversation of conversations.value) {
      if (conversation.type === 'dm') {
        const targetId = conversation.targetId ?? conversation.memberIds.find((id) => id !== currentUserId.value);
        if (!targetId || !memberSet.has(targetId)) {
          continue;
        }
        const nextMembers = uniqueMemberIds([currentUserId.value, targetId]);
        updates.push({
          ...conversation,
          targetId,
          memberIds: nextMembers
        });
        continue;
      }

      if (conversation.isDefault) {
        const nextMembers = uniqueMemberIds(memberIds);
        if (!hasSameMembers(conversation.memberIds, nextMembers)) {
          remoteUpdates.push({ id: conversation.id, memberIds: nextMembers });
        }
        updates.push({
          ...conversation,
          memberIds: nextMembers
        });
        continue;
      }

      const filtered = conversation.memberIds.filter((id) => memberSet.has(id));
      const nextMembers = uniqueMemberIds([currentUserId.value, ...filtered]);
      if (!hasSameMembers(conversation.memberIds, nextMembers)) {
        remoteUpdates.push({ id: conversation.id, memberIds: nextMembers });
      }
      updates.push({
        ...conversation,
        memberIds: nextMembers
      });
    }

    conversations.value = updates;

    if (remoteUpdates.length === 0) {
      return;
    }
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId) return;
    for (const update of remoteUpdates) {
      try {
        await setConversationMembersRemote(workspaceId, update.id, update.memberIds);
      } catch (error) {
        console.error('Failed to sync conversation members.', error);
      }
    }
  };

  /**
   * 重置聊天状态并清理定时器/缓存。
   * 输入：无。
   * 输出：无。
   */
  const reset = () => {
    isReady.value = false;
    chatError.value = null;
    pendingTerminalMessages.value = [];
    streamMessageIds.clear();
    defaultChannelId.value = null;
    totalUnreadCount.value = 0;
    conversations.value = [];
    loadedMessages.clear();
    loadingMessages.clear();
    conversationPaging.clear();
    loadSequence += 1;
  };

  const memberAuthorSignature = computed(() =>
    JSON.stringify(
      members.value
        .map((member) => ({ id: member.id, name: member.name, avatar: member.avatar }))
        .sort((a, b) => a.id.localeCompare(b.id))
    )
  );

  watch(memberAuthorSignature, (next, prev) => {
    if (!isReady.value || !next || next === prev) {
      return;
    }
    refreshMessageAuthors();
  });

  watch(
    isReady,
    (ready) => {
      if (!ready || pendingTerminalMessages.value.length === 0) {
        return;
      }
      const pending = [...pendingTerminalMessages.value];
      pendingTerminalMessages.value = [];
      for (const item of pending) {
        void appendTerminalMessage(item.payload, { readThrough: item.readThrough });
      }
    },
    { flush: 'post' }
  );

  watch(
    () => members.value.map((member) => member.id).sort().join('|'),
    (next, prev) => {
      if (!isReady.value || next === prev || !next) return;
      void syncConversationMembers(next.split('|'));
    }
  );

  registerUnreadSyncListener();
  registerMessageStatusListener();

  return {
    conversations,
    currentUser,
    maxMessageLength: MAX_MESSAGE_LENGTH,
    sendMessage,
    ensureDirectMessage,
    createGroupConversation,
    setConversationMembers,
    loadConversationMessages,
    loadOlderMessages,
    toggleConversationPin,
    toggleConversationMute,
    renameConversation,
    clearConversationMessages,
    deleteConversation,
    deleteMemberConversations,
    markConversationRead,
    markAllConversationsRead,
    appendTerminalMessage,
    applyTerminalStreamMessage,
    getConversationTitle: resolveConversationTitle,
    ensureConversationLatestMessage,
    getConversationPaging: (conversationId: string) => getPagingState(conversationId),
    isReady,
    chatError,
    defaultChannelId,
    totalUnreadCount,
    loadSession,
    reset
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useChatStore, import.meta.hot));
}

