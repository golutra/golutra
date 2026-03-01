<template>
  <div
    :class="[
      'relative flex items-center gap-3 p-2 rounded-xl hover:bg-white/5 group transition-all duration-200',
      menuOpen ? 'z-40' : 'z-0 group-hover:z-30'
    ]"
  >
    <button
      type="button"
      @mousedown.prevent
      @click.stop="handleAvatarClick"
      :aria-disabled="!canOpenTerminal"
      :class="[
        'member-avatar-button relative transition-all duration-300 focus:outline-none focus-visible:outline-none focus:ring-0 focus-visible:ring-0',
        resolvedStatus === 'offline' ? 'grayscale opacity-60 group-hover:grayscale-0 group-hover:opacity-100' : '',
        canOpenTerminal ? 'cursor-pointer' : 'cursor-default'
      ]"
    >
      <AvatarBadge :avatar="member.avatar" :label="displayName" class="w-10 h-10 rounded-full shadow-md" />
      <MemberStatusDots
        class="absolute -bottom-0.5 -right-0.5"
        :status="resolvedStatus"
        :terminal-status="member.terminalStatus"
        :show-terminal-status="canOpenTerminal"
      />
    </button>
    <div class="flex items-center justify-between gap-2 min-w-0 flex-1">
      <div class="flex flex-col min-w-0">
        <div class="flex items-center gap-2">
          <span :class="[member.roleType === 'owner' ? 'text-primary font-bold' : 'text-white font-medium group-hover:text-primary', 'text-[13px] leading-none transition-colors']">{{ displayName }}</span>
          <span v-if="member.roleType === 'owner'" class="px-1.5 py-0.5 bg-yellow-500/20 text-yellow-500 text-[9px] rounded border border-yellow-500/20 font-bold uppercase tracking-wide">{{ t('members.roles.owner') }}</span>
          <span v-if="member.roleType === 'admin'" class="px-1.5 py-0.5 bg-primary/20 text-primary text-[9px] rounded border border-primary/20 font-bold uppercase tracking-wide">{{ t('members.roles.admin') }}</span>
        </div>
        <span v-if="displayRole" class="text-white/30 text-[10px] mt-1.5 font-medium truncate">{{ displayRole }}</span>
      </div>
      <div class="relative shrink-0">
        <button
          type="button"
          @click.stop="emit('toggle-menu', member)"
          data-member-menu-toggle
          :class="[
            'w-8 h-8 rounded-lg flex items-center justify-center transition-colors',
            menuOpen ? 'bg-white/10 text-white' : 'text-white/30 hover:text-white hover:bg-white/10'
          ]"
        >
          <span class="material-symbols-outlined text-[18px]">more_vert</span>
        </button>
        <div
          v-if="menuOpen"
          data-member-menu
          class="absolute right-0 top-full mt-2 w-52 rounded-xl glass-modal bg-panel-strong/95 flex flex-col py-1.5 shadow-2xl overflow-hidden z-50 animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10"
          @click.stop
        >
          <button
            v-if="canSendMessage"
            type="button"
            class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/15 hover:text-white hover:ring-1 hover:ring-white/10 transition-colors flex items-center gap-3"
            @click="emit('action', { action: 'send-message', member })"
          >
            <span class="material-symbols-outlined text-lg opacity-70">chat_bubble</span>
            {{ t('members.actions.sendMessage') }}
          </button>
          <button
            v-if="canMention"
            type="button"
            class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/15 hover:text-white hover:ring-1 hover:ring-white/10 transition-colors flex items-center gap-3"
            @click="emit('action', { action: 'mention', member })"
          >
            <span class="material-symbols-outlined text-lg opacity-70">alternate_email</span>
            @{{ member.name }}
          </button>
          <button
            v-if="canRename"
            type="button"
            class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/15 hover:text-white hover:ring-1 hover:ring-white/10 transition-colors flex items-center gap-3"
            @click="emit('action', { action: 'rename', member })"
          >
            <span class="material-symbols-outlined text-lg opacity-70">edit</span>
            {{ t('members.actions.rename') }}
          </button>
          <div class="h-px bg-white/10 my-1 mx-2"></div>
          <div class="px-4 py-1 text-[10px] font-semibold uppercase tracking-wider text-white/40">
            {{ t('settings.status') }}
          </div>
          <button
            v-for="option in menuStatusOptions"
            :key="option.id"
            type="button"
            class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/15 hover:text-white hover:ring-1 hover:ring-white/10 transition-colors flex items-center gap-3"
            @click="emit('action', { action: 'set-status', member, status: option.id })"
          >
            <span :class="['w-2.5 h-2.5 rounded-full', option.dotClass]"></span>
            {{ t(option.labelKey) }}
            <span v-if="resolvedStatus === option.id" class="material-symbols-outlined text-[16px] ml-auto text-white/60">check</span>
          </button>
          <template v-if="canRemove">
            <div class="h-px bg-white/10 my-1 mx-2"></div>
            <button
              type="button"
              class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-red-400 hover:bg-red-500/20 hover:text-red-400 hover:ring-1 hover:ring-red-400/40 transition-colors flex items-center gap-3"
              @click="emit('action', { action: 'remove', member })"
            >
              <span class="material-symbols-outlined text-lg opacity-70">person_remove</span>
              {{ t('members.manage.remove') }}
            </button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// 成员行组件：展示单个成员信息与操作入口。
