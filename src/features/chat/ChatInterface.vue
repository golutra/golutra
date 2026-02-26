<template>
  <div class="flex h-full w-full relative">
    <ChatSidebar
      :conversations="conversations"
      :members="displayMembers"
      :current-user-id="currentUserId"
      :active-conversation-id="activeConversationId"
      :workspace-name="workspaceName"
      :default-channel-id="defaultChannelId"
      @select-conversation="handleSelectConversation"
      @conversation-action="handleConversationAction"
    />

    <div class="flex-1 flex flex-col min-w-0 bg-transparent relative z-0">
      <div
        v-if="workspaceReadOnly"
        class="px-4 py-2 border-b border-red-500/30 bg-red-500/10 text-red-200 text-xs flex items-start gap-3"
      >
        <span class="material-symbols-outlined text-sm text-red-300 mt-0.5">lock</span>
        <div class="flex flex-col">
          <span class="font-semibold">{{ t('workspace.readOnlyTitle') }}</span>
          <span class="selectable text-red-200/80">{{ workspaceWarning || t('workspace.readOnlySubtitle') }}</span>
        </div>
      </div>
      <ChatHeader
        :title="headerTitle"
        :description="headerDescription"
        :member-count="conversationMembers.length"
        @open-roadmap="activeModal = 'roadmap'"
        @open-skills="activeModal = 'skills'"
        @open-members="showMembersDrawer = true"
      />
      <MessagesList
        :messages="activeMessages"
        :current-user-id="currentUserId"
        :current-user-name="currentUserName"
        :is-typing="isAssistantTyping"
        :typing-name="assistantName"
        :typing-avatar="assistantAvatar"
        :has-more="hasMoreMessages"
        :is-loading-more="isLoadingMore"
        @load-more="handleLoadOlderMessages"
        @open-roadmap="activeModal = 'roadmap'"
      />
      <div class="relative">
        <ChatInput
          ref="chatInputRef"
          v-model="inputValue"
          :max-length="maxMessageLength"
          :is-generating="isAssistantTyping"
          :quick-prompts="quickPrompts"
          :placeholder="inputPlaceholder"
          :members="mentionableMembers"
          @send="handleSendMessage"
          @stop="cancelAssistantReply"
        />
      </div>
    </div>

    <MembersSidebar
      :members="conversationMembers"
      :current-user-id="currentUserId"
      variant="sidebar"
      @open-invite="openFriendsInvite"
      @member-action="handleMemberAction"
    />

    <template v-if="showMembersDrawer">
      <div class="fixed inset-0 bg-black/40 backdrop-blur-[1px] z-40 xl:hidden" @click="showMembersDrawer = false"></div>
      <div class="fixed right-0 top-0 bottom-0 z-50 xl:hidden">
        <MembersSidebar
          :members="conversationMembers"
          :current-user-id="currentUserId"
          variant="drawer"
          @open-invite="openFriendsInvite"
          @member-action="handleMemberAction"
        />
      </div>
    </template>

    <RoadmapModal v-if="activeModal === 'roadmap'" @close="activeModal = null" />
    <SkillManagementModal
      v-if="activeModal === 'skills'"
      @close="activeModal = null"
      @configure="activeModal = 'skillDetail'"
    />
    <SkillDetailModal v-if="activeModal === 'skillDetail'" @close="activeModal = null" @back="activeModal = 'skills'" />

    <InviteFriendsModal
      v-if="showFriendsInviteModal"
      :friends="invitableFriends"
      :title="inviteModalTitle"
      :action-label="inviteActionLabel"
      @close="showFriendsInviteModal = false"
      @invite="handleInviteFriends"
    />

    <ManageMemberModal
      v-if="managingMember"
      :member="managingMember"
      :show-remove="false"
      @close="managingMember = null"
      @save="handleUpdateMember"
      @remove="handleRemoveMember"
    />

    <RenameConversationModal
      v-if="renamingConversation"
      :name="renamingConversationName"
      @close="renamingConversation = null"
      @save="handleRenameConversation"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import ChatSidebar from './components/ChatSidebar.vue';
