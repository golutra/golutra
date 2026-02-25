<template>
  <div class="fixed inset-0 z-[150] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="w-full max-w-sm bg-panel-strong border border-white/10 rounded-2xl shadow-2xl p-6 ring-1 ring-white/5">
      <div class="flex justify-between items-center mb-6">
        <h3 class="text-white font-bold text-lg">{{ t('members.manage.title') }}</h3>
        <button type="button" @click="emit('close')" class="text-white/40 hover:text-white transition-colors">
          <span class="material-symbols-outlined">close</span>
        </button>
      </div>

      <div class="mb-6 text-center">
        <img :src="member.avatar" class="w-20 h-20 rounded-full mx-auto mb-3 object-cover border-2 border-white/10" />
        <p class="text-white/40 text-xs uppercase font-bold tracking-widest">{{ roleLabel }}</p>
      </div>

      <div class="space-y-4">
        <div>
          <label class="text-xs font-bold text-white/50 uppercase tracking-wider mb-1.5 block">{{ t('members.manage.displayName') }}</label>
          <input
            v-model="name"
            class="w-full bg-surface/80 border border-white/10 rounded-xl px-4 py-2.5 text-white focus:border-primary/50 focus:ring-1 focus:ring-primary/50 outline-none transition-all"
          />
        </div>

        <button type="button" @click="emit('save', member.id, name)" class="w-full py-2.5 bg-primary text-on-primary font-bold rounded-xl hover:bg-primary-hover transition-colors">
          {{ t('settings.saveChanges') }}
        </button>

        <template v-if="showRemove">
          <div class="h-px bg-white/5 my-2"></div>

          <button type="button" @click="emit('remove', member.id)" class="w-full py-2.5 text-red-400 hover:bg-red-500/10 font-medium rounded-xl transition-colors flex items-center justify-center gap-2">
            <span class="material-symbols-outlined text-[18px]">person_remove</span>
            {{ t('members.manage.remove') }}
          </button>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Member } from '../types';

const props = defineProps<{ member: Member; showRemove?: boolean }>();
const emit = defineEmits<{ (e: 'close'): void; (e: 'save', id: string, name: string): void; (e: 'remove', id: string): void }>();

const member = toRef(props, 'member');
const name = ref(member.value.name);
const showRemove = computed(() => props.showRemove !== false);

const { t } = useI18n();

const roleLabel = computed(() => {
  if (member.value.roleKey) {
    return t(member.value.roleKey);
  }
  if (member.value.role) {
    return member.value.role;
  }
  return t('members.roles.member');
});

watch(member, (nextMember) => {
  name.value = nextMember.name;
});
</script>
