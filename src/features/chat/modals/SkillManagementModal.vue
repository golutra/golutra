<!-- [2026-01-23 00:59] 目的: 提供技能管理弹窗入口，集中展示当前工作区技能与已安装库，避免管理入口分散; 边界: 仅负责 UI 与交互编排，状态读写与持久化交由各 store 处理以降低耦合; 设计: 采用双标签视图与统一文案键，确保信息结构稳定且可扩展。 -->
<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="skill-modal w-full max-w-4xl max-h-[80vh] backdrop-blur-2xl border border-white/10 rounded-2xl shadow-2xl flex flex-col overflow-hidden relative ring-1 ring-white/10">
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
            <span :class="['ml-1.5 px-1.5 py-0.5 rounded text-[10px] font-bold', activeTab === 'library' ? 'bg-primary/20 text-primary' : 'bg-white/5 text-white/30']">{{ libraryCount }}</span>
          </button>
        </div>
      </div>

      <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar p-8">
        <div v-if="activeTab === 'current'" class="space-y-6 animate-in fade-in duration-200">
            <div>
              <div class="flex items-center justify-between">
                <h3 class="text-sm font-bold text-white/60 uppercase tracking-wider">{{ t('skills.project.title') }}</h3>
                <button
                  type="button"
                  class="text-primary text-xs font-medium hover:text-primary-hover flex items-center gap-1 transition-colors"
                  @click="handleSyncProjectSkills"
                >
                  <span :class="['material-symbols-outlined text-sm', projectSyncSpinning ? 'spin-once' : '']">sync</span>
                  {{ t('skills.current.syncAll') }}
                </button>
              </div>
            <p v-if="workspaceReadOnly" class="mt-2 text-[11px] text-white/40">{{ t('skills.project.readOnlyHint') }}</p>

            <div v-if="loadingProjectSkills" class="mt-4 text-xs text-white/40">
              {{ t('skills.project.loading') }}
            </div>
            <div v-else-if="projectSkillLinksView.length === 0" class="mt-4 text-xs text-white/40">
              {{ t('skills.project.empty') }}
            </div>
            <div v-else class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
              <div
                v-for="link in projectSkillLinksView"
                :key="link.name"
                class="skill-card rounded-xl border border-white/5 p-4 flex items-start justify-between gap-3 min-w-0 relative overflow-hidden"
              >
                <div class="skill-card-accent"></div>
                <div class="min-w-0">
                  <h4 class="text-white font-semibold text-sm mb-1 truncate">{{ link.name }}</h4>
                  <p class="text-[11px] text-white/35 font-mono skill-path-multiline">{{ link.displayTargetPath }}</p>
                  <span class="inline-block mt-2 text-[10px] text-white/50">{{ t('skills.detail.source.local') }}</span>
                </div>
                <div class="flex flex-col items-center gap-2">
                  <button
                    v-if="canOpenSkillPath"
                    type="button"
                    class="w-8 h-8 rounded-lg bg-white/5 hover:bg-white/10 text-white/50 hover:text-white flex items-center justify-center transition-colors"
                    :aria-label="t('common.openFolder')"
                    @click="handleOpenSkillPath(link.targetPath)"
                  >
                    <span class="material-symbols-outlined text-[18px]">folder_open</span>
                  </button>
                  <button
                    type="button"
                    class="w-8 h-8 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-300 flex items-center justify-center transition-colors"
                    :disabled="unlinkingSkillName === link.name || workspaceReadOnly"
                    :aria-label="t('common.remove')"
                    @click="handleRemoveProjectSkill(link)"
                  >
                    <span class="material-symbols-outlined text-[18px]">delete</span>
                  </button>
                </div>
              </div>
            </div>
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
          <div class="flex items-center gap-3 mb-8">
            <div class="relative flex-1">
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <span class="material-symbols-outlined text-white/30 text-lg">search</span>
              </div>
              <input
                class="block w-full pl-10 pr-3 py-2.5 border border-white/10 rounded-xl leading-5 bg-white/5 text-white placeholder-white/30 focus:outline-none focus:bg-white/10 focus:ring-1 focus:ring-primary/50 focus:border-primary/50 sm:text-sm transition-all shadow-inner"
                :placeholder="t('skills.library.searchPlaceholder')"
                type="text"
              />
              <div class="absolute inset-y-0 right-0 pr-2 flex items-center">
                <kbd class="inline-flex items-center border border-white/10 rounded px-2 text-sm font-sans font-medium text-white/30">⌘K</kbd>
              </div>
            </div>
            <button
              type="button"
              class="flex items-center gap-2 px-3 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-white text-xs font-semibold border border-white/10 transition-colors"
              @click="handleLibraryRefresh"
            >
              <span :class="['material-symbols-outlined text-[18px]', libraryRefreshSpinning ? 'spin-once' : '']">sync</span>
              {{ t('skills.library.refresh') }}
            </button>
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

            <div
              v-for="folder in localSkillFolders"
              :key="folder.id"
              class="skill-card p-5 rounded-2xl border border-white/5 hover:border-white/10 transition-all group hover:shadow-lg hover:shadow-black/20 flex flex-col h-full relative overflow-hidden min-w-0"
            >
              <div class="skill-card-accent"></div>
              <div class="flex justify-between items-start mb-4 relative z-10">
                <div class="w-12 h-12 rounded-xl flex items-center justify-center ring-1 shadow-glow bg-white/5 text-primary border border-white/10">
                  <span class="material-symbols-outlined text-2xl">folder</span>
                </div>
                <div class="flex items-center gap-2">
                  <button
                    v-if="canOpenSkillPath"
                    type="button"
                    class="w-8 h-8 rounded-lg bg-white/5 hover:bg-white/10 text-white/50 hover:text-white flex items-center justify-center transition-colors"
                    :aria-label="t('common.openFolder')"
                    @click="handleOpenSkillPath(folder.path)"
                  >
                    <span class="material-symbols-outlined text-[18px]">folder_open</span>
                  </button>
                  <button
                    type="button"
                    class="w-8 h-8 rounded-lg bg-red-500/10 hover:bg-red-500/20 text-red-300 flex items-center justify-center transition-colors"
                    :aria-label="t('common.remove')"
                    :disabled="removingFolderId === folder.id"
                    @click="handleRemoveFolder(folder)"
                  >
                    <span class="material-symbols-outlined text-[18px]">delete</span>
                  </button>
                </div>
              </div>
              <h3 class="text-white font-semibold text-base mb-1 tracking-tight">{{ folder.name }}</h3>
              <p class="text-white/40 text-xs leading-relaxed mb-4 flex-grow font-mono skill-path-multiline">{{ folder.displayPath }}</p>
              <div class="flex items-center gap-2 mt-auto">
                <span class="text-[10px] font-medium text-white/50">{{ t('skills.detail.source.local') }}</span>
              </div>
            </div>

            <button
              type="button"
              @click="handleImportFolder"
              :disabled="importingFolder"
              :class="[
                'p-5 rounded-2xl bg-white/[0.02] border border-dashed border-white/10 transition-all group flex flex-col items-center justify-center h-full min-h-[180px] text-center',
                importingFolder ? 'opacity-60 cursor-not-allowed' : 'hover:border-primary/50 hover:bg-white/5'
              ]"
            >
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
          :disabled="workspaceReadOnly"
          :class="[
            'flex items-center gap-2 px-3 py-1.5 rounded-lg bg-white/5 text-white transition-colors border border-white/5',
            workspaceReadOnly ? 'opacity-60 cursor-not-allowed' : 'hover:bg-white/10'
          ]"
          type="button"
          @click="openProjectSkillPicker"
        >
          <span class="material-symbols-outlined text-sm">add</span> {{ t('skills.project.import') }}
        </button>
        <div v-else class="flex gap-2">
          <span class="text-white/20">{{ t('skills.footer.lastSynced') }}</span>
        </div>
      </div>

      <div
        v-if="showProjectSkillPicker"
        class="absolute inset-0 z-30 flex items-center justify-center bg-black/60 backdrop-blur-sm"
        @click.self="closeProjectSkillPicker"
      >
        <div class="w-full max-w-2xl mx-6 bg-panel/90 border border-white/10 rounded-2xl shadow-2xl overflow-hidden">
          <div class="flex items-center justify-between px-6 py-4 border-b border-white/10 bg-white/[0.03]">
            <div>
              <h3 class="text-sm font-bold text-white/80 uppercase tracking-wider">{{ t('skills.project.import') }}</h3>
              <p class="text-xs text-white/40 mt-1">{{ t('skills.project.pickerSubtitle') }}</p>
            </div>
            <button
              type="button"
              class="w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 text-white/60 hover:text-white flex items-center justify-center transition-colors"
              @click="closeProjectSkillPicker"
            >
              <span class="material-symbols-outlined text-lg">close</span>
            </button>
          </div>

          <div class="px-6 py-5">
            <div class="relative">
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <span class="material-symbols-outlined text-white/40 text-lg">search</span>
              </div>
              <input
                v-model="projectSkillQuery"
                class="block w-full pl-10 pr-3 py-3 border border-white/15 rounded-xl leading-5 bg-white/5 text-white placeholder-white/30 focus:outline-none focus:bg-white/10 focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-sm transition-all shadow-inner"
                :placeholder="t('skills.project.searchPlaceholder')"
                type="text"
              />
            </div>

            <div class="mt-4 max-h-[360px] overflow-y-auto custom-scrollbar space-y-2 pr-1">
              <div v-if="availableProjectSkills.length === 0" class="text-sm text-white/40 text-center py-8">
                {{ t('skills.project.emptyLibrary') }}
              </div>
              <div
                v-else-if="filteredProjectSkills.length === 0"
                class="text-sm text-white/40 text-center py-8"
              >
                {{ t('skills.project.emptySearch') }}
              </div>
              <div
                v-for="folder in filteredProjectSkills"
                :key="folder.id"
                class="flex items-center justify-between gap-3 rounded-xl bg-white/5 border border-white/10 px-4 py-3 hover:border-white/20 transition-colors"
              >
                <div class="min-w-0">
                  <div class="text-sm font-semibold text-white">{{ folder.name }}</div>
                  <div class="text-[11px] text-white/35 font-mono skill-path-multiline">{{ folder.displayPath }}</div>
                </div>
                <button
                  type="button"
                  class="px-3 py-1.5 rounded-md bg-primary/20 text-primary text-[11px] font-semibold hover:bg-primary/30 transition-colors"
                  :disabled="linkingSkillId === folder.id"
                  @click="handleLinkProjectSkill(folder)"
                >
                  {{ t('skills.project.linkAction') }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// 技能管理弹窗：维护技能启用状态与展示详情入口。
import { computed, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useI18n } from 'vue-i18n';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useGlobalStore } from '@/features/global/globalStore';
import { createLibrarySkills } from '@/features/skills/skillLibrary';
import { formatSkillPath } from '@/features/skills/skillPath';
import { useSpinOnce } from '@/shared/animations/useSpinOnce';
import {
  importSkillFolder,
  linkProjectSkill,
  listProjectSkillLinks,
  openSkillFolder,
  removeSkillFolder,
  unlinkProjectSkill
} from '@/features/skills/skillsBridge';
import { useNavigationStore } from '@/stores/navigationStore';
import { ask } from '@tauri-apps/plugin-dialog';
import { isTauri } from '@tauri-apps/api/core';
import type { ProjectSkillLink } from '@/features/skills/skillsBridge';

