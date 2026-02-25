<template>
  <nav class="w-full md:w-[88px] md:h-full h-16 flex md:flex-col flex-row items-center md:py-6 py-2 md:gap-6 gap-2 bg-panel/50 glass-panel md:border-r border-t border-white/5 shrink-0 z-50 fixed md:static bottom-0 left-0 right-0">
    <div class="mb-2 relative group cursor-pointer hidden md:block">
      <img
        :src="PRIMARY_USER_AVATAR_URL"
        :alt="t('common.userAvatarAlt')"
        class="w-[52px] h-[52px] rounded-[18px] object-cover border-2 border-white/10 group-hover:border-primary/50 transition-colors"
      />
      <div class="absolute -top-1 -right-1 w-3.5 h-3.5 bg-red-500 border-[3px] border-panel rounded-full"></div>
    </div>

    <div class="w-8 h-[1px] bg-white/10 rounded-full hidden md:block"></div>

    <div class="flex md:flex-col flex-row gap-4 w-full md:w-full justify-around md:justify-start">
      <div v-for="item in navItems" :key="item.id" class="relative group flex justify-center w-full">
        <div
          v-if="activeTab === item.id"
          class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-8 bg-primary rounded-r-full shadow-[0_0_15px_rgb(var(--color-primary)_/_0.6)] hidden md:block"
        ></div>
        <button
          type="button"
          :title="t(item.tooltipKey)"
          @click="handleNavClick(item.id)"
          :class="[
            'w-10 h-10 md:w-12 md:h-12 flex items-center justify-center rounded-2xl transition-all duration-300 group-hover:scale-105',
            activeTab === item.id
              ? 'bg-gradient-to-br from-primary/80 to-primary-hover/80 text-on-primary shadow-glow'
              : 'bg-white/5 text-white/40 hover:bg-white/10 hover:text-white'
          ]"
        >
          <span class="material-symbols-outlined text-[24px]">{{ item.icon }}</span>
        </button>
      </div>

      <div class="relative group flex justify-center w-full md:hidden">
        <button
          type="button"
          :title="t('nav.settings')"
          @click="emitChange('settings')"
          :class="[
            'w-10 h-10 flex items-center justify-center rounded-2xl transition-all duration-300 group-hover:scale-105',
            activeTab === 'settings'
              ? 'bg-gradient-to-br from-primary/80 to-primary-hover/80 text-on-primary shadow-glow'
              : 'bg-white/5 text-white/40 hover:bg-white/10 hover:text-white'
          ]"
        >
          <span class="material-symbols-outlined text-[24px]">settings</span>
        </button>
      </div>
    </div>

    <div class="flex-1 hidden md:block"></div>

    <button
      type="button"
      @click="emitChange('settings')"
      :title="t('nav.settings')"
      :class="[
        'w-12 h-12 flex items-center justify-center rounded-2xl transition-all hidden md:flex',
        activeTab === 'settings' ? 'text-white bg-white/10' : 'text-white/40 hover:text-white hover:bg-white/5'
      ]"
    >
      <span class="material-symbols-outlined text-[26px]">settings</span>
    </button>
  </nav>
</template>

<script setup lang="ts">
import { toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import { PRIMARY_USER_AVATAR_URL } from '../constants/avatars';
type TabId = 'chat' | 'workspaces' | 'store' | 'plugins' | 'settings';

type NavItem = {
  id: TabId;
  icon: string;
  tooltipKey: string;
};

const props = defineProps<{ activeTab: TabId }>();
const emit = defineEmits<{ (e: 'change', tab: TabId): void }>();

const navItems: NavItem[] = [
  { id: 'chat', icon: 'chat_bubble', tooltipKey: 'nav.chat' },
  { id: 'workspaces', icon: 'folder_open', tooltipKey: 'nav.workspaces' },
  { id: 'store', icon: 'storefront', tooltipKey: 'nav.store' },
  { id: 'plugins', icon: 'extension', tooltipKey: 'nav.plugins' }
];

const { t } = useI18n();

const emitChange = (tab: TabId) => {
  emit('change', tab);
};

const handleNavClick = (id: TabId) => {
  // Preserve original behavior: the workspaces icon does not switch tabs.
  if (id === 'workspaces') return;
  emitChange(id);
};

const activeTab = toRef(props, 'activeTab');
</script>
