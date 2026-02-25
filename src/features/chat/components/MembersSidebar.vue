<template>
  <aside
    ref="containerRef"
    :class="[
      'bg-panel/50 glass-panel border-l border-white/5 shrink-0 flex-col py-6 px-4 h-full',
      variant === 'drawer' ? 'flex w-72' : 'hidden xl:flex w-[260px]'
    ]"
  >
    <div class="mb-6 flex items-center justify-between px-2">
      <h2 class="text-white font-bold text-[15px]">{{ t('members.title') }}</h2>
      <button
        type="button"
        @click="emit('toggle-invite')"
        :class="[
          'w-8 h-8 rounded-full flex items-center justify-center transition-colors shadow-glow ring-1 ring-white/20 relative z-50',
          showInviteMenu ? 'bg-primary text-on-primary shadow-glow' : 'bg-panel/60 text-white/70 hover:bg-panel/80 hover:text-white'
        ]"
      >
        <span class="material-symbols-outlined text-[20px]">person_add</span>
      </button>
    </div>

    <div class="space-y-2 overflow-y-auto custom-scrollbar flex-1">
      <div v-if="owners.length" class="mb-6">
        <h3 class="text-white/30 text-[10px] font-bold uppercase tracking-widest mb-3 px-2">{{ t('members.sections.owner', { count: owners.length }) }}</h3>
        <div class="space-y-1">
          <MemberRow
            v-for="member in owners"
            :key="member.id"
            :member="member"
            :current-user-id="currentUserId"
            :menu-open="openMenuId === member.id"
            @toggle-menu="toggleMenu"
            @action="handleAction"
          />
        </div>
      </div>

      <div v-if="admins.length" class="mb-6">
        <h3 class="text-white/30 text-[10px] font-bold uppercase tracking-widest mb-3 px-2">{{ t('members.sections.admin', { count: admins.length }) }}</h3>
        <div class="space-y-1">
          <MemberRow
            v-for="member in admins"
            :key="member.id"
            :member="member"
            :current-user-id="currentUserId"
            :menu-open="openMenuId === member.id"
            @toggle-menu="toggleMenu"
            @action="handleAction"
          />
        </div>
      </div>

      <div v-if="assistants.length" class="mb-6">
        <h3 class="text-white/30 text-[10px] font-bold uppercase tracking-widest mb-3 px-2">{{ t('members.sections.assistant', { count: assistants.length }) }}</h3>
        <div class="space-y-1">
          <MemberRow
            v-for="member in assistants"
            :key="member.id"
            :member="member"
            :current-user-id="currentUserId"
            :menu-open="openMenuId === member.id"
            @toggle-menu="toggleMenu"
            @action="handleAction"
          />
        </div>
      </div>

      <div v-if="membersGroup.length" class="mb-6">
        <h3 class="text-white/30 text-[10px] font-bold uppercase tracking-widest mb-3 px-2">{{ t('members.sections.member', { count: membersGroup.length }) }}</h3>
        <div class="space-y-1">
          <MemberRow
            v-for="member in membersGroup"
            :key="member.id"
            :member="member"
            :current-user-id="currentUserId"
            :menu-open="openMenuId === member.id"
            @toggle-menu="toggleMenu"
            @action="handleAction"
          />
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Member, MemberAction } from '../types';
import MemberRow from './MemberRow.vue';

const props = defineProps<{ members: Member[]; showInviteMenu: boolean; currentUserId?: string; variant?: 'sidebar' | 'drawer' }>();
const emit = defineEmits<{ (e: 'toggle-invite'): void; (e: 'member-action', payload: { action: MemberAction; member: Member }): void }>();

const owners = computed(() => props.members.filter((member) => member.roleType === 'owner'));
const admins = computed(() => props.members.filter((member) => member.roleType === 'admin'));
const assistants = computed(() => props.members.filter((member) => member.roleType === 'assistant'));
const membersGroup = computed(() => props.members.filter((member) => member.roleType === 'member'));

const showInviteMenu = toRef(props, 'showInviteMenu');
const currentUserId = toRef(props, 'currentUserId');
const variant = computed(() => props.variant ?? 'sidebar');
const openMenuId = ref<string | null>(null);
const containerRef = ref<HTMLElement | null>(null);

const { t } = useI18n();

const toggleMenu = (member: Member) => {
  openMenuId.value = openMenuId.value === member.id ? null : member.id;
};

const handleAction = (payload: { action: MemberAction; member: Member }) => {
  openMenuId.value = null;
  emit('member-action', payload);
};

const handleClickOutside = (event: MouseEvent) => {
  if (containerRef.value && !containerRef.value.contains(event.target as Node)) {
    openMenuId.value = null;
  }
};

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside);
});
</script>
