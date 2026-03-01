<template>
  <div class="px-6 pb-6 pt-0">
    <div class="flex items-center justify-between px-2">
      <div class="flex items-center gap-2">
        <span class="text-[11px] font-bold uppercase tracking-widest text-white/40">{{ t('friends.title') }}</span>
        <span class="text-[11px] text-white/30 font-medium">{{ friends.length }}</span>
      </div>
    </div>

    <div v-if="friends.length" class="mt-3 flex items-center gap-2 overflow-x-auto no-scrollbar px-2">
      <div
        v-for="friend in friends"
        :key="friend.id"
        class="flex items-center gap-2 px-3 py-2 rounded-xl bg-white/5 border border-white/10 shrink-0"
      >
        <AvatarBadge :avatar="friend.avatar" :label="friend.name" class="w-7 h-7 rounded-full" />
        <div class="text-xs text-white/80 font-medium truncate max-w-[120px]">{{ friend.name }}</div>
      </div>
    </div>
    <div v-else class="mt-3 px-2 text-[11px] text-white/30 font-medium">
      {{ t('friends.empty') }}
    </div>
  </div>
</template>

<script setup lang="ts">
// 好友快捷栏组件：展示常用好友头像入口。
import { toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import type { FriendEntry } from '../types';
import AvatarBadge from '@/shared/components/AvatarBadge.vue';

const props = defineProps<{ friends: FriendEntry[] }>();

const { t } = useI18n();

const friends = toRef(props, 'friends');
</script>
