<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
    <div class="w-full max-w-2xl bg-panel-strong/90 backdrop-blur-2xl border border-white/10 rounded-2xl shadow-2xl flex flex-col max-h-[85vh] ring-1 ring-white/5">
      <div class="px-8 py-6 border-b border-white/5 flex justify-between items-start bg-white/[0.02]">
        <div class="space-y-1.5 w-full mr-8">
          <h2 class="text-xl font-bold text-white tracking-tight flex items-center gap-3 mb-3">
            <div class="w-8 h-8 rounded-lg bg-primary/20 flex items-center justify-center text-primary border border-primary/20 shadow-[0_0_15px_rgb(var(--color-primary)_/_0.2)]">
              <span class="material-symbols-outlined text-[20px]">checklist</span>
            </div>
            {{ t('roadmap.title') }}
          </h2>
          <div class="pl-[44px] flex items-center w-full">
            <div class="flex-1 relative group">
              <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
                <span class="text-white/30 text-[13px] font-bold">{{ t('roadmap.objectiveLabel') }}</span>
              </div>
              <input
                class="w-full bg-white/[0.03] border border-white/5 rounded-lg py-2 pl-[84px] pr-8 text-white/80 text-[13px] font-medium placeholder-white/20 focus:outline-none focus:bg-white/[0.06] focus:border-primary/40 focus:ring-1 focus:ring-primary/40 transition-all duration-200"
                type="text"
                :value="objectiveValue"
              />
              <div class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
                <span class="material-symbols-outlined text-white/20 text-[14px]">edit</span>
              </div>
            </div>
          </div>
        </div>
        <button type="button" @click="emit('close')" class="w-8 h-8 rounded-full hover:bg-white/10 text-white/40 hover:text-white flex items-center justify-center transition-colors shrink-0">
          <span class="material-symbols-outlined text-xl">close</span>
        </button>
      </div>

      <div ref="listRef" class="p-6 overflow-y-auto custom-scrollbar space-y-3">
        <div
          v-for="task in tasks"
          :key="task.id"
          :class="[
            'relative group flex items-center gap-4 p-3 pr-4 rounded-xl border transition-all duration-200',
            openMenuId === task.id
              ? 'bg-white/[0.04] border-white/10 z-20'
              : 'bg-white/[0.02] border-white/5 hover:border-white/10 hover:bg-white/[0.04]'
          ]"
        >
          <div :class="['w-8 flex justify-center font-mono text-xs font-medium', task.status === 'in-progress' ? 'text-primary font-bold' : 'text-white/20']">
            {{ task.number }}
          </div>
          <div class="flex-1 min-w-0">
            <input
              v-if="editingId === task.id"
              class="w-full bg-surface/80 border border-primary/50 rounded px-2 py-1 text-white text-[14px] focus:outline-none"
              v-model="editValue"
              @blur="saveTask"
              @keydown.enter.prevent="saveTask"
              autofocus
              :placeholder="t('roadmap.taskPlaceholder')"
            />
            <h3
              v-else
              :class="['font-medium text-[14px]', task.status === 'done' ? 'text-white/60 line-through decoration-white/20' : 'text-white']"
            >
              {{ task.title }}
            </h3>
          </div>
          <div :class="['flex items-center gap-1.5 px-2.5 py-1 rounded-full shadow-sm border', statusConfig(task.status).bg, statusConfig(task.status).border]">
            <div v-if="statusConfig(task.status).indicator" class="w-2 h-2 rounded-full bg-amber-400 animate-pulse"></div>
            <span v-if="statusConfig(task.status).icon" :class="['material-symbols-outlined text-[16px]', statusConfig(task.status).text]">
              {{ statusConfig(task.status).icon }}
            </span>
            <span :class="['text-[11px] font-bold tracking-wide uppercase', statusConfig(task.status).text]">
              {{ statusConfig(task.status).label }}
            </span>
          </div>
          <button
            type="button"
            @click.stop="openMenuId = openMenuId === task.id ? null : task.id"
            :class="['w-8 h-8 flex items-center justify-center rounded-lg transition-colors', openMenuId === task.id ? 'bg-white/10 text-white' : 'text-white/20 hover:text-white hover:bg-white/10']"
          >
            <span class="material-symbols-outlined">more_horiz</span>
          </button>

          <div
            v-if="openMenuId === task.id"
            class="absolute right-10 top-8 w-48 bg-panel-soft border border-white/10 rounded-xl shadow-2xl z-30 overflow-hidden ring-1 ring-black/50 origin-top-right animate-in fade-in zoom-in-95 duration-100 backdrop-blur-3xl"
          >
            <div class="p-1.5 space-y-0.5">
              <button
                v-for="action in taskActions"
                :key="action.key"
                type="button"
                @click="handleTaskAction(action.id, task)"
                class="w-full text-left px-3 py-2 text-[13px] font-medium text-white rounded-lg hover:bg-primary/20 hover:text-primary transition-colors flex items-center gap-2.5 group/item"
              >
                <span class="material-symbols-outlined text-[18px] text-white/50 group-hover/item:text-primary">
                  {{ action.icon }}
                </span>
                {{ t(action.key) }}
              </button>
            </div>
            <div class="h-px bg-white/5 mx-2"></div>
            <div class="p-1.5">
              <button
                type="button"
                @click="handleDelete(task.id)"
                class="w-full text-left px-3 py-2 text-[13px] font-medium text-red-400 rounded-lg hover:bg-red-500/10 transition-colors flex items-center gap-2.5"
              >
                <span class="material-symbols-outlined text-[18px]">delete</span> {{ t('roadmap.actions.delete') }}
              </button>
            </div>
          </div>

          <div v-if="openMenuId === task.id" class="fixed inset-0 z-10" @click="openMenuId = null"></div>
        </div>
      </div>

      <div class="p-5 border-t border-white/5 bg-white/[0.02] flex justify-between items-center rounded-b-2xl">
        <div class="text-[12px] text-white/30 font-medium">
          {{ t('roadmap.footer', { count: tasks.length, percent: completion }) }}
        </div>
        <button
          type="button"
          @click="handleAddTask"
          class="px-5 py-2.5 bg-primary hover:bg-primary-hover text-on-primary text-sm font-bold rounded-xl shadow-glow transition-all active:scale-95 flex items-center gap-2"
        >
          <span class="material-symbols-outlined text-[18px]">add</span> {{ t('roadmap.addTask') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { RoadmapTask, RoadmapTaskStatus } from '../types';

const emit = defineEmits<{ (e: 'close'): void }>();

const openMenuId = ref<number | null>(null);
const editingId = ref<number | null>(null);
const editValue = ref('');
const listRef = ref<HTMLDivElement | null>(null);

const { t } = useI18n();

const objectiveValue = 'Q4 界面重构与功能实现';

const taskActions = [
  { id: 0, key: 'roadmap.actions.edit', icon: 'edit' },
  { id: 1, key: 'roadmap.actions.changeOrder', icon: 'swap_vert' },
  { id: 2, key: 'roadmap.actions.markPriority', icon: 'flag' }
];

const tasks = ref<RoadmapTask[]>([
  { id: 1, number: '01', title: 'Initial Research Phase', status: 'done' },
  { id: 2, number: '02', title: 'UI Wireframes & Prototyping', status: 'in-progress' },
  { id: 3, number: '03', title: 'Legacy Database Migration', status: 'pending' }
]);

watch([editingId, () => tasks.value.length], async () => {
  if (editingId.value && listRef.value) {
    await nextTick();
    listRef.value.scrollTop = listRef.value.scrollHeight;
  }
});

const statusConfig = (status: RoadmapTaskStatus) => {
  switch (status) {
    case 'done':
      return {
        bg: 'bg-emerald-500/10',
        border: 'border-emerald-500/20',
        text: 'text-emerald-400',
        icon: 'check_circle',
        label: t('roadmap.status.done')
      };
    case 'in-progress':
      return {
        bg: 'bg-amber-400/10',
        border: 'border-amber-400/20',
        text: 'text-amber-400',
        icon: null,
        label: t('roadmap.status.inProgress'),
        indicator: true
      };
    default:
      return {
        bg: 'bg-white/5',
        border: 'border-white/10',
        text: 'text-white/40',
        icon: 'radio_button_unchecked',
        label: t('roadmap.status.pending')
      };
  }
};

const handleAddTask = () => {
  const newId = Math.max(0, ...tasks.value.map((task) => task.id)) + 1;
  tasks.value = [
    ...tasks.value,
    {
      id: newId,
      number: newId.toString().padStart(2, '0'),
      title: '',
      status: 'pending'
    }
  ];
  editingId.value = newId;
  editValue.value = '';
};

const saveTask = () => {
  if (editingId.value === null) return;

  tasks.value = tasks.value.map((task) =>
    task.id === editingId.value ? { ...task, title: editValue.value.trim() || t('roadmap.newTask') } : task
  );
  editingId.value = null;
};

const handleDelete = (id: number) => {
  tasks.value = tasks.value.filter((task) => task.id !== id);
  if (openMenuId.value === id) {
    openMenuId.value = null;
  }
};

const handleTaskAction = (index: number, task: RoadmapTask) => {
  if (index === 0) {
    editingId.value = task.id;
    editValue.value = task.title;
    openMenuId.value = null;
  }
};

const completion = computed(() => {
  const doneCount = tasks.value.filter((task) => task.status === 'done').length;
  return Math.round((doneCount / Math.max(tasks.value.length, 1)) * 100);
});
</script>