const emit = defineEmits<{ (e: 'close'): void; (e: 'configure'): void }>();
const activeTab = ref<'current' | 'library'>('current');

const { t } = useI18n();

const workspaceStore = useWorkspaceStore();
const { currentWorkspace, workspaceReadOnly } = storeToRefs(workspaceStore);
const projectStore = useProjectStore();
const { defaultChannelName } = storeToRefs(workspaceStore);
const { currentSkills } = storeToRefs(projectStore);
const globalStore = useGlobalStore();
const { installedSkillIds, importedSkillFolders } = storeToRefs(globalStore);
const { removeSkill, addImportedSkillFolder, removeImportedSkillFolder, refreshGlobalData } = globalStore;
const navigationStore = useNavigationStore();
const { setActiveTab, setSkillStoreTab } = navigationStore;
const channelLabel = computed(() => `#${defaultChannelName.value}`);
const canOpenSkillPath = isTauri();

const librarySkills = ref(createLibrarySkills());
const installedLibrarySkills = computed(() =>
  librarySkills.value.filter((skill) => installedSkillIds.value.includes(skill.id))
);
const localSkillFolders = computed(() =>
  importedSkillFolders.value.map((folder) => ({
    ...folder,
    displayPath: formatSkillPath(folder.path)
  }))
);
const libraryCount = computed(() => installedLibrarySkills.value.length + localSkillFolders.value.length);

