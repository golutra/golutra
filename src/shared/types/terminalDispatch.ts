import type { ConversationType } from './conversation';

// 终端派发请求：封装聊天上下文与目标成员。
export type TerminalDispatchRequest = {
  memberId: string;
  conversationId: string;
  conversationType: ConversationType;
  senderId: string;
  senderName: string;
  text: string;
};
