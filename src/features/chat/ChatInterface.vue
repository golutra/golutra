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
        ref="messagesListRef"
        :messages="activeMessages"
        :current-user-id="currentUserId"
        :current-user-name="currentUserName"
        :has-more="hasMoreMessages"
        :is-loading-more="isLoadingMore"
        :terminal-member-ids="terminalMemberIds"
        @load-more="handleLoadOlderMessages"
        @open-roadmap="activeModal = 'roadmap'"
        @open-terminal="handleMessageAvatarOpen"
      />
      <div class="relative">
        <ChatInput
          ref="chatInputRef"
          v-model="inputValue"
          :max-length="maxMessageLength"
          :quick-prompts="quickPrompts"
          :placeholder="inputPlaceholder"
          :members="mentionableMembers"
          @send="handleSendMessage"
        />
      </div>
    </div>

    <MembersSidebar
      :members="conversationMembers"
      :current-user-id="currentUserId"
      variant="sidebar"
      @open-invite="openFriendsInvite"
      @member-action="handleMemberAction"
    >
      <template v-if="isDefaultChannelActive" #header-action>
        <div class="relative">
          <button
            type="button"
            @click="toggleAddFriendMenu"
            :class="[
              'h-9 px-4 rounded-xl flex items-center gap-2 text-[12px] font-semibold transition-colors border',
              showAddFriendMenu
                ? 'bg-primary/20 text-primary border-primary/30'
                : 'bg-white/5 text-white/70 border-white/10 hover:bg-white/10 hover:text-white'
            ]"
          >
            <span class="material-symbols-outlined text-[18px]">person_add</span>
            {{ t('friends.add') }}
          </button>
          <template v-if="showAddFriendMenu">
            <div
              class="fixed inset-0 bg-background/60 backdrop-blur-[2px] z-40 transition-opacity duration-300"
              @click="showAddFriendMenu = false"
            ></div>
            <InviteMenu position-class="absolute right-0 top-full mt-3" @select="handleAddFriendSelect" />
          </template>
        </div>
      </template>
    </MembersSidebar>

    <template v-if="showMembersDrawer">
      <div class="fixed inset-0 bg-black/40 backdrop-blur-[1px] z-40 md:hidden" @click="showMembersDrawer = false"></div>
      <div class="fixed right-0 top-0 bottom-0 z-50 md:hidden">
        <MembersSidebar
          :members="conversationMembers"
          :current-user-id="currentUserId"
          variant="drawer"
          @open-invite="openFriendsInvite"
          @member-action="handleMemberAction"
        >
          <template v-if="isDefaultChannelActive" #header-action>
            <div class="relative">
              <button
                type="button"
                @click="toggleAddFriendMenu"
                :class="[
                  'h-9 px-4 rounded-xl flex items-center gap-2 text-[12px] font-semibold transition-colors border',
                  showAddFriendMenu
                    ? 'bg-primary/20 text-primary border-primary/30'
                    : 'bg-white/5 text-white/70 border-white/10 hover:bg-white/10 hover:text-white'
                ]"
              >
                <span class="material-symbols-outlined text-[18px]">person_add</span>
                {{ t('friends.add') }}
              </button>
              <template v-if="showAddFriendMenu">
                <div
                  class="fixed inset-0 bg-background/60 backdrop-blur-[2px] z-40 transition-opacity duration-300"
                  @click="showAddFriendMenu = false"
                ></div>
                <InviteMenu position-class="absolute right-0 top-full mt-3" @select="handleAddFriendSelect" />
              </template>
            </div>
          </template>
        </MembersSidebar>
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
    <InviteAdminModal v-if="addFriendModalType === 'admin'" @close="addFriendModalType = null" @invite="handleAddFriendAdminInvite" />
    <InviteAssistantModal
      v-if="addFriendModalType === 'assistant'"
      :title="t('invite.assistant.title')"
      invite-role="assistant"
      @close="addFriendModalType = null"
      @invite="handleAddFriendInvite($event, 'assistant')"
    />
    <InviteAssistantModal
      v-if="addFriendModalType === 'member'"
      :title="t('invite.member.title')"
      invite-role="member"
      @close="addFriendModalType = null"
      @invite="handleAddFriendInvite($event, 'member')"
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
// 聊天主界面：协调会话列表、消息输入、成员侧栏与终端联动。
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import ChatSidebar from './components/ChatSidebar.vue';
import ChatHeader from './components/ChatHeader.vue';
import MessagesList from './components/MessagesList.vue';
import ChatInput from './components/ChatInput.vue';
import MembersSidebar from './components/MembersSidebar.vue';
import InviteMenu from './components/InviteMenu.vue';
import RoadmapModal from './modals/RoadmapModal.vue';
import SkillManagementModal from './modals/SkillManagementModal.vue';
import SkillDetailModal from './modals/SkillDetailModal.vue';
import InviteFriendsModal from './modals/InviteFriendsModal.vue';
import InviteAdminModal from './modals/InviteAdminModal.vue';
import InviteAssistantModal from './modals/InviteAssistantModal.vue';
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
import { ensureAvatar } from '@/shared/utils/avatar';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';
import { createFrontLogger } from '@/shared/monitoring/passiveMonitor';
import { resolveMemberDisplayName } from '@/shared/utils/memberDisplay';
import { registerKeybindRule, unregisterKeybindRule } from '@/shared/keyboard/registry';
import { resolveKeybindKeys, type KeybindActionId } from '@/shared/keyboard/profiles';
import type { KeybindItem } from '@/shared/keyboard/types';
import { useTerminalOrchestratorStore } from '@/stores/terminalOrchestratorStore';
import { useToastStore } from '@/stores/toastStore';
import { useNotificationOrchestratorStore } from '@/stores/notificationOrchestratorStore';
import { onChatMessageCreated } from './chatBridge';
import { useFriendInvites } from './useFriendInvites';
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

