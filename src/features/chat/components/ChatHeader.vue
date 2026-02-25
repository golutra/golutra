<template>
  <header class="h-[72px] px-8 flex items-center justify-between border-b border-white/5 bg-glass-header glass-panel sticky top-0 z-10 backdrop-blur-md">
    <button
      type="button"
      @click="emit('open-roadmap')"
      class="w-10 h-10 rounded-xl bg-gradient-to-br from-primary/60 to-primary-hover/50 backdrop-blur-md border border-white/20 shadow-lg text-on-primary flex items-center justify-center transition-all group active:scale-95 hover:shadow-glow"
      :title="t('chat.header.todo')"
    >
      <span class="material-symbols-outlined text-[22px] drop-shadow-sm group-hover:rotate-12 transition-transform">checklist</span>
    </button>
    <div class="flex flex-col items-center text-center">
      <h1 class="text-white font-semibold text-[17px] tracking-tight leading-tight mb-0.5">{{ headerTitle }}</h1>
      <p class="text-white/40 text-[11px] font-medium tracking-wide truncate max-w-xs">{{ headerDescription }}</p>
    </div>
    <div class="flex items-center gap-2">
      <button
        v-if="memberCount !== undefined"
        type="button"
        class="xl:hidden inline-flex items-center gap-2 px-3 py-2 rounded-full bg-white/10 text-white/70 border border-white/10 hover:bg-white/20 hover:text-white transition-colors"
        @click="emit('open-members')"
      >
        <span class="material-symbols-outlined text-[18px]">group</span>
        <span class="text-[11px] font-semibold">Members</span>
        <span class="text-[10px] font-bold bg-white/10 px-1.5 py-0.5 rounded-full">{{ memberCount }}</span>
      </button>
      <button
        type="button"
        @click="emit('open-skills')"
        class="w-10 h-10 rounded-xl bg-gradient-to-br from-primary/60 to-primary-hover/50 backdrop-blur-md border border-white/20 shadow-lg text-on-primary flex items-center justify-center transition-all group active:scale-95 hover:shadow-glow"
        :title="t('chat.header.inventory')"
      >
        <span class="material-symbols-outlined text-[22px] drop-shadow-sm group-hover:rotate-12 transition-transform">backpack</span>
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';

const props = defineProps<{ title?: string; description?: string; memberCount?: number }>();
const emit = defineEmits<{ (e: 'open-roadmap'): void; (e: 'open-skills'): void; (e: 'open-members'): void }>();

const { t } = useI18n();

const headerTitle = computed(() => props.title ?? t('chat.channelName'));
const headerDescription = computed(() => props.description ?? t('chat.channelDescription'));
</script>
