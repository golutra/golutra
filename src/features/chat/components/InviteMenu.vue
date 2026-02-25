<template>
  <div class="fixed top-[74px] right-6 z-50 w-[260px] bg-panel/80 backdrop-blur-2xl border border-white/10 rounded-2xl shadow-2xl flex flex-col p-1.5 animate-in fade-in zoom-in-95 duration-200 origin-top-right ring-1 ring-white/5">
    <div class="px-3 py-2 mb-1">
      <h3 class="text-white font-semibold text-[13px] tracking-tight">{{ t('invite.menu.title') }}</h3>
      <p class="text-white/40 text-[11px] font-medium mt-0.5">{{ t('invite.menu.subtitle') }}</p>
    </div>
    <div class="h-[1px] bg-white/5 mx-2 mb-1.5"></div>
    <button
      type="button"
      @click="emit('select', 'admin')"
      class="flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-white/10 group transition-all duration-200 text-left w-full"
    >
      <div class="w-8 h-8 rounded-[10px] bg-red-500/10 text-red-400 flex items-center justify-center group-hover:bg-red-500 group-hover:text-on-primary group-hover:shadow-lg group-hover:shadow-red-500/20 transition-all duration-200">
        <span class="material-symbols-outlined text-[18px]">admin_panel_settings</span>
      </div>
      <div class="flex flex-col">
        <span class="text-white/90 text-[13px] font-medium group-hover:text-white tracking-tight">{{ t('invite.menu.admin') }}</span>
        <span class="text-white/40 text-[10px] group-hover:text-white/60">{{ t('invite.menu.adminDesc') }}</span>
      </div>
    </button>
    <button
      type="button"
      @click="emit('select', 'assistant')"
      class="flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-white/10 group transition-all duration-200 text-left w-full"
    >
      <div class="w-8 h-8 rounded-[10px] bg-yellow-500/10 text-yellow-400 flex items-center justify-center group-hover:bg-yellow-500 group-hover:text-on-primary group-hover:shadow-lg group-hover:shadow-yellow-500/20 transition-all duration-200">
        <span class="material-symbols-outlined text-[18px]">manage_accounts</span>
      </div>
      <div class="flex flex-col">
        <span class="text-white/90 text-[13px] font-medium group-hover:text-white tracking-tight">{{ t('invite.menu.assistant') }}</span>
        <span class="text-white/40 text-[10px] group-hover:text-white/60">{{ t('invite.menu.assistantDesc') }}</span>
      </div>
    </button>
    <button
      type="button"
      @click="emit('select', 'member')"
      class="flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-white/10 group transition-all duration-200 text-left w-full"
    >
      <div class="w-8 h-8 rounded-[10px] bg-primary/10 text-primary flex items-center justify-center group-hover:bg-primary group-hover:text-on-primary group-hover:shadow-lg group-hover:shadow-primary/20 transition-all duration-200">
        <span class="material-symbols-outlined text-[18px]">person</span>
      </div>
      <div class="flex flex-col">
        <span class="text-white/90 text-[13px] font-medium group-hover:text-white tracking-tight">{{ t('invite.menu.member') }}</span>
        <span class="text-white/40 text-[10px] group-hover:text-white/60">{{ t('invite.menu.memberDesc') }}</span>
      </div>
    </button>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const emit = defineEmits<{ (e: 'select', type: 'admin' | 'assistant' | 'member'): void }>();

const { t } = useI18n();
</script>
