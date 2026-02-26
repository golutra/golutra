import { readAppData, readCacheData, writeAppData, writeCacheData } from '@/shared/tauri/storage';

export type ChatSessionCache = {
  activeConversationId?: string;
};

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

export const chatDataPath = (workspaceId: string) => `${workspaceId}/chat.json`;
export const chatCachePath = (workspaceId: string) => `${workspaceId}/session.json`;
const conversationMessagesDir = (workspaceId: string) => `${workspaceId}/chat/conversations`;
const conversationMessagesPath = (workspaceId: string, conversationId: string) =>
  `${conversationMessagesDir(workspaceId)}/${encodeConversationId(conversationId)}.json`;
const legacyChatDataPath = (workspaceId: string) => `workspaces/${workspaceId}/chat.json`;
const legacyChatCachePath = (workspaceId: string) => `workspaces/${workspaceId}/session.json`;

type LegacyConversation = { id?: string; messages?: unknown };
type LegacySession = { conversations?: LegacyConversation[]; nextMessageId?: number };

const hasLegacyMessages = (session: LegacySession | null) =>
  Array.isArray(session?.conversations) &&
  session.conversations.some(
    (conversation) =>
      conversation &&
      typeof conversation === 'object' &&
      'messages' in conversation &&
      Array.isArray((conversation as LegacyConversation).messages)
  );

const migrateLegacySession = async (workspaceId: string, legacy: LegacySession) => {
  const conversations = Array.isArray(legacy.conversations) ? legacy.conversations : [];
  const nextMessageId = typeof legacy.nextMessageId === 'number' ? legacy.nextMessageId : 1;
  const normalized = conversations
    .map((conversation) => {
      const id = typeof conversation.id === 'string' ? conversation.id : '';
      if (!id) return null;
      const { messages, ...meta } = conversation;
      return { id, meta, messages };
    })
    .filter((item): item is { id: string; meta: Record<string, unknown>; messages?: unknown } => Boolean(item));

  for (const item of normalized) {
    const payload = Array.isArray(item.messages) ? item.messages : [];
    try {
      await writeAppData(conversationMessagesPath(workspaceId, item.id), payload);
    } catch (error) {
      console.error('Failed to migrate conversation messages.', error);
    }
  }

  const indexPayload = {
    conversations: normalized.map((item) => item.meta),
    nextMessageId
  };
  try {
    await writeAppData(chatDataPath(workspaceId), indexPayload);
  } catch (error) {
    console.error('Failed to write chat index data.', error);
  }

  return indexPayload;
};

export const loadChatSession = async <T>(workspaceId: string): Promise<T | null> => {
  try {
    const current = await readAppData<T>(chatDataPath(workspaceId));
    if (current !== null) {
      if (hasLegacyMessages(current as LegacySession)) {
        return (await migrateLegacySession(workspaceId, current as LegacySession)) as T;
      }
      return current;
    }
  } catch (error) {
    console.error('Failed to read chat session data.', error);
  }
  try {
    const legacy = await readAppData<T>(legacyChatDataPath(workspaceId));
    if (legacy !== null) {
      if (hasLegacyMessages(legacy as LegacySession)) {
        return (await migrateLegacySession(workspaceId, legacy as LegacySession)) as T;
      }
      try {
        await writeAppData(chatDataPath(workspaceId), legacy);
      } catch (error) {
        console.error('Failed to migrate chat session data.', error);
      }
      return legacy;
    }
  } catch (error) {
    console.error('Failed to read legacy chat session data.', error);
  }
  return null;
};

export const saveChatSession = async (workspaceId: string, payload: unknown) =>
  writeAppData(chatDataPath(workspaceId), payload);

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

export const saveConversationMessages = async (workspaceId: string, conversationId: string, payload: unknown) =>
  writeAppData(conversationMessagesPath(workspaceId, conversationId), payload);

export const loadChatCache = async (workspaceId: string): Promise<ChatSessionCache | null> => {
  try {
    const current = await readCacheData<ChatSessionCache>(chatCachePath(workspaceId));
    if (current !== null) {
      return current;
    }
  } catch (error) {
    console.error('Failed to read chat cache data.', error);
  }
  try {
    const legacy = await readCacheData<ChatSessionCache>(legacyChatCachePath(workspaceId));
    if (legacy !== null) {
      try {
        await writeCacheData(chatCachePath(workspaceId), legacy);
      } catch (error) {
        console.error('Failed to migrate chat cache data.', error);
      }
      return legacy;
    }
  } catch (error) {
    console.error('Failed to read legacy chat cache data.', error);
  }
  return null;
};

export const saveChatCache = async (workspaceId: string, payload: ChatSessionCache) =>
  writeCacheData(chatCachePath(workspaceId), payload);