type ActiveModal = 'roadmap' | 'skills' | 'skillDetail' | null;

const activeModal = ref<ActiveModal>(null);
const showFriendsInviteModal = ref(false);
const showMembersDrawer = ref(false);
const managingMember = ref<Member | null>(null);
const renamingConversation = ref<Conversation | null>(null);
const {
  showInviteMenu: showAddFriendMenu,
  activeModalType: addFriendModalType,
  toggleInviteMenu: toggleAddFriendMenu,
  handleInviteSelect: handleAddFriendSelect,
  handleAdminInvite: handleAddFriendAdminInvite,
  handleInvite: handleAddFriendInvite
} = useFriendInvites();
const chatInputRef = ref<{ focus: () => void; registerMention: (member: Member) => void } | null>(null);
const messagesListRef = ref<{ jumpToLatest: () => void } | null>(null);
const activeConversationId = ref<string>('');

const inputValue = ref('');

const isTauriEnv = isTauri();
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
const notificationOrchestratorStore = useNotificationOrchestratorStore();
const { pendingOpenConversationId } = storeToRefs(notificationOrchestratorStore);
const { clearPendingOpenConversation } = notificationOrchestratorStore;
const terminalOrchestratorStore = useTerminalOrchestratorStore();
const { ensureMemberSession, openMemberTerminal, stopMemberSession, onTerminalStreamMessage } = terminalOrchestratorStore;
const toastStore = useToastStore();
const { pushToast } = toastStore;
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
  isReady,
  defaultChannelId,
  totalUnreadCount
} = storeToRefs(chatStore);
  const {
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
    applyTerminalStreamMessage,
    getConversationPaging,
    markConversationRead
  } = chatStore;
const maxMessageLength = chatStore.maxMessageLength;

const DEFAULT_OWNER_NAME = 'Owner';
const DEFAULT_MEMBER_NAME = 'Member';
const accountDisplayName = computed(() => settings.value.account.displayName.trim());
const resolvedAccountName = computed(() => accountDisplayName.value || DEFAULT_OWNER_NAME);
const accountAvatar = computed(() => ensureAvatar(settings.value.account.avatar));
const currentUserId = computed(() => currentUser.value?.id ?? CURRENT_USER_ID);
const currentUserName = computed(() => resolvedAccountName.value);
const quickPrompts = computed(() => [
  t('chat.input.quickPrompts.summarize'),
  t('chat.input.quickPrompts.draftReply'),
  t('chat.input.quickPrompts.extractTasks')
]);

