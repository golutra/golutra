// Tauri 通知封装：集中处理桌面 badge 与悬浮窗调用。
import { invoke } from '@tauri-apps/api/core';

export type NotificationPreviewItem = {
  workspaceId?: string | null;
  workspaceName?: string | null;
  workspacePath?: string | null;
  conversationId?: string | null;
  conversationName?: string | null;
  conversationUnread: number;
  conversationType?: 'dm' | 'channel' | null;
  memberCount?: number | null;
  senderId?: string | null;
  senderName?: string | null;
  senderAvatar?: string | null;
  senderCanOpenTerminal?: boolean;
  preview: string;
  lastMessageAt?: number | null;
};

export type NotificationPreviewState = {
  title: string;
  totalUnread: number;
  items: NotificationPreviewItem[];
};

export type NotificationUpdatePayload = NotificationPreviewState & {
  windowLabel: string;
  unreadCount: number;
  avatarPng?: number[] | null;
};

/**
 * 更新托盘状态与悬浮窗预览数据。
 * 输入：窗口 label、未读数、预览信息与可选头像 PNG。
 * 输出：无。
 */
export const updateNotificationState = async (payload: NotificationUpdatePayload): Promise<void> =>
  invoke('notification_update_state', { payload });

/**
 * 获取悬浮窗预览数据。
 */
export const getNotificationState = async (): Promise<NotificationPreviewState> =>
  invoke<NotificationPreviewState>('notification_get_state');

/**
 * 同步悬浮窗 hover 状态。
 */
export const setNotificationPreviewHovered = async (hovered: boolean): Promise<void> =>
  invoke('notification_preview_hover', { hovered });

/**
 * 立即隐藏悬浮窗（用于点击动作）。
 */
export const hideNotificationPreview = async (): Promise<void> => {
  await invoke('notification_preview_hide');
};

/**
 * 请求忽略全部未读。
 */
export const requestIgnoreAll = async (userId: string): Promise<void> => {
  await hideNotificationPreview();
  await invoke('notification_request_ignore_all', { userId });
};

/**
 * 请求查看全部未读。
 */
export const requestViewAll = async (
  userId: string,
  items?: NotificationPreviewItem[]
): Promise<void> => {
  await hideNotificationPreview();
  const payload: Record<string, unknown> = { userId };
  if (items && items.length > 0) {
    payload.items = items;
  }
  await invoke('notification_open_all_unread', payload);
};

/**
 * 请求打开指定会话。
 */
export const requestOpenConversation = async (payload: {
  workspaceId: string;
  conversationId: string;
}): Promise<void> => {
  await hideNotificationPreview();
  await invoke('notification_open_unread_conversation', payload);
};

/**
 * 请求打开发送者终端。
 */
export const requestOpenTerminal = async (payload: {
  workspaceId: string;
  conversationId: string;
  senderId: string;
}): Promise<void> => {
  await hideNotificationPreview();
  await invoke('notification_open_terminal', payload);
};

/**
 * 设置当前主发布窗口。
 */
export const setNotificationActiveWindow = async (windowLabel: string): Promise<boolean> =>
  invoke<boolean>('notification_set_active_window', { windowLabel });
