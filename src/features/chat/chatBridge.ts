// 聊天桥接层：封装 Tauri 命令与事件监听，统一数据契约。
import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';

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
  spanId?: string;
};

export type ChatMessageStatusPayload = {
  workspaceId: string;
  conversationId: string;
  messageId: string;
  status: MessageStatus;
};

export type ChatDispatchMentions = {
  mentionIds: string[];
  mentionAll: boolean;
};

export type ChatDispatchRequest = {
  workspaceId: string;
  workspacePath: string;
  conversationId: string;
  conversationType: 'channel' | 'dm';
  text: string;
  senderId: string;
  senderName: string;
  mentions: ChatDispatchMentions;
  messageId: string;
  clientTraceId: string;
  timestamp: number;
};

export type ChatUnreadSyncPayload = {
  workspaceId: string;
  totalUnreadCount: number;
  conversationId?: string | null;
  conversationUnreadCount?: number | null;
  resetAll?: boolean;
};

/**
 * 生成 ULID，用于本地创建会话/成员等标识。
 * 输入：无。
 * 输出：ULID 字符串。
 * 错误语义：非 Tauri 环境会抛出异常。
 */
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

/**
 * 获取工作区会话列表与首页信息。
 * 输入：workspaceId、userId、workspaceName 与成员 id 列表。
 * 输出：ChatHomeFeed。
 */
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

/**
 * 拉取会话消息列表。
 * 输入：workspaceId、conversationId、可选 limit 与 beforeId。
 * 输出：消息数组。
 */
export const getConversationMessages = (workspaceId: string, conversationId: string, limit?: number, beforeId?: string) =>
  invoke<MessageDto[]>('chat_get_messages', {
    workspaceId,
    conversationId,
    limit,
    beforeId
  });

/**
 * 标记会话已读到最新。
 * 输入：workspaceId、userId、conversationId。
 * 输出：无。
 */
export const markConversationRead = (workspaceId: string, userId: string, conversationId: string) =>
  invoke('chat_mark_conversation_read_latest', {
    workspaceId,
    userId,
    conversationId
  });

/**
 * 发送会话消息。
 * 输入：工作区/会话/发送者/查看者与内容。
 * 输出：创建后的消息 DTO。
 */
export const sendConversationMessage = (
  workspaceId: string,
  conversationId: string,
  senderId: string | null,
  viewerId: string | null,
  content: MessageContent,
  isAi?: boolean,
  attachment?: MessageAttachment
) =>
  invoke<MessageDto>('chat_send_message', {
    workspaceId,
    conversationId,
    senderId: senderId ?? undefined,
    viewerId: viewerId ?? undefined,
    content,
    isAi,
    attachment
  });

/**
 * 发送会话消息并由后端执行终端编排。
 * 输入：消息文本与 mention 元数据。
 * 输出：创建后的消息 DTO。
 */
export const sendConversationMessageAndDispatch = (payload: ChatDispatchRequest) =>
  invoke<MessageDto>('chat_send_message_and_dispatch', { payload });

/**
 * 创建群聊会话。
 * 输入：工作区、发起用户、成员 id 列表与可选名称。
 * 输出：会话 DTO。
 */
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

/**
 * 确保私聊会话存在。
 * 输入：工作区、当前用户与目标用户 id。
 * 输出：会话 DTO。
 */
export const ensureDirectConversation = (workspaceId: string, userId: string, targetId: string) =>
  invoke<ConversationDto>('chat_ensure_direct', { workspaceId, userId, targetId });

/**
 * 设置会话固定/静音状态。
 * 输入：工作区、用户、会话 id 与可选开关。
 * 输出：无。
 */
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

/**
 * 重命名会话。
 * 输入：工作区、会话 id 与自定义名称。
 * 输出：无。
 */
export const renameConversation = (workspaceId: string, conversationId: string, customName?: string | null) =>
  invoke('chat_rename_conversation', { workspaceId, conversationId, customName });

/**
 * 清空会话消息。
 * 输入：工作区与会话 id。
 * 输出：无。
 */
export const clearConversation = (workspaceId: string, conversationId: string) =>
  invoke('chat_clear_conversation', { workspaceId, conversationId });

