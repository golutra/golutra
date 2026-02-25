export type MemberRole = 'owner' | 'admin' | 'assistant' | 'member';
export type MemberStatus = 'online' | 'offline' | 'dnd';

export type Member = {
  id: string;
  name: string;
  role: string;
  roleKey?: string;
  roleType: MemberRole;
  avatar: string;
  status: MemberStatus;
};

export type MemberAction = 'send-message' | 'mention' | 'rename' | 'remove';

export type ConversationType = 'channel' | 'dm';

export type Conversation = {
  id: string;
  type: ConversationType;
  targetId: string;
  nameKey?: string;
  customName?: string;
  descriptionKey?: string;
  pinned: boolean;
  muted: boolean;
  messages: Message[];
};

export type ConversationAction = 'pin' | 'unpin' | 'rename' | 'mute' | 'unmute' | 'clear' | 'delete';

export type MessageAttachment =
  | {
      type: 'image';
      name: string;
      size: string;
      url: string;
    }
  | {
      type: 'roadmap';
      title: string;
    };

export type MessageStatus = 'sending' | 'sent' | 'failed';

export type Message = {
  id: number;
  senderId?: string;
  user: string;
  avatar: string;
  text: string;
  time: string;
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
