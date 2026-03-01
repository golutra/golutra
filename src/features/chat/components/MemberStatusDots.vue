<template>
  <div ref="anchorRef" class="relative group/status" @mouseenter="handleEnter" @mouseleave="handleLeave">
    <div class="flex items-center gap-0.5">
      <span :class="['w-3 h-3 rounded-full border-2', borderClass, manualDotClass]"></span>
      <span
        v-if="showTerminalStatus"
        :class="['w-3 h-3 rounded-full border-2', borderClass, terminalDotClass]"
      ></span>
    </div>
    <Teleport to="body">
      <div
        v-if="showTooltip"
        ref="tooltipRef"
        class="fixed min-w-[180px] rounded-xl bg-panel-strong/95 px-3 py-2 text-[10px] text-white/70 shadow-2xl ring-1 ring-white/10 pointer-events-none z-[99999]"
        :style="{ top: `${tooltipPosition.top}px`, left: `${tooltipPosition.left}px`, transform: tooltipPosition.transform }"
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
// 状态指示点组件：展示在线与终端连接状态的组合信息。
import { computed, nextTick, onBeforeUnmount, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { MemberStatus } from '../types';
import type { TerminalConnectionStatus } from '@/shared/types/terminal';

const props = defineProps<{
  status: MemberStatus;
  terminalStatus?: TerminalConnectionStatus;
  showTerminalStatus?: boolean;
  clampTooltip?: boolean;
  borderClass?: string;
}>();

const { t } = useI18n();
const anchorRef = ref<HTMLElement | null>(null);
const tooltipRef = ref<HTMLElement | null>(null);
const showTooltip = ref(false);
const tooltipPosition = ref({ top: 0, left: 0, transform: 'translateX(-50%)' });

const baseManualOptions: Array<{ id: MemberStatus; labelKey: string; dotClass: string }> = [
  { id: 'online', labelKey: 'settings.statusOptions.online', dotClass: 'bg-green-500' },
  { id: 'working', labelKey: 'settings.statusOptions.working', dotClass: 'bg-amber-400' },
  { id: 'dnd', labelKey: 'settings.statusOptions.dnd', dotClass: 'bg-red-500' },
  { id: 'offline', labelKey: 'settings.statusOptions.offline', dotClass: 'bg-slate-500' }
];

const terminalOptions: Array<{ id: TerminalConnectionStatus; labelKey: string; dotClass: string }> = [
  { id: 'connecting', labelKey: 'terminal.statusOptions.connecting', dotClass: 'bg-sky-400' },
  { id: 'connected', labelKey: 'terminal.statusOptions.connected', dotClass: 'bg-emerald-500' },
  { id: 'working', labelKey: 'terminal.statusOptions.working', dotClass: 'bg-amber-400' },
  { id: 'disconnected', labelKey: 'terminal.statusOptions.disconnected', dotClass: 'bg-slate-500' },
  { id: 'pending', labelKey: 'terminal.statusOptions.pending', dotClass: 'bg-white/20' }
];

const showTerminalStatus = computed(() => props.showTerminalStatus ?? false);
const borderClass = computed(() => props.borderClass?.trim() || 'border-panel');
const manualOptions = computed(() =>
  showTerminalStatus.value ? baseManualOptions.filter((option) => option.id !== 'working') : baseManualOptions
);
const resolvedTerminalStatus = computed<TerminalConnectionStatus | null>(() => {
  if (!showTerminalStatus.value) {
    return null;
  }
  return props.terminalStatus ?? 'pending';
});

const manualDotClass = computed(() => {
  const match = baseManualOptions.find((option) => option.id === props.status);
  return match?.dotClass ?? 'bg-slate-500';
});

const terminalDotClass = computed(() => {
  const match = terminalOptions.find((option) => option.id === resolvedTerminalStatus.value);
  return match?.dotClass ?? 'bg-slate-500';
});

// Tooltip 最小宽度 180px，按半宽做夹紧以避免出屏。
const TOOLTIP_HALF_WIDTH = 110;
const TOOLTIP_GAP = 8;
const TOOLTIP_EDGE_GAP = 8;

const updatePosition = () => {
  const rect = anchorRef.value?.getBoundingClientRect();
  if (!rect) {
    return;
  }
  const tooltipHeight = tooltipRef.value?.offsetHeight ?? 0;
  const tooltipWidth = tooltipRef.value?.offsetWidth ?? 0;
  const viewportHeight = window.innerHeight || document.documentElement.clientHeight;
  const viewportWidth = window.innerWidth || document.documentElement.clientWidth;
  const center = rect.left + rect.width / 2;
  let left = props.clampTooltip
    ? Math.min(Math.max(center, TOOLTIP_HALF_WIDTH), viewportWidth - TOOLTIP_HALF_WIDTH)
    : center;
  let transform = 'translateX(-50%)';
  if (tooltipWidth > 0) {
    // 水平放置规则：优先居中；若溢出则择宽侧靠边，并在极窄视口下夹紧到可视范围。
    const centerLeft = center - tooltipWidth / 2;
    const centerRight = center + tooltipWidth / 2;
    const centerFits = centerLeft >= TOOLTIP_EDGE_GAP && centerRight <= viewportWidth - TOOLTIP_EDGE_GAP;
    if (!centerFits) {
      const spaceRight = viewportWidth - rect.right - TOOLTIP_EDGE_GAP;
      const spaceLeft = rect.left - TOOLTIP_EDGE_GAP;
      if (spaceRight >= spaceLeft) {
        left = rect.right + TOOLTIP_EDGE_GAP;
        transform = 'translateX(0)';
      } else {
        left = rect.left - TOOLTIP_EDGE_GAP;
        transform = 'translateX(-100%)';
      }
    }
    if (transform === 'translateX(0)') {
      const min = TOOLTIP_EDGE_GAP;
      const max = Math.max(min, viewportWidth - TOOLTIP_EDGE_GAP - tooltipWidth);
      left = Math.min(Math.max(left, min), max);
    } else if (transform === 'translateX(-100%)') {
      const max = viewportWidth - TOOLTIP_EDGE_GAP;
      const min = Math.min(max, TOOLTIP_EDGE_GAP + tooltipWidth);
      left = Math.min(Math.max(left, min), max);
    } else {
      const min = TOOLTIP_EDGE_GAP + tooltipWidth / 2;
      const max = Math.max(min, viewportWidth - TOOLTIP_EDGE_GAP - tooltipWidth / 2);
      left = Math.min(Math.max(left, min), max);
    }
  }
  let top = rect.bottom + TOOLTIP_GAP;
  if (tooltipHeight > 0) {
    const spaceBelow = Math.max(0, viewportHeight - rect.bottom);
    const spaceAbove = Math.max(0, rect.top);
    // 底部空间不足时翻到上方，并做兜底夹紧避免出屏。
    if (spaceBelow < tooltipHeight + TOOLTIP_GAP && spaceAbove > spaceBelow) {
      top = rect.top - tooltipHeight - TOOLTIP_GAP;
    }
    const maxTop = Math.max(TOOLTIP_GAP, viewportHeight - tooltipHeight - TOOLTIP_GAP);
    top = Math.min(Math.max(TOOLTIP_GAP, top), maxTop);
  }
  tooltipPosition.value = { top, left, transform };
};

const handleEnter = () => {
  showTooltip.value = true;
  void nextTick(() => {
    updatePosition();
    window.addEventListener('scroll', updatePosition, true);
    window.addEventListener('resize', updatePosition);
  });
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