import ChatHeader from './components/ChatHeader.vue';
import MessagesList from './components/MessagesList.vue';
import ChatInput from './components/ChatInput.vue';
import MembersSidebar from './components/MembersSidebar.vue';
import RoadmapModal from './modals/RoadmapModal.vue';
import SkillManagementModal from './modals/SkillManagementModal.vue';
import SkillDetailModal from './modals/SkillDetailModal.vue';
import InviteFriendsModal from './modals/InviteFriendsModal.vue';
import ManageMemberModal from './modals/ManageMemberModal.vue';
import RenameConversationModal from './modals/RenameConversationModal.vue';
import type { Conversation, ConversationAction, FriendEntry, Member, MemberActionPayload, MessageMentionsPayload } from './types';
import { useChatStore } from './chatStore';
import { useContactsStore } from './contactsStore';
import { loadChatCache, saveChatCache } from './chatStorage';
import { CURRENT_USER_ID } from './data';
import { buildGroupConversationTitle } from './utils';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useSettingsStore } from '@/features/global/settingsStore';
import { buildSeededAvatar, ensureAvatar } from '@/shared/utils/avatar';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import { useTerminalMemberStore } from '@/features/terminal/terminalMemberStore';
import { onChatMessageCreated } from './chatBridge';
import { useToastStore } from '@/stores/toastStore';
import { isTauri } from '@tauri-apps/api/core';

type ActiveModal = 'roadmap' | 'skills' | 'skillDetail' | null;

const activeModal = ref<ActiveModal>(null);
const showFriendsInviteModal = ref(false);
const showMembersDrawer = ref(false);
const managingMember = ref<Member | null>(null);
const renamingConversation = ref<Conversation | null>(null);
const chatInputRef = ref<{ focus: () => void; registerMention: (member: Member) => void } | null>(null);
const activeConversationId = ref<string>('');

const inputValue = ref('');

const { t } = useI18n();
const workspaceStore = useWorkspaceStore();
const {
  currentWorkspace,
  defaultChannelName: defaultChannelNameRef,
  workspaceReadOnly,
  workspaceWarning
} = storeToRefs(workspaceStore);
const projectStore = useProjectStore();
const { members } = storeToRefs(projectStore);
const { addMember, updateMember, removeMember } = projectStore;
const contactsStore = useContactsStore();
const { contacts } = storeToRefs(contactsStore);
const settingsStore = useSettingsStore();
const { settings } = storeToRefs(settingsStore);
const { setAccountStatus } = settingsStore;
const toastStore = useToastStore();
const { pushToast } = toastStore;
const terminalMemberStore = useTerminalMemberStore();
const { ensureMemberSession, openMemberTerminal, stopMemberSession } = terminalMemberStore;
const workspaceName = computed(() => currentWorkspace.value?.name ?? t('chat.sidebar.workspaceName'));
const workspaceId = computed(() => currentWorkspace.value?.id ?? null);
const defaultChannelName = computed(() => defaultChannelNameRef.value?.trim() || '');
const defaultChannelDisplay = computed(() => {
  if (!defaultChannelName.value) return '';
  return defaultChannelName.value.startsWith('#') ? defaultChannelName.value : `#${defaultChannelName.value}`;
});
const fallbackChannelTitle = computed(() => defaultChannelName.value || workspaceName.value);

const chatStore = useChatStore();
const {
  conversations,
  currentUser,
  assistantMember,
  assistantTypingConversationId,
  isReady,
  defaultChannelId
} = storeToRefs(chatStore);
  const {
    cancelAssistantReply,
    sendMessage,
    ensureDirectMessage,
    createGroupConversation,
    setConversationMembers,
    loadConversationMessages,
    loadOlderMessages,
    toggleConversationPin,
    toggleConversationMute,
    renameConversation,
    clearConversationMessages,
    deleteConversation,
    deleteMemberConversations,
    appendTerminalMessage,
    getConversationPaging,
    markConversationRead
  } = chatStore;
