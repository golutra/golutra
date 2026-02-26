<template>
  <div class="flex h-full w-full flex-col overflow-hidden">
    <header class="px-8 pt-8 pb-4 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-primary/10 text-primary flex items-center justify-center border border-primary/20">
          <span class="material-symbols-outlined text-[20px]">group</span>
        </div>
        <div class="flex flex-col">
          <span class="text-white font-semibold text-[18px]">{{ t('friends.title') }}</span>
          <span class="text-white/40 text-[12px]">{{ totalFriends }}</span>
        </div>
      </div>
      <div class="relative">
        <button
          type="button"
          @click="toggleInviteMenu"
          :class="[
            'h-9 px-4 rounded-xl flex items-center gap-2 text-[12px] font-semibold transition-colors border',
            showInviteMenu
              ? 'bg-primary/20 text-primary border-primary/30'
              : 'bg-white/5 text-white/70 border-white/10 hover:bg-white/10 hover:text-white'
          ]"
        >
          <span class="material-symbols-outlined text-[18px]">person_add</span>
          {{ t('friends.add') }}
        </button>
        <template v-if="showInviteMenu">
          <div class="fixed inset-0 bg-background/60 backdrop-blur-[2px] z-40 transition-opacity duration-300" @click="showInviteMenu = false"></div>
          <InviteMenu position-class="absolute right-0 top-full mt-3" @select="handleInviteSelect" />
        </template>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto px-8 pb-10 space-y-8 custom-scrollbar">
      <div v-if="totalFriends === 0" class="text-white/40 text-sm py-12 text-center">
        {{ t('friends.empty') }}
      </div>

      <section v-if="projectFriends.length">
        <div class="flex items-center justify-between mb-3">
          <span class="text-[11px] font-bold uppercase tracking-widest text-white/40">{{ t('friends.sections.project') }}</span>
          <span class="text-[11px] text-white/30">{{ projectFriends.length }}</span>
        </div>
        <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          <div
            v-for="friend in projectFriends"
            :key="friend.id"
            class="relative z-0 flex items-center gap-3 p-3 rounded-2xl bg-panel/50 border border-white/10 shadow-sm hover:z-30"
          >
            <div class="relative">
              <button type="button" class="rounded-full" @click="handleFriendAvatarClick(friend)">
                <AvatarBadge :avatar="friend.avatar" :label="friend.name" class="w-11 h-11 rounded-full shadow-md" />
              </button>
              <MemberStatusDots
                class="absolute -bottom-0.5 -right-0.5"
                :status="friend.status"
                :terminal-status="friend.terminalStatus"
                :show-terminal-status="hasTerminalConfig(friend.terminalType, friend.terminalCommand)"
              />
            </div>
            <div class="min-w-0 flex-1">
              <div class="text-sm text-white font-semibold truncate">{{ friend.name }}</div>
              <div class="text-[11px] text-white/40">{{ friend.roleType }}</div>
            </div>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                :title="t('members.actions.sendMessage')"
                :aria-label="t('members.actions.sendMessage')"
                @click="handleStartDirectChat(friend)"
              >
                <span class="material-symbols-outlined text-[18px]">chat_bubble</span>
              </button>
              <div class="relative">
                <button
                  type="button"
                  class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                  data-friend-menu-toggle
                  @click="toggleStatusMenu(friend.id)"
                >
                  <span class="material-symbols-outlined text-[18px]">more_vert</span>
                </button>
                <div
                  v-if="openMenuId === friend.id"
                  data-friend-menu
                  class="absolute right-0 top-full mt-2 w-52 rounded-xl glass-modal bg-panel-strong/95 flex flex-col py-1.5 shadow-2xl overflow-hidden z-50 animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10"
                  @click.stop
                >
                  <div class="px-4 py-1 text-[10px] font-semibold uppercase tracking-wider text-white/40">
                    {{ t('settings.status') }}
                  </div>
                  <button
                    v-for="option in statusOptionsFor(friend)"
                    :key="option.id"
                    type="button"
                    class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/15 hover:text-white hover:ring-1 hover:ring-white/10 transition-colors flex items-center gap-3"
                    @click="handleStatusChange(friend, option.id)"
                  >
                    <span :class="['w-2.5 h-2.5 rounded-full', option.dotClass]"></span>
                    {{ t(option.labelKey) }}
                    <span v-if="friend.status === option.id" class="material-symbols-outlined text-[16px] ml-auto text-white/60">check</span>
                  </button>
                </div>
              </div>
              <button
                type="button"
                class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                @click="handleFriendDelete(friend)"
              >
                <span class="material-symbols-outlined text-[18px]">delete</span>
              </button>
            </div>
          </div>
        </div>
      </section>

      <section v-if="globalFriends.length">
        <div class="flex items-center justify-between mb-3">
          <span class="text-[11px] font-bold uppercase tracking-widest text-white/40">{{ t('friends.sections.global') }}</span>
          <span class="text-[11px] text-white/30">{{ globalFriends.length }}</span>
        </div>
        <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          <div
            v-for="friend in globalFriends"
            :key="friend.id"
            class="relative z-0 flex items-center gap-3 p-3 rounded-2xl bg-panel/40 border border-white/10 shadow-sm hover:z-30"
          >
            <div class="relative">
              <AvatarBadge :avatar="friend.avatar" :label="friend.name" class="w-11 h-11 rounded-full shadow-md" />
              <MemberStatusDots
                class="absolute -bottom-0.5 -right-0.5"
                :status="friend.status"
                :terminal-status="friend.terminalStatus"
                :show-terminal-status="hasTerminalConfig(friend.terminalType, friend.terminalCommand)"
              />
            </div>
            <div class="min-w-0 flex-1">
              <div class="text-sm text-white font-semibold truncate">{{ friend.name }}</div>
              <div class="text-[11px] text-white/40">{{ friend.roleType }}</div>
            </div>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                :title="t('members.actions.sendMessage')"
                :aria-label="t('members.actions.sendMessage')"
                @click="handleStartDirectChat(friend)"
              >
                <span class="material-symbols-outlined text-[18px]">chat_bubble</span>
              </button>
              <div class="relative">
                <button
                  type="button"
                  class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                  data-friend-menu-toggle
                  @click="toggleStatusMenu(friend.id)"
                >
                  <span class="material-symbols-outlined text-[18px]">more_vert</span>
                </button>
                <div
                  v-if="openMenuId === friend.id"
                  data-friend-menu
                  class="absolute right-0 top-full mt-2 w-52 rounded-xl glass-modal bg-panel-strong/95 flex flex-col py-1.5 shadow-2xl overflow-hidden z-50 animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10"
                  @click.stop
                >
                  <div class="px-4 py-1 text-[10px] font-semibold uppercase tracking-wider text-white/40">
                    {{ t('settings.status') }}
                  </div>
                  <button
                    v-for="option in statusOptionsFor(friend)"
                    :key="option.id"
                    type="button"
                    class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/15 hover:text-white hover:ring-1 hover:ring-white/10 transition-colors flex items-center gap-3"
                    @click="handleStatusChange(friend, option.id)"
                  >
                    <span :class="['w-2.5 h-2.5 rounded-full', option.dotClass]"></span>
                    {{ t(option.labelKey) }}
                    <span v-if="friend.status === option.id" class="material-symbols-outlined text-[16px] ml-auto text-white/60">check</span>
                  </button>
                </div>
              </div>
              <button
                type="button"
                class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                @click="handleFriendDelete(friend)"
              >
                <span class="material-symbols-outlined text-[18px]">delete</span>
              </button>
            </div>
          </div>
        </div>
      </section>
    </div>

    <InviteAdminModal v-if="activeModalType === 'admin'" @close="activeModalType = null" @invite="handleAdminInvite" />
    <InviteAssistantModal
      v-if="activeModalType === 'assistant'"
      :title="t('invite.assistant.title')"
      invite-role="assistant"
      @close="activeModalType = null"
      @invite="handleInvite($event, 'assistant')"
    />
    <InviteAssistantModal
      v-if="activeModalType === 'member'"
      :title="t('invite.member.title')"
      invite-role="member"
      @close="activeModalType = null"
      @invite="handleInvite($event, 'member')"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import { isTauri } from '@tauri-apps/api/core';

