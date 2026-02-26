import { computed, ref, watch } from 'vue';
import { acceptHMRUpdate, defineStore, storeToRefs } from 'pinia';
import { AI_ASSISTANT_ID, CURRENT_USER_ID } from './data';
import { DEFAULT_AVATAR } from '@/shared/constants/avatars';
import { buildSeededAvatar, ensureAvatar } from '@/shared/utils/avatar';
import { useSettingsStore } from '@/features/global/settingsStore';
import type { Conversation, Member, Message, MessageContent, MessageMentionsPayload } from './types';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useTerminalMemberStore, type TerminalDispatchRequest } from '@/features/terminal/terminalMemberStore';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import type { ChatMessageCreatedPayload, ConversationDto, MessageDto } from './chatBridge';
import {
  clearConversation as clearConversationRemote,
  createGroupConversation as createGroupConversationRemote,
  deleteConversation as deleteConversationRemote,
  ensureDirectConversation,
  getConversationMessages,
  listConversations,
  markConversationRead as markConversationReadRemote,
  renameConversation as renameConversationRemote,
  sendConversationMessage,
  setConversationMembers as setConversationMembersRemote,
  setConversationSettings as setConversationSettingsRemote
} from './chatBridge';

const ASSISTANT_REPLY_DELAY = 900;
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

let loadSequence = 0;