const maxMessageLength = chatStore.maxMessageLength;

const DEFAULT_OWNER_NAME = 'Owner';
const DEFAULT_MEMBER_NAME = 'Member';
const DEFAULT_ASSISTANT_NAME = 'Assistant';

const accountDisplayName = computed(() => settings.value.account.displayName.trim());
const resolvedAccountName = computed(() => accountDisplayName.value || DEFAULT_OWNER_NAME);
const accountAvatar = computed(() => ensureAvatar(settings.value.account.avatar));
const currentUserId = computed(() => currentUser.value?.id ?? CURRENT_USER_ID);
const currentUserName = computed(() => resolvedAccountName.value);
const assistantName = computed(() => assistantMember.value?.name ?? DEFAULT_ASSISTANT_NAME);
const assistantAvatar = computed(() =>
  ensureAvatar(assistantMember.value?.avatar ?? buildSeededAvatar(assistantName.value))
);
const quickPrompts = computed(() => [
  t('chat.input.quickPrompts.summarize'),
  t('chat.input.quickPrompts.draftReply'),
  t('chat.input.quickPrompts.extractTasks')
]);

const buildMentionLabel = (name: string) => `@${name}`;
const CACHE_SAVE_DEBOUNCE_MS = 1200;
const cacheSaveTimers = new Map<string, number>();
const cacheSavePending = new Map<string, { activeConversationId: string }>();
let cacheSaveChain = Promise.resolve();
let unlistenTerminalChat: (() => void) | null = null;
const debugLog = (...args: unknown[]) => {
  if (!import.meta.env.DEV) {
    return;
  }
  try {
    if (window.localStorage.getItem('terminal-debug') !== '1') {
      return;
    }
  } catch {
    return;
  }
  console.info('[chat]', ...args);
};

const uniqueIds = (ids: string[]) => Array.from(new Set(ids.filter((id) => id)));

const handleTerminalSessionError = (error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes('terminal buffer limit reached')) {
    pushToast(t('terminal.resourceLimit'), { tone: 'error' });
    return true;
  }
  return false;
};

const activeConversation = computed(() => conversations.value.find((conversation) => conversation.id === activeConversationId.value) ?? null);
const activeMessages = computed(() => activeConversation.value?.messages ?? []);
const latestActiveMessageId = computed(() => {
  const messages = activeMessages.value;
  return messages.length ? messages[messages.length - 1].id : null;
});
const activePaging = computed(() =>
  activeConversationId.value ? getConversationPaging(activeConversationId.value) : { hasMore: false, loading: false }
);
const hasMoreMessages = computed(() => Boolean(activePaging.value?.hasMore));
const isLoadingMore = computed(() => Boolean(activePaging.value?.loading));
const lastMarkedMessageId = ref<string | null>(null);

const displayMembers = computed(() => {
  const status = settings.value.account.status;
  const name = resolvedAccountName.value;
  return members.value.map((member) => {
    if (member.id !== CURRENT_USER_ID) {
      return member;
    }
    return {
      ...member,
      name: name || member.name,
      status,
      avatar: accountAvatar.value
    };
  });
});
const memberById = computed(() => new Map(displayMembers.value.map((member) => [member.id, member])));

const friendEntries = computed<FriendEntry[]>(() => {
  const entries: FriendEntry[] = [];
  const seen = new Set<string>();
  for (const member of members.value) {
    if (!member || member.id === currentUserId.value) continue;
    entries.push({
      id: member.id,
      name: member.name,
      avatar: member.avatar,
      roleType: member.roleType,
      status: member.status,
      terminalStatus: member.terminalStatus,
      scope: 'project',
      terminalType: member.terminalType,
      terminalCommand: member.terminalCommand,
      terminalPath: member.terminalPath
    });
    seen.add(member.id);
  }
  for (const contact of contacts.value) {
    if (!contact || seen.has(contact.id)) continue;
    entries.push({
      id: contact.id,
      name: contact.name,
      avatar: contact.avatar,
      roleType: contact.roleType,
      status: contact.status,
      scope: 'global'
    });
    seen.add(contact.id);
  }
  return entries;
});

