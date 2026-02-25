<template>
  <div class="flex h-full w-full relative">
    <ChatSidebar
      :conversations="conversations"
      :members="members"
      :active-conversation-id="activeConversationId"
      @select-conversation="handleSelectConversation"
      @conversation-action="handleConversationAction"
    />

    <div class="flex-1 flex flex-col min-w-0 bg-transparent relative z-0">
      <ChatHeader
        :title="headerTitle"
        :description="headerDescription"
        :member-count="members.length"
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
        @open-roadmap="activeModal = 'roadmap'"
      />
      <ChatInput
        ref="chatInputRef"
        v-model="inputValue"
        :max-length="maxMessageLength"
        :is-generating="isAssistantTyping"
        :quick-prompts="quickPrompts"
        :placeholder="inputPlaceholder"
        :members="members"
        @send="handleSendMessage"
        @stop="cancelAssistantReply"
      />
    </div>

    <MembersSidebar
      :members="members"
      :show-invite-menu="showInviteMenu"
      :current-user-id="currentUserId"
      variant="sidebar"
      @toggle-invite="toggleInviteMenu"
      @member-action="handleMemberAction"
    />

    <template v-if="showMembersDrawer">
      <div class="fixed inset-0 bg-black/40 backdrop-blur-[1px] z-40 xl:hidden" @click="showMembersDrawer = false"></div>
      <div class="fixed right-0 top-0 bottom-0 z-50 xl:hidden">
        <MembersSidebar
          :members="members"
          :show-invite-menu="showInviteMenu"
          :current-user-id="currentUserId"
          variant="drawer"
          @toggle-invite="toggleInviteMenu"
          @member-action="handleMemberAction"
        />
      </div>
    </template>

    <template v-if="showInviteMenu">
      <div class="fixed inset-0 bg-background/60 backdrop-blur-[2px] z-40 transition-opacity duration-300" @click="showInviteMenu = false"></div>
      <InviteMenu @select="handleInviteSelect" />
    </template>

    <RoadmapModal v-if="activeModal === 'roadmap'" @close="activeModal = null" />
    <SkillManagementModal
      v-if="activeModal === 'skills'"
      @close="activeModal = null"
      @configure="activeModal = 'skillDetail'"
    />
    <SkillDetailModal v-if="activeModal === 'skillDetail'" @close="activeModal = null" @back="activeModal = 'skills'" />

    <InviteAdminModal v-if="activeModalType === 'admin'" @close="activeModalType = null" />
    <InviteAssistantModal
      v-if="activeModalType === 'assistant'"
      :title="t('invite.assistant.title')"
      @close="activeModalType = null"
      @invite="handleInvite($event, 'assistant')"
    />
    <InviteAssistantModal
      v-if="activeModalType === 'member'"
      :title="t('invite.member.title')"
      @close="activeModalType = null"
      @invite="handleInvite($event, 'member')"
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
import { computed, nextTick, ref, watch } from 'vue';
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
import InviteAdminModal from './modals/InviteAdminModal.vue';
import InviteAssistantModal from './modals/InviteAssistantModal.vue';
import ManageMemberModal from './modals/ManageMemberModal.vue';
import RenameConversationModal from './modals/RenameConversationModal.vue';
import type { Conversation, ConversationAction, Member, MemberAction } from './types';
import { useChatSession } from './composables/useChatSession';
import { DEFAULT_CHANNEL_ID } from './data';

type ActiveModal = 'roadmap' | 'skills' | 'skillDetail' | null;

type InviteModal = 'admin' | 'assistant' | 'member' | null;

type InviteRole = 'assistant' | 'member';

const activeModal = ref<ActiveModal>(null);
const activeModalType = ref<InviteModal>(null);
const showInviteMenu = ref(false);
const showMembersDrawer = ref(false);
const managingMember = ref<Member | null>(null);
const renamingConversation = ref<Conversation | null>(null);
const chatInputRef = ref<{ focus: () => void } | null>(null);
const buildConversationId = (type: 'channel' | 'dm', targetId: string) => `${type}:${targetId}`;
const activeConversationId = ref<string>(buildConversationId('channel', DEFAULT_CHANNEL_ID));

const inputValue = ref('');

const { t } = useI18n();
const {
  members,
  conversations,
  currentUser,
  assistantMember,
  assistantTypingConversationId,
  cancelAssistantReply,
  maxMessageLength,
  sendMessage,
  addMember,
  updateMember,
  removeMember,
  createInviteMember,
  ensureDirectMessage,
  toggleConversationPin,
  toggleConversationMute,
  renameConversation,
  clearConversationMessages,
  deleteConversation
} = useChatSession();