export const useChatStore = defineStore('chat', () => {
  const workspaceStore = useWorkspaceStore();
  const projectStore = useProjectStore();
  const settingsStore = useSettingsStore();
  const terminalMemberStore = useTerminalMemberStore();
  const { currentWorkspace } = storeToRefs(workspaceStore);
  const { members } = storeToRefs(projectStore);
  const { settings } = storeToRefs(settingsStore);
  const { enqueueTerminalDispatch } = terminalMemberStore;

  const defaultState = () => ({
    conversations: [] as Conversation[],
    assistantTypingConversationId: null as string | null,
    isReady: false,
    chatError: null as string | null,
    defaultChannelId: null as string | null,
    totalUnreadCount: 0
  });

  const conversations = ref<Conversation[]>(defaultState().conversations);
  const assistantTypingConversationId = ref<string | null>(defaultState().assistantTypingConversationId);
  const assistantReplyTimeoutId = ref<number | null>(null);
  const isReady = ref(defaultState().isReady);
  const chatError = ref<string | null>(defaultState().chatError);
  const defaultChannelId = ref<string | null>(defaultState().defaultChannelId);
  const totalUnreadCount = ref<number>(defaultState().totalUnreadCount);
  const pendingTerminalMessages = ref<ChatMessageCreatedPayload[]>([]);
  const loadedMessages = new Set<string>();
  const loadingMessages = new Set<string>();
  const conversationPaging = new Map<string, { hasMore: boolean; loading: boolean }>();

  const currentUser = computed(
    () => members.value.find((member) => member.id === CURRENT_USER_ID) ?? members.value[0]
  );
  const currentUserId = computed(() => currentUser.value?.id ?? CURRENT_USER_ID);
  const accountAvatar = computed(() => ensureAvatar(settings.value.account.avatar));
  const assistantMember = computed(
    () =>
      members.value.find((member) => member.id === AI_ASSISTANT_ID) ??
      members.value.find((member) => member.roleType === 'assistant') ??
      null
  );
  const resolveSenderName = () => {
    const displayName = settings.value.account.displayName.trim();
    if (displayName) {
      return displayName;
    }
    return currentUser.value?.name ?? 'Owner';
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

  const applyMessageToConversation = (conversationId: string, message: Message) => {
    updateConversation(conversationId, (conversation) => {
      const messages = [...conversation.messages, message];
      const currentUnread = conversation.unreadCount ?? 0;
      const nextUnreadCount =
        message.senderId && message.senderId === currentUserId.value ? 0 : currentUnread + 1;
      return {
        ...conversation,
        messages,
        lastMessageAt: message.createdAt,
        lastMessagePreview: resolvePreviewText(message.content),
        unreadCount: nextUnreadCount
      };
    });
    updateConversationOrder();
  };

  const resolveTerminalTargets = (conversation: Conversation, mentions?: MessageMentionsPayload) => {
    const memberMap = new Map(members.value.map((member) => [member.id, member]));
    const isTerminalMember = (memberId: string) => {
      if (memberId === currentUserId.value) {
        return false;
      }
      const member = memberMap.get(memberId);
      if (!member) {
        return false;
      }
      return hasTerminalConfig(member.terminalType, member.terminalCommand);
    };

    if (conversation.type === 'dm') {
      const targetId =
        conversation.targetId ?? conversation.memberIds.find((id) => id !== currentUserId.value) ?? '';
      if (!targetId || !isTerminalMember(targetId)) {
        return [];
      }
      return [targetId];
    }

    if (!mentions) {
      return [];
    }
    const mentionIds = uniqueMemberIds(mentions.mentionIds ?? []);
    if (!mentions.mentionAll && mentionIds.length === 0) {
      return [];
    }
    const memberSet = new Set(conversation.memberIds ?? []);
    const sourceIds = mentions.mentionAll ? conversation.memberIds : mentionIds;
    if (mentions.mentionAll) {
      // TODO(seekskyworld): Notify admin user on @all without triggering terminal events (single-user mode only).
    }
    return uniqueMemberIds(sourceIds).filter((id) => memberSet.has(id)).filter(isTerminalMember);
  };

  const setConversationMessages = (conversationId: string, messages: Message[]) => {
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      messages
    }));
  };

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

  const queueAssistantReply = (prompt: string, conversationId: string) => {
    if (assistantReplyTimeoutId.value !== null) {
      window.clearTimeout(assistantReplyTimeoutId.value);
    }
    assistantTypingConversationId.value = conversationId;
    const replyKey = prompt.includes('?') ? 'chat.messages.autoReplyQuestion' : 'chat.messages.autoReply';

    assistantReplyTimeoutId.value = window.setTimeout(async () => {
      try {
        const workspaceId = currentWorkspace.value?.id;
        if (!workspaceId) return;
        const assistantId = assistantMember.value?.id ?? AI_ASSISTANT_ID;
        const result = await sendConversationMessage(
          workspaceId,
          conversationId,
          assistantId,
          { type: 'system', key: replyKey },
          true
        );
        const message = normalizeMessage(result);
        applyMessageToConversation(conversationId, message);
      } catch (error) {
        console.error('Failed to send assistant reply.', error);
      } finally {
        assistantTypingConversationId.value = null;
        assistantReplyTimeoutId.value = null;
      }
    }, ASSISTANT_REPLY_DELAY);
  };

  const cancelAssistantReply = () => {
    if (assistantReplyTimeoutId.value !== null) {
      window.clearTimeout(assistantReplyTimeoutId.value);
      assistantReplyTimeoutId.value = null;
    }
    assistantTypingConversationId.value = null;
  };

  const sendMessage = async (payload: { text: string; conversationId: string; mentions?: MessageMentionsPayload }) => {
    const trimmed = payload.text.trim();
    if (!trimmed) return null;
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId) return null;
    const conversation = conversations.value.find((item) => item.id === payload.conversationId);
    if (!conversation) return null;

    const text = trimmed.slice(0, MAX_MESSAGE_LENGTH);
    try {
      const result = await sendConversationMessage(
        workspaceId,
        payload.conversationId,
        currentUserId.value,
        { type: 'text', text },
        false
      );
      const message = normalizeMessage(result);
      applyMessageToConversation(payload.conversationId, message);

      if (assistantMember.value) {
        queueAssistantReply(text, payload.conversationId);
      }

      const targets = resolveTerminalTargets(conversation, payload.mentions);
      if (targets.length > 0) {
        const senderName = resolveSenderName();
        for (const memberId of targets) {
          const request: TerminalDispatchRequest = {
            memberId,
            conversationId: conversation.id,
            conversationType: conversation.type,
            senderId: currentUserId.value,
            senderName,
            text
          };
          void enqueueTerminalDispatch(request).catch((error) => {
            console.error('Failed to dispatch terminal message.', error);
          });
        }
      }

      return message;
    } catch (error) {
      console.error('Failed to send message.', error);
      return null;
    }
  };

  const appendTerminalMessage = (payload: ChatMessageCreatedPayload) => {
    if (!payload || !payload.conversationId) {
      return;
    }
    if (!isReady.value) {
      pendingTerminalMessages.value.push(payload);
      return;
    }
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId) return;
    if (payload.workspaceId && payload.workspaceId !== workspaceId) return;
    const conversationId = payload.conversationId;
    const conversation = conversations.value.find((item) => item.id === conversationId);
    if (!conversation) return;
    const message = normalizeMessage(payload.message);
    applyMessageToConversation(conversationId, message);
    if (typeof payload.totalUnreadCount === 'number') {
      totalUnreadCount.value = payload.totalUnreadCount;
    }
  };

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

  const setConversationMembers = async (conversationId: string, memberIds: string[]) => {
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId) return;
    const nextMembers = uniqueMemberIds([currentUserId.value, ...memberIds]);
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      memberIds: nextMembers
    }));
    try {
      await setConversationMembersRemote(workspaceId, conversationId, nextMembers);
    } catch (error) {
      console.error('Failed to update conversation members.', error);
    }
  };

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

  const markConversationRead = async (conversationId: string) => {
    if (!conversationId) return;
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      unreadCount: 0
    }));
    totalUnreadCount.value = conversations.value.reduce(
      (sum, conversation) => sum + (conversation.unreadCount ?? 0),
      0
    );
    const workspaceId = currentWorkspace.value?.id;
    if (!workspaceId) return;
    try {
      await markConversationReadRemote(workspaceId, currentUserId.value, conversationId);
    } catch (error) {
      console.error('Failed to mark conversation read.', error);
    }
  };

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

  const reset = () => {
    isReady.value = false;
    chatError.value = null;
    pendingTerminalMessages.value = [];
    defaultChannelId.value = null;
    totalUnreadCount.value = 0;
    conversations.value = [];
    loadedMessages.clear();
    loadingMessages.clear();
    conversationPaging.clear();
    if (assistantReplyTimeoutId.value !== null) {
      window.clearTimeout(assistantReplyTimeoutId.value);
      assistantReplyTimeoutId.value = null;
    }
    assistantTypingConversationId.value = null;
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
      for (const payload of pending) {
        void appendTerminalMessage(payload);
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

  return {
    conversations,
    currentUser,
    assistantMember,
    assistantTypingConversationId,
    cancelAssistantReply,
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
    appendTerminalMessage,
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
