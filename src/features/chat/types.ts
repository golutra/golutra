// 聊天领域模型：定义成员、会话与消息的共享类型契约。
import type { TerminalConnectionStatus, TerminalType } from '@/shared/types/terminal';
import type { ConversationType } from '@/shared/types/conversation';
export type { ConversationType } from '@/shared/types/conversation';

export type MemberRole = 'owner' | 'admin' | 'assistant' | 'member';
export type MemberStatus = 'online' | 'working' | 'dnd' | 'offline';

export type MessageMentionsPayload = {
  mentionIds: string[];
  mentionAll: boolean;
};

export type Member = {
  id: string;
  name: string;
  role: string;
  roleKey?: string;
  roleType: MemberRole;
  avatar: string;
  status: MemberStatus;
  terminalStatus?: TerminalConnectionStatus;
  terminalType?: TerminalType;
  terminalCommand?: string;
  terminalPath?: string;
  autoStartTerminal?: boolean;
  manualStatus?: MemberStatus;
  unlimitedAccess?: boolean;
  sandboxed?: boolean;
};

export type Contact = {
  id: string;
  name: string;
  avatar: string;
  roleType: MemberRole;
  status: MemberStatus;
  createdAt: number;
};

export type FriendScope = 'project' | 'global';

export type FriendEntry = {
  id: string;
  name: string;
  avatar: string;
  roleType: MemberRole;
  status: MemberStatus;
  manualStatus?: MemberStatus;
  terminalStatus?: TerminalConnectionStatus;
  scope: FriendScope;
  terminalType?: TerminalType;
  terminalCommand?: string;
  terminalPath?: string;
};

export type MemberAction = 'send-message' | 'mention' | 'rename' | 'remove' | 'set-status' | 'open-terminal';
export type MemberActionPayload = {
  action: MemberAction;
  member: Member;
  status?: MemberStatus;
};

export type Conversation = {
  id: string;
  type: ConversationType;
  memberIds: string[];
  targetId?: string;
  nameKey?: string;
  customName?: string;
  descriptionKey?: string;
  pinned: boolean;
  muted: boolean;
  lastMessageAt?: number;
  lastMessagePreview?: string;
  lastMessageSenderId?: string;
  lastMessageSenderName?: string;
  lastMessageSenderAvatar?: string;
  lastMessageAttachment?: MessageAttachment;
  isDefault?: boolean;
  unreadCount?: number;
  messages: Message[];
};

export type ConversationAction = 'pin' | 'unpin' | 'rename' | 'mute' | 'unmute' | 'clear' | 'delete';

export type MessageAttachment =
  | {
      type: 'image';
      filePath: string;
      fileName: string;
      fileSize: number;
      mimeType: string;
      width?: number;
      height?: number;
      thumbnailPath?: string;
    }
  | {
      type: 'roadmap';
      title: string;
    };

export type MessageStatus = 'sending' | 'sent' | 'failed';

export type MessageContent =
  | {
      type: 'text';
      text: string;
    }
  | {
      type: 'system';
      key: string;
      args?: Record<string, string>;
    };

export type Message = {
  id: string;
  senderId?: string;
  user: string;
  userKey?: string;
  userArgs?: Record<string, string | number>;
  avatar: string;
  content: MessageContent;
  createdAt: number;
  isAi: boolean;
  attachment?: MessageAttachment;
  status?: MessageStatus;
};

export type RoadmapTaskStatus = 'done' | 'in-progress' | 'pending';

export type RoadmapTask = {
  id: number;
  number: string;
  title: string;
  status: RoadmapTaskStatus;
};
