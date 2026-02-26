<template>
  <div class="flex flex-col h-full w-full">
    <header class="px-10 py-10 flex flex-col items-center justify-center shrink-0 z-10">
      <h1 class="text-3xl font-bold text-white mb-8 tracking-tight drop-shadow-lg">{{ t('skillStore.title') }}</h1>

      <div class="relative w-full max-w-xl group">
        <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
          <span class="material-symbols-outlined text-white/40 group-focus-within:text-primary transition-colors">search</span>
        </div>
        <input
          type="text"
          class="w-full bg-white/5 border border-white/10 rounded-2xl py-3.5 pl-12 pr-16 text-white placeholder-white/30 focus:bg-white/10 focus:border-primary/50 focus:ring-0 transition-all shadow-glass backdrop-blur-md text-[15px]"
          :placeholder="t('skillStore.searchPlaceholder')"
        />
        <div class="absolute inset-y-0 right-0 pr-3 flex items-center">
          <button class="bg-white/10 hover:bg-white/20 text-[11px] px-2 py-1 rounded-md text-white/60 font-medium transition-colors border border-white/5">CMD+K</button>
        </div>
      </div>

      <div class="mt-8 bg-panel/60 backdrop-blur-md p-1.5 rounded-xl border border-white/5 inline-flex relative z-10 w-80 shadow-lg">
        <span
          class="absolute inset-y-1.5 left-1.5 w-[calc(50%-0.375rem)] rounded-lg bg-gradient-to-br from-primary/25 via-white/[0.08] to-white/[0.03] border border-primary/20 shadow-[0_8px_20px_rgba(0,0,0,0.25)] backdrop-blur-md transition-transform duration-300 ease-out pointer-events-none"
          :class="activeTab === 'installed' ? 'translate-x-full' : 'translate-x-0'"
        ></span>
        <button
          type="button"
          @click="setSkillStoreTab('store')"
          :class="[
            'flex-1 py-2 rounded-lg text-sm font-bold transition-all relative z-10',
            activeTab === 'store'
              ? 'text-white text-shadow'
              : 'text-white/60 hover:text-white'
          ]"
        >
          {{ t('skillStore.tabs.store') }}
        </button>
        <button
          type="button"
          @click="setSkillStoreTab('installed')"
          :class="[
            'flex-1 py-2 rounded-lg text-sm font-bold transition-all relative z-10',
            activeTab === 'installed'
              ? 'text-white text-shadow'
              : 'text-white/60 hover:text-white'
          ]"
        >
          {{ t('skillStore.tabs.installed') }}
        </button>
      </div>

      <div class="flex items-center gap-2 mt-6 overflow-x-auto max-w-full pb-2 no-scrollbar">
        <button class="px-5 py-2 rounded-full bg-primary/20 text-primary border border-primary/20 text-sm font-bold whitespace-nowrap">{{ t('skillStore.filters.all') }}</button>
        <button
          v-for="filter in filters"
          :key="filter"
          class="px-5 py-2 rounded-full bg-white/5 hover:bg-white/10 text-white/60 hover:text-white border border-white/5 text-sm font-bold transition-colors whitespace-nowrap"
        >
          {{ t(filter) }}
        </button>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto px-10 pb-12">
      <div v-if="activeTab === 'store'" class="grid grid-cols-[repeat(auto-fit,minmax(260px,1fr))] gap-6 max-w-7xl mx-auto">
        <div
          v-for="skill in storeSkills"
          :key="skill.id"
          class="glass-panel bg-panel-strong/60 rounded-3xl p-6 border border-white/5 hover:border-primary/30 transition-all duration-300 hover:shadow-glow hover:-translate-y-1 group flex flex-col h-full relative overflow-hidden"
        >
          <div class="flex items-start justify-between mb-4">
            <div :class="['w-14 h-14 rounded-2xl border border-white/5 flex items-center justify-center shadow-lg', skill.bg, skill.color]">
              <span class="material-symbols-outlined text-[32px]">{{ skill.icon }}</span>
            </div>
            <div class="flex items-center gap-1 bg-panel/60 px-2 py-1 rounded-lg border border-white/5 backdrop-blur-sm">
              <span class="material-symbols-outlined text-yellow-400 text-[14px] fill-current">star</span>
              <span class="text-[11px] font-bold text-white">{{ skill.rating }}</span>
            </div>
          </div>

          <h3 class="text-white font-bold text-lg mb-1.5">{{ t(skill.titleKey) }}</h3>
          <p class="text-white/50 text-[13px] mb-6 flex-1 leading-relaxed">{{ t(skill.descKey) }}</p>

          <div class="mb-4">
            <div class="flex items-center gap-2 bg-panel/60 rounded-xl p-1.5 border border-white/5 focus-within:border-primary/50 transition-colors group/input">
              <span class="material-symbols-outlined text-white/30 text-[18px] ml-2 group-focus-within/input:text-primary transition-colors">link</span>
              <input
                type="text"
                class="bg-transparent border-none text-[11px] text-white placeholder-white/20 w-full focus:ring-0 p-0 h-8"
                :placeholder="t('skillStore.syncPlaceholder')"
              />
              <button class="w-8 h-8 rounded-lg bg-white/5 hover:bg-primary/20 hover:text-primary text-white/40 flex items-center justify-center transition-all" :title="t('skillStore.syncNow')">
                <span class="material-symbols-outlined text-[16px]">sync</span>
              </button>
            </div>
          </div>

          <button
            v-if="skill.installed"
            class="w-full py-3 rounded-xl bg-white/5 border border-white/10 text-white/60 font-bold text-[13px] cursor-default flex items-center justify-center gap-2"
          >
            {{ t('skillStore.installed') }}
          </button>
          <button
            v-else
            class="w-full py-3 rounded-xl bg-primary hover:bg-primary-hover text-on-primary font-bold text-[13px] shadow-glow transition-all active:scale-95 flex items-center justify-center gap-2"
            @click="handleInstall(skill.id)"
          >
            <span class="material-symbols-outlined text-[18px]">add_to_drive</span>
            {{ t('skillStore.installFolder') }}
          </button>
        </div>
      </div>

      <div v-else class="max-w-[1600px] mx-auto">
        <div class="grid grid-cols-[repeat(auto-fit,minmax(260px,320px))] gap-6 justify-start">
          <div
            v-for="skill in installedSkills"
            :key="skill.id"
            class="glass-panel bg-panel-strong/60 rounded-3xl p-6 border border-white/5 hover:border-primary/30 transition-all duration-300 hover:shadow-glow hover:-translate-y-1 group flex flex-col h-full relative overflow-hidden"
          >
            <div :class="['absolute top-0 right-0 w-16 h-16 bg-gradient-to-br to-transparent rounded-bl-3xl -mr-4 -mt-4', skill.gradient]"></div>
            <div class="flex justify-between items-start mb-4 relative z-10">
              <div :class="['w-14 h-14 rounded-2xl border border-white/5 flex items-center justify-center shadow-lg', skill.bg, skill.color]">
                <span class="material-symbols-outlined text-[32px]">{{ skill.icon }}</span>
              </div>
            </div>
            <h3 class="text-white font-bold text-lg mb-1.5">{{ t(skill.nameKey) }}</h3>
            <p class="text-white/50 text-[13px] mb-6 flex-1 leading-relaxed">{{ t(skill.descKey) }}</p>
            <div class="flex items-center justify-between gap-3 mt-auto">
              <div class="flex items-center gap-2">
                <span class="text-[10px] font-mono text-white/30 bg-white/5 px-1.5 py-0.5 rounded border border-white/5">{{ skill.ver }}</span>
                <span :class="['text-[10px] font-medium', skill.color]">{{ t(skill.assetsKey) }}</span>
              </div>
              <button
                type="button"
                class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border border-red-500/20 bg-red-500/5 text-red-300 text-[11px] font-semibold hover:bg-red-500/10 hover:border-red-500/40 transition-colors"
                @click="handleRemove(skill.id)"
              >
                <span class="material-symbols-outlined text-[14px]">delete</span>
                {{ t('common.remove') }}
              </button>
            </div>
          </div>

          <button
            type="button"
            class="glass-panel bg-panel-strong/40 rounded-3xl p-4 border border-dashed border-white/10 hover:border-primary/40 transition-all duration-300 hover:shadow-glow hover:-translate-y-1 group flex flex-col items-center justify-center h-full min-h-[160px] text-center"
          >
            <div class="w-10 h-10 rounded-full bg-white/5 group-hover:bg-primary/10 flex items-center justify-center text-white/30 group-hover:text-primary transition-colors mb-2">
              <span class="material-symbols-outlined text-xl">add</span>
            </div>
            <span class="text-white/60 font-medium text-[13px] group-hover:text-white transition-colors">{{ t('skills.library.importTitle') }}</span>
            <span class="text-white/30 text-[11px] mt-1">{{ t('skills.library.importSubtitle') }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import { createLibrarySkills } from '@/features/skills/skillLibrary';
import { useGlobalStore } from '@/features/global/globalStore';
import { useNavigationStore } from '@/stores/navigationStore';

type StoreSkill = {
  id: number;
  titleKey: string;
  descKey: string;
  icon: string;
  color: string;
  bg: string;
  rating: string;
};

const skills: StoreSkill[] = [];

const navigationStore = useNavigationStore();
const { skillStoreTab } = storeToRefs(navigationStore);
const { setSkillStoreTab } = navigationStore;
const activeTab = skillStoreTab;
const { installedSkillIds, installSkill, removeSkill } = useGlobalStore();
const librarySkills = createLibrarySkills();
const installedSkills = computed(() =>
  librarySkills.filter((skill) => installedSkillIds.value.includes(skill.id))
);

const storeSkills = computed(() =>
  skills.map((skill) => ({
    ...skill,
    installed: installedSkillIds.value.includes(skill.id)
  }))
);

const filters = [
  'skillStore.filters.engineering',
  'skillStore.filters.design',
  'skillStore.filters.management',
  'skillStore.filters.marketing',
  'skillStore.filters.finance'
];

const { t } = useI18n();

const handleInstall = (id: number) => {
  void installSkill(id);
};

const handleRemove = (id: number) => {
  void removeSkill(id);
};
</script>
