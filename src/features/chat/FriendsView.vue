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

    <div ref="scrollContainerRef" class="flex-1 overflow-y-auto px-8 pb-10 space-y-8 custom-scrollbar">
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
            :class="[
              'relative flex items-center gap-3 p-3 rounded-2xl bg-panel/50 border border-white/10 shadow-sm',
              openMenuId === friend.id ? 'z-40' : 'z-0 hover:z-30'
            ]"
          >
            <div class="relative">
              <button
                type="button"
                class="member-avatar-button rounded-full focus:outline-none focus-visible:outline-none focus:ring-0 focus-visible:ring-0"
                @click.stop="handleFriendAvatarClick(friend)"
              >
                <AvatarBadge :avatar="friend.avatar" :label="resolveFriendName(friend)" class="w-11 h-11 rounded-full shadow-md" />
              </button>
              <MemberStatusDots
                class="absolute -bottom-0.5 -right-0.5"
                :status="resolveFriendStatus(friend)"
                :terminal-status="friend.terminalStatus"
                :show-terminal-status="hasTerminalConfig(friend.terminalType, friend.terminalCommand)"
              />
            </div>
            <div class="min-w-0 flex-1">
              <button
                type="button"
                class="flex items-center gap-1 min-w-0 w-full overflow-hidden text-left group"
                @click.stop="openFriendManage(friend)"
              >
                <span class="friend-name-clamp text-sm text-white font-semibold break-words flex-1 min-w-0 transition-colors group-hover:text-primary group-hover:underline cursor-pointer">
                  {{ resolveFriendName(friend) }}
                </span>
                <span class="material-symbols-outlined text-[14px] text-primary/70 opacity-0 transition-opacity group-hover:opacity-100 shrink-0" aria-hidden="true">
                  edit
                </span>
              </button>
              <div class="text-[11px] text-white/40">{{ formatFriendRole(friend.roleType) }}</div>
            </div>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                :title="t('members.actions.sendMessage')"
                :aria-label="t('members.actions.sendMessage')"
                @click.stop="handleStartDirectChat(friend)"
              >
                <span class="material-symbols-outlined text-[18px]">chat_bubble</span>
              </button>
              <div class="relative">
                <button
                  type="button"
                  class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                  data-friend-menu-toggle
                  @click.stop="toggleStatusMenu(friend.id, $event)"
                >
                  <span class="material-symbols-outlined text-[18px]">more_vert</span>
                </button>
                <div
                  v-if="openMenuId === friend.id"
                  data-friend-menu
                  :ref="setStatusMenuRef"
                  :class="[
                    'absolute right-0 w-52 rounded-xl glass-modal bg-panel-strong/95 flex flex-col py-1.5 shadow-2xl overflow-x-hidden overflow-y-auto z-50 animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10',
                    statusMenuPlacement === 'top' ? 'bottom-full mb-2' : 'top-full mt-2'
                  ]"
                  :style="statusMenuMaxHeight ? { maxHeight: statusMenuMaxHeight } : undefined"
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
                    <span v-if="resolveFriendStatus(friend) === option.id" class="material-symbols-outlined text-[16px] ml-auto text-white/60">check</span>
                  </button>
                </div>
              </div>
              <button
                type="button"
                class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                @click.stop="handleFriendDelete(friend)"
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
            :class="[
              'relative flex items-center gap-3 p-3 rounded-2xl bg-panel/40 border border-white/10 shadow-sm',
              openMenuId === friend.id ? 'z-40' : 'z-0 hover:z-30'
            ]"
          >
            <div class="relative">
              <AvatarBadge :avatar="friend.avatar" :label="resolveFriendName(friend)" class="w-11 h-11 rounded-full shadow-md" />
              <MemberStatusDots
                class="absolute -bottom-0.5 -right-0.5"
                :status="resolveFriendStatus(friend)"
                :terminal-status="friend.terminalStatus"
                :show-terminal-status="hasTerminalConfig(friend.terminalType, friend.terminalCommand)"
              />
            </div>
            <div class="min-w-0 flex-1">
              <button
                type="button"
                class="flex items-center gap-1 min-w-0 w-full overflow-hidden text-left group"
                @click.stop="openFriendManage(friend)"
              >
                <span class="friend-name-clamp text-sm text-white font-semibold break-words flex-1 min-w-0 transition-colors group-hover:text-primary group-hover:underline cursor-pointer">
                  {{ resolveFriendName(friend) }}
                </span>
                <span class="material-symbols-outlined text-[14px] text-primary/70 opacity-0 transition-opacity group-hover:opacity-100 shrink-0" aria-hidden="true">
                  edit
                </span>
              </button>
              <div class="text-[11px] text-white/40">{{ formatFriendRole(friend.roleType) }}</div>
            </div>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                :title="t('members.actions.sendMessage')"
                :aria-label="t('members.actions.sendMessage')"
                @click.stop="handleStartDirectChat(friend)"
              >
                <span class="material-symbols-outlined text-[18px]">chat_bubble</span>
              </button>
              <div class="relative">
                <button
                  type="button"
                  class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                  data-friend-menu-toggle
                  @click.stop="toggleStatusMenu(friend.id, $event)"
                >
                  <span class="material-symbols-outlined text-[18px]">more_vert</span>
                </button>
                <div
                  v-if="openMenuId === friend.id"
                  data-friend-menu
                  :ref="setStatusMenuRef"
                  :class="[
                    'absolute right-0 w-52 rounded-xl glass-modal bg-panel-strong/95 flex flex-col py-1.5 shadow-2xl overflow-x-hidden overflow-y-auto z-50 animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10',
                    statusMenuPlacement === 'top' ? 'bottom-full mb-2' : 'top-full mt-2'
                  ]"
                  :style="statusMenuMaxHeight ? { maxHeight: statusMenuMaxHeight } : undefined"
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
                    <span v-if="resolveFriendStatus(friend) === option.id" class="material-symbols-outlined text-[16px] ml-auto text-white/60">check</span>
                  </button>
                </div>
              </div>
              <button
                type="button"
                class="w-8 h-8 rounded-lg border border-white/10 text-white/50 hover:text-white hover:bg-white/10 transition-colors flex items-center justify-center"
                @click.stop="handleFriendDelete(friend)"
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

    <ManageMemberModal
      v-if="managingMember"
      :member="managingMember"
      :show-remove="false"
      @close="closeFriendManage"
      @save="handleFriendRename"
    />
  </div>
