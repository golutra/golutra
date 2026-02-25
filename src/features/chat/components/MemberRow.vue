<template>
  <div class="flex items-center gap-3 p-2 rounded-xl hover:bg-white/5 group transition-all duration-200">
    <div :class="['relative transition-all duration-300', member.status === 'offline' ? 'grayscale opacity-60 group-hover:grayscale-0 group-hover:opacity-100' : '']">
      <div class="w-10 h-10 rounded-full bg-cover bg-center shadow-md" :style="{ backgroundImage: `url('${member.avatar}')` }"></div>
      <div v-if="member.roleType === 'owner'" class="absolute -bottom-0.5 -right-0.5 w-4 h-4 bg-green-500 border-2 border-panel rounded-full flex items-center justify-center text-panel"></div>
      <div v-else-if="member.roleType === 'admin'" class="absolute bottom-0 right-0 w-3 h-3 bg-[#2ecc71] border-2 border-panel rounded-full"></div>
      <div v-else-if="member.roleType === 'assistant'" class="absolute bottom-0 right-0 w-3 h-3 bg-[#e74c3c] border-2 border-panel rounded-full"></div>
      <div v-else class="absolute bottom-0 right-0 w-3 h-3 bg-gray-500 border-2 border-panel rounded-full"></div>
    </div>
    <div class="flex items-center justify-between gap-2 min-w-0 flex-1">
      <div class="flex flex-col min-w-0">
        <div class="flex items-center gap-2">
          <span :class="[member.roleType === 'owner' ? 'text-primary font-bold' : 'text-white font-medium group-hover:text-primary', 'text-[13px] leading-none transition-colors']">{{ member.name }}</span>
          <span v-if="member.roleType === 'owner'" class="px-1.5 py-0.5 bg-yellow-500/20 text-yellow-500 text-[9px] rounded border border-yellow-500/20 font-bold uppercase tracking-wide">{{ t('members.roles.owner') }}</span>
          <span v-if="member.roleType === 'admin'" class="px-1.5 py-0.5 bg-primary/20 text-primary text-[9px] rounded border border-primary/20 font-bold uppercase tracking-wide">{{ t('members.roles.admin') }}</span>
        </div>
        <span v-if="displayRole" class="text-white/30 text-[10px] mt-1.5 font-medium truncate">{{ displayRole }}</span>
      </div>
      <div class="relative shrink-0">
        <button
          type="button"
          @click.stop="emit('toggle-menu', member)"
          :class="[
            'w-8 h-8 rounded-lg flex items-center justify-center transition-colors',
            menuOpen ? 'bg-white/10 text-white' : 'text-white/30 hover:text-white hover:bg-white/10'
          ]"
        >
          <span class="material-symbols-outlined text-[18px]">more_vert</span>
        </button>
        <div
          v-if="menuOpen"
          class="absolute right-0 top-full mt-2 w-52 rounded-xl glass-modal bg-panel-strong/95 flex flex-col py-1.5 shadow-2xl overflow-hidden z-50 animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10"
          @click.stop
        >
          <button
            type="button"
            class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/5 hover:text-white transition-colors flex items-center gap-3"
            @click="emit('action', { action: 'send-message', member })"
          >
            <span class="material-symbols-outlined text-lg opacity-70">chat_bubble</span>
            {{ t('members.actions.sendMessage') }}
          </button>
          <button
            type="button"
            class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/5 hover:text-white transition-colors flex items-center gap-3"
            @click="emit('action', { action: 'mention', member })"
          >
            <span class="material-symbols-outlined text-lg opacity-70">alternate_email</span>
            {{ t('members.actions.mention') }}
          </button>
          <button
            type="button"
            class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/5 hover:text-white transition-colors flex items-center gap-3"
            @click="emit('action', { action: 'rename', member })"
          >
            <span class="material-symbols-outlined text-lg opacity-70">edit</span>
            {{ t('members.actions.rename') }}
          </button>
          <template v-if="canRemove">
            <div class="h-px bg-white/10 my-1 mx-2"></div>
            <button
              type="button"
              class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-red-400 hover:bg-red-500/10 hover:text-red-500 transition-colors flex items-center gap-3"
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
import { computed, toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Member, MemberAction } from '../types';

const props = defineProps<{ member: Member; menuOpen?: boolean; currentUserId?: string }>();
const emit = defineEmits<{
  (e: 'toggle-menu', member: Member): void;
  (e: 'action', payload: { action: MemberAction; member: Member }): void;
}>();
const member = toRef(props, 'member');
const menuOpen = computed(() => props.menuOpen ?? false);
const canRemove = computed(() => (props.currentUserId ? member.value.id !== props.currentUserId : true));

const { t } = useI18n();

const displayRole = computed(() => {
  if (member.value.roleKey) {
    return t(member.value.roleKey);
  }
  return member.value.role;
});
</script>
