<template>
  <div class="fixed top-5 right-5 z-[60] flex flex-col gap-2 items-end pointer-events-none">
    <div
      v-for="toast in toasts"
      :key="toast.id"
      :class="[
        'pointer-events-auto flex items-center gap-3 px-4 py-3 rounded-xl border shadow-2xl backdrop-blur bg-panel/90',
        toneClasses(toast.tone)
      ]"
      @click="removeToast(toast.id)"
    >
      <span class="material-symbols-outlined text-[18px] opacity-80">{{ toneIcon(toast.tone) }}</span>
      <span class="text-xs font-semibold leading-snug">{{ toast.message }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useToastStore, type ToastTone } from '@/stores/toastStore';

const toastStore = useToastStore();
const { toasts } = storeToRefs(toastStore);
const { removeToast } = toastStore;

const toneClasses = (tone: ToastTone) => {
  if (tone === 'error') {
    return 'border-red-500/40 text-red-100 bg-red-500/10';
  }
  if (tone === 'success') {
    return 'border-emerald-400/40 text-emerald-100 bg-emerald-500/10';
  }
  return 'border-white/10 text-white/90 bg-white/5';
};

const toneIcon = (tone: ToastTone) => {
  if (tone === 'error') {
    return 'error';
  }
  if (tone === 'success') {
    return 'check_circle';
  }
  return 'info';
};
</script>
