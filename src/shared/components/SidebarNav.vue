<template>
  <nav class="w-full md:w-[88px] md:h-full h-16 flex md:flex-col flex-row items-center md:py-6 py-2 md:gap-6 gap-2 bg-panel/50 glass-panel md:border-r border-t md:border-t-0 border-white/5 shrink-0 z-50 fixed md:static bottom-0 left-0 right-0">
    <div ref="statusMenuRef" class="mb-2 relative hidden md:block">
      <button
        type="button"
        class="relative group"
        @click="toggleStatusMenu"
        :aria-expanded="statusMenuOpen"
        :title="t('settings.status')"
      >
        <AvatarBadge
          :avatar="accountAvatar"
          :label="t('common.userAvatarAlt')"
          class="w-[52px] h-[52px] rounded-[18px] border-2 border-white/10 group-hover:border-primary/50 transition-colors"
        />
        <div :class="['absolute -top-1 -right-1 w-3.5 h-3.5 border-[3px] border-panel rounded-full', statusDotClass]"></div>
      </button>
      <div
        v-if="statusMenuOpen"
        class="absolute left-full top-0 ml-3 w-52 glass-modal bg-panel-strong/95 rounded-xl shadow-2xl ring-1 ring-white/10 overflow-hidden z-50"
      >
        <div class="px-3 py-2 text-[10px] font-semibold uppercase tracking-widest text-white/40">
          {{ t('settings.status') }}
        </div>
        <div class="py-1">
          <button
            v-for="option in statusOptions"
            :key="option.id"
            type="button"
            class="w-full text-left px-3 py-2 text-xs font-semibold text-white/70 hover:text-white hover:bg-white/10 transition-colors flex items-center gap-2"
            @click="selectStatus(option.id)"
          >
            <span :class="['w-2.5 h-2.5 rounded-full', option.dotClass]"></span>
            {{ t(option.labelKey) }}
            <span v-if="accountStatus === option.id" class="material-symbols-outlined text-[16px] ml-auto text-white/50">check</span>
          </button>
        </div>
      </div>
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
            'relative w-10 h-10 md:w-12 md:h-12 flex items-center justify-center rounded-2xl transition-all duration-300 group-hover:scale-105',
            activeTab === item.id
              ? 'bg-gradient-to-br from-primary/80 to-primary-hover/80 text-on-primary shadow-glow'
              : 'bg-white/5 text-white/40 hover:bg-white/10 hover:text-white'
          ]"
        >
          <span class="material-symbols-outlined text-[24px]">{{ item.icon }}</span>
          <span
            v-if="shouldShowUnread(item.id)"
            class="absolute -top-1 -right-1 min-w-[18px] h-[18px] px-1.5 rounded-full bg-[rgb(231,93,88)] text-on-primary text-[10px] font-bold flex items-center justify-center border border-[rgb(199,75,70)] shadow-md"
          >
            {{ formatUnreadCount(totalUnreadCount) }}
          </span>
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
import { computed, onBeforeUnmount, onMounted, ref, toRef } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import AvatarBadge from './AvatarBadge.vue';
import { useSettingsStore, type AccountStatus } from '@/features/global/settingsStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { CURRENT_USER_ID } from '@/features/chat/data';
import { ensureAvatar } from '@/shared/utils/avatar';
import { useChatStore } from '@/features/chat/chatStore';
type TabId = 'chat' | 'friends' | 'workspaces' | 'store' | 'plugins' | 'settings';

type NavItem = {
  id: TabId;
  icon: string;
  tooltipKey: string;
};

const props = defineProps<{ activeTab: TabId }>();
const emit = defineEmits<{ (e: 'change', tab: TabId): void }>();

const navItems: NavItem[] = [
  { id: 'chat', icon: 'chat_bubble', tooltipKey: 'nav.chat' },
  { id: 'friends', icon: 'group', tooltipKey: 'nav.friends' },
  { id: 'workspaces', icon: 'folder_open', tooltipKey: 'nav.workspaces' },
  { id: 'store', icon: 'storefront', tooltipKey: 'nav.store' },
  { id: 'plugins', icon: 'extension', tooltipKey: 'nav.plugins' }
];

const { t } = useI18n();
const settingsStore = useSettingsStore();
const projectStore = useProjectStore();
const chatStore = useChatStore();
const { settings } = storeToRefs(settingsStore);
const { setAccountStatus } = settingsStore;
const { updateMember } = projectStore;
const { totalUnreadCount } = storeToRefs(chatStore);

const statusMenuRef = ref<HTMLElement | null>(null);
const statusMenuOpen = ref(false);
const accountStatus = computed(() => settings.value.account.status);
const accountAvatar = computed(() => ensureAvatar(settings.value.account.avatar));
const statusOptions: Array<{ id: AccountStatus; labelKey: string; dotClass: string }> = [
  { id: 'online', labelKey: 'settings.statusOptions.online', dotClass: 'bg-green-500' },
  { id: 'working', labelKey: 'settings.statusOptions.working', dotClass: 'bg-amber-400' },
  { id: 'dnd', labelKey: 'settings.statusOptions.dnd', dotClass: 'bg-red-500' },
  { id: 'offline', labelKey: 'settings.statusOptions.offline', dotClass: 'bg-white/30' }
];
const statusDotClass = computed(() => {
  if (accountStatus.value === 'online') return 'bg-green-500';
  if (accountStatus.value === 'working') return 'bg-amber-400';
  if (accountStatus.value === 'dnd') return 'bg-red-500';
  return 'bg-white/30';
});

const toggleStatusMenu = () => {
  statusMenuOpen.value = !statusMenuOpen.value;
};

const selectStatus = (status: AccountStatus) => {
  if (accountStatus.value !== status) {
    setAccountStatus(status);
    void updateMember(CURRENT_USER_ID, { status });
  }
  statusMenuOpen.value = false;
};

const handleClickOutside = (event: MouseEvent) => {
  if (statusMenuRef.value && !statusMenuRef.value.contains(event.target as Node)) {
    statusMenuOpen.value = false;
  }
};

const emitChange = (tab: TabId) => {
  emit('change', tab);
};

const handleNavClick = (id: TabId) => {
  emitChange(id);
};

const shouldShowUnread = (id: TabId) => {
  if (id !== 'chat') return false;
  return totalUnreadCount.value > 0;
};

const formatUnreadCount = (value: number) => (value > 99 ? '99+' : String(value));

const activeTab = toRef(props, 'activeTab');

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside);
});
</script>
