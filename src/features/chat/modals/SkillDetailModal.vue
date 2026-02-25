<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center p-6 bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="w-full max-w-2xl h-auto bg-panel-strong/90 backdrop-blur-2xl border border-white/10 rounded-2xl flex flex-col overflow-hidden relative shadow-2xl ring-1 ring-white/10">
      <button type="button" @click="emit('close')" class="absolute top-4 right-4 w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 text-white/50 hover:text-white flex items-center justify-center transition-colors z-20">
        <span class="material-symbols-outlined text-lg">close</span>
      </button>
      <div class="px-8 pt-8 pb-4 shrink-0 border-b border-white/5 bg-white/[0.02]">
        <div class="flex items-center gap-3">
          <button type="button" @click="emit('back')" class="w-8 h-8 rounded-lg bg-white/5 hover:bg-white/10 text-white/50 hover:text-white flex items-center justify-center transition-colors">
            <span class="material-symbols-outlined text-lg">arrow_back</span>
          </button>
          <div class="h-6 w-[1px] bg-white/10"></div>
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-400 ring-1 ring-blue-500/20">
              <span class="material-symbols-outlined text-lg">palette</span>
            </div>
            <h2 class="text-xl font-bold text-white tracking-tight">{{ t('skills.items.designSystemCore.name') }}</h2>
          </div>
        </div>
      </div>
      <div class="flex-1 overflow-y-auto custom-scrollbar p-8">
        <div class="space-y-8">
          <div>
            <h3 class="text-sm font-bold text-white/60 uppercase tracking-wider mb-4">{{ t('skills.detail.sourceConfig') }}</h3>
            <div class="p-6 rounded-2xl bg-white/5 border border-white/5">
              <div class="flex mb-6 bg-panel/60 p-1 rounded-xl border border-white/5">
                <button type="button" class="flex-1 py-2 px-3 rounded-lg bg-white/10 text-white text-sm font-medium shadow-sm ring-1 ring-white/5 transition-all flex items-center justify-center gap-2">
                  <span class="material-symbols-outlined text-base">code</span>
                  {{ t('skills.detail.source.github') }}
                </button>
                <button type="button" class="flex-1 py-2 px-3 rounded-lg text-white/40 hover:text-white text-sm font-medium transition-colors flex items-center justify-center gap-2">
                  <span class="material-symbols-outlined text-base">terminal</span>
                  {{ t('skills.detail.source.command') }}
                </button>
                <button type="button" class="flex-1 py-2 px-3 rounded-lg text-white/40 hover:text-white text-sm font-medium transition-colors flex items-center justify-center gap-2">
                  <span class="material-symbols-outlined text-base">folder_open</span>
                  {{ t('skills.detail.source.local') }}
                </button>
              </div>
              <div class="space-y-2">
                <label class="block text-xs font-medium text-white/70">{{ t('skills.detail.repoLabel') }}</label>
                <div class="relative group">
                  <span class="absolute left-3 top-1/2 -translate-y-1/2 text-white/30">
                    <span class="material-symbols-outlined text-lg">link</span>
                  </span>
                  <input
                    class="w-full bg-surface border border-white/10 rounded-xl py-2.5 pl-10 pr-10 text-white placeholder-white/20 focus:outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all text-sm"
                    :placeholder="t('skills.detail.repoPlaceholder')"
                    type="text"
                    value="https://github.com/acme-inc/design-system-core.git"
                  />
                  <div class="absolute right-2 top-1/2 -translate-y-1/2">
                    <button type="button" class="p-1 rounded-md hover:bg-white/10 text-white/30 hover:text-white transition-colors">
                      <span class="material-symbols-outlined text-sm">content_paste</span>
                    </button>
                  </div>
                </div>
                <p class="text-[11px] text-white/30 mt-1.5 pl-1">{{ t('skills.detail.repoHint') }}</p>
              </div>
            </div>
          </div>
          <div>
            <h3 class="text-sm font-bold text-white/60 uppercase tracking-wider mb-4">{{ t('skills.detail.syncPreferences') }}</h3>
            <div class="p-6 rounded-2xl bg-white/5 border border-white/5 space-y-6">
              <div class="flex items-center justify-between pb-6 border-b border-white/5">
                <div class="flex flex-col">
                  <span class="text-white font-medium text-sm">{{ t('skills.detail.autoSync') }}</span>
                  <span class="text-white/40 text-xs mt-0.5">{{ t('skills.detail.autoSyncDesc') }}</span>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input checked class="sr-only peer" type="checkbox" />
                  <div class="w-11 h-6 bg-panel-strong/80 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-border/40 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary shadow-inner transition-colors"></div>
                </label>
              </div>
              <div class="grid grid-cols-2 gap-6 items-center">
                <div class="flex flex-col">
                  <label class="text-white font-medium text-sm mb-1">{{ t('skills.detail.updateFrequency') }}</label>
                  <span class="text-white/40 text-xs">{{ t('skills.detail.updateFrequencyDesc') }}</span>
                </div>
                <div class="relative">
                  <select class="w-full appearance-none bg-surface border border-white/10 rounded-xl py-2.5 px-4 text-white text-sm cursor-pointer hover:bg-white/5 transition-colors focus:outline-none focus:border-primary/50">
                    <option>{{ t('skills.detail.frequency.every15') }}</option>
                    <option selected>{{ t('skills.detail.frequency.hour') }}</option>
                    <option>{{ t('skills.detail.frequency.daily') }}</option>
                    <option>{{ t('skills.detail.frequency.manual') }}</option>
                  </select>
                  <div class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-white/50">
                    <span class="material-symbols-outlined text-lg">expand_more</span>
                  </div>
                </div>
              </div>
              <div class="grid grid-cols-2 gap-6 items-center">
                <div class="flex flex-col">
                  <label class="text-white font-medium text-sm mb-1">{{ t('skills.detail.targetBranch') }}</label>
                  <span class="text-white/40 text-xs">{{ t('skills.detail.targetBranchDesc') }}</span>
                </div>
                <div class="relative">
                  <select class="w-full appearance-none bg-surface border border-white/10 rounded-xl py-2.5 px-4 text-white text-sm cursor-pointer hover:bg-white/5 transition-colors focus:outline-none focus:border-primary/50">
                    <option selected>main</option>
                    <option>develop</option>
                    <option>release/v2.4</option>
                  </select>
                  <div class="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-white/50">
                    <span class="material-symbols-outlined text-lg">call_split</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="flex items-center justify-end gap-3 pt-6 border-t border-white/5 mt-2">
          <button class="px-4 py-2 rounded-lg text-sm font-medium text-red-400 hover:text-red-300 hover:bg-red-500/10 transition-colors mr-auto" type="button">
            {{ t('skills.detail.deleteSkill') }}
          </button>
          <button type="button" @click="emit('back')" class="px-5 py-2.5 rounded-xl bg-white/5 hover:bg-white/10 text-white text-sm font-medium transition-colors border border-white/10">
            {{ t('skills.detail.cancel') }}
          </button>
          <button type="button" @click="emit('back')" class="px-5 py-2.5 rounded-xl bg-primary hover:bg-primary-hover text-on-primary font-bold text-sm shadow-glow transition-all active:scale-95">
            {{ t('skills.detail.saveChanges') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const emit = defineEmits<{ (e: 'close'): void; (e: 'back'): void }>();

const { t } = useI18n();
</script>
