import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import {
  AI_ASSISTANT_ID,
  CURRENT_USER_ID,
  DEFAULT_CHANNEL_ID,
  DEFAULT_CHANNEL_NAME_KEY,
  initialMembers,
  initialMessages
} from '../data';
import { PRIMARY_USER_AVATAR_URL } from '../../../shared/constants/avatars';
import type { Conversation, ConversationType, Member, Message, MessageStatus } from '../types';
import { ensureUniqueName, formatMessageTime } from '../utils';

const STORAGE_KEY = 'nexus-chat-session-v1';
const SEND_SIMULATION_DELAY = 200;
const ASSISTANT_REPLY_DELAY = 900;
const MAX_MESSAGE_LENGTH = 1200;

type StoredSession = {
  members: Member[];
  conversations: Conversation[];
  nextMessageId: number;
  messages?: Message[];
  directMessageIds?: string[];
};

const buildConversationId = (type: ConversationType, targetId: string) => `${type}:${targetId}`;

const getMemberAvatar = (members: Member[], id?: string) => {
  if (!id) return undefined;
  return members.find((member) => member.id === id)?.avatar;
};

const getCurrentUserName = (members: Member[]) => members.find((member) => member.id === CURRENT_USER_ID)?.name ?? 'You';

const buildUiAvatar = (name: string) =>
  `https://ui-avatars.com/api/?name=${encodeURIComponent(name)}&background=0b57d0&color=fff`;

const resolveMessageAvatar = (message: Message, members: Member[]) => {
  const memberAvatar = getMemberAvatar(members, message.senderId);
  if (memberAvatar) return memberAvatar;

  if (message.avatar && message.avatar !== PRIMARY_USER_AVATAR_URL) {
    return message.avatar;
  }

  const currentUserName = getCurrentUserName(members);
  if (message.user && message.user !== currentUserName) {
    return buildUiAvatar(message.user);
  }

  return message.avatar || PRIMARY_USER_AVATAR_URL;
};

const getNextMessageIdFromConversations = (conversations: Conversation[]) =>
  conversations.reduce((max, conversation) => {
    const conversationMax = conversation.messages.reduce((messageMax, message) => Math.max(messageMax, message.id), 0);
    return Math.max(max, conversationMax);
  }, 0) + 1;

const normalizeMessage = (message: Message, members: Member[]): Message => {
  const createdAt = typeof message.createdAt === 'number' ? message.createdAt : Date.now();
  const status: MessageStatus = message.status === 'failed' ? 'failed' : 'sent';
  const avatar = resolveMessageAvatar(message, members);

  return {
    ...message,
    createdAt,
    time: message.time || formatMessageTime(createdAt),
    status,
    avatar
  };
};

const createChannelConversation = (messages: Message[], members: Member[]): Conversation => ({
  id: buildConversationId('channel', DEFAULT_CHANNEL_ID),
  type: 'channel',
  targetId: DEFAULT_CHANNEL_ID,
  nameKey: DEFAULT_CHANNEL_NAME_KEY,
  pinned: false,
  muted: false,
  messages: messages.map((message) => normalizeMessage(message, members))
});

const createDirectConversation = (memberId: string): Conversation => ({
  id: buildConversationId('dm', memberId),
  type: 'dm',
  targetId: memberId,
  pinned: false,
  muted: false,
  messages: []
});

const normalizeConversation = (conversation: Partial<Conversation>, members: Member[]): Conversation | null => {
  if (conversation.type !== 'channel' && conversation.type !== 'dm') return null;
  if (typeof conversation.id !== 'string' || !conversation.id) return null;
  if (typeof conversation.targetId !== 'string' || !conversation.targetId) return null;

  return {
    id: conversation.id,
    type: conversation.type,
    targetId: conversation.targetId,
    nameKey: typeof conversation.nameKey === 'string' ? conversation.nameKey : undefined,
    customName: typeof conversation.customName === 'string' ? conversation.customName : undefined,
    descriptionKey: typeof conversation.descriptionKey === 'string' ? conversation.descriptionKey : undefined,
    pinned: Boolean(conversation.pinned),
    muted: Boolean(conversation.muted),
    messages: Array.isArray(conversation.messages)
      ? conversation.messages.map((message) => normalizeMessage(message as Message, members))
      : []
  };
};

const buildDefaultSession = (): StoredSession => {
  const members = [...initialMembers];
  const defaultConversation = createChannelConversation(initialMessages, members);
  return {
    members,
    conversations: [defaultConversation],
    nextMessageId: getNextMessageIdFromConversations([defaultConversation])
  };
};

