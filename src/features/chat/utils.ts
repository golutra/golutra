import type { Member, Message } from './types';

export const splitMentions = (text: string) => text.split(/(@[\w\s]+)/g).filter(Boolean);

export const ensureUniqueName = (name: string, members: Member[]) => {
  // Enforce case-insensitive uniqueness to match the UI's duplicate prevention.
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

export const formatMessageTime = (timestamp: number, locale?: string) => {
  const formatter = new Intl.DateTimeFormat(locale, {
    hour: '2-digit',
    minute: '2-digit'
  });
  return formatter.format(new Date(timestamp));
};

export const getMessageDayKey = (timestamp: number) => {
  const date = new Date(timestamp);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
};

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