const projectSkillLinks = ref<ProjectSkillLink[]>([]);
const loadingProjectSkills = ref(false);
const showProjectSkillPicker = ref(false);
const linkingSkillId = ref<string | null>(null);
const unlinkingSkillName = ref<string | null>(null);
const projectSkillQuery = ref('');
const { spinning: projectSyncSpinning, triggerSpin: triggerProjectSyncSpin } = useSpinOnce();
const { spinning: libraryRefreshSpinning, triggerSpin: triggerLibraryRefreshSpin } = useSpinOnce();

const projectSkillLinksView = computed(() =>
  projectSkillLinks.value.map((link) => ({
    ...link,
    displayTargetPath: formatSkillPath(link.targetPath)
  }))
);

const availableProjectSkills = computed(() =>
  localSkillFolders.value.filter(
    (folder) => !projectSkillLinks.value.some((link) => link.targetPath === folder.path)
  )
);

const filteredProjectSkills = computed(() => {
  const query = projectSkillQuery.value.trim().toLowerCase();
  if (!query) {
    return availableProjectSkills.value;
  }
  return availableProjectSkills.value.filter((folder) => {
    const name = folder.name.toLowerCase();
    const path = folder.displayPath.toLowerCase();
    return name.includes(query) || path.includes(query);
  });
});