const CHAT_KEYBIND_RULE_ID = 'chat-keybinds';

const buildChatKeybindItems = (): KeybindItem[] => {
  if (!settings.value.keybinds.enabled) {
    return [];
  }
  const profile = settings.value.keybinds.profile;
  const items: KeybindItem[] = [];
  const addItem = (actionId: KeybindActionId, item: Omit<KeybindItem, 'id' | 'keys'>) => {
    const keys = resolveKeybindKeys(profile, actionId);
    if (keys.length === 0) {
      return;
    }
    items.push({ id: `chat-${actionId}`, keys, ...item });
  };

  addItem('jump-to-latest', {
    allowInEditable: true,
    action: () => {
      messagesListRef.value?.jumpToLatest();
    }
  });

  addItem('toggle-mute', {
    allowInEditable: true,
    enabled: () => Boolean(activeConversationId.value),
    action: () => {
      if (!activeConversationId.value) {
        return;
      }
      toggleConversationMute(activeConversationId.value);
    }
  });

  return items;
};

registerKeybindRule({
  id: CHAT_KEYBIND_RULE_ID,
  order: 10,
  mode: 'merge',
  stop: false,
  matches: () => true,
  items: () => buildChatKeybindItems()
});

const buildMentionLabel = (name: string) => `@${name}`;
const CACHE_SAVE_DEBOUNCE_MS = 1200;
const cacheSaveTimers = new Map<string, number>();
const cacheSavePending = new Map<string, { activeConversationId: string }>();
let cacheSaveChain = Promise.resolve();
let unlistenTerminalChat: (() => void) | null = null;
let unlistenTerminalStream: (() => void) | null = null;
let unlistenWindowFocus: (() => void) | null = null;
const appWindow = isTauriEnv ? getCurrentWindow() : null;
const windowFocused = ref(true);
const debugLog = createFrontLogger('chat');
const isChatWindowActive = () => {
  if (typeof document === 'undefined') {
    return false;
  }
  if (document.visibilityState !== 'visible') {
    return false;
  }
  if (appWindow) {
    return windowFocused.value;
  }
  if (typeof document.hasFocus === 'function') {
    return document.hasFocus();
  }
  return true;
};

// 窗口重新获得焦点时仅清空当前会话未读。
const clearUnreadOnFocus = () => {
  if (!isReady.value || totalUnreadCount.value <= 0) {
    return;
  }
  if (!isChatWindowActive()) {
    return;
  }
  const conversationId = activeConversationId.value;
  if (!conversationId) {
    return;
  }
  const conversation = conversations.value.find((item) => item.id === conversationId);
  if (!conversation || (conversation.unreadCount ?? 0) <= 0) {
    return;
  }
  void markConversationRead(conversationId);
};

watch([windowFocused, isReady], ([focused, ready], [prevFocused, prevReady]) => {
  if (!focused || !ready) {
    return;
  }
  if (prevFocused && prevReady) {
    return;
  }
  clearUnreadOnFocus();
});

const uniqueIds = (ids: string[]) => Array.from(new Set(ids.filter((id) => id)));

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
const displayMembersForTitle = computed(() =>
  displayMembers.value.map((member) => ({ ...member, name: resolveMemberDisplayName(member) }))
);
const resolveDisplayName = (member?: Member | null) => (member ? resolveMemberDisplayName(member) : '');
const memberById = computed(() => new Map(displayMembers.value.map((member) => [member.id, member])));
const terminalMemberIds = computed(() =>
  members.value
    .filter((member) => member.id !== currentUserId.value)
    .filter((member) => member.roleType === 'assistant' || member.roleType === 'member')
    .filter((member) => hasTerminalConfig(member.terminalType, member.terminalCommand))
    .map((member) => member.id)
);