/**
 * 删除会话。
 * 输入：工作区与会话 id。
 * 输出：无。
 */
export const deleteConversation = (workspaceId: string, conversationId: string) =>
  invoke('chat_delete_conversation', { workspaceId, conversationId });

/**
 * 更新会话成员列表。
 * 输入：工作区、会话 id 与成员 id 列表。
 * 输出：无。
 */
export const setConversationMembers = (workspaceId: string, conversationId: string, memberIds: string[]) =>
  invoke('chat_set_conversation_members', { workspaceId, conversationId, memberIds });

/**
 * 修复聊天数据并返回删除数量。
 * 输入：工作区 id。
 * 输出：修复结果。
 */
export const repairChatMessages = (workspaceId: string) =>
  invoke<ChatRepairResult>('chat_repair_messages', { workspaceId });

/**
 * 清空工作区全部消息。
 * 输入：工作区 id。
 * 输出：清理结果。
 */
export const clearAllChatMessages = (workspaceId: string) =>
  invoke<ChatClearResult>('chat_clear_all_messages', { workspaceId });

type ChatMessageListener = (payload: ChatMessageCreatedPayload) => void;
const chatMessageListeners = new Set<ChatMessageListener>();
let chatMessageListenerInitialized = false;

// 全局只初始化一次事件监听，避免重复订阅。
const ensureChatMessageListener = async () => {
  if (chatMessageListenerInitialized) {
    return;
  }
  chatMessageListenerInitialized = true;
  await listen<ChatMessageCreatedPayload>('chat-message-created', (event) => {
    void logDiagnosticsEvent('on-chat-message-created', {
      conversationId: event.payload.conversationId,
      messageId: event.payload.message.id,
      senderId: event.payload.message.senderId,
      content: event.payload.message.content
    });
    for (const handler of chatMessageListeners) {
      handler(event.payload);
    }
  });
};

/**
 * 订阅聊天消息创建事件。
 * 输入：事件处理函数。
 * 输出：取消订阅函数。
 */
export const onChatMessageCreated = (handler: ChatMessageListener) => {
  void ensureChatMessageListener();
  chatMessageListeners.add(handler);
  return () => chatMessageListeners.delete(handler);
};

type ChatMessageStatusListener = (payload: ChatMessageStatusPayload) => void;
const chatMessageStatusListeners = new Set<ChatMessageStatusListener>();
let chatMessageStatusListenerInitialized = false;

const ensureChatMessageStatusListener = async () => {
  if (chatMessageStatusListenerInitialized) {
    return;
  }
  chatMessageStatusListenerInitialized = true;
  await listen<ChatMessageStatusPayload>('chat-message-status', (event) => {
    void logDiagnosticsEvent('on-chat-message-status', {
      conversationId: event.payload.conversationId,
      messageId: event.payload.messageId,
      status: event.payload.status
    });
    for (const handler of chatMessageStatusListeners) {
      handler(event.payload);
    }
  });
};

export const onChatMessageStatus = (handler: ChatMessageStatusListener) => {
  void ensureChatMessageStatusListener();
  chatMessageStatusListeners.add(handler);
  return () => chatMessageStatusListeners.delete(handler);
};

type ChatUnreadListener = (payload: ChatUnreadSyncPayload) => void;
const chatUnreadListeners = new Set<ChatUnreadListener>();
let chatUnreadListenerInitialized = false;

const ensureChatUnreadListener = async () => {
  if (chatUnreadListenerInitialized) {
    return;
  }
  chatUnreadListenerInitialized = true;
  await listen<ChatUnreadSyncPayload>('chat-unread-sync', (event) => {
    void logDiagnosticsEvent('on-chat-unread-sync', {
      workspaceId: event.payload.workspaceId,
      totalUnreadCount: event.payload.totalUnreadCount,
      conversationId: event.payload.conversationId,
      conversationUnreadCount: event.payload.conversationUnreadCount,
      resetAll: event.payload.resetAll
    });
    for (const handler of chatUnreadListeners) {
      handler(event.payload);
    }
  });
};

export const onChatUnreadSync = (handler: ChatUnreadListener) => {
  void ensureChatUnreadListener();
  chatUnreadListeners.add(handler);
  return () => chatUnreadListeners.delete(handler);
};