const handleRemoveSkill = (id: number) => {
  void removeSkill(id);
};

const handleBrowseStore = () => {
  setSkillStoreTab('store');
  setActiveTab('store');
  emit('close');
};

const handleOpenSkillPath = async (path: string) => {
  if (!canOpenSkillPath || !path) {
    return;
  }
  try {
    await openSkillFolder(path);
  } catch (error) {
    console.error('Failed to open skill path.', error);
  }
};

const loadProjectSkillLinks = async () => {
  const workspace = currentWorkspace.value;
  if (!workspace || !isTauri()) {
    projectSkillLinks.value = [];
    return;
  }
  if (loadingProjectSkills.value) {
    return;
  }
  loadingProjectSkills.value = true;
  try {
    projectSkillLinks.value = await listProjectSkillLinks(workspace.path);
  } catch (error) {
    console.error('Failed to load project skill links.', error);
  } finally {
    loadingProjectSkills.value = false;
  }
};

const importingFolder = ref(false);
const removingFolderId = ref<string | null>(null);

const confirmRemoveFolder = async (name: string) => {
  const message = t('skills.library.removeConfirmMessage', { name });
  const title = t('skills.library.removeConfirmTitle');
  if (isTauri()) {
    return ask(message, {
      title,
      kind: 'warning',
      okLabel: t('skills.library.removeConfirmOk'),
      cancelLabel: t('skills.library.removeConfirmCancel')
    });
  }
  return window.confirm(`${title}\n${message}`);
};