</template>

<script setup lang="ts">
// 好友列表视图：展示成员状态、邀请与终端操作入口。
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import { isTauri } from '@tauri-apps/api/core';

import AvatarBadge from '@/shared/components/AvatarBadge.vue';
import MemberStatusDots from './components/MemberStatusDots.vue';
import InviteMenu from './components/InviteMenu.vue';
import InviteAdminModal from './modals/InviteAdminModal.vue';
import InviteAssistantModal from './modals/InviteAssistantModal.vue';
import ManageMemberModal from './modals/ManageMemberModal.vue';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useChatStore } from '@/features/chat/chatStore';
import { useTerminalOrchestratorStore } from '@/stores/terminalOrchestratorStore';
import { useNavigationStore } from '@/stores/navigationStore';
import { useToastStore } from '@/stores/toastStore';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import { createFrontLogger } from '@/shared/monitoring/passiveMonitor';
import { saveChatCache } from './chatStorage';
import { useContactsStore } from './contactsStore';
import type { FriendEntry, Member, MemberStatus } from './types';
import { CURRENT_USER_ID } from './data';
import { useFriendInvites } from './useFriendInvites';
import { resolveMemberDisplayName } from '@/shared/utils/memberDisplay';

const { t } = useI18n();
const projectStore = useProjectStore();
const workspaceStore = useWorkspaceStore();
const { currentWorkspace } = storeToRefs(workspaceStore);
const navigationStore = useNavigationStore();
const { setActiveTab } = navigationStore;
const chatStore = useChatStore();
const { members } = storeToRefs(projectStore);
const { addMember, updateMember, removeMember } = projectStore;
const { deleteMemberConversations, ensureDirectMessage } = chatStore;
const terminalOrchestratorStore = useTerminalOrchestratorStore();
const { ensureMemberSession, stopMemberSession, openMemberTerminal } = terminalOrchestratorStore;
const toastStore = useToastStore();
const { pushToast } = toastStore;
const contactsStore = useContactsStore();
const { contacts } = storeToRefs(contactsStore);
const { upsertContact } = contactsStore;
const debugLog = createFrontLogger('friends');
const resolveFriendName = (friend: FriendEntry) => resolveMemberDisplayName(friend);

const openMenuId = ref<string | null>(null);
const scrollContainerRef = ref<HTMLElement | null>(null);
const statusMenuRef = ref<HTMLElement | null>(null);
const statusMenuPlacement = ref<'top' | 'bottom'>('bottom');
const statusMenuMaxHeight = ref('');
const managingFriend = ref<FriendEntry | null>(null);
const managingMember = ref<Member | null>(null);
const setStatusMenuRef = (element: HTMLElement | null) => {
  statusMenuRef.value = element;
};
const {
  showInviteMenu,
  activeModalType,
  toggleInviteMenu,
  handleInviteSelect,
  handleAdminInvite,
  handleInvite,
  syncDefaultChannelMembers
} = useFriendInvites();