const loadStoredSession = (): StoredSession => {
  if (typeof window === 'undefined') {
    return buildDefaultSession();
  }

  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return buildDefaultSession();
  }

  try {
    const parsed = JSON.parse(raw) as Partial<StoredSession>;
    const members = Array.isArray(parsed.members) ? parsed.members : initialMembers;
    let conversations: Conversation[] = [];

    if (Array.isArray(parsed.conversations)) {
      conversations = parsed.conversations
        .map((conversation) => normalizeConversation(conversation, members))
        .filter((conversation): conversation is Conversation => Boolean(conversation));
    } else {
      const messages = Array.isArray(parsed.messages)
        ? parsed.messages.map((message) => normalizeMessage(message as Message, members))
        : initialMessages.map((message) => normalizeMessage(message, members));
      conversations = [createChannelConversation(messages, members)];

      if (Array.isArray(parsed.directMessageIds)) {
        parsed.directMessageIds
          .filter((id) => typeof id === 'string')
          .forEach((memberId) => {
            if (members.some((member) => member.id === memberId)) {
              conversations.push(createDirectConversation(memberId));
            }
          });
      }
    }

    conversations = conversations.filter((conversation) => {
      if (conversation.type === 'dm') {
        return members.some((member) => member.id === conversation.targetId);
      }
      return true;
    });

    const computedNextId = getNextMessageIdFromConversations(conversations);
    const nextMessageId =
      typeof parsed.nextMessageId === 'number' ? Math.max(parsed.nextMessageId, computedNextId) : computedNextId;

    return {
      members,
      conversations,
      nextMessageId
    };
  } catch {
    return buildDefaultSession();
  }
};

const persistSession = (session: StoredSession) => {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(session));
};

