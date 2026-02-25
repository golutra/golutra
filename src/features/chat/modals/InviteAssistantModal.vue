<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="w-full max-w-[340px] bg-panel/80 backdrop-blur-2xl border border-white/10 rounded-3xl shadow-2xl flex flex-col relative ring-1 ring-white/5 overflow-hidden">
      <button type="button" @click="emit('close')" class="absolute top-4 right-4 w-7 h-7 rounded-full bg-white/5 text-white/40 hover:text-white flex items-center justify-center transition-colors z-10 hover:bg-white/10">
        <span class="material-symbols-outlined text-[16px]">close</span>
      </button>
      <div class="px-6 pt-8 pb-3 text-center">
        <h2 class="text-white font-bold text-[17px] tracking-tight">{{ title }}</h2>
        <p class="text-white/40 text-[12px] font-medium mt-1">{{ t('invite.assistant.subtitle') }}</p>
      </div>

      <div class="px-3 py-2 space-y-1">
        <div
          v-for="model in models"
          :key="model.id"
          @click="selectedModel = model.id"
          :class="[
            'flex items-center gap-3 p-2.5 rounded-xl border cursor-pointer group transition-all',
            selectedModel === model.id ? 'bg-primary/10 border-primary/20 relative overflow-hidden' : 'border-transparent hover:bg-white/5'
          ]"
        >
          <div v-if="selectedModel === model.id" class="absolute left-0 top-0 bottom-0 w-0.5 bg-primary"></div>
          <div
            :class="[
              'w-9 h-9 rounded-lg flex items-center justify-center',
              selectedModel === model.id
                ? 'bg-primary text-on-primary shadow-lg shadow-primary/20'
                : 'bg-white/5 text-white/70 border border-white/5 group-hover:bg-white/10'
            ]"
          >
            <span class="material-symbols-outlined text-[20px]">{{ model.icon }}</span>
          </div>
          <div class="flex flex-col z-10">
            <span :class="['text-[13px] font-medium', selectedModel === model.id ? 'text-white font-semibold' : 'text-white/80 group-hover:text-white']">{{ t(model.labelKey) }}</span>
          </div>
          <div v-if="selectedModel === model.id" class="ml-auto text-primary">
            <span class="material-symbols-outlined text-[18px]">check_circle</span>
          </div>
        </div>
      </div>

      <div class="px-5 py-5 space-y-4 mt-1 border-t border-white/5 bg-white/[0.02]">
        <div class="flex items-center justify-between">
          <span class="text-[13px] text-white/90 font-medium tracking-tight">{{ t('invite.assistant.instances') }}</span>
          <div class="flex items-center bg-panel/60 rounded-lg p-0.5 border border-white/10 shadow-inner">
            <button
              type="button"
              :disabled="!canDecreaseInstances"
              :class="[
                'w-7 h-6 rounded-md flex items-center justify-center transition-colors',
                canDecreaseInstances ? 'text-white/60 hover:text-white hover:bg-white/10' : 'text-white/20 cursor-not-allowed'
              ]"
              @click="decreaseInstances"
            >
              <span class="material-symbols-outlined text-[16px]">remove</span>
            </button>
            <input
              :value="instanceText"
              type="text"
              inputmode="numeric"
              class="w-10 bg-transparent text-center text-[13px] font-semibold text-white tabular-nums outline-none border-none focus:ring-0"
              aria-label="Instance count"
              @input="handleInstanceInput"
              @blur="commitInstanceInput"
              @keydown.enter.prevent="commitInstanceInput"
            />
            <button
              type="button"
              :disabled="!canIncreaseInstances"
              :class="[
                'w-7 h-6 rounded-md flex items-center justify-center transition-colors',
                canIncreaseInstances
                  ? 'text-primary hover:bg-primary/10 shadow-[0_0_8px_rgb(var(--color-primary)_/_0.2)]'
                  : 'text-white/20 cursor-not-allowed'
              ]"
              @click="increaseInstances"
            >
              <span class="material-symbols-outlined text-[16px]">add</span>
            </button>
          </div>
        </div>

        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-[13px] text-white/90 font-medium tracking-tight">{{ t('invite.assistant.unlimitedAccess') }}</span>
            <span class="text-[11px] text-white/30 font-medium">{{ t('invite.assistant.unlimitedAccessDesc') }}</span>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input v-model="unlimitedAccess" type="checkbox" class="sr-only peer" />
            <div class="w-11 h-6 bg-panel-strong/80 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-border/40 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary shadow-inner"></div>
          </label>
        </div>
        <div class="flex items-center justify-between">
          <div class="flex flex-col">
            <span class="text-[13px] text-white/90 font-medium tracking-tight">{{ t('invite.assistant.sandboxed') }}</span>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input v-model="sandboxed" type="checkbox" class="sr-only peer" />
            <div class="w-11 h-6 bg-panel-strong/80 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-border/40 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary shadow-inner"></div>
          </label>
        </div>
      </div>

      <div class="p-4 pt-2">
        <button
          type="button"
          @click="emitInvite"
          class="w-full py-3 bg-gradient-to-r from-primary to-primary-hover hover:brightness-110 text-on-primary text-[13px] font-bold rounded-xl shadow-[0_0_20px_rgb(var(--color-primary)_/_0.3)] transition-all active:scale-[0.98] border-t border-white/20"
        >
          {{ t('invite.assistant.send') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';

const props = defineProps<{ title?: string }>();
const emit = defineEmits<{ (e: 'close'): void; (e: 'invite', model: string): void }>();

const { t } = useI18n();

const models = [
  { id: 'gemini', labelKey: 'invite.assistant.models.gemini', icon: 'terminal' },
  { id: 'codex', labelKey: 'invite.assistant.models.codex', icon: 'code' },
  { id: 'claude', labelKey: 'invite.assistant.models.claude', icon: 'psychology' },
  { id: 'custom', labelKey: 'invite.assistant.models.custom', icon: 'settings_suggest' }
];

const selectedModel = ref('claude');
const minInstances = 1;
const maxInstances = 10;
const instances = ref(minInstances);
const instanceText = ref(String(minInstances));
const unlimitedAccess = ref(true);
const sandboxed = ref(false);

const title = computed(() => props.title ?? t('invite.assistant.title'));
const canDecreaseInstances = computed(() => instances.value > minInstances);
const canIncreaseInstances = computed(() => instances.value < maxInstances);

const clampInstances = (value: number) => Math.min(maxInstances, Math.max(minInstances, value));

const handleInstanceInput = (event: Event) => {
  const nextValue = (event.target as HTMLInputElement).value;
  instanceText.value = nextValue;
  if (!nextValue.trim()) return;
  const parsed = Number(nextValue);
  if (!Number.isFinite(parsed)) return;
  instances.value = clampInstances(Math.round(parsed));
};

const commitInstanceInput = () => {
  const parsed = Number(instanceText.value);
  if (!Number.isFinite(parsed)) {
    instanceText.value = String(instances.value);
    return;
  }
  instances.value = clampInstances(Math.round(parsed));
  instanceText.value = String(instances.value);
};

const decreaseInstances = () => {
  if (instances.value > minInstances) {
    instances.value -= 1;
    instanceText.value = String(instances.value);
  }
};

const increaseInstances = () => {
  if (instances.value < maxInstances) {
    instances.value += 1;
    instanceText.value = String(instances.value);
  }
};

const emitInvite = () => {
  const model = models.find((item) => item.id === selectedModel.value);
  if (model) {
    emit('invite', t(model.labelKey));
  }
};
</script>
