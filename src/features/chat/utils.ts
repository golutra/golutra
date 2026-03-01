// 聊天工具函数：处理提及、名称去重与消息时间分组。
import type { Member, Message } from './types';

/**
 * 拆分文本中的提及片段。
 * 输入：原始文本。
 * 输出：包含提及与普通文本的片段数组。
 */
const MENTION_TOKEN_PATTERN = /@[^\s]+/u;
const MENTION_SPLIT_PATTERN = /(@[^\s]+)/gu;

export const splitMentions = (text: string) => text.split(MENTION_SPLIT_PATTERN).filter(Boolean);
export const isMentionToken = (value: string) => MENTION_TOKEN_PATTERN.test(value);

const truncateByChars = (value: string, limit: number) => {
  const chars = Array.from(value);
  if (chars.length <= limit) {
    return value;
  }
  return chars.slice(0, limit).join('');
};

/**
 * 构建群聊标题。
 * 输入：成员 id 列表、成员数据、当前用户 id 与最大字符数（默认 18）。
 * 输出：拼接后的标题字符串。
 */
export const buildGroupConversationTitle = (
  memberIds: string[] | undefined,
  members: Member[],
  currentUserId: string,
  limit = 18
) => {
  if (!memberIds || memberIds.length === 0) {
    return '';
  }
  const memberById = new Map(members.map((member) => [member.id, member]));
  const names = memberIds
    .filter((id) => id && id !== currentUserId)
    .map((id) => memberById.get(id)?.name?.trim())
    .filter((name): name is string => Boolean(name));
  if (names.length === 0) {
    return '';
  }
  const joined = names.join(',');
  return truncateByChars(joined, limit);
};

/**
 * 生成不冲突的成员名称。
 * 输入：期望名称与已有成员列表。
 * 输出：保证大小写不冲突的名称。
 */
export const ensureUniqueName = (name: string, members: Member[]) => {
  // 以不区分大小写的规则与 UI 校验保持一致。
  const lowerNames = new Set(members.map((member) => member.name.toLowerCase()));

  if (!lowerNames.has(name.toLowerCase())) {
    return name;
  }

  let counter = 1;
  let candidate = `${name}-${counter}`;
  while (lowerNames.has(candidate.toLowerCase())) {
    counter += 1;
    candidate = `${name}-${counter}`;
  }

  return candidate;
};

/**
 * 格式化消息时间。
 * 输入：时间戳与可选 locale。
 * 输出：时:分 字符串。
 */
export const formatMessageTime = (timestamp: number, locale?: string) => {
  const formatter = new Intl.DateTimeFormat(locale, {
    hour: '2-digit',
    minute: '2-digit'
  });
  return formatter.format(new Date(timestamp));
};

/**
 * 获取消息所属日期的 key。
 * 输入：时间戳。
 * 输出：YYYY-MM-DD 字符串。
 */
export const getMessageDayKey = (timestamp: number) => {
  const date = new Date(timestamp);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
};

/**
 * 格式化日期分隔符文本。
 * 输入：时间戳与可选 locale。
 * 输出：日期标签字符串。
 */
export const formatDayLabel = (timestamp: number, locale?: string) => {
  const formatter = new Intl.DateTimeFormat(locale, {
    month: 'long',
    day: 'numeric',
    year: 'numeric'
  });
  return formatter.format(new Date(timestamp));
};

export type MessageDisplayItem =
  | { type: 'separator'; id: string; label: string }
  | { type: 'message'; id: string; message: Message };

/**
 * 按日期对消息分组并插入分隔条目。
 * 输入：消息列表与可选 locale。
 * 输出：包含分隔符与消息的展示数组。
 */
export const groupMessagesByDay = (messages: Message[], locale?: string): MessageDisplayItem[] => {
  const items: MessageDisplayItem[] = [];
  let lastDayKey = '';

  [...messages]
    .sort((a, b) => a.createdAt - b.createdAt)
    .forEach((message) => {
      const dayKey = getMessageDayKey(message.createdAt);
      if (dayKey !== lastDayKey) {
        items.push({
          type: 'separator',
          id: `separator-${dayKey}`,
          label: formatDayLabel(message.createdAt, locale)
        });
        lastDayKey = dayKey;
      }

      items.push({
        type: 'message',
        id: `message-${message.id}`,
        message
      });
    });

  return items;
};