import AvatarBadge from '@/shared/components/AvatarBadge.vue';
import MemberStatusDots from './components/MemberStatusDots.vue';
import InviteMenu from './components/InviteMenu.vue';
import InviteAdminModal from './modals/InviteAdminModal.vue';
import InviteAssistantModal from './modals/InviteAssistantModal.vue';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useChatStore } from '@/features/chat/chatStore';
import { useTerminalMemberStore } from '@/features/terminal/terminalMemberStore';
import { useNavigationStore } from '@/stores/navigationStore';
import { useToastStore } from '@/stores/toastStore';
import { buildSeededAvatar } from '@/shared/utils/avatar';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import { ensureUniqueName } from './utils';
import { generateUlid } from './chatBridge';
import { saveChatCache } from './chatStorage';
import { useContactsStore } from './contactsStore';
import type { Contact, FriendEntry, Member, MemberStatus } from './types';
import { CURRENT_USER_ID } from './data';

type InviteModal = 'admin' | 'assistant' | 'member' | null;
type InviteRole = 'assistant' | 'member';
type InviteModel = {
  id: string;
  label: string;
  command: string;
  terminalType: 'shell' | 'codex' | 'gemini' | 'claude';
  instances: number;
  unlimitedAccess: boolean;
  sandboxed: boolean;
};

