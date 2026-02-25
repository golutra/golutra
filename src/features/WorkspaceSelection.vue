<template>
  <div ref="containerRef" class="flex-grow flex flex-col items-center justify-center p-6 w-full max-w-7xl mx-auto z-10 relative">
    <div class="fixed inset-0 overflow-hidden pointer-events-none z-0">
      <div class="absolute -top-[20%] left-1/2 -translate-x-1/2 w-[60%] h-[60%] bg-primary/5 rounded-full blur-[120px]"></div>
      <div class="absolute bottom-0 right-0 w-[40%] h-[40%] bg-blue-900/10 rounded-full blur-[100px]"></div>
    </div>

    <div class="w-full max-w-3xl mb-16 mt-12 md:mt-0 z-10">
      <button
        type="button"
        @click="emit('select-workspace')"
        class="group w-full glass-panel bg-panel/40 rounded-3xl p-16 flex flex-col items-center justify-center text-center relative overflow-hidden transition-all duration-300 hover:bg-panel/60 hover:shadow-[0_0_40px_rgb(var(--color-primary)_/_0.15)] hover:border-primary/20 border border-white/5"
      >
        <div class="absolute inset-0 bg-gradient-to-b from-white/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"></div>
        <div class="relative w-20 h-20 rounded-full bg-white/5 flex items-center justify-center mb-6 group-hover:scale-110 group-hover:bg-primary/20 transition-all duration-300 border border-white/10 group-hover:border-primary/40 shadow-lg group-hover:shadow-primary/20">
          <span class="material-symbols-outlined text-4xl text-gray-400 group-hover:text-primary transition-colors duration-300">add</span>
        </div>
        <h1 class="relative text-3xl font-bold text-white mb-3 tracking-tight">{{ t('workspace.createTitle') }}</h1>
        <p class="relative text-gray-400 font-medium text-lg group-hover:text-gray-300 transition-colors">{{ t('workspace.createSubtitle') }}</p>
      </button>
    </div>

    <div class="w-full max-w-6xl z-10">
      <div class="flex items-center space-x-4 mb-8 px-2">
        <h2 class="text-xs font-bold text-gray-500 tracking-[0.25em] uppercase">{{ t('workspace.recentTitle') }}</h2>
        <div class="h-px bg-gradient-to-r from-white/10 to-transparent flex-grow"></div>
        <div class="relative group">
          <button class="flex items-center space-x-2 text-xs font-bold text-gray-400 hover:text-white transition-colors py-1.5 px-3 rounded-lg hover:bg-white/5">
            <span class="tracking-wider uppercase">{{ t('workspace.more') }}</span>
            <span class="material-symbols-outlined text-base">expand_more</span>
          </button>

          <div class="absolute right-0 top-full mt-2 w-72 glass-modal bg-panel-strong/95 rounded-xl shadow-2xl flex flex-col overflow-hidden invisible group-focus-within:visible group-hover:visible opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 transition-all duration-200 transform origin-top-right z-50">
            <div class="p-3 border-b border-white/5">
              <div class="relative">
                  <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-gray-500 text-[18px]">search</span>
                <input
                  type="text"
                  class="w-full bg-surface/80 text-xs text-gray-300 placeholder-gray-500 rounded-lg py-2.5 pl-10 pr-3 border border-white/10 focus:border-primary/50 focus:outline-none"
                  :placeholder="t('workspace.searchPlaceholder')"
                />
              </div>
            </div>
            <div class="py-2">
              <button
                v-for="item in recentTeams"
                :key="item"
                class="w-full text-left px-4 py-2.5 hover:bg-white/5 flex items-center gap-3 transition-colors"
              >
                <div class="w-8 h-8 rounded-lg bg-white/10 flex items-center justify-center text-white/50">
                  <span class="material-symbols-outlined text-sm">group</span>
                </div>
                <span class="text-sm font-medium text-gray-300">{{ t(item) }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-8 pb-20">
        <div
          v-for="ws in workspaces"
          :key="ws.id"
          @click="handleSelectWorkspace"
          :class="[
            'glass-panel bg-panel/40 rounded-3xl p-8 flex flex-col justify-between h-72 cursor-pointer group relative transition-all hover:bg-panel/60 hover:shadow-[0_0_30px_rgb(var(--color-primary)_/_0.12)]',
            activeMenu === ws.id ? 'z-50 ring-1 ring-white/10' : 'z-0 hover:z-40'
          ]"
        >
          <div v-if="ws.active" class="absolute top-6 right-16 z-20">
            <span class="bg-primary/10 text-primary text-[10px] font-bold px-3 py-1.5 rounded-full tracking-wider border border-primary/20 shadow-[0_0_15px_rgb(var(--color-primary)_/_0.18)] backdrop-blur-md">{{ t('workspace.activeLabel') }}</span>
          </div>

          <div class="absolute top-4 right-4 z-30">
            <button
              type="button"
              @click.stop="toggleMenu(ws.id)"
              :class="[
                'p-2 rounded-lg transition-colors ring-1 ring-transparent',
                activeMenu === ws.id ? 'bg-white/10 text-white ring-white/10 shadow-lg' : 'hover:bg-white/10 text-gray-400 hover:text-white'
              ]"
            >
              <span class="material-symbols-outlined text-xl">more_horiz</span>
            </button>

            <div
              v-if="activeMenu === ws.id"
              class="absolute right-0 top-12 w-56 rounded-xl glass-modal bg-panel-strong/95 flex flex-col py-1.5 shadow-2xl overflow-hidden z-50 animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10"
              @click.stop
            >
              <button class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-gray-300 hover:bg-primary/10 hover:text-primary transition-colors flex items-center gap-3">
                <span class="material-symbols-outlined text-lg opacity-70">settings</span> {{ t('workspace.menu.settings') }}
              </button>
              <button class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-gray-300 hover:bg-primary/10 hover:text-primary transition-colors flex items-center gap-3">
                <span class="material-symbols-outlined text-lg opacity-70">person_add</span> {{ t('workspace.menu.invite') }}
              </button>
              <button class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-gray-300 hover:bg-primary/10 hover:text-primary transition-colors flex items-center gap-3">
                <span class="material-symbols-outlined text-lg opacity-70">content_copy</span> {{ t('workspace.menu.copyId') }}
              </button>
              <div class="h-px bg-white/10 my-1 mx-2"></div>
              <button class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-red-400 hover:bg-red-500/10 hover:text-red-300 transition-colors flex items-center gap-3">
                <span class="material-symbols-outlined text-lg opacity-70">logout</span> {{ t('workspace.menu.leave') }}
              </button>
            </div>
          </div>

          <div>
            <div :class="['w-14 h-14 rounded-2xl flex items-center justify-center mb-6 border transition-colors duration-300', ws.iconBg, ws.iconHoverBg, ws.borderColor, ws.groupHoverBorder]">
              <span :class="['material-symbols-outlined text-3xl transition-colors', ws.iconColor]">{{ ws.icon }}</span>
            </div>
            <h3 class="text-2xl font-bold text-white mb-3 group-hover:text-white/90 transition-colors tracking-tight">{{ t(ws.titleKey) }}</h3>
            <p class="text-sm text-gray-400 leading-relaxed font-medium">{{ t(ws.descKey) }}</p>
          </div>

          <div class="flex items-center justify-between mt-6 border-t border-white/5 pt-5">
            <div class="flex -space-x-3 overflow-hidden pl-1">
              <img
                v-for="(memberId, idx) in ws.members"
                :key="idx"
                :src="`https://picsum.photos/id/${memberId}/100/100`"
                class="inline-block h-8 w-8 rounded-full ring-2 ring-white/10"
                alt=""
              />
            </div>
            <span class="text-xs text-gray-500 font-bold">{{ t(ws.timeKey) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';

type Workspace = {
  id: string;
  titleKey: string;
  descKey: string;
  icon: string;
  iconColor: string;
  iconBg: string;
  borderColor: string;
  groupHoverBorder: string;
  iconHoverBg: string;
  active: boolean;
  members: number[];
  timeKey: string;
};

const emit = defineEmits<{ (e: 'select-workspace'): void }>();

const workspaces: Workspace[] = [
  {
    id: 'frontend',
    titleKey: 'workspace.list.frontend.title',
    descKey: 'workspace.list.frontend.desc',
    icon: 'layers',
    iconColor: 'text-primary',
    iconBg: 'bg-primary/10',
    borderColor: 'border-primary/20',
    groupHoverBorder: '',
    iconHoverBg: 'group-hover:bg-primary/20',
    active: true,
    members: [10, 12],
    timeKey: 'workspace.times.twoMinutes'
  },
  {
    id: 'mobile',
    titleKey: 'workspace.list.mobile.title',
    descKey: 'workspace.list.mobile.desc',
    icon: 'hub',
    iconColor: 'text-orange-400',
    iconBg: 'bg-orange-500/10',
    borderColor: 'border-orange-500/20',
    groupHoverBorder: 'group-hover:border-orange-500/40',
    iconHoverBg: '',
    active: false,
    members: [22, 32],
    timeKey: 'workspace.times.fourHours'
  },
  {
    id: 'infra',
    titleKey: 'workspace.list.infrastructure.title',
    descKey: 'workspace.list.infrastructure.desc',
    icon: 'dns',
    iconColor: 'text-purple-400',
    iconBg: 'bg-purple-500/10',
    borderColor: 'border-purple-500/20',
    groupHoverBorder: 'group-hover:border-purple-500/40',
    iconHoverBg: '',
    active: false,
    members: [45],
    timeKey: 'workspace.times.oneDay'
  }
];

const recentTeams = ['workspace.recent.designSystems', 'workspace.recent.marketingTeam', 'workspace.recent.backendCore'];
const activeMenu = ref<string | null>(null);
const containerRef = ref<HTMLElement | null>(null);

const { t } = useI18n();

const handleSelectWorkspace = () => {
  activeMenu.value = null;
  emit('select-workspace');
};

const toggleMenu = (id: string) => {
  activeMenu.value = activeMenu.value === id ? null : id;
};

const handleClickOutside = (event: MouseEvent) => {
  if (containerRef.value && !containerRef.value.contains(event.target as Node)) {
    activeMenu.value = null;
  }
};

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside);
});
</script>