import { computed, toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Member, MemberActionPayload, MemberStatus } from '../types';
import AvatarBadge from '@/shared/components/AvatarBadge.vue';
import MemberStatusDots from './MemberStatusDots.vue';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';
import { resolveMemberDisplayName } from '@/shared/utils/memberDisplay';

const props = defineProps<{ member: Member; menuOpen?: boolean; currentUserId?: string }>();
const emit = defineEmits<{
  (e: 'toggle-menu', member: Member): void;
  (e: 'action', payload: MemberActionPayload): void;
}>();
const member = toRef(props, 'member');
const menuOpen = computed(() => props.menuOpen ?? false);
const canRemove = computed(() => (props.currentUserId ? member.value.id !== props.currentUserId : true));
const canSendMessage = computed(() => (props.currentUserId ? member.value.id !== props.currentUserId : true));
const isCurrentUser = computed(() => (props.currentUserId ? member.value.id === props.currentUserId : false));
const canMention = computed(() => !isCurrentUser.value);
const canRename = computed(() => !isCurrentUser.value);
const canOpenTerminal = computed(() => hasTerminalConfig(member.value.terminalType, member.value.terminalCommand));
const displayName = computed(() => resolveMemberDisplayName(member.value));
const resolvedStatus = computed(() => member.value.manualStatus ?? member.value.status);

const { t } = useI18n();

const baseStatusOptions: Array<{ id: MemberStatus; labelKey: string; dotClass: string }> = [
  { id: 'online', labelKey: 'settings.statusOptions.online', dotClass: 'bg-green-500' },
  { id: 'working', labelKey: 'settings.statusOptions.working', dotClass: 'bg-amber-400' },
  { id: 'dnd', labelKey: 'settings.statusOptions.dnd', dotClass: 'bg-red-500' },
  { id: 'offline', labelKey: 'settings.statusOptions.offline', dotClass: 'bg-slate-500' }
];
const menuStatusOptions = computed(() =>
  canOpenTerminal.value ? baseStatusOptions.filter((option) => option.id !== 'working') : baseStatusOptions
);

const displayRole = computed(() => {
  if (isCurrentUser.value) {
    const match = baseStatusOptions.find((option) => option.id === member.value.status);
    return match ? t(match.labelKey) : '';
  }
  if (member.value.roleType === 'assistant') {
    return t('members.roles.member');
  }
  if (member.value.roleKey) {
    return t(member.value.roleKey);
  }
  return member.value.role;
});

const handleAvatarClick = (event?: MouseEvent) => {
  const target = event?.currentTarget as HTMLElement | null;
  target?.blur();
  if (!canOpenTerminal.value) {
    return;
  }
  void logDiagnosticsEvent('avatar-click', {
    memberId: member.value.id,
    terminalType: member.value.terminalType,
    terminalCommand: member.value.terminalCommand
  });
  emit('action', { action: 'open-terminal', member: member.value });
};
</script>

