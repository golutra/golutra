<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="relative w-full max-w-[480px] bg-panel/90 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl flex flex-col overflow-hidden ring-1 ring-white/5">
      <div class="px-6 py-5 border-b border-white/5 flex items-center gap-4 bg-gradient-to-r from-white/[0.03] to-transparent">
        <div class="w-10 h-10 rounded-[12px] bg-red-500/10 text-red-400 flex items-center justify-center shadow-lg shadow-red-500/10 shrink-0 border border-red-500/10">
          <span class="material-symbols-outlined text-[20px]">admin_panel_settings</span>
        </div>
        <div class="flex-1">
          <h3 class="text-white font-semibold text-[17px] tracking-tight leading-tight">{{ t('invite.admin.title') }}</h3>
          <p class="text-white/40 text-[12px] font-medium mt-0.5">{{ t('invite.admin.subtitle') }}</p>
        </div>
        <button type="button" @click="emit('close')" class="text-white/20 hover:text-white transition-colors rounded-lg p-1 hover:bg-white/5">
          <span class="material-symbols-outlined text-[20px]">close</span>
        </button>
      </div>

      <div class="p-6 space-y-6 max-h-[70vh] overflow-y-auto custom-scrollbar">
        <div class="space-y-2.5">
          <div class="flex items-center justify-between">
            <label class="text-[11px] font-bold text-white/40 uppercase tracking-widest">{{ t('invite.admin.uniqueLink') }}</label>
            <span class="text-[11px] text-primary cursor-pointer hover:underline">{{ t('invite.admin.regenerate') }}</span>
          </div>
          <div class="flex gap-2 group focus-within:ring-2 ring-primary/20 rounded-xl transition-all">
            <div class="flex-1 bg-surface border border-white/10 rounded-xl px-3.5 py-2.5 flex items-center gap-2 transition-colors group-focus-within:border-primary/50 group-focus-within:bg-panel/70">
              <span class="material-symbols-outlined text-white/20 text-[18px]">link</span>
              <input class="bg-transparent border-none p-0 text-white/70 text-[13px] font-mono w-full focus:ring-0 truncate select-all" readonly value="https://sky.chat/invite/adm_9x82m..." />
            </div>
            <button type="button" class="px-4 bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl text-white/70 hover:text-white transition-all font-medium text-[13px] flex items-center gap-2 active:scale-95">
              <span class="material-symbols-outlined text-[18px]">content_copy</span>
            </button>
          </div>
        </div>

        <div class="space-y-2.5">
          <label class="text-[11px] font-bold text-white/40 uppercase tracking-widest">{{ t('invite.admin.userIdentifier') }}</label>
          <div class="relative group">
            <div class="absolute left-3.5 top-2.5 text-white/30 group-focus-within:text-primary transition-colors">
              <span class="material-symbols-outlined text-[20px]">alternate_email</span>
            </div>
            <input class="w-full bg-surface border border-white/10 rounded-xl py-2.5 pl-11 pr-4 text-white placeholder-white/20 focus:outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all text-[14px]" :placeholder="t('invite.admin.userPlaceholder')" type="text" />
          </div>
        </div>

        <div class="space-y-2.5">
          <label class="text-[11px] font-bold text-white/40 uppercase tracking-widest">{{ t('invite.admin.permissions') }}</label>
          <div class="bg-surface/50 border border-white/10 rounded-xl overflow-hidden">
            <label v-for="perm in permissions" :key="perm.title" class="flex items-start gap-3 p-3.5 hover:bg-white/5 cursor-pointer transition-colors group border-b border-white/5 last:border-0 relative">
              <div class="relative flex items-center mt-0.5">
                <input type="checkbox" :checked="perm.checked" class="peer sr-only" />
                <div class="w-4 h-4 border-2 border-white/20 rounded-[4px] peer-checked:bg-primary peer-checked:border-primary transition-all flex items-center justify-center">
                  <span class="material-symbols-outlined text-[12px] text-on-primary font-bold opacity-0 peer-checked:opacity-100">check</span>
                </div>
              </div>
              <div class="flex flex-col">
                <span class="text-[13px] font-medium text-white/90 group-hover:text-white transition-colors">{{ perm.title }}</span>
                <span class="text-[11px] text-white/30 mt-0.5 font-light">{{ perm.desc }}</span>
              </div>
            </label>
          </div>
        </div>
      </div>

      <div class="p-6 pt-2 bg-gradient-to-t from-panel-strong/80 to-transparent">
        <button type="button" class="w-full py-3 rounded-xl bg-gradient-to-r from-primary to-primary-hover hover:brightness-110 text-on-primary font-bold text-[14px] shadow-glow flex items-center justify-center gap-2 transition-all active:scale-[0.98] group">
          <span>{{ t('invite.admin.send') }}</span>
          <span class="material-symbols-outlined text-[18px] group-hover:translate-x-0.5 transition-transform">send</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

const emit = defineEmits<{ (e: 'close'): void }>();

const { t } = useI18n();

const permissions = computed(() => [
  {
    title: t('invite.admin.permissionsList.fullAccess.title'),
    desc: t('invite.admin.permissionsList.fullAccess.desc'),
    checked: true
  },
  {
    title: t('invite.admin.permissionsList.billing.title'),
    desc: t('invite.admin.permissionsList.billing.desc'),
    checked: false
  },
  {
    title: t('invite.admin.permissionsList.memberManagement.title'),
    desc: t('invite.admin.permissionsList.memberManagement.desc'),
    checked: true
  }
]);
</script>