const { t } = useI18n();
const projectStore = useProjectStore();
const workspaceStore = useWorkspaceStore();
const { currentWorkspace } = storeToRefs(workspaceStore);
const navigationStore = useNavigationStore();
const { setActiveTab } = navigationStore;
const chatStore = useChatStore();
const { members } = storeToRefs(projectStore);
const { addMember, updateMember, removeMember } = projectStore;
const { conversations, currentUser } = storeToRefs(chatStore);
const { setConversationMembers, deleteMemberConversations, ensureDirectMessage } = chatStore;
const terminalMemberStore = useTerminalMemberStore();
const { ensureMemberSession, stopMemberSession } = terminalMemberStore;
const toastStore = useToastStore();
const { pushToast } = toastStore;
const contactsStore = useContactsStore();
const { contacts } = storeToRefs(contactsStore);
const { upsertContact } = contactsStore;

const showInviteMenu = ref(false);
const activeModalType = ref<InviteModal>(null);
const openMenuId = ref<string | null>(null);

const projectFriends = computed<FriendEntry[]>(() =>
  members.value
    .filter((member) => member.id !== CURRENT_USER_ID)
    .map((member) => ({
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
    }))
);

const globalFriends = computed<FriendEntry[]>(() => {
  const projectIds = new Set(projectFriends.value.map((entry) => entry.id));
  return contacts.value
    .filter((contact) => !projectIds.has(contact.id))
    .map((contact) => ({
      id: contact.id,
      name: contact.name,
      avatar: contact.avatar,
      roleType: contact.roleType,
      status: contact.status,
      scope: 'global'
    }));
});

const totalFriends = computed(() => projectFriends.value.length + globalFriends.value.length);

