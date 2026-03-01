<!-- [2026-01-23 11:56] 目的: 统一渲染终端调用链步骤与通过状态; 边界: 仅负责展示与样式，不参与测试逻辑; 设计: 根据步骤状态映射图标与颜色，保持诊断与审计一致呈现。 -->
<template>
  <div class="space-y-3">
    <div
      v-for="chain in callChains"
      :key="chain.id"
      class="rounded-xl border border-white/10 bg-white/5 px-3 py-3"
    >
      <div class="flex items-center justify-between">
        <div class="text-[12px] font-semibold text-white/80">{{ chain.title }}</div>
        <span
          class="material-symbols-outlined text-[16px]"
          :class="chainStatusClass(chain)"
        >
          {{ chainStatusIcon(chain) }}
        </span>
      </div>
      <div class="mt-2 space-y-1.5">
        <div
          v-for="step in chain.steps"
          :key="step.id"
          class="flex items-start gap-2 text-[12px] text-white/70"
        >
          <span class="material-symbols-outlined text-[16px]" :class="stepStatusClass(step.status)">
            {{ stepStatusIcon(step.status) }}
          </span>
          <div class="flex-1 space-y-0.5">
            <div class="leading-relaxed">{{ step.label }}</div>
            <div v-if="step.detail" :class="['text-[11px] leading-relaxed break-words', stepDetailClass(step.status)]">
              {{ step.detail }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
type StepStatus = 'pending' | 'passed' | 'failed' | 'skipped';
type CallChainStep = { id: string; label: string; status: StepStatus; detail?: string };
type CallChain = { id: string; title: string; steps: CallChainStep[] };

defineProps<{ callChains: CallChain[] }>();

const stepStatusIcon = (status: StepStatus) => {
  if (status === 'passed') return 'check_circle';
  if (status === 'failed') return 'cancel';
  if (status === 'skipped') return 'remove_circle';
  return 'radio_button_unchecked';
};

const stepStatusClass = (status: StepStatus) => {
  if (status === 'passed') return 'text-emerald-400';
  if (status === 'failed') return 'text-red-400';
  if (status === 'skipped') return 'text-white/30';
  return 'text-white/30';
};

const stepDetailClass = (status: StepStatus) => {
  if (status === 'failed') return 'text-red-300';
  if (status === 'passed') return 'text-emerald-300/80';
  if (status === 'skipped') return 'text-white/40';
  return 'text-white/40';
};

const chainStatusIcon = (chain: CallChain) => {
  if (chain.steps.some((step) => step.status === 'failed')) return 'cancel';
  if (chain.steps.every((step) => step.status === 'passed' || step.status === 'skipped')) {
    return 'check_circle';
  }
  return 'radio_button_unchecked';
};

const chainStatusClass = (chain: CallChain) => {
  if (chain.steps.some((step) => step.status === 'failed')) return 'text-red-400';
  if (chain.steps.every((step) => step.status === 'passed' || step.status === 'skipped')) {
    return 'text-emerald-400';
  }
  return 'text-white/30';
};
</script>
