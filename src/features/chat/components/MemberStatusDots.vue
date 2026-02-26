<template>
  <div ref="anchorRef" class="relative group/status" @mouseenter="handleEnter" @mouseleave="handleLeave">
    <div class="flex items-center gap-0.5">
      <span :class="['w-3 h-3 rounded-full border-2 border-panel', manualDotClass]"></span>
      <span
        v-if="showTerminalStatus"
        :class="['w-3 h-3 rounded-full border-2 border-panel', terminalDotClass]"
      ></span>
    </div>
    <Teleport to="body">
      <div
        v-if="showTooltip"
        class="fixed min-w-[180px] -translate-x-1/2 rounded-xl bg-panel-strong/95 px-3 py-2 text-[10px] text-white/70 shadow-2xl ring-1 ring-white/10 pointer-events-none z-[99999]"
        :style="{ top: `${tooltipPosition.top}px`, left: `${tooltipPosition.left}px` }"
      >
        <div class="text-[10px] font-semibold uppercase tracking-wider text-white/50">{{ t('settings.status') }}</div>
        <div class="mt-1 space-y-1">
          <div
            v-for="option in manualOptions"
            :key="option.id"
            :class="[
              'flex items-center gap-2 rounded-md px-2 py-1',
              option.id === status ? 'bg-white/10 text-white ring-1 ring-white/10' : 'text-white/60'
            ]"
          >
            <span
              :class="[
                'w-2 h-2 rounded-full',
                option.dotClass,
                option.id === status ? 'shadow-[0_0_8px_rgba(255,255,255,0.45)]' : 'opacity-40'
              ]"
            ></span>
            <span :class="[option.id === status ? 'text-white font-semibold' : 'text-white/50']">
              {{ t(option.labelKey) }}
            </span>
          </div>
        </div>
        <div v-if="showTerminalStatus" class="mt-2 pt-2 border-t border-white/10">
          <div class="text-[10px] font-semibold uppercase tracking-wider text-white/50">
            {{ t('terminal.statusLabel') }}
          </div>
          <div class="mt-1 space-y-1">
            <div
              v-for="option in terminalOptions"
              :key="option.id"
              :class="[
                'flex items-center gap-2 rounded-md px-2 py-1',
                option.id === resolvedTerminalStatus ? 'bg-white/10 text-white ring-1 ring-white/10' : 'text-white/60'
              ]"
            >
              <span
                :class="[
                  'w-2 h-2 rounded-full',
                  option.dotClass,
                  option.id === resolvedTerminalStatus
                    ? 'shadow-[0_0_8px_rgba(255,255,255,0.45)]'
                    : 'opacity-40'
                ]"
              ></span>
              <span :class="[option.id === resolvedTerminalStatus ? 'text-white font-semibold' : 'text-white/50']">
                {{ t(option.labelKey) }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { MemberStatus } from '../types';
import type { TerminalConnectionStatus } from '@/shared/types/terminal';

const props = defineProps<{
  status: MemberStatus;
  terminalStatus?: TerminalConnectionStatus;
  showTerminalStatus?: boolean;
}>();

const { t } = useI18n();
const anchorRef = ref<HTMLElement | null>(null);
const showTooltip = ref(false);
const tooltipPosition = ref({ top: 0, left: 0 });

const baseManualOptions: Array<{ id: MemberStatus; labelKey: string; dotClass: string }> = [
  { id: 'online', labelKey: 'settings.statusOptions.online', dotClass: 'bg-green-500' },
  { id: 'working', labelKey: 'settings.statusOptions.working', dotClass: 'bg-amber-400' },
  { id: 'dnd', labelKey: 'settings.statusOptions.dnd', dotClass: 'bg-red-500' },
  { id: 'offline', labelKey: 'settings.statusOptions.offline', dotClass: 'bg-white/30' }
];

const terminalOptions: Array<{ id: TerminalConnectionStatus; labelKey: string; dotClass: string }> = [
  { id: 'connecting', labelKey: 'terminal.statusOptions.connecting', dotClass: 'bg-sky-400' },
  { id: 'connected', labelKey: 'terminal.statusOptions.connected', dotClass: 'bg-emerald-500' },
  { id: 'working', labelKey: 'terminal.statusOptions.working', dotClass: 'bg-amber-400' },
  { id: 'disconnected', labelKey: 'terminal.statusOptions.disconnected', dotClass: 'bg-white/30' }
];

const showTerminalStatus = computed(() => props.showTerminalStatus ?? false);
const manualOptions = computed(() =>
  showTerminalStatus.value ? baseManualOptions.filter((option) => option.id !== 'working') : baseManualOptions
);
const resolvedTerminalStatus = computed<TerminalConnectionStatus | null>(() => {
  if (!showTerminalStatus.value) {
    return null;
  }
  return props.terminalStatus ?? 'disconnected';
});

const manualDotClass = computed(() => {
  const match = baseManualOptions.find((option) => option.id === props.status);
  return match?.dotClass ?? 'bg-white/30';
});

const terminalDotClass = computed(() => {
  const match = terminalOptions.find((option) => option.id === resolvedTerminalStatus.value);
  return match?.dotClass ?? 'bg-white/30';
});

const updatePosition = () => {
  const rect = anchorRef.value?.getBoundingClientRect();
  if (!rect) {
    return;
  }
  tooltipPosition.value = {
    top: rect.bottom + 8,
    left: rect.left + rect.width / 2
  };
};

const handleEnter = () => {
  showTooltip.value = true;
  updatePosition();
  window.addEventListener('scroll', updatePosition, true);
  window.addEventListener('resize', updatePosition);
};

const handleLeave = () => {
  showTooltip.value = false;
  window.removeEventListener('scroll', updatePosition, true);
  window.removeEventListener('resize', updatePosition);
};

onBeforeUnmount(() => {
  handleLeave();
});
</script>