const baseStatusOptions: Array<{ id: MemberStatus; labelKey: string; dotClass: string }> = [
  { id: 'online', labelKey: 'settings.statusOptions.online', dotClass: 'bg-green-500' },
  { id: 'working', labelKey: 'settings.statusOptions.working', dotClass: 'bg-amber-400' },
  { id: 'dnd', labelKey: 'settings.statusOptions.dnd', dotClass: 'bg-red-500' },
  { id: 'offline', labelKey: 'settings.statusOptions.offline', dotClass: 'bg-white/30' }
];
const statusOptionsFor = (friend: FriendEntry) => {
  if (hasTerminalConfig(friend.terminalType, friend.terminalCommand)) {
    return baseStatusOptions.filter((option) => option.id !== 'working');
  }
  return baseStatusOptions;
};

const roleKeyForType = (roleType: FriendEntry['roleType']) => {
  if (roleType === 'owner') return 'members.roles.owner';
  if (roleType === 'admin') return 'members.roles.admin';
  if (roleType === 'assistant') return 'members.roles.aiAssistant';
  return 'members.roles.member';
};

const toggleInviteMenu = () => {
  showInviteMenu.value = !showInviteMenu.value;
};

const toggleStatusMenu = (friendId: string) => {
  openMenuId.value = openMenuId.value === friendId ? null : friendId;
};

const closeStatusMenu = () => {
  openMenuId.value = null;
};

const handleInviteSelect = (type: 'admin' | 'assistant' | 'member') => {
  showInviteMenu.value = false;
  activeModalType.value = type;
};

const ensureUniqueContactName = (name: string, roster: Array<{ name: string }>) => {
  const trimmed = name.trim();
  if (!trimmed) return '';
  const lowerNames = new Set(roster.map((entry) => entry.name.toLowerCase()));
  if (!lowerNames.has(trimmed.toLowerCase())) {
    return trimmed;
  }
  let counter = 1;
  let candidate = `${trimmed}-${counter}`;
  while (lowerNames.has(candidate.toLowerCase())) {
    counter += 1;
    candidate = `${trimmed}-${counter}`;
  }
  return candidate;
};

const createMemberId = async () => {
  try {
    return await generateUlid();
  } catch (error) {
    console.error('Failed to generate member id.', error);
    pushToast('Failed to generate member id.', { tone: 'error' });
    return null;
  }
};

const createInviteMember = async (
  name: string,
  roleType: 'assistant' | 'member',
  roleKey: string,
  status: Member['status'],
  roster: Member[],
  access?: { unlimitedAccess: boolean; sandboxed: boolean }
) => {
  const id = await createMemberId();
  if (!id) return null;
  const finalName = ensureUniqueName(name, roster);
  const avatar = buildSeededAvatar(`${roleType}:${finalName}`);

  return {
    id,
    name: finalName,
    role: '',
    roleKey,
    roleType,
    avatar,
    status,
    unlimitedAccess: access?.unlimitedAccess,
    sandboxed: access?.sandboxed
  };
};

const extractTerminalBase = (command: string) => {
  const trimmed = command.trim();
  const segment = trimmed.split(/\s+/)[0] || '';
  return segment.replace(/[\W_]+/g, ' ').trim() || t('settings.memberOptions.terminal');
};

const normalizeTerminalBase = (label: string, command: string) => {
  const trimmedLabel = label.trim();
  if (trimmedLabel) {
    return trimmedLabel;
  }
  return extractTerminalBase(command);
};

const nextTerminalIndex = (base: string, roster: Member[]) => {
  const pattern = new RegExp(`^${base.replace(/[-/\\^$*+?.()|[\]{}]/g, '\\$&')}-([\\d]+)$`, 'i');
  let maxIndex = 0;
  for (const member of roster) {
    const match = member.name.match(pattern);
    if (!match) continue;
    const parsed = Number(match[1]);
    if (Number.isFinite(parsed)) {
      maxIndex = Math.max(maxIndex, parsed);
    }
  }
  return maxIndex + 1;
};