const invitableFriends = computed(() => {
  const active = activeConversation.value;
  if (!active) return friendEntries.value;
  const memberIds = new Set(active.memberIds ?? []);
  return friendEntries.value.filter((friend) => !memberIds.has(friend.id));
});

const inviteModalTitle = computed(() =>
  activeConversation.value?.type === 'dm' ? t('friends.inviteModal.titleDm') : t('friends.inviteModal.titleChannel')
);

const inviteActionLabel = computed(() =>
  activeConversation.value?.type === 'dm'
    ? t('friends.inviteModal.actionCreate')
    : t('friends.inviteModal.actionInvite')
);

const conversationMembers = computed(() => {
  if (!activeConversation.value) return displayMembers.value;
  const memberIds = Array.isArray(activeConversation.value.memberIds) ? activeConversation.value.memberIds : [];
  if (memberIds.length === 0) return displayMembers.value;
  const membersById = memberById.value;
  return memberIds
    .map((id) => membersById.get(id))
    .filter((member): member is Member => Boolean(member));
});

const mentionableMembers = computed(() =>
  conversationMembers.value.filter((member) => member.id !== currentUserId.value)
);

const activeDirectMember = computed(() => {
  if (activeConversation.value?.type !== 'dm') return null;
  const targetId = activeConversation.value.targetId ?? '';
  return memberById.value.get(targetId) ?? null;
});

const isMainChannel = (conversation: Conversation) =>
  conversation.type === 'channel' && conversation.id === defaultChannelId.value;

const getConversationTitle = (conversation: Conversation) => {
  if (conversation.type === 'dm') {
    const targetId = conversation.targetId ?? '';
    return memberById.value.get(targetId)?.name ?? DEFAULT_MEMBER_NAME;
  }
  if (isMainChannel(conversation) && defaultChannelName.value) {
    return defaultChannelName.value;
  }
  if (conversation.customName) {
    return conversation.customName;
  }
  if (conversation.nameKey) {
    return t(conversation.nameKey);
  }
  const groupTitle = buildGroupConversationTitle(
    conversation.memberIds,
    displayMembers.value,
    currentUserId.value,
    25
  );
  return groupTitle || conversation.id;
};

const headerTitle = computed(() => {
  if (!activeConversation.value) {
    return fallbackChannelTitle.value;
  }
  if (activeConversation.value.type === 'dm') {
    return activeDirectMember.value?.name ?? fallbackChannelTitle.value;
  }
  return getConversationTitle(activeConversation.value);
});

const headerDescription = computed(() => {
  if (!activeConversation.value) {
    return '';
  }
  if (activeConversation.value.type === 'dm') {
    return activeDirectMember.value ? t('chat.directMessageDescription', { name: activeDirectMember.value.name }) : '';
  }
  if (activeConversation.value.descriptionKey) {
    return t(activeConversation.value.descriptionKey);
  }
  return '';
});

const inputPlaceholder = computed(() => {
  if (activeConversation.value?.type === 'dm' && activeDirectMember.value) {
    const displayName = activeDirectMember.value.name.trim();
    const name = displayName ? `@${displayName}` : '@';
    return t('chat.input.directPlaceholder', { name });
  }
  if (activeConversation.value?.type === 'channel') {
    const label = getConversationTitle(activeConversation.value);
    const display = label.startsWith('#') ? label : `#${label}`;
    return t('chat.input.placeholder', { channel: display });
  }
  return t('chat.input.placeholder', { channel: defaultChannelDisplay.value });
});

