<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="w-full max-w-[520px] bg-panel/90 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl flex flex-col overflow-hidden ring-1 ring-white/5">
      <div class="px-6 py-5 border-b border-white/5 flex items-center justify-between">
        <div class="flex flex-col">
          <h3 class="text-white font-semibold text-[17px] tracking-tight leading-tight">{{ title }}</h3>
          <p class="text-white/40 text-[12px] font-medium mt-0.5">{{ selectedCount }}</p>
        </div>
        <button type="button" @click="emit('close')" class="text-white/20 hover:text-white transition-colors rounded-lg p-1 hover:bg-white/5">
          <span class="material-symbols-outlined text-[20px]">close</span>
        </button>
      </div>

      <div class="px-6 py-4 border-b border-white/5">
        <div class="relative">
          <span class="material-symbols-outlined text-[18px] text-white/20 absolute left-3 top-2.5">search</span>
          <input
            v-model="query"
            type="text"
            :placeholder="t('friends.inviteModal.search')"
            class="w-full bg-surface border border-white/10 rounded-xl py-2.5 pl-10 pr-3 text-white placeholder-white/20 focus:outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all text-[14px]"
          />
        </div>
      </div>

      <div class="flex-1 overflow-y-auto px-4 py-4 space-y-2 custom-scrollbar">
        <div
          v-for="friend in filteredFriends"
          :key="friend.id"
          class="flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-white/5 transition-colors"
          @click="toggleFriend(friend.id)"
        >
          <label class="relative flex items-center">
            <input type="checkbox" class="sr-only" :checked="selectedIds.has(friend.id)" />
            <span
              :class="[
                'w-4 h-4 rounded-[4px] border-2 flex items-center justify-center transition-all',
                selectedIds.has(friend.id) ? 'bg-primary border-primary text-on-primary' : 'border-white/20 text-transparent'
              ]"
            >
              <span class="material-symbols-outlined text-[12px]">check</span>
            </span>
          </label>
          <AvatarBadge :avatar="friend.avatar" :label="friend.name" class="w-9 h-9 rounded-full shadow-md" />
          <div class="min-w-0 flex-1">
            <div class="text-[13px] font-semibold text-white truncate">{{ friend.name }}</div>
            <div class="text-[11px] text-white/40">{{ friend.roleType }}</div>
          </div>
          <div :class="['w-2.5 h-2.5 rounded-full border-2 border-panel', statusDotClass(friend.status)]"></div>
        </div>
        <div v-if="filteredFriends.length === 0" class="text-center text-white/30 text-sm py-8">
          {{ t('friends.empty') }}
        </div>
      </div>

      <div class="px-6 py-4 border-t border-white/5 flex items-center justify-between bg-gradient-to-t from-panel-strong/60 to-transparent">
        <span class="text-[12px] text-white/40">{{ selectedCount }}</span>
        <button
          type="button"
          :disabled="selectedIds.size === 0"
          @click="confirmInvite"
          :class="[
            'h-10 px-5 rounded-xl font-bold text-[13px] transition-all',
            selectedIds.size === 0
              ? 'bg-white/10 text-white/30 cursor-not-allowed'
              : 'bg-gradient-to-r from-primary to-primary-hover text-on-primary shadow-glow hover:brightness-110 active:scale-[0.98]'
          ]"
        >
          {{ actionLabel }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import AvatarBadge from '@/shared/components/AvatarBadge.vue';
import type { FriendEntry } from '../types';

const props = defineProps<{ friends: FriendEntry[]; title: string; actionLabel: string }>();
const emit = defineEmits<{ (e: 'close'): void; (e: 'invite', ids: string[]): void }>();

const { t } = useI18n();
const query = ref('');
const selectedIds = ref<Set<string>>(new Set());

const filteredFriends = computed(() => {
  const keyword = query.value.trim().toLowerCase();
  if (!keyword) {
    return props.friends;
  }
  return props.friends.filter((friend) => friend.name.toLowerCase().includes(keyword));
});

const selectedCount = computed(() => `${selectedIds.value.size}/${props.friends.length}`);

const statusDotClass = (status: FriendEntry['status']) => {
  if (status === 'online') return 'bg-green-500';
  if (status === 'working') return 'bg-amber-400';
  if (status === 'dnd') return 'bg-red-500';
  return 'bg-white/20';
};

const toggleFriend = (id: string) => {
  const next = new Set(selectedIds.value);
  if (next.has(id)) {
    next.delete(id);
  } else {
    next.add(id);
  }
  selectedIds.value = next;
};

const confirmInvite = () => {
  if (selectedIds.value.size === 0) return;
  emit('invite', Array.from(selectedIds.value));
};
</script>
