<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="w-full max-w-4xl max-h-[80vh] bg-panel-strong/95 backdrop-blur-2xl border border-white/10 rounded-2xl shadow-2xl flex flex-col overflow-hidden relative ring-1 ring-white/10">
      <button type="button" @click="emit('close')" class="absolute top-4 right-4 w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 text-white/50 hover:text-white flex items-center justify-center transition-colors z-20">
        <span class="material-symbols-outlined text-lg">close</span>
      </button>

      <div class="px-8 pt-8 pb-4 shrink-0 border-b border-white/5 bg-white/[0.02]">
        <div class="flex items-center gap-3 mb-6">
          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-primary/20 to-primary-hover/10 flex items-center justify-center text-primary ring-1 ring-primary/20 shadow-glow">
            <span class="material-symbols-outlined text-2xl">backpack</span>
          </div>
          <div>
            <h2 class="text-xl font-bold text-white tracking-tight">{{ t('skills.management.title') }}</h2>
            <p class="text-white/40 text-xs font-medium">{{ t('skills.management.subtitle', { channel: channelLabel }) }}</p>
          </div>
        </div>
        <div class="flex bg-panel/60 p-1 rounded-lg w-full max-w-md border border-white/5">
          <button
            type="button"
            @click="activeTab = 'current'"
            :class="[
              'flex-1 py-1.5 px-3 rounded-md text-sm font-medium transition-all',
              activeTab === 'current' ? 'bg-white/10 text-white shadow-sm ring-1 ring-white/5' : 'text-white/40 hover:text-white'
            ]"
          >
            {{ t('skills.management.tabs.current') }}
            <span :class="['ml-1.5 px-1.5 py-0.5 rounded text-[10px] font-bold', activeTab === 'current' ? 'bg-primary/20 text-primary' : 'bg-white/5 text-white/30']">{{ currentSkills.length }}</span>
          </button>
          <button
            type="button"
            @click="activeTab = 'library'"
            :class="[
              'flex-1 py-1.5 px-3 rounded-md text-sm font-medium transition-all',
              activeTab === 'library' ? 'bg-white/10 text-white shadow-sm ring-1 ring-white/5' : 'text-white/40 hover:text-white'
            ]"
          >
            {{ t('skills.management.tabs.library') }}
            <span :class="['ml-1.5 px-1.5 py-0.5 rounded text-[10px] font-bold', activeTab === 'library' ? 'bg-primary/20 text-primary' : 'bg-white/5 text-white/30']">{{ installedLibrarySkills.length }}</span>
          </button>
        </div>
      </div>

      <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar p-8">
        <div v-if="activeTab === 'current'" class="space-y-6 animate-in fade-in duration-200">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-bold text-white/60 uppercase tracking-wider">{{ t('skills.current.activeFolders') }}</h3>
            <button type="button" class="text-primary text-xs font-medium hover:text-primary-hover flex items-center gap-1 transition-colors">
              <span class="material-symbols-outlined text-sm">sync</span> {{ t('skills.current.syncAll') }}
            </button>
          </div>

          <div
            v-for="skill in currentSkills"
            :key="skill.name"
            class="p-4 rounded-xl bg-white/5 border border-white/5 hover:border-white/10 transition-all group hover:bg-white/[0.07]"
          >
            <div class="flex items-start justify-between">
              <div class="flex gap-4">
                <div :class="['w-12 h-12 rounded-lg flex items-center justify-center ring-1', skill.bg, skill.color, skill.ring]">
                  <span class="material-symbols-outlined text-2xl">{{ skill.icon }}</span>
                </div>
                <div>
                  <h4 class="text-white font-semibold text-base mb-1">{{ t(skill.nameKey) }}</h4>
                  <div class="flex items-center gap-2 text-xs text-white/40 mb-2">
                    <span class="bg-white/5 px-2 py-0.5 rounded text-white/60 font-medium">{{ skill.ver }}</span>
                    <span>•</span>
                    <span>{{ t('skills.current.updated') }}</span>
                  </div>
                  <div class="flex gap-2">
                    <span class="px-2 py-1 rounded bg-green-500/10 text-green-400 text-[10px] font-bold uppercase tracking-wide border border-green-500/10 flex items-center gap-1">
                      <span class="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse"></span> {{ t('skills.current.active') }}
                    </span>
                  </div>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <button type="button" @click="emit('configure')" class="w-8 h-8 rounded-lg hover:bg-white/10 text-white/40 hover:text-white flex items-center justify-center transition-colors">
                  <span class="material-symbols-outlined text-lg">settings</span>
                </button>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" checked class="sr-only peer" readonly />
                  <div class="w-11 h-6 bg-panel-strong/80 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-border/40 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary shadow-inner"></div>
                </label>
              </div>
            </div>
            <div v-if="skill.tags" class="mt-4 pt-4 border-t border-white/5 flex gap-2 overflow-x-auto pb-1 no-scrollbar">
              <span v-for="tag in skill.tags" :key="tag" class="text-xs px-2 py-1 rounded-md bg-panel/60 text-white/50 border border-white/5 whitespace-nowrap">{{ t(tag) }}</span>
            </div>
          </div>
        </div>

        <div v-else class="animate-in fade-in duration-200">
          <div class="relative mb-8">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <span class="material-symbols-outlined text-white/30 text-lg">search</span>
            </div>
            <input class="block w-full pl-10 pr-3 py-2.5 border border-white/10 rounded-xl leading-5 bg-white/5 text-white placeholder-white/30 focus:outline-none focus:bg-white/10 focus:ring-1 focus:ring-primary/50 focus:border-primary/50 sm:text-sm transition-all shadow-inner" :placeholder="t('skills.library.searchPlaceholder')" type="text" />
            <div class="absolute inset-y-0 right-0 pr-2 flex items-center">
              <kbd class="inline-flex items-center border border-white/10 rounded px-2 text-sm font-sans font-medium text-white/30">⌘K</kbd>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
            <div
            v-for="skill in installedLibrarySkills"
            :key="skill.id"
            class="p-5 rounded-2xl bg-white/5 border border-white/5 hover:border-white/10 transition-all group hover:bg-white/[0.07] hover:shadow-lg hover:shadow-black/20 flex flex-col h-full relative overflow-hidden"
            >
              <div :class="['absolute top-0 right-0 w-16 h-16 bg-gradient-to-br to-transparent rounded-bl-3xl -mr-4 -mt-4', skill.gradient]"></div>
              <div class="flex justify-between items-start mb-4 relative z-10">
                <div :class="['w-12 h-12 rounded-xl flex items-center justify-center ring-1 shadow-glow', skill.bg, skill.color, skill.ring]">
                  <span class="material-symbols-outlined text-2xl">{{ skill.icon }}</span>
                </div>
                <button
                  type="button"
                  class="w-8 h-8 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-300 flex items-center justify-center transition-colors"
                  @click="handleRemoveSkill(skill.id)"
                  :aria-label="t('common.remove')"
                >
                  <span class="material-symbols-outlined text-[18px]">delete</span>
                </button>
              </div>
              <h3 class="text-white font-semibold text-base mb-1 tracking-tight">{{ t(skill.nameKey) }}</h3>
              <p class="text-white/40 text-xs leading-relaxed mb-4 flex-grow">{{ t(skill.descKey) }}</p>
              <div class="flex items-center gap-2 mt-auto">
                <span class="text-[10px] font-mono text-white/30 bg-white/5 px-1.5 py-0.5 rounded border border-white/5">{{ skill.ver }}</span>
                <span :class="['text-[10px] font-medium', skill.color]">{{ t(skill.assetsKey) }}</span>
              </div>
            </div>

            <button class="p-5 rounded-2xl bg-white/[0.02] border border-dashed border-white/10 hover:border-primary/50 hover:bg-white/5 transition-all group flex flex-col items-center justify-center h-full min-h-[180px] text-center" type="button">
              <div class="w-12 h-12 rounded-full bg-white/5 group-hover:bg-primary/10 flex items-center justify-center text-white/30 group-hover:text-primary transition-colors mb-3">
                <span class="material-symbols-outlined text-2xl">add</span>
              </div>
              <span class="text-white/60 font-medium text-sm group-hover:text-white transition-colors">{{ t('skills.library.importTitle') }}</span>
              <span class="text-white/30 text-xs mt-1">{{ t('skills.library.importSubtitle') }}</span>
            </button>
          </div>

          <div class="mt-10 flex justify-center pb-2">
            <button
              class="flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-primary/10 to-blue-500/10 border border-primary/20 hover:border-primary/40 text-white font-semibold text-sm shadow-lg hover:scale-[1.02] active:scale-[0.98] transition-all"
              type="button"
              @click="handleBrowseStore"
            >
              <span class="material-symbols-outlined text-lg">storefront</span>
              {{ t('skills.library.browseShop') }}
            </button>
          </div>
        </div>
      </div>

      <div class="p-4 bg-panel/60 border-t border-white/5 flex justify-between items-center text-xs text-white/30 px-8 rounded-b-2xl">
        <div class="flex gap-4">
          <span class="hover:text-white/60 cursor-pointer transition-colors">{{ t('skills.footer.documentation') }}</span>
          <span class="hover:text-white/60 cursor-pointer transition-colors">{{ t('skills.footer.privacy') }}</span>
        </div>
        <button
          v-if="activeTab === 'current'"
          class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-white transition-colors border border-white/5"
          type="button"
        >
          <span class="material-symbols-outlined text-sm">add</span> {{ t('skills.footer.newFolder') }}
        </button>
        <div v-else class="flex gap-2">
          <span class="text-white/20">{{ t('skills.footer.lastSynced') }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useGlobalStore } from '@/features/global/globalStore';
import { createLibrarySkills } from '@/features/skills/skillLibrary';
import { useNavigationStore } from '@/stores/navigationStore';

const emit = defineEmits<{ (e: 'close'): void; (e: 'configure'): void }>();
const activeTab = ref<'current' | 'library'>('current');

const { t } = useI18n();

const workspaceStore = useWorkspaceStore();
const projectStore = useProjectStore();
const { defaultChannelName } = storeToRefs(workspaceStore);
const { currentSkills } = storeToRefs(projectStore);
const { installedSkillIds, removeSkill } = useGlobalStore();
const navigationStore = useNavigationStore();
const { setActiveTab, setSkillStoreTab } = navigationStore;
const channelLabel = computed(() => `#${defaultChannelName.value}`);

const librarySkills = ref(createLibrarySkills());
const installedLibrarySkills = computed(() =>
  librarySkills.value.filter((skill) => installedSkillIds.value.includes(skill.id))
);

const handleRemoveSkill = (id: number) => {
  void removeSkill(id);
};

const handleBrowseStore = () => {
  setSkillStoreTab('store');
  setActiveTab('store');
  emit('close');
};
</script>