const currentUserId = computed(() => currentUser.value?.id ?? 'me');
const currentUserName = computed(() => currentUser.value?.name ?? t('members.roles.member'));
const assistantName = computed(() => assistantMember.value?.name ?? t('members.roles.aiAssistant'));
const assistantAvatar = computed(() => assistantMember.value?.avatar ?? '');
const quickPrompts = computed(() => [
  t('chat.input.quickPrompts.summarize'),
  t('chat.input.quickPrompts.draftReply'),
  t('chat.input.quickPrompts.extractTasks')
]);

const buildMentionLabel = (name: string) => `@${name},`;

const activeConversation = computed(() => conversations.value.find((conversation) => conversation.id === activeConversationId.value) ?? null);
const activeMessages = computed(() => activeConversation.value?.messages ?? []);

const memberById = computed(() => new Map(members.value.map((member) => [member.id, member])));

const activeDirectMember = computed(() => {
  if (activeConversation.value?.type !== 'dm') return null;
  return memberById.value.get(activeConversation.value.targetId) ?? null;
});

const getConversationTitle = (conversation: Conversation) => {
  if (conversation.type === 'dm') {
    return memberById.value.get(conversation.targetId)?.name ?? t('members.roles.member');
  }
  return conversation.customName ?? (conversation.nameKey ? t(conversation.nameKey) : conversation.targetId);
};

const headerTitle = computed(() => {
  if (!activeConversation.value) {
    return t('chat.channelName');
  }
  if (activeConversation.value.type === 'dm') {
    return activeDirectMember.value?.name ?? t('chat.channelName');
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
    return t('chat.input.directPlaceholder', { name: activeDirectMember.value.name });
  }
  if (activeConversation.value?.type === 'channel') {
    const label = getConversationTitle(activeConversation.value);
    const display = label.startsWith('#') ? label : `#${label}`;
    return t('chat.input.placeholder', { channel: display });
  }
  return t('chat.input.placeholder', { channel: t('chat.channelDisplay') });
});

const isAssistantTyping = computed(() => assistantTypingConversationId.value === activeConversationId.value);

const renamingConversationName = computed(() => {
  if (!renamingConversation.value) return '';
  if (renamingConversation.value.type === 'dm') {
    return memberById.value.get(renamingConversation.value.targetId)?.name ?? '';
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
  void focusChatInput();
};

const toggleInviteMenu = () => {
  showInviteMenu.value = !showInviteMenu.value;
};

const handleSelectConversation = (conversationId: string) => {
  activeConversationId.value = conversationId;
};

const handleInviteSelect = (type: 'admin' | 'assistant' | 'member') => {
  showInviteMenu.value = false;
  activeModalType.value = type;
};

const handleInvite = (name: string, type: InviteRole) => {
  const isAssistant = type === 'assistant';
  const newMember = createInviteMember(
    name,
    isAssistant ? 'assistant' : 'member',
    isAssistant ? 'members.roles.aiAssistant' : 'members.roles.member',
    isAssistant ? 'online' : 'offline'
  );

  addMember(newMember);
  activeModalType.value = null;
};

const handleUpdateMember = (id: string, newName: string) => {
  updateMember(id, { name: newName });
  managingMember.value = null;
};

const handleRemoveMember = (id: string) => {
  if (id === currentUserId.value) return;
  removeMember(id);
  managingMember.value = null;
};

const handleMemberAction = ({ action, member }: { action: MemberAction; member: Member }) => {
  if (action === 'send-message') {
    const conversationId = ensureDirectMessage(member.id);
    if (conversationId) {
      activeConversationId.value = conversationId;
    }
    setMessageTarget();
    return;
  }

  if (action === 'mention') {
    appendMention(member);
    return;
  }

  if (action === 'rename') {
    managingMember.value = member;
    return;
  }

  if (action === 'remove') {
    if (member.id === currentUserId.value) {
      return;
    }
    if (managingMember.value?.id === member.id) {
      managingMember.value = null;
    }
    removeMember(member.id);
  }
};

const handleSendMessage = () => {
  if (!activeConversation.value) return;
  const result = sendMessage(inputValue.value, activeConversation.value.id);
  if (result) {
    inputValue.value = '';
  }
};

const handleConversationAction = ({ conversationId, action }: { conversationId: string; action: ConversationAction }) => {
  if (action === 'rename') {
    const conversation = conversations.value.find((item) => item.id === conversationId);
    if (conversation?.type === 'channel') {
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
    if (conversationId === buildConversationId('channel', DEFAULT_CHANNEL_ID)) {
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

watch(conversations, (nextConversations) => {
  if (renamingConversation.value && !nextConversations.some((conversation) => conversation.id === renamingConversation.value?.id)) {
    renamingConversation.value = null;
  }
  if (!nextConversations.length) {
    activeConversationId.value = '';
    return;
  }
  if (!nextConversations.some((conversation) => conversation.id === activeConversationId.value)) {
    activeConversationId.value = nextConversations[0].id;
  }
});
</script>

