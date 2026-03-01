// 终端调用链清单：用于测试结果展示与状态标记。
export type TerminalCallChain = {
  id: 'ui-open' | 'chat-backend' | 'create-terminal-friend';
  title: string;
  steps: Array<{ id: string; label: string }>;
};

export const TERMINAL_CALL_CHAINS: TerminalCallChain[] = [
  {
    id: 'ui-open',
    title: 'UI open terminal',
    steps: [
      { id: 'ui-open.avatar-click', label: 'MemberRow avatar click' },
      { id: 'ui-open.chatinterface-handle-member-action', label: 'ChatInterface.handleMemberAction' },
      { id: 'ui-open.open-member-terminal', label: 'terminalMemberStore.openMemberTerminal' },
      { id: 'ui-open.ensure-member-session', label: 'ensureMemberSession/startMemberSession' },
      { id: 'ui-open.create-session', label: 'terminalBridge.createSession -> terminal_create' },
      { id: 'ui-open.open-terminal-window', label: 'openTerminalWindow -> terminal_open_window' },
      { id: 'ui-open.terminal-open-tab', label: 'TerminalWorkspace terminal-open-tab' },
      { id: 'ui-open.terminal-attach', label: 'TerminalPane.attachSession -> terminal_attach' }
    ]
  },
  {
    id: 'chat-backend',
    title: 'Chat background dispatch',
    steps: [
      { id: 'chat-backend.send-message', label: 'chatStore.sendMessage' },
      { id: 'chat-backend.parse-targets', label: 'resolve terminal targets' },
      { id: 'chat-backend.enqueue-dispatch', label: 'enqueueTerminalDispatch' },
      { id: 'chat-backend.ensure-member-session', label: 'terminalMemberStore.ensureMemberSession(openTab: false)' },
      { id: 'chat-backend.dispatch-session', label: 'terminalBridge.dispatchSession -> terminal_dispatch' },
      { id: 'chat-backend.backend-write-append', label: 'PTY write + chat_append_terminal_message' },
      { id: 'chat-backend.chat-message-created', label: 'chat-message-created' },
      { id: 'chat-backend.on-chat-message-created', label: 'onChatMessageCreated' },
      { id: 'chat-backend.append-terminal-message', label: 'chatStore.appendTerminalMessage' }
    ]
  },
  {
    id: 'create-terminal-friend',
    title: 'Create terminal friend',
    steps: [
      { id: 'friend.create.invite-click', label: 'FriendsView invite click' },
      { id: 'friend.create.invite-emit', label: 'InviteAssistantModal emit invite' },
      { id: 'friend.create.handle-invite', label: 'FriendsView.handleInvite(role=member)' },
      { id: 'friend.create.build-member', label: 'build terminal member payload' },
      { id: 'friend.create.add-member', label: 'projectStore.addMember' },
      { id: 'friend.create.set-conversation-members', label: 'chatStore.setConversationMembers' },
      { id: 'friend.create.ensure-member-session', label: 'terminalMemberStore.ensureMemberSession(openTab: false)' },
      { id: 'friend.create.create-session', label: 'terminalBridge.createSession -> terminal_create' }
    ]
  }
];
