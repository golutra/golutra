import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import type { MessageAttachment, MessageContent, MessageStatus } from './types';

export type ConversationDto = {
  id: string;
  type: 'channel' | 'dm';
  memberIds: string[];
  targetId?: string;
  customName?: string | null;
  pinned: boolean;
  muted: boolean;
  lastMessageAt?: number | null;
  lastMessagePreview?: string | null;
  isDefault?: boolean | null;
  unreadCount?: number | null;
};

export type ChatHomeFeed = {
  pinned: ConversationDto[];
  timeline: ConversationDto[];
  defaultChannelId?: string | null;
  totalUnreadCount?: number | null;
};

export type MessageDto = {
  id: string;
  senderId?: string;
  content: MessageContent;
  createdAt: number;
  isAi: boolean;
  status: MessageStatus;
  attachment?: MessageAttachment;
};

export type ChatMessageCreatedPayload = {
  workspaceId: string;
  conversationId: string;
  message: MessageDto;
  totalUnreadCount: number;
};

export const generateUlid = async () => {
  if (!isTauri()) {
    throw new Error('ULID generation requires Tauri runtime.');
  }
  return invoke<string>('chat_ulid_new');
};

export type ChatRepairResult = {
  removedMessages: number;
};

export type ChatClearResult = {
  removedMessages: number;
  removedAttachments: number;
  clearedTimeline: number;
};

export const listConversations = (
  workspaceId: string,
  userId: string,
  workspaceName: string,
  memberIds: string[]
) =>
  invoke<ChatHomeFeed>('chat_list_conversations', {
    workspaceId,
    userId,
    workspaceName,
    memberIds
  });

export const getConversationMessages = (workspaceId: string, conversationId: string, limit?: number, beforeId?: string) =>
  invoke<MessageDto[]>('chat_get_messages', {
    workspaceId,
    conversationId,
    limit,
    beforeId
  });

export const markConversationRead = (workspaceId: string, userId: string, conversationId: string) =>
  invoke('chat_mark_conversation_read_latest', {
    workspaceId,
    userId,
    conversationId
  });

export const sendConversationMessage = (
  workspaceId: string,
  conversationId: string,
  senderId: string | null,
  content: MessageContent,
  isAi?: boolean,
  attachment?: MessageAttachment
) =>
  invoke<MessageDto>('chat_send_message', {
    workspaceId,
    conversationId,
    senderId: senderId ?? undefined,
    content,
    isAi,
    attachment
  });

export const createGroupConversation = (
  workspaceId: string,
  userId: string,
  memberIds: string[],
  customName?: string | null
) =>
  invoke<ConversationDto>('chat_create_group', {
    workspaceId,
    userId,
    memberIds,
    customName
  });

export const ensureDirectConversation = (workspaceId: string, userId: string, targetId: string) =>
  invoke<ConversationDto>('chat_ensure_direct', { workspaceId, userId, targetId });

export const setConversationSettings = (
  workspaceId: string,
  userId: string,
  conversationId: string,
  pinned?: boolean,
  muted?: boolean
) =>
  invoke('chat_set_conversation_settings', {
    workspaceId,
    userId,
    conversationId,
    pinned,
    muted
  });

export const renameConversation = (workspaceId: string, conversationId: string, customName?: string | null) =>
  invoke('chat_rename_conversation', { workspaceId, conversationId, customName });

export const clearConversation = (workspaceId: string, conversationId: string) =>
  invoke('chat_clear_conversation', { workspaceId, conversationId });

export const deleteConversation = (workspaceId: string, conversationId: string) =>
  invoke('chat_delete_conversation', { workspaceId, conversationId });

export const setConversationMembers = (workspaceId: string, conversationId: string, memberIds: string[]) =>
  invoke('chat_set_conversation_members', { workspaceId, conversationId, memberIds });

export const repairChatMessages = (workspaceId: string) =>
  invoke<ChatRepairResult>('chat_repair_messages', { workspaceId });

export const clearAllChatMessages = (workspaceId: string) =>
  invoke<ChatClearResult>('chat_clear_all_messages', { workspaceId });

type ChatMessageListener = (payload: ChatMessageCreatedPayload) => void;
const chatMessageListeners = new Set<ChatMessageListener>();
let chatMessageListenerInitialized = false;

const ensureChatMessageListener = async () => {
  if (chatMessageListenerInitialized) {
    return;
  }
  chatMessageListenerInitialized = true;
  await listen<ChatMessageCreatedPayload>('chat-message-created', (event) => {
    for (const handler of chatMessageListeners) {
      handler(event.payload);
    }
  });
};

export const onChatMessageCreated = (handler: ChatMessageListener) => {
  void ensureChatMessageListener();
  chatMessageListeners.add(handler);
  return () => chatMessageListeners.delete(handler);
};
