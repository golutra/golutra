<template>
  <div class="flex flex-col h-full w-full">
    <header class="px-10 py-10 flex flex-col items-center justify-center shrink-0 z-10">
      <h1 class="text-3xl font-bold text-white mb-8 tracking-tight drop-shadow-lg">{{ t('marketplace.title') }}</h1>

      <div class="relative w-full max-w-xl group">
        <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
          <span class="material-symbols-outlined text-white/40 group-focus-within:text-primary transition-colors">search</span>
        </div>
        <input
          type="text"
          class="w-full bg-white/5 border border-white/10 rounded-2xl py-3.5 pl-12 pr-16 text-white placeholder-white/30 focus:bg-white/10 focus:border-primary/50 focus:ring-0 transition-all shadow-glass backdrop-blur-md text-[15px]"
          :placeholder="t('marketplace.searchPlaceholder')"
        />
        <div class="absolute inset-y-0 right-0 pr-3 flex items-center">
          <button class="bg-white/10 hover:bg-white/20 text-[11px] px-2 py-1 rounded-md text-white/60 font-medium transition-colors border border-white/5">CMD+K</button>
        </div>
      </div>

      <div class="flex items-center p-1 bg-panel/60 border border-white/10 rounded-full w-full max-w-[320px] mt-8 backdrop-blur-xl relative z-10 shadow-lg">
        <span
          class="absolute inset-y-1 left-1 w-[calc(50%-0.25rem)] rounded-full bg-gradient-to-br from-primary/25 via-white/[0.08] to-white/[0.03] border border-primary/20 shadow-[0_8px_20px_rgba(0,0,0,0.25)] backdrop-blur-md transition-transform duration-300 ease-out pointer-events-none"
          :class="activeTab === 'installed' ? 'translate-x-full' : 'translate-x-0'"
        ></span>
        <button
          type="button"
          @click="activeTab = 'store'"
          :class="[
            'flex-1 py-1.5 rounded-full font-bold text-[13px] transition-all flex items-center justify-center tracking-wide relative z-10',
            activeTab === 'store'
              ? 'text-white text-shadow'
              : 'text-white/60 hover:text-white'
          ]"
        >
          {{ t('marketplace.browseStore') }}
        </button>
        <button
          type="button"
          @click="activeTab = 'installed'"
          :class="[
            'flex-1 py-1.5 rounded-full font-bold text-[13px] transition-all flex items-center justify-center tracking-wide relative z-10',
            activeTab === 'installed'
              ? 'text-white text-shadow'
              : 'text-white/60 hover:text-white'
          ]"
        >
          {{ t('marketplace.myPlugins') }}
        </button>
      </div>

      <div class="flex items-center gap-2 mt-8 overflow-x-auto max-w-full pb-2 no-scrollbar">
        <button class="px-4 py-1.5 rounded-full bg-primary/20 text-primary border border-primary/20 text-xs font-bold whitespace-nowrap">{{ t('marketplace.categories.all') }}</button>
        <button
          v-for="cat in categories"
          :key="cat"
          class="px-4 py-1.5 rounded-full bg-white/5 hover:bg-white/10 text-white/70 hover:text-white border border-white/5 text-xs font-bold transition-colors whitespace-nowrap"
        >
          {{ t(cat) }}
        </button>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto px-10 pb-12">
      <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-6 max-w-[1600px] mx-auto">
        <div
          v-for="plugin in visiblePlugins"
          :key="plugin.id"
          class="glass-panel bg-panel-strong/60 rounded-3xl p-6 border border-white/5 hover:border-primary/30 transition-all duration-300 hover:shadow-glow hover:-translate-y-1 group flex flex-col h-full relative overflow-hidden"
        >
          <div class="absolute top-0 right-0 p-5 opacity-0 group-hover:opacity-100 transition-opacity duration-300">
            <span class="material-symbols-outlined text-white/20 hover:text-white cursor-pointer">more_horiz</span>
          </div>

          <div class="flex items-start justify-between mb-4">
            <div :class="['w-14 h-14 rounded-2xl bg-gradient-to-br border border-white/10 flex items-center justify-center shadow-lg text-white', plugin.bg]">
              <span class="material-symbols-outlined text-[28px]">{{ plugin.icon }}</span>
            </div>
            <div class="flex items-center gap-1 bg-white/5 px-2 py-1 rounded-lg border border-white/5 backdrop-blur-sm">
              <span class="material-symbols-outlined text-yellow-400 text-[14px] fill-current">star</span>
              <span class="text-[11px] font-bold text-white">{{ plugin.rating }}</span>
            </div>
          </div>

          <h3 class="text-white font-bold text-lg mb-1.5">{{ t(plugin.titleKey) }}</h3>
          <p class="text-white/50 text-[13px] mb-6 flex-1 leading-relaxed">{{ t(plugin.descKey) }}</p>

          <div
            v-if="plugin.installed && activeTab === 'installed'"
            class="flex items-center gap-2"
          >
            <div class="flex-1 py-2.5 rounded-xl bg-white/5 border border-white/10 text-white/70 font-bold text-[13px] flex items-center justify-center gap-2">
              {{ t('marketplace.installed') }}
            </div>
            <button
              type="button"
              class="flex-1 py-2.5 rounded-xl bg-red-500/10 hover:bg-red-500/20 border border-red-500/20 text-red-200 font-bold text-[13px] transition-all active:scale-95 flex items-center justify-center gap-2"
            >
              <span class="material-symbols-outlined text-[18px]">delete</span>
              {{ t('common.remove') }}
            </button>
          </div>
          <button
            v-else-if="plugin.installed"
            class="w-full py-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-white font-bold text-[13px] transition-all active:scale-95 flex items-center justify-center gap-2"
          >
            {{ t('marketplace.installed') }}
          </button>
          <button
          v-else
            class="w-full py-2.5 rounded-xl bg-primary hover:bg-primary-hover text-on-primary font-bold text-[13px] shadow-glow transition-all active:scale-95 flex items-center justify-center gap-2"
          >
            <span class="material-symbols-outlined text-[18px]">download</span>
            {{ t('marketplace.install') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';

const plugins = [
  { id: 1, titleKey: 'marketplace.plugins.github.title', descKey: 'marketplace.plugins.github.desc', icon: 'terminal', bg: 'from-gray-800 to-black', rating: '4.9' },
  { id: 2, titleKey: 'marketplace.plugins.spotify.title', descKey: 'marketplace.plugins.spotify.desc', icon: 'music_note', bg: 'from-green-600 to-green-900', rating: '4.7' },
  { id: 3, titleKey: 'marketplace.plugins.taskManager.title', descKey: 'marketplace.plugins.taskManager.desc', icon: 'check_circle', bg: 'from-blue-500 to-indigo-600', rating: '4.8' },
  { id: 4, titleKey: 'marketplace.plugins.calendar.title', descKey: 'marketplace.plugins.calendar.desc', icon: 'calendar_month', bg: 'from-orange-400 to-red-500', rating: '4.5', installed: true },
  { id: 5, titleKey: 'marketplace.plugins.aiAssistant.title', descKey: 'marketplace.plugins.aiAssistant.desc', icon: 'smart_toy', bg: 'from-purple-500 to-pink-500', rating: '4.2' },
  { id: 6, titleKey: 'marketplace.plugins.terminal.title', descKey: 'marketplace.plugins.terminal.desc', icon: 'code', bg: 'from-emerald-400 to-teal-600', rating: '5.0' },
  { id: 7, titleKey: 'marketplace.plugins.figma.title', descKey: 'marketplace.plugins.figma.desc', icon: 'design_services', bg: 'from-[#F24E1E] to-[#A259FF]', rating: '4.6' },
  { id: 8, titleKey: 'marketplace.plugins.quickNotes.title', descKey: 'marketplace.plugins.quickNotes.desc', icon: 'sticky_note_2', bg: 'from-yellow-400 to-orange-500', rating: '4.3' }
];

const activeTab = ref<'store' | 'installed'>('store');
const visiblePlugins = computed(() =>
  activeTab.value === 'installed' ? plugins.filter((plugin) => plugin.installed) : plugins
);

const categories = [
  'marketplace.categories.productivity',
  'marketplace.categories.development',
  'marketplace.categories.design',
  'marketplace.categories.communication',
  'marketplace.categories.music'
];

const { t } = useI18n();
</script>