const applyProjectPrefix = (base: string, projectName: string) => {
  const trimmedBase = base.trim();
  const trimmedProject = projectName.trim();
  if (!trimmedProject) {
    return trimmedBase;
  }
  if (!trimmedBase) {
    return trimmedProject;
  }
  const prefix = `${trimmedProject.toLowerCase()}-`;
  if (trimmedBase.toLowerCase().startsWith(prefix)) {
    return trimmedBase;
  }
  return `${trimmedProject}-${trimmedBase}`;
};

const buildTerminalMemberName = (projectName: string, label: string, command: string, roster: Member[]) => {
  const base = applyProjectPrefix(normalizeTerminalBase(label, command), projectName);
  const index = nextTerminalIndex(base, roster);
  return `${base}-${index}`;
};

const handleTerminalSessionError = (error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes('terminal buffer limit reached')) {
    pushToast(t('terminal.resourceLimit'), { tone: 'error' });
    return true;
  }
  return false;
};

const normalizeInviteInstances = (value: number) => {
  if (!Number.isFinite(value)) {
    return 1;
  }
  return Math.max(1, Math.round(value));
};

const handleAdminInvite = async (payload: { identifier: string }) => {
  const trimmed = payload.identifier.trim();
  if (!trimmed) {
    return;
  }
  const id = await createMemberId();
  if (!id) {
    return;
  }
  const name = ensureUniqueContactName(trimmed, [
    ...members.value.map((member) => ({ name: member.name })),
    ...contacts.value.map((contact) => ({ name: contact.name }))
  ]);
  if (!name) {
    return;
  }
  const contact: Contact = {
    id,
    name,
    avatar: buildSeededAvatar(`admin:${name}`),
    roleType: 'admin',
    status: 'offline',
    createdAt: Date.now()
  };
  await contactsStore.upsertContact(contact);
  activeModalType.value = null;
};

const handleInvite = async (model: InviteModel, type: InviteRole) => {
  const isAssistant = type === 'assistant';
  const instanceCount = normalizeInviteInstances(model.instances);
  const access = { unlimitedAccess: model.unlimitedAccess, sandboxed: model.sandboxed };
  if (isAssistant) {
    const roster = [...members.value];
    const newMembers: Member[] = [];
    for (let i = 0; i < instanceCount; i += 1) {
      const newMember = await createInviteMember(
        model.label,
        'assistant',
        'members.roles.aiAssistant',
        'online',
        roster,
        access
      );
      if (!newMember) {
        activeModalType.value = null;
        return;
      }
      roster.push(newMember);
      newMembers.push(newMember);
    }
    for (const member of newMembers) {
      await addMember(member);
    }
    await syncDefaultChannelMembers();
    activeModalType.value = null;
    return;
  }

  const command = model.command?.trim() ?? '';
  const roster = [...members.value];
  const newMembers: Member[] = [];
  for (let i = 0; i < instanceCount; i += 1) {
    const terminalName = buildTerminalMemberName(currentWorkspace.value?.name ?? '', model.label, command, roster);
    const memberId = await createMemberId();
    if (!memberId) {
      activeModalType.value = null;
      return;
    }
    const newMember: Member = {
      id: memberId,
      name: terminalName,
      role: '',
      roleKey: 'members.roles.member',
      roleType: 'member',
      avatar: buildSeededAvatar(`member:${terminalName}`),
      status: 'online',
      terminalType: model.terminalType,
      terminalCommand: command || undefined,
      autoStartTerminal: true,
      unlimitedAccess: access.unlimitedAccess,
      sandboxed: access.sandboxed
    };
    roster.push(newMember);
    newMembers.push(newMember);
  }

  for (const member of newMembers) {
    await addMember(member);
  }
  await syncDefaultChannelMembers();
  activeModalType.value = null;
  for (const member of newMembers) {
    try {
      await ensureMemberSession(member, { openTab: false });
    } catch (error) {
      handleTerminalSessionError(error);
      console.error('Failed to start terminal session.', error);
      await updateMember(member.id, { status: 'offline', terminalStatus: 'disconnected' });
    }
  }
};