const projectFriends = computed<FriendEntry[]>(() =>
  members.value
    .filter((member) => member.id !== CURRENT_USER_ID)
    .map((member) => ({
      id: member.id,
      name: member.name,
      avatar: member.avatar,
      roleType: member.roleType,
      status: member.status,
      manualStatus: member.manualStatus,
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
  { id: 'offline', labelKey: 'settings.statusOptions.offline', dotClass: 'bg-slate-500' }
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

const formatFriendRole = (roleType: FriendEntry['roleType']) => {
  if (roleType === 'assistant') {
    return t('members.roles.member');
  }
  return t(roleKeyForType(roleType));
};

const buildFriendMember = (friend: FriendEntry): Member => ({
  id: friend.id,
  name: friend.name,
  role: '',
  roleKey: roleKeyForType(friend.roleType),
  roleType: friend.roleType,
  avatar: friend.avatar,
  status: friend.status,
  terminalStatus: friend.terminalStatus,
  terminalType: friend.terminalType,
  terminalCommand: friend.terminalCommand,
  terminalPath: friend.terminalPath,
  manualStatus: friend.manualStatus
});

const openFriendManage = (friend: FriendEntry) => {
  closeStatusMenu();
  managingFriend.value = friend;
  if (friend.scope === 'project') {
    managingMember.value = members.value.find((member) => member.id === friend.id) ?? buildFriendMember(friend);
    return;
  }
  managingMember.value = buildFriendMember(friend);
};

const closeFriendManage = () => {
  managingFriend.value = null;
  managingMember.value = null;
};

const toggleStatusMenu = (friendId: string, event?: MouseEvent) => {
  if (openMenuId.value === friendId) {
    closeStatusMenu();
    return;
  }
  openMenuId.value = friendId;
  statusMenuPlacement.value = 'bottom';
  statusMenuMaxHeight.value = '';
  const anchor = event?.currentTarget as HTMLElement | null;
  void nextTick(() => updateStatusMenuPlacement(anchor));
};

const closeStatusMenu = () => {
  openMenuId.value = null;
  statusMenuPlacement.value = 'bottom';
  statusMenuMaxHeight.value = '';
};

const resolveFriendStatus = (friend: FriendEntry) => friend.manualStatus ?? friend.status;

const updateStatusMenuPlacement = (anchor?: HTMLElement | null) => {
  const menu = statusMenuRef.value;
  if (!menu) return;
  const anchorRect = anchor?.getBoundingClientRect() ?? menu.getBoundingClientRect();
  const containerRect =
    scrollContainerRef.value?.getBoundingClientRect() ?? {
      top: 0,
      bottom: window.innerHeight || document.documentElement.clientHeight
    };
  const spaceBelow = Math.max(0, containerRect.bottom - anchorRect.bottom);
  const spaceAbove = Math.max(0, anchorRect.top - containerRect.top);
  const menuHeight = menu.offsetHeight;
  const preferTop = spaceBelow < menuHeight && spaceAbove > spaceBelow;
  statusMenuPlacement.value = preferTop ? 'top' : 'bottom';
  const available = preferTop ? spaceAbove : spaceBelow;
  if (available > 0) {
    const safe = Math.max(0, Math.floor(available - 12));
    statusMenuMaxHeight.value = `${safe}px`;
  }
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
    await stopMemberSession(friend.id, { preserve: false, fireAndForget: true, deleteSessionMap: true });
  }
  await removeMember(friend.id);
  await syncDefaultChannelMembers();
};

const updateProjectFriendStatus = async (friend: FriendEntry, status: MemberStatus) => {
  const member = members.value.find((item) => item.id === friend.id);
  if (!member) return;
  if (hasTerminalConfig(member.terminalType, member.terminalCommand)) {
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
};

const updateGlobalFriendStatus = async (friend: FriendEntry, status: MemberStatus) => {
  const contact = contacts.value.find((item) => item.id === friend.id);
  if (!contact) return;
  await upsertContact({ ...contact, status });
};

const handleStatusChange = async (friend: FriendEntry, status: MemberStatus) => {
  if (resolveFriendStatus(friend) === status) {
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
  if (member.terminalStatus === 'connecting' && member.terminalType !== 'shell') {
    pushToast(t('terminal.statusOptions.connecting'), { tone: 'info' });
    return;
  }
  debugLog('open terminal from avatar', { memberId: member.id });
  await openMemberTerminal(member);
};

const handleFriendRename = async (id: string, nextName: string) => {
  const friend = managingFriend.value;
  if (!friend) return;
  const name = nextName.trim();
  if (friend.scope === 'project') {
    await updateMember(id, { name });
    const contact = contacts.value.find((item) => item.id === id);
    if (contact) {
      await upsertContact({ ...contact, name });
    }
    closeFriendManage();
    return;
  }
  const contact = contacts.value.find((item) => item.id === id);
  if (contact) {
    await upsertContact({ ...contact, name });
  }
  closeFriendManage();
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

<style scoped>
.friend-name-clamp {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