const isAssistantTyping = computed(() => assistantTypingConversationId.value === activeConversationId.value);

const renamingConversationName = computed(() => {
  if (!renamingConversation.value) return '';
  if (renamingConversation.value.type === 'dm') {
    const targetId = renamingConversation.value.targetId ?? '';
    return memberById.value.get(targetId)?.name ?? '';
  }
  return getConversationTitle(renamingConversation.value);
});

const focusChatInput = async () => {
  await nextTick();
  chatInputRef.value?.focus();
};

const setMessageTarget = () => {
  inputValue.value = '';
  void focusChatInput();
};

const appendMention = (member: Member) => {
  const mention = buildMentionLabel(member.name);
  const trimmed = inputValue.value.replace(/[ \t]+$/, '');
  const separator = trimmed ? (trimmed.endsWith('\n') ? '' : ' ') : '';
  inputValue.value = `${trimmed}${separator}${mention} `;
  chatInputRef.value?.registerMention(member);
  void focusChatInput();
};

const openFriendsInvite = () => {
  if (!activeConversation.value) return;
  showFriendsInviteModal.value = true;
};

const handleSelectConversation = (conversationId: string) => {
  activeConversationId.value = conversationId;
};

const roleKeyForType = (roleType: FriendEntry['roleType']) => {
  if (roleType === 'owner') return 'members.roles.owner';
  if (roleType === 'admin') return 'members.roles.admin';
  if (roleType === 'assistant') return 'members.roles.aiAssistant';
  return 'members.roles.member';
};

const ensureProjectMembersFromFriends = async (ids: string[]) => {
  const existingIds = new Set(members.value.map((member) => member.id));
  for (const id of ids) {
    if (existingIds.has(id)) continue;
    const friend = friendEntries.value.find((entry) => entry.id === id);
    if (!friend) continue;
    const newMember: Member = {
      id: friend.id,
      name: friend.name,
      role: '',
      roleKey: roleKeyForType(friend.roleType),
      roleType: friend.roleType,
      avatar: friend.avatar,
      status: friend.status
    };
    await addMember(newMember);
    existingIds.add(id);
  }
};

const handleInviteFriends = async (ids: string[]) => {
  const conversation = activeConversation.value;
  if (!conversation) return;
  const selected = uniqueIds(ids);
  if (selected.length === 0) {
    showFriendsInviteModal.value = false;
    return;
  }

  await ensureProjectMembersFromFriends(selected);

  if (conversation.type === 'dm') {
    const baseMembers = uniqueIds([
      currentUserId.value,
      ...(conversation.memberIds ?? []),
      ...(conversation.targetId ? [conversation.targetId] : [])
    ]);
    const memberIds = uniqueIds([...baseMembers, ...selected]);
    const created = await createGroupConversation(memberIds);
    if (created) {
      activeConversationId.value = created.id;
    }
  } else {
    const memberIds = uniqueIds([...(conversation.memberIds ?? []), ...selected, currentUserId.value]);
    await setConversationMembers(conversation.id, memberIds);
  }

  showFriendsInviteModal.value = false;
};


const handleUpdateMember = async (id: string, newName: string) => {
  await updateMember(id, { name: newName });
  managingMember.value = null;
};

const handleRemoveMember = async (id: string) => {
  if (id === currentUserId.value) return;
  await deleteMemberConversations(id);
  await removeMember(id);
  managingMember.value = null;
};