const friendEntries = computed<FriendEntry[]>(() => {
  const entries: FriendEntry[] = [];
  const seen = new Set<string>();
  for (const member of members.value) {
    if (!member || member.id === currentUserId.value) continue;
    entries.push({
      id: member.id,
      name: resolveMemberDisplayName(member),
      avatar: member.avatar,
      roleType: member.roleType,
      status: member.status,
      manualStatus: member.manualStatus,
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

const isDefaultChannelActive = computed(() => {
  const conversation = activeConversation.value;
  return conversation ? isMainChannel(conversation) : false;
});

const getConversationTitle = (conversation: Conversation) => {
  if (conversation.type === 'dm') {
    const targetId = conversation.targetId ?? '';
    return resolveDisplayName(memberById.value.get(targetId)) || DEFAULT_MEMBER_NAME;
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
    displayMembersForTitle.value,
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
    return resolveDisplayName(activeDirectMember.value) || fallbackChannelTitle.value;
  }
  return getConversationTitle(activeConversation.value);
});

const headerDescription = computed(() => {
  if (!activeConversation.value) {
    return '';
  }
  if (activeConversation.value.type === 'dm') {
    return activeDirectMember.value ? t('chat.directMessageDescription', { name: resolveDisplayName(activeDirectMember.value) }) : '';
  }
  if (activeConversation.value.descriptionKey) {
    return t(activeConversation.value.descriptionKey);
  }
  return '';
});

const inputPlaceholder = computed(() => {
  if (activeConversation.value?.type === 'dm' && activeDirectMember.value) {
    const displayName = resolveDisplayName(activeDirectMember.value).trim();
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

const renamingConversationName = computed(() => {
  if (!renamingConversation.value) return '';
  if (renamingConversation.value.type === 'dm') {
    const targetId = renamingConversation.value.targetId ?? '';
    return resolveDisplayName(memberById.value.get(targetId));
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
  if (isDefaultChannelActive.value) {
    showAddFriendMenu.value = true;
    return;
  }
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
    debugLog('open terminal from member action', { memberId: member.id });
    void logDiagnosticsEvent('chatinterface-handle-member-action', {
      memberId: member.id,
      terminalType: member.terminalType,
      terminalCommand: member.terminalCommand,
      action
    });
    if (member.terminalStatus === 'connecting' && member.terminalType !== 'shell') {
      pushToast(t('terminal.statusOptions.connecting'), { tone: 'info' });
      return;
    }
    await openMemberTerminal(member);
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
      const resolvedStatus = member.manualStatus ?? member.status;
      if (resolvedStatus === status) {
        return;
      }
      if (member.id === CURRENT_USER_ID) {
        setAccountStatus(status);
      }
      if (member.id !== CURRENT_USER_ID && hasTerminalConfig(member.terminalType, member.terminalCommand)) {
        if (status === 'offline') {
          await stopMemberSession(member.id, { preserve: false, fireAndForget: true });
          void updateMember(member.id, { autoStartTerminal: false, manualStatus: status });
          return;
        }
        if (status === 'online') {
          const session = await ensureMemberSession(member, { openTab: false });
          if (!session) {
            return;
          }
          void updateMember(member.id, { autoStartTerminal: true, manualStatus: status });
          return;
        }
        void updateMember(member.id, { manualStatus: status });
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
        await stopMemberSession(member.id, { preserve: false, fireAndForget: true, deleteSessionMap: true });
      } catch (error) {
        console.error('Failed to stop terminal session.', error);
      }
    }
    await deleteMemberConversations(member.id);
    await removeMember(member.id);
  }
};

const handleMessageAvatarOpen = async (memberId: string) => {
  if (!memberId || memberId === currentUserId.value) {
    return;
  }
  const member = memberById.value.get(memberId);
  if (!member) {
    return;
  }
  if (member.roleType !== 'assistant' && member.roleType !== 'member') {
    return;
  }
  if (!hasTerminalConfig(member.terminalType, member.terminalCommand)) {
    return;
  }
  debugLog('open terminal from message avatar', { memberId: member.id });
  void logDiagnosticsEvent('chatinterface-message-avatar-open', {
    memberId: member.id,
    terminalType: member.terminalType,
    terminalCommand: member.terminalCommand
  });
  if (member.terminalStatus === 'connecting' && member.terminalType !== 'shell') {
    pushToast(t('terminal.statusOptions.connecting'), { tone: 'info' });
    return;
  }
  await openMemberTerminal(member);
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
    clearConversationMessages(conversationId);
    return;
  }

  if (action === 'delete') {
    if (conversationId === defaultChannelId.value) {
      return;
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

type NotificationOpenConversationPayload = { conversationId?: string };
const preferredConversationId = ref<string | null>(null);

const applyNotificationConversation = async () => {
  const conversationId = pendingOpenConversationId.value;
  if (!conversationId || !isReady.value) {
    return;
  }
  if (!conversations.value.some((conversation) => conversation.id === conversationId)) {
    return;
  }
  clearPendingOpenConversation();
  preferredConversationId.value = conversationId;
  activeConversationId.value = conversationId;
  await loadConversationMessages(conversationId);
  await nextTick();
  messagesListRef.value?.jumpToLatest();
};

const queueNotificationConversation = (conversationId?: string | null) => {
  const trimmed = conversationId?.trim() ?? '';
  if (!trimmed) {
    return;
  }
  pendingOpenConversationId.value = trimmed;
  void applyNotificationConversation();
};

const resolveNotificationBootstrap = () => {
  if (typeof window === 'undefined') {
    return null;
  }
  const payload = (window as typeof window & { __GOLUTRA_NOTIFICATION_OPEN__?: NotificationOpenConversationPayload })
    .__GOLUTRA_NOTIFICATION_OPEN__;
  const conversationId = typeof payload?.conversationId === 'string' ? payload.conversationId.trim() : '';
  if (!conversationId) {
    return null;
  }
  return { conversationId };
};

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
    // 当前私聊正打开且窗口聚焦时，直接视为已读，避免托盘短暂闪未读。
    const readThrough = payload.conversationId === activeConversationId.value && isChatWindowActive();
    void appendTerminalMessage(payload, { readThrough });
  });
};

const registerTerminalStreamListener = () => {
  if (!isTauri() || unlistenTerminalStream) {
    return;
  }
  unlistenTerminalStream = onTerminalStreamMessage((payload) => {
    applyTerminalStreamMessage(payload);
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

watch(isDefaultChannelActive, (isDefault) => {
  if (!isDefault) {
    showAddFriendMenu.value = false;
  }
});

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
    if (!isChatWindowActive()) return;
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

watch(
  [() => pendingOpenConversationId.value, () => isReady.value, () => conversations.value.length],
  ([conversationId, ready]) => {
    if (!conversationId || !ready) return;
    void applyNotificationConversation();
  }
);

onBeforeUnmount(() => {
  unregisterKeybindRule(CHAT_KEYBIND_RULE_ID);
  flushCacheSaves();
  if (unlistenTerminalChat) {
    unlistenTerminalChat();
    unlistenTerminalChat = null;
  }
  if (unlistenTerminalStream) {
    unlistenTerminalStream();
    unlistenTerminalStream = null;
  }
  if (unlistenWindowFocus) {
    unlistenWindowFocus();
    unlistenWindowFocus = null;
  }
});

onMounted(() => {
  if (appWindow) {
    appWindow
      .isFocused()
      .then((focused) => {
        windowFocused.value = focused;
      })
      .catch(() => {});
    appWindow
      .onFocusChanged((event) => {
        windowFocused.value = event.payload;
      })
      .then((unlisten) => {
        unlistenWindowFocus = unlisten;
      })
      .catch(() => {});
  }
  registerTerminalChatListener();
  registerTerminalStreamListener();
  void contactsStore.load();
  const boot = resolveNotificationBootstrap();
  if (boot) {
    queueNotificationConversation(boot.conversationId);
    try {
      delete (window as typeof window & { __GOLUTRA_NOTIFICATION_OPEN__?: NotificationOpenConversationPayload })
        .__GOLUTRA_NOTIFICATION_OPEN__;
    } catch {
      // ignore
    }
  }
});
</script>