export const useChatSession = () => {
  const { t, locale } = useI18n();
  const stored = loadStoredSession();

  const members = ref<Member[]>(stored.members);
  const conversations = ref<Conversation[]>(stored.conversations);
  const nextMessageId = ref(stored.nextMessageId);
  const assistantReplyTimeoutId = ref<number | null>(null);
  const assistantTypingConversationId = ref<string | null>(null);

  const currentUser = computed(() => members.value.find((member) => member.id === CURRENT_USER_ID) ?? members.value[0]);
  const assistantMember = computed(
    () =>
      members.value.find((member) => member.id === AI_ASSISTANT_ID) ??
      members.value.find((member) => member.roleType === 'assistant') ??
      null
  );

  const updateConversation = (conversationId: string, updater: (conversation: Conversation) => Conversation) => {
    conversations.value = conversations.value.map((conversation) =>
      conversation.id === conversationId ? updater(conversation) : conversation
    );
  };

  const updateMessageStatus = (conversationId: string, id: number, status: MessageStatus) => {
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      messages: conversation.messages.map((message) => (message.id === id ? { ...message, status } : message))
    }));
  };

  const addMessage = (conversationId: string, message: Message) => {
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      messages: [...conversation.messages, message]
    }));
  };

  const sendMessage = (rawText: string, conversationId: string) => {
    const trimmed = rawText.trim();
    if (!trimmed) return null;
    const conversation = conversations.value.find((item) => item.id === conversationId);
    if (!conversation) return null;

    const text = trimmed.slice(0, MAX_MESSAGE_LENGTH);
    const createdAt = Date.now();
    const message: Message = {
      id: nextMessageId.value,
      senderId: CURRENT_USER_ID,
      user: currentUser.value?.name ?? 'You',
      avatar: currentUser.value?.avatar ?? PRIMARY_USER_AVATAR_URL,
      text,
      time: formatMessageTime(createdAt, locale.value),
      createdAt,
      isAi: false,
      status: 'sending'
    };

    nextMessageId.value += 1;
    addMessage(conversation.id, message);

    window.setTimeout(() => {
      updateMessageStatus(conversation.id, message.id, 'sent');
    }, SEND_SIMULATION_DELAY);

    if (assistantMember.value) {
      queueAssistantReply(text, conversation.id);
    }

    return message;
  };

  const queueAssistantReply = (prompt: string, conversationId: string) => {
    if (assistantReplyTimeoutId.value !== null) {
      window.clearTimeout(assistantReplyTimeoutId.value);
    }
    assistantTypingConversationId.value = conversationId;
    const replyText = prompt.includes('?') ? t('chat.messages.autoReplyQuestion') : t('chat.messages.autoReply');

    assistantReplyTimeoutId.value = window.setTimeout(() => {
      const createdAt = Date.now();
      const reply: Message = {
        id: nextMessageId.value,
        senderId: assistantMember.value?.id,
        user: assistantMember.value?.name ?? t('members.roles.aiAssistant'),
        avatar: assistantMember.value?.avatar ?? PRIMARY_USER_AVATAR_URL,
        text: replyText,
        time: formatMessageTime(createdAt, locale.value),
        createdAt,
        isAi: true,
        status: 'sent'
      };

      nextMessageId.value += 1;
      addMessage(conversationId, reply);
      assistantTypingConversationId.value = null;
      assistantReplyTimeoutId.value = null;
    }, ASSISTANT_REPLY_DELAY);
  };

  const cancelAssistantReply = () => {
    if (assistantReplyTimeoutId.value !== null) {
      window.clearTimeout(assistantReplyTimeoutId.value);
      assistantReplyTimeoutId.value = null;
    }
    assistantTypingConversationId.value = null;
  };

  const addMember = (member: Omit<Member, 'id'> & { id?: string }) => {
    const id = member.id ?? Date.now().toString();
    members.value = [...members.value, { ...member, id }];
    return id;
  };

  const updateMember = (id: string, updates: Partial<Member>) => {
    const updatedMember = members.value.find((member) => member.id === id);
    if (!updatedMember) return;

    const nextMember = { ...updatedMember, ...updates };
    members.value = members.value.map((member) => (member.id === id ? nextMember : member));

    conversations.value = conversations.value.map((conversation) => ({
      ...conversation,
      messages: conversation.messages.map((message) => {
        if (message.senderId !== id) return message;
        return {
          ...message,
          user: updates.name ?? message.user,
          avatar: updates.avatar ?? message.avatar
        };
      })
    }));
  };

  const removeMember = (id: string) => {
    if (id === CURRENT_USER_ID) return;
    members.value = members.value.filter((member) => member.id !== id);
    conversations.value = conversations.value.filter((conversation) =>
      conversation.type === 'dm' ? conversation.targetId !== id : true
    );
  };

  const ensureDirectMessage = (memberId: string) => {
    if (!memberId || memberId === CURRENT_USER_ID) return null;
    if (!members.value.some((member) => member.id === memberId)) return null;

    const existing = conversations.value.find((conversation) => conversation.type === 'dm' && conversation.targetId === memberId);
    if (existing) {
      return existing.id;
    }

    const conversation = createDirectConversation(memberId);
    conversations.value = [...conversations.value, conversation];
    return conversation.id;
  };

  const toggleConversationPin = (conversationId: string) => {
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      pinned: !conversation.pinned
    }));
  };

  const toggleConversationMute = (conversationId: string) => {
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      muted: !conversation.muted
    }));
  };

  const renameConversation = (conversationId: string, name: string) => {
    const trimmed = name.trim();
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      customName: trimmed ? trimmed : undefined
    }));
  };

  const clearConversationMessages = (conversationId: string) => {
    updateConversation(conversationId, (conversation) => ({
      ...conversation,
      messages: []
    }));
  };

  const deleteConversation = (conversationId: string) => {
    if (conversationId === buildConversationId('channel', DEFAULT_CHANNEL_ID)) {
      return;
    }
    conversations.value = conversations.value.filter((conversation) => conversation.id !== conversationId);
  };

  const createInviteMember = (name: string, roleType: 'assistant' | 'member', roleKey: string, status: Member['status']) => {
    const finalName = ensureUniqueName(name, members.value);
    const avatar = roleType === 'assistant'
      ? `https://ui-avatars.com/api/?name=${encodeURIComponent(finalName)}&background=0b57d0&color=fff`
      : `https://picsum.photos/seed/${encodeURIComponent(finalName)}/200/200`;

    return {
      id: Date.now().toString(),
      name: finalName,
      role: '',
      roleKey,
      roleType,
      avatar,
      status
    };
  };

  watch(
    [conversations, members, nextMessageId],
    () => {
      persistSession({
        conversations: conversations.value,
        members: members.value,
        nextMessageId: nextMessageId.value
      });
    },
    { deep: true }
  );

  return {
    members,
    conversations,
    currentUser,
    assistantMember,
    assistantTypingConversationId,
    cancelAssistantReply,
    maxMessageLength: MAX_MESSAGE_LENGTH,
    sendMessage,
    addMember,
    updateMember,
    removeMember,
    createInviteMember,
    ensureDirectMessage,
    toggleConversationPin,
    toggleConversationMute,
    renameConversation,
    clearConversationMessages,
    deleteConversation
  };
};