const confirmRemoveProjectSkill = async (name: string) => {
  const message = t('skills.project.removeConfirmMessage', { name });
  const title = t('skills.project.removeConfirmTitle');
  if (isTauri()) {
    return ask(message, {
      title,
      kind: 'warning',
      okLabel: t('skills.project.removeConfirmOk'),
      cancelLabel: t('skills.project.removeConfirmCancel')
    });
  }
  return window.confirm(`${title}\n${message}`);
};

const handleImportFolder = async () => {
  if (importingFolder.value) return;
  importingFolder.value = true;
  try {
    const result = await importSkillFolder();
    if (!result) {
      return;
    }
    await addImportedSkillFolder({
      name: result.folderName,
      path: result.destPath
    });
    console.info('Imported skill folder.', result);
  } catch (error) {
    console.error('Failed to import skill folder.', error);
  } finally {
    importingFolder.value = false;
  }
};

const handleLinkProjectSkill = async (folder: { id: string; path: string }) => {
  const workspace = currentWorkspace.value;
  if (!workspace || workspaceReadOnly.value) {
    return;
  }
  if (linkingSkillId.value) {
    return;
  }
  linkingSkillId.value = folder.id;
  try {
    const link = await linkProjectSkill(workspace.path, folder.path);
    projectSkillLinks.value = [...projectSkillLinks.value, link];
    closeProjectSkillPicker();
  } catch (error) {
    console.error('Failed to link project skill.', error);
  } finally {
    linkingSkillId.value = null;
  }
};

const handleSyncProjectSkills = () => {
  triggerProjectSyncSpin();
  void loadProjectSkillLinks();
};

const handleLibraryRefresh = () => {
  triggerLibraryRefreshSpin();
  void refreshGlobalData();
};

const handleRemoveFolder = async (folder: { id: string; name: string; path: string }) => {
  if (removingFolderId.value) return;
  const confirmed = await confirmRemoveFolder(folder.name);
  if (!confirmed) return;
  removingFolderId.value = folder.id;
  try {
    await removeSkillFolder(folder.path);
    await removeImportedSkillFolder(folder.id);
  } catch (error) {
    console.error('Failed to remove skill folder.', error);
  } finally {
    removingFolderId.value = null;
  }
};

const handleRemoveProjectSkill = async (link: { name: string }) => {
  const workspace = currentWorkspace.value;
  if (!workspace || workspaceReadOnly.value) {
    return;
  }
  if (unlinkingSkillName.value) {
    return;
  }
  const confirmed = await confirmRemoveProjectSkill(link.name);
  if (!confirmed) {
    return;
  }
  unlinkingSkillName.value = link.name;
  try {
    await unlinkProjectSkill(workspace.path, link.name);
    projectSkillLinks.value = projectSkillLinks.value.filter((item) => item.name !== link.name);
  } catch (error) {
    console.error('Failed to unlink project skill.', error);
  } finally {
    unlinkingSkillName.value = null;
  }
};

watch(activeTab, (nextTab) => {
  if (nextTab !== 'current') {
    closeProjectSkillPicker();
  }
});

watch(
  () => currentWorkspace.value?.path,
  () => {
    void loadProjectSkillLinks();
  }
);

onMounted(() => {
  void loadProjectSkillLinks();
});

const openProjectSkillPicker = () => {
  showProjectSkillPicker.value = true;
  projectSkillQuery.value = '';
};

const closeProjectSkillPicker = () => {
  showProjectSkillPicker.value = false;
  projectSkillQuery.value = '';
};
</script>

<style scoped>
.skill-path-multiline {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 3;
  line-clamp: 3;
  overflow: hidden;
  white-space: normal;
  overflow-wrap: anywhere;
}
</style>
