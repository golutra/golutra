import { describe, expect, it } from 'vitest';
import { ensureUniqueName, groupMessagesByDay, splitMentions } from '../features/chat/utils';
import type { Member, Message } from '../features/chat/types';

describe('chat utils', () => {
  it('splits mentions while preserving plain text', () => {
    const parts = splitMentions('Hello @Jane Doe and @Sam');
    expect(parts).toEqual(['Hello ', '@Jane Doe and ', '@Sam']);
  });

  it('ensures unique member names case-insensitively', () => {
    const members: Member[] = [
      { id: '1', name: 'Alex', role: '', roleType: 'member', avatar: '', status: 'offline' },
      { id: '2', name: 'Alex-1', role: '', roleType: 'member', avatar: '', status: 'offline' }
    ];

    expect(ensureUniqueName('Alex', members)).toBe('Alex-2');
    expect(ensureUniqueName('Jamie', members)).toBe('Jamie');
  });

  it('groups messages by day with separators', () => {
    const dayOne = new Date('2024-02-01T09:00:00').getTime();
    const dayTwo = new Date('2024-02-02T10:00:00').getTime();

    const messages: Message[] = [
      {
        id: 1,
        senderId: 'me',
        user: 'You',
        avatar: '',
        text: 'First message',
        time: '09:00',
        createdAt: dayOne,
        isAi: false,
        status: 'sent'
      },
      {
        id: 2,
        senderId: 'me',
        user: 'You',
        avatar: '',
        text: 'Second message',
        time: '09:05',
        createdAt: dayOne + 1000 * 60 * 5,
        isAi: false,
        status: 'sent'
      },
      {
        id: 3,
        senderId: 'assistant',
        user: 'AI',
        avatar: '',
        text: 'Next day message',
        time: '10:00',
        createdAt: dayTwo,
        isAi: true,
        status: 'sent'
      }
    ];

    const grouped = groupMessagesByDay(messages, 'en-US');
    const separatorCount = grouped.filter((item) => item.type === 'separator').length;
    const messageCount = grouped.filter((item) => item.type === 'message').length;

    expect(separatorCount).toBe(2);
    expect(messageCount).toBe(3);
  });
});
