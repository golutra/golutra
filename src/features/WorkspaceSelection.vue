<template>
  <div ref="containerRef" class="flex-grow min-h-full flex flex-col items-center justify-start md:justify-center p-6 w-full max-w-7xl mx-auto z-10 relative">
    <div
      v-if="workspaceError"
      class="fixed top-6 right-6 z-50 max-w-sm w-full bg-red-500/15 border border-red-400/40 text-red-200 px-4 py-3 rounded-xl shadow-2xl backdrop-blur-md"
    >
      <div class="flex items-start gap-3">
        <span class="material-symbols-outlined text-lg text-red-300 mt-0.5">error</span>
        <div class="flex-1">
          <div class="text-sm font-semibold">{{ t('workspace.openErrorTitle') }}</div>
          <div class="selectable text-xs text-red-200/80 mt-1 break-words">{{ workspaceError }}</div>
        </div>
        <button
          type="button"
          class="text-red-200/60 hover:text-red-200 transition-colors"
          @click="clearWorkspaceError"
          aria-label="Dismiss"
        >
          <span class="material-symbols-outlined text-base">close</span>
        </button>
      </div>
    </div>
    <div class="fixed inset-0 overflow-hidden pointer-events-none z-0">
      <div class="absolute -top-[20%] left-1/2 -translate-x-1/2 w-[60%] h-[60%] bg-primary/5 rounded-full blur-[120px]"></div>
      <div class="absolute bottom-0 right-0 w-[40%] h-[40%] bg-blue-900/10 rounded-full blur-[100px]"></div>
    </div>

    <div class="w-full max-w-3xl mb-16 mt-12 md:mt-10 z-10">
      <button
        type="button"
        @click="handleOpenFolder"
        class="group w-full glass-panel bg-panel/40 rounded-3xl p-16 flex flex-col items-center justify-center text-center relative overflow-hidden transition-all duration-300 hover:bg-panel/60 hover:shadow-[0_0_40px_rgb(var(--color-primary)_/_0.15)] hover:border-primary/20 border border-white/5"
      >
        <div class="absolute inset-0 bg-gradient-to-b from-white/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"></div>
        <div class="relative w-20 h-20 rounded-full bg-white/5 flex items-center justify-center mb-6 group-hover:scale-110 group-hover:bg-primary/20 transition-all duration-300 border border-white/10 group-hover:border-primary/40 shadow-lg group-hover:shadow-primary/20">
          <span class="material-symbols-outlined text-4xl text-gray-400 group-hover:text-primary transition-colors duration-300">folder_open</span>
        </div>
        <h1 class="relative text-3xl font-bold text-white mb-3 tracking-tight">{{ t('workspace.openTitle') }}</h1>
        <p class="relative text-gray-400 font-medium text-lg group-hover:text-gray-300 transition-colors">{{ t('workspace.openSubtitle') }}</p>
      </button>
    </div>

    <div class="w-full max-w-6xl z-10">
      <div class="flex items-center space-x-4 mb-8 px-2">
        <h2 class="text-xs font-bold text-gray-500 tracking-[0.25em] uppercase">{{ t('workspace.recentTitle') }}</h2>
        <div class="h-px bg-gradient-to-r from-white/10 to-transparent flex-grow"></div>
        <div v-if="recentMore.length" class="relative group">
          <button class="flex items-center space-x-2 text-xs font-bold text-gray-400 hover:text-white transition-colors py-1.5 px-3 rounded-lg hover:bg-white/5">
            <span class="tracking-wider uppercase">{{ t('workspace.more') }}</span>
            <span class="material-symbols-outlined text-base">expand_more</span>
          </button>

          <div class="absolute right-0 top-full mt-2 w-72 glass-modal bg-panel-strong/95 rounded-xl shadow-2xl flex flex-col overflow-hidden invisible group-focus-within:visible group-hover:visible opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 transition-all duration-200 transform origin-top-right z-50">
            <div class="p-3 border-b border-white/5">
              <div class="relative">
                <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-gray-500 text-[18px]">search</span>
                <input
                  v-model="searchQuery"
                  type="text"
                  class="w-full bg-surface/80 text-xs text-gray-300 placeholder-gray-500 rounded-lg py-2.5 pl-10 pr-3 border border-white/10 focus:border-primary/50 focus:outline-none"
                  :placeholder="t('workspace.searchPlaceholder')"
                />
              </div>
            </div>
            <div class="py-2 max-h-72 overflow-y-auto custom-scrollbar">
              <button
                v-for="item in filteredMore"
                :key="item.id"
                class="w-full text-left px-4 py-2.5 hover:bg-white/5 flex items-start gap-3 transition-colors"
                @click="handleOpenRecent(item.path)"
              >
                <div class="w-8 h-8 rounded-lg bg-white/10 flex items-center justify-center text-white/50">
                  <span class="material-symbols-outlined text-sm">folder</span>
                </div>
                <div class="min-w-0 flex-1">
                  <span class="text-sm font-medium text-gray-300 block truncate">{{ item.name }}</span>
                  <span class="text-[11px] text-gray-500 block truncate">{{ formatWorkspacePath(item.path) }}</span>
                </div>
              </button>
              <p v-if="!filteredMore.length" class="px-4 py-3 text-xs text-white/40">
                {{ t('workspace.noResults') }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <div v-if="!recentPrimary.length" class="flex flex-col items-center justify-center text-center text-white/50 pb-20">
        <div class="w-14 h-14 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center mb-4">
          <span class="material-symbols-outlined text-[28px]">history</span>
        </div>
        <p class="text-sm font-semibold text-white/70">{{ t('workspace.emptyTitle') }}</p>
        <p class="text-xs text-white/40 mt-1">{{ t('workspace.emptySubtitle') }}</p>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-3 gap-8 pb-20">
        <button
          v-for="workspace in recentPrimary"
          :key="workspace.id"
          @click="handleOpenRecent(workspace.path)"
          class="glass-panel bg-panel/40 rounded-3xl p-7 flex flex-col justify-between h-64 text-left group relative transition-all hover:bg-panel/60 hover:shadow-[0_0_30px_rgb(var(--color-primary)_/_0.12)]"
        >
          <div>
            <div class="w-12 h-12 rounded-2xl bg-white/10 flex items-center justify-center border border-white/10 mb-6 group-hover:border-primary/30 transition-colors">
              <span class="material-symbols-outlined text-2xl text-white/60 group-hover:text-primary">folder</span>
            </div>
            <h3 class="text-xl font-bold text-white mb-2 group-hover:text-white/90 transition-colors tracking-tight truncate">
              {{ workspace.name }}
            </h3>
            <p class="text-xs text-white/40 leading-relaxed font-medium truncate">{{ formatWorkspacePath(workspace.path) }}</p>
          </div>
          <div class="flex items-center justify-between mt-6 border-t border-white/5 pt-4">
            <span class="text-[11px] text-white/30 font-semibold uppercase tracking-wider">
              {{ formatRelativeTime(workspace.lastOpenedAt) }}
            </span>
            <span class="text-[11px] text-white/40">{{ t('workspace.openAction') }}</span>
          </div>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import { useWorkspaceStore } from './workspace/workspaceStore';

const { t, locale } = useI18n();
const workspaceStore = useWorkspaceStore();
const { recentPrimary, recentMore, workspaceError } = storeToRefs(workspaceStore);
const { loadRecent, openWorkspaceDialog, openWorkspaceByPath, clearWorkspaceError } = workspaceStore;

const containerRef = ref<HTMLElement | null>(null);
const searchQuery = ref('');
const errorTimer = ref<number | null>(null);

const handleOpenFolder = async () => {
  await openWorkspaceDialog();
};

const handleOpenRecent = async (path: string) => {
  await openWorkspaceByPath(path);
};

const formatWorkspacePath = (path: string) => {
  if (!path) return path;
  if (!path.startsWith('\\\\?\\')) return path;
  const trimmed = path.slice(4);
  if (trimmed.toLowerCase().startsWith('unc\\')) {
    return `\\\\${trimmed.slice(4)}`;
  }
  return trimmed;
};

const formatRelativeTime = (timestamp: number) => {
  if (!timestamp) return '';
  const diff = Date.now() - timestamp;
  const rtf = new Intl.RelativeTimeFormat(locale.value, { numeric: 'auto' });
  const seconds = Math.round(diff / 1000);
  if (Math.abs(seconds) < 60) return rtf.format(-seconds, 'second');
  const minutes = Math.round(seconds / 60);
  if (Math.abs(minutes) < 60) return rtf.format(-minutes, 'minute');
  const hours = Math.round(minutes / 60);
  if (Math.abs(hours) < 24) return rtf.format(-hours, 'hour');
  const days = Math.round(hours / 24);
  if (Math.abs(days) < 30) return rtf.format(-days, 'day');
  const months = Math.round(days / 30);
  return rtf.format(-months, 'month');
};

const filteredMore = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) return recentMore.value;
  return recentMore.value.filter((item) => {
    if (item.name.toLowerCase().includes(query)) return true;
    return formatWorkspacePath(item.path).toLowerCase().includes(query);
  });
});

onMounted(() => {
  void loadRecent();
});

watch(
  () => workspaceError.value,
  (message) => {
    if (errorTimer.value !== null) {
      window.clearTimeout(errorTimer.value);
      errorTimer.value = null;
    }
    if (message) {
      errorTimer.value = window.setTimeout(() => {
        clearWorkspaceError();
        errorTimer.value = null;
      }, 5000);
    }
  }
);

onBeforeUnmount(() => {
  if (errorTimer.value !== null) {
    window.clearTimeout(errorTimer.value);
  }
});
</script>