const handleMemberAction = async ({ action, member, status }: MemberActionPayload) => {
  if (action === 'send-message') {
    const conversationId = await ensureDirectMessage(member.id);
    if (conversationId) {
      activeConversationId.value = conversationId;
    }
    setMessageTarget();
    return;
  }

  if (action === 'mention') {
    if (member.id === currentUserId.value) {
      return;
    }
    appendMention(member);
    return;
  }

  if (action === 'open-terminal') {
    try {
      debugLog('open terminal from member action', { memberId: member.id });
      await openMemberTerminal(member);
    } catch (error) {
      handleTerminalSessionError(error);
      console.error('Failed to open member terminal.', error);
    }
    return;
  }

  if (action === 'rename') {
    if (member.id === currentUserId.value) {
      return;
    }
    managingMember.value = member;
    return;
  }

  if (action === 'set-status') {
    if (status) {
      if (member.id === CURRENT_USER_ID) {
        setAccountStatus(status);
      }
      if (member.id !== CURRENT_USER_ID && hasTerminalConfig(member.terminalType, member.terminalCommand)) {
        if (status === 'offline') {
          try {
            await stopMemberSession(member.id, { preserve: false, fireAndForget: true });
          } catch (error) {
            console.error('Failed to stop terminal session.', error);
          }
          void updateMember(member.id, { status, autoStartTerminal: false, manualStatus: status });
          return;
        }
        if (status === 'online') {
          try {
            await ensureMemberSession(member, { openTab: false });
          } catch (error) {
            handleTerminalSessionError(error);
            console.error('Failed to start terminal session.', error);
            return;
          }
          void updateMember(member.id, { status, autoStartTerminal: true, manualStatus: status });
          return;
        }
        void updateMember(member.id, { status, manualStatus: status });
        return;
      }
      void updateMember(member.id, { status });
    }
    return;
  }

  if (action === 'remove') {
    if (member.id === currentUserId.value) {
      return;
    }
    if (managingMember.value?.id === member.id) {
      managingMember.value = null;
    }
    if (hasTerminalConfig(member.terminalType, member.terminalCommand)) {
      try {
        await stopMemberSession(member.id, { preserve: false, fireAndForget: true });
      } catch (error) {
        console.error('Failed to stop terminal session.', error);
      }
    }
    await deleteMemberConversations(member.id);
    await removeMember(member.id);
  }
};

const handleSendMessage = async (mentions: MessageMentionsPayload) => {
  if (!activeConversation.value) return;
  const messageText = inputValue.value.trim();
  if (!messageText) return;
  const result = await sendMessage({
    text: messageText,
    conversationId: activeConversation.value.id,
    mentions
  });
  if (result) {
    inputValue.value = '';
  }
};

const handleConversationAction = ({ conversationId, action }: { conversationId: string; action: ConversationAction }) => {
  if (action === 'rename') {
    const conversation = conversations.value.find((item) => item.id === conversationId);
    if (conversation?.type === 'channel' && !isMainChannel(conversation)) {
      renamingConversation.value = conversation;
    }
    return;
  }

  if (action === 'pin' || action === 'unpin') {
    toggleConversationPin(conversationId);
    return;
  }

  if (action === 'mute' || action === 'unmute') {
    toggleConversationMute(conversationId);
    return;
  }

  if (action === 'clear') {
    if (assistantTypingConversationId.value === conversationId) {
      cancelAssistantReply();
    }
    clearConversationMessages(conversationId);
    return;
  }

  if (action === 'delete') {
    if (conversationId === defaultChannelId.value) {
      return;
    }
    if (assistantTypingConversationId.value === conversationId) {
      cancelAssistantReply();
    }
    deleteConversation(conversationId);
  }
};

const handleRenameConversation = (name: string) => {
  if (!renamingConversation.value) return;
  renameConversation(renamingConversation.value.id, name);
  renamingConversation.value = null;
};

const handleLoadOlderMessages = async () => {
  if (!activeConversationId.value) return;
  await loadOlderMessages(activeConversationId.value);
};

const preferredConversationId = ref<string | null>(null);

