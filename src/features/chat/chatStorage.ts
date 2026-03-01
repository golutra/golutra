// 聊天本地存储：封装会话索引与消息分文件的持久化。
import { readAppData, readCacheData, writeAppData, writeCacheData } from '@/shared/tauri/storage';

export type ChatSessionCache = {
  activeConversationId?: string;
};

// 将会话 id 编码为可用文件名，避免路径分隔符带来的风险。
const encodeConversationId = (value: string) => {
  const bytes = new TextEncoder().encode(value);
  let binary = '';
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  if (typeof btoa === 'function') {
    const base64 = btoa(binary).replace(/=+$/g, '');
    return base64.replace(/\+/g, '-').replace(/\//g, '_');
  }
  return Array.from(bytes)
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
};

/**
 * 会话索引数据路径。
 * 输入：workspaceId。
 * 输出：相对路径字符串。
 */
export const chatDataPath = (workspaceId: string) => `${workspaceId}/chat.json`;
/**
 * 会话缓存路径。
 * 输入：workspaceId。
 * 输出：相对路径字符串。
 */
export const chatCachePath = (workspaceId: string) => `${workspaceId}/session.json`;
const conversationMessagesDir = (workspaceId: string) => `${workspaceId}/chat/conversations`;
const conversationMessagesPath = (workspaceId: string, conversationId: string) =>
  `${conversationMessagesDir(workspaceId)}/${encodeConversationId(conversationId)}.json`;
/**
 * 读取聊天会话索引数据。
 * 输入：workspaceId。
 * 输出：索引数据或 null。
 */
export const loadChatSession = async <T>(workspaceId: string): Promise<T | null> => {
  try {
    const current = await readAppData<T>(chatDataPath(workspaceId));
    return current ?? null;
  } catch (error) {
    console.error('Failed to read chat session data.', error);
  }
  return null;
};

/**
 * 保存会话索引数据。
 * 输入：workspaceId 与 payload。
 * 输出：无。
 */
export const saveChatSession = async (workspaceId: string, payload: unknown) =>
  writeAppData(chatDataPath(workspaceId), payload);

/**
 * 读取会话消息列表。
 * 输入：workspaceId 与 conversationId。
 * 输出：消息数组，失败时返回空数组。
 */
export const loadConversationMessages = async <T>(workspaceId: string, conversationId: string): Promise<T[]> => {
  try {
    const current = await readAppData<T[]>(conversationMessagesPath(workspaceId, conversationId));
    if (Array.isArray(current)) {
      return current;
    }
  } catch (error) {
    console.error('Failed to read conversation messages.', error);
  }
  return [];
};

/**
 * 保存会话消息列表。
 * 输入：workspaceId、conversationId 与 payload。
 * 输出：无。
 */
export const saveConversationMessages = async (workspaceId: string, conversationId: string, payload: unknown) =>
  writeAppData(conversationMessagesPath(workspaceId, conversationId), payload);

/**
 * 读取会话缓存数据。
 * 输入：workspaceId。
 * 输出：缓存数据或 null。
 */
export const loadChatCache = async (workspaceId: string): Promise<ChatSessionCache | null> => {
  try {
    const current = await readCacheData<ChatSessionCache>(chatCachePath(workspaceId));
    return current ?? null;
  } catch (error) {
    console.error('Failed to read chat cache data.', error);
  }
  return null;
};

/**
 * 保存会话缓存数据。
 * 输入：workspaceId 与缓存 payload。
 * 输出：无。
 */
export const saveChatCache = async (workspaceId: string, payload: ChatSessionCache) =>
  writeCacheData(chatCachePath(workspaceId), payload);