const syncDefaultChannelMembers = async () => {
  const defaultConversation = conversations.value.find((item) => item.isDefault);
  if (!defaultConversation) return;
  const currentUserId = currentUser.value?.id ?? CURRENT_USER_ID;
  const memberIds = Array.from(new Set([currentUserId, ...members.value.map((member) => member.id)]));
  await setConversationMembers(defaultConversation.id, memberIds);
};

const ensureFriendMember = async (friend: FriendEntry) => {
  if (members.value.some((member) => member.id === friend.id)) {
    return;
  }
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
  await syncDefaultChannelMembers();
};

const handleStartDirectChat = async (friend: FriendEntry) => {
  closeStatusMenu();
  const workspaceId = currentWorkspace.value?.id;
  if (!workspaceId) return;
  await ensureFriendMember(friend);
  const conversationId = await ensureDirectMessage(friend.id);
  if (!conversationId) return;
  try {
    await saveChatCache(workspaceId, { activeConversationId: conversationId });
  } catch (error) {
    console.error('Failed to save chat cache.', error);
  }
  setActiveTab('chat');
};

const handleFriendDelete = async (friend: FriendEntry) => {
  closeStatusMenu();
  await deleteMemberConversations(friend.id);
  if (friend.scope === 'global') {
    await contactsStore.removeContact(friend.id);
    return;
  }
  if (hasTerminalConfig(friend.terminalType, friend.terminalCommand)) {
    try {
      await stopMemberSession(friend.id, { preserve: false, fireAndForget: true });
    } catch (error) {
      console.error('Failed to stop terminal session.', error);
    }
  }
  await removeMember(friend.id);
  await syncDefaultChannelMembers();
};

const updateProjectFriendStatus = async (friend: FriendEntry, status: MemberStatus) => {
  const member = members.value.find((item) => item.id === friend.id);
  if (!member) return;
  if (hasTerminalConfig(member.terminalType, member.terminalCommand)) {
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
};

const updateGlobalFriendStatus = async (friend: FriendEntry, status: MemberStatus) => {
  const contact = contacts.value.find((item) => item.id === friend.id);
  if (!contact) return;
  await upsertContact({ ...contact, status });
};

const handleStatusChange = async (friend: FriendEntry, status: MemberStatus) => {
  if (friend.status === status) {
    closeStatusMenu();
    return;
  }
  try {
    if (friend.scope === 'global') {
      await updateGlobalFriendStatus(friend, status);
    } else {
      await updateProjectFriendStatus(friend, status);
    }
  } finally {
    closeStatusMenu();
  }
};

const handleFriendAvatarClick = async (friend: FriendEntry) => {
  if (friend.scope !== 'project') return;
  if (!hasTerminalConfig(friend.terminalType, friend.terminalCommand)) return;
  const member = members.value.find((item) => item.id === friend.id);
  if (!member) return;
  try {
    if (import.meta.env.DEV) {
      try {
        if (window.localStorage.getItem('terminal-debug') === '1') {
          console.info('[friends] open terminal from avatar', { memberId: member.id });
        }
      } catch {
        // Ignore logging failures.
      }
    }
    await terminalMemberStore.openMemberTerminal(member);
  } catch (error) {
    handleTerminalSessionError(error);
    console.error('Failed to open member terminal.', error);
  }
};

const handleClickOutside = (event: MouseEvent) => {
  if (!openMenuId.value) return;
  const target = event.target as HTMLElement | null;
  if (target?.closest('[data-friend-menu]') || target?.closest('[data-friend-menu-toggle]')) {
    return;
  }
  closeStatusMenu();
};

onMounted(() => {
  if (isTauri()) {
    void contactsStore.load();
  }
  document.addEventListener('click', handleClickOutside, true);
});

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside, true);
});
</script>