const scheduleCacheSave = (workspaceId: string, conversationId: string) => {
  cacheSavePending.set(workspaceId, { activeConversationId: conversationId });
  const existing = cacheSaveTimers.get(workspaceId);
  if (existing !== undefined) {
    window.clearTimeout(existing);
  }
  const timer = window.setTimeout(() => {
    cacheSaveTimers.delete(workspaceId);
    const pending = cacheSavePending.get(workspaceId);
    if (!pending) return;
    cacheSavePending.delete(workspaceId);
    const commit = async () => {
      try {
        await saveChatCache(workspaceId, pending);
      } catch (error) {
        console.error('Failed to save chat cache.', error);
      }
    };
    cacheSaveChain = cacheSaveChain.then(commit, commit);
  }, CACHE_SAVE_DEBOUNCE_MS);
  cacheSaveTimers.set(workspaceId, timer);
};

const registerTerminalChatListener = () => {
  if (!isTauri() || unlistenTerminalChat) {
    return;
  }
  unlistenTerminalChat = onChatMessageCreated((payload) => {
    void appendTerminalMessage(payload);
  });
};

const flushCacheSaves = () => {
  for (const timer of cacheSaveTimers.values()) {
    window.clearTimeout(timer);
  }
  cacheSaveTimers.clear();
  if (cacheSavePending.size === 0) {
    return;
  }
  const pending = Array.from(cacheSavePending.entries());
  cacheSavePending.clear();
  for (const [workspaceId, payload] of pending) {
    const commit = async () => {
      try {
        await saveChatCache(workspaceId, payload);
      } catch (error) {
        console.error('Failed to save chat cache.', error);
      }
    };
    cacheSaveChain = cacheSaveChain.then(commit, commit);
  }
};

const applyActiveConversation = (preferredId?: string) => {
  if (!conversations.value.length) {
    activeConversationId.value = '';
    return;
  }
  if (activeConversationId.value && conversations.value.some((conversation) => conversation.id === activeConversationId.value)) {
    return;
  }
  const targetId = preferredId ?? preferredConversationId.value ?? '';
  if (targetId && conversations.value.some((conversation) => conversation.id === targetId)) {
    activeConversationId.value = targetId;
    return;
  }
  const mainId = defaultChannelId.value ?? '';
  const fallback =
    (mainId ? conversations.value.find((conversation) => conversation.id === mainId)?.id : undefined) ??
    conversations.value[0].id;
  activeConversationId.value = fallback;
};

watch(
  () => workspaceId.value,
  async (nextId) => {
    if (!nextId) {
      activeConversationId.value = '';
      return;
    }
    const cache = await loadChatCache(nextId);
    preferredConversationId.value = cache?.activeConversationId ?? null;
    applyActiveConversation(preferredConversationId.value ?? undefined);
  },
  { immediate: true }
);

watch(
  conversations,
  (nextConversations) => {
    if (renamingConversation.value && !nextConversations.some((conversation) => conversation.id === renamingConversation.value?.id)) {
      renamingConversation.value = null;
    }
    applyActiveConversation();
  },
  { deep: true }
);

watch(
  [() => activeConversationId.value, () => isReady.value],
  ([conversationId, ready]) => {
    if (!ready || !conversationId) return;
    void loadConversationMessages(conversationId);
  },
  { immediate: true }
);

watch(
  [() => activeConversationId.value, () => latestActiveMessageId.value, () => isReady.value],
  ([conversationId, messageId, ready]) => {
    if (!ready || !conversationId || !messageId) return;
    if (lastMarkedMessageId.value === messageId) return;
    lastMarkedMessageId.value = messageId;
    void markConversationRead(conversationId);
  }
);


watch(
  [() => activeConversationId.value, () => workspaceId.value, () => isReady.value],
  ([conversationId, nextWorkspaceId, ready]) => {
    if (!nextWorkspaceId || !ready) return;
    scheduleCacheSave(nextWorkspaceId, conversationId);
  }
);

onBeforeUnmount(() => {
  flushCacheSaves();
  if (unlistenTerminalChat) {
    unlistenTerminalChat();
    unlistenTerminalChat = null;
  }
});

onMounted(() => {
  registerTerminalChatListener();
  void contactsStore.load();
});
</script>

