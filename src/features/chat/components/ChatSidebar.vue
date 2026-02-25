<template>
  <div ref="containerRef" class="w-16 md:w-64 bg-panel/50 border-r border-white/5 flex flex-col shrink-0">
    <div class="h-16 flex items-center px-2 md:px-4 border-b border-white/5 justify-center md:justify-start">
      <h2 class="text-white font-bold text-sm tracking-wide flex items-center gap-2">
        <span class="material-symbols-outlined text-primary text-[18px]">layers</span>
        <span class="hidden md:inline">{{ t('chat.sidebar.workspaceName') }}</span>
      </h2>
    </div>

    <div class="flex-1 overflow-y-auto py-4 px-1 md:px-2 space-y-6 custom-scrollbar">
      <div>
        <div class="px-2 mb-2 items-center justify-between group hidden md:flex">
          <h3 class="text-[11px] font-bold text-white/40 uppercase tracking-wider">{{ t('chat.sidebar.channels') }}</h3>
          <button class="text-white/20 hover:text-white transition-colors opacity-0 group-hover:opacity-100" type="button">
            <span class="material-symbols-outlined text-[16px]">add</span>
          </button>
        </div>
        <div class="space-y-1">
          <div
            v-for="item in channelItems"
            :key="item.conversation.id"
            :class="[
              'w-full px-2 md:px-3 py-2 rounded-xl flex items-center md:items-start gap-3 transition-all group cursor-pointer justify-center md:justify-start',
              item.conversation.id === activeConversationId ? 'bg-white/10 text-white' : 'text-white/60 hover:text-white hover:bg-white/5'
            ]"
            @click="selectConversation(item.conversation.id)"
          >
            <span :class="['text-[18px] font-semibold leading-none mt-0.5', item.conversation.id === activeConversationId ? 'text-primary' : 'text-white/30']">#</span>
            <div class="hidden md:flex items-start gap-3 min-w-0 flex-1">
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 min-w-0">
                  <span class="text-[13px] font-semibold truncate">{{ item.title }}</span>
                  <span v-if="item.conversation.pinned" class="material-symbols-outlined text-[12px] text-white/40">push_pin</span>
                  <span v-if="item.conversation.muted" class="material-symbols-outlined text-[12px] text-white/40">notifications_off</span>
                </div>
                <div class="text-[11px] text-white/40 truncate">{{ item.preview }}</div>
              </div>
            </div>
            <div class="relative shrink-0 hidden md:block">
              <button
                type="button"
                @click.stop="toggleMenu(item.conversation.id)"
                :class="[
                  'w-7 h-7 rounded-lg flex items-center justify-center transition-colors',
                  openMenuId === item.conversation.id ? 'bg-white/10 text-white' : 'text-white/30 hover:text-white hover:bg-white/10'
                ]"
              >
                <span class="material-symbols-outlined text-[16px]">more_vert</span>
              </button>
              <div
                v-if="openMenuId === item.conversation.id"
                class="absolute right-0 top-full mt-2 w-56 max-h-64 rounded-xl glass-modal bg-panel-strong/95 flex flex-col py-1.5 shadow-2xl overflow-y-auto custom-scrollbar z-50 animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10"
                @click.stop
              >
                <button
                  type="button"
                  class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/5 hover:text-white transition-colors flex items-center gap-3"
                  @click="handleAction(item.conversation, item.conversation.pinned ? 'unpin' : 'pin')"
                >
                  <span class="material-symbols-outlined text-lg opacity-70">push_pin</span>
                  {{ item.conversation.pinned ? t('chat.conversation.actions.unpin') : t('chat.conversation.actions.pin') }}
                </button>
                <button
                  type="button"
                  class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/5 hover:text-white transition-colors flex items-center gap-3"
                  @click="handleAction(item.conversation, 'rename')"
                >
                  <span class="material-symbols-outlined text-lg opacity-70">edit</span>
                  {{ t('chat.conversation.actions.rename') }}
                </button>
                <button
                  type="button"
                  class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/5 hover:text-white transition-colors flex items-center gap-3"
                  @click="handleAction(item.conversation, item.conversation.muted ? 'unmute' : 'mute')"
                >
                  <span class="material-symbols-outlined text-lg opacity-70">notifications_off</span>
                  {{ item.conversation.muted ? t('chat.conversation.actions.unmute') : t('chat.conversation.actions.mute') }}
                </button>
                <div class="h-px bg-white/10 my-1 mx-2"></div>
                <button
                  type="button"
                  class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/5 hover:text-white transition-colors flex items-center gap-3"
                  @click="handleAction(item.conversation, 'clear')"
                >
                  <span class="material-symbols-outlined text-lg opacity-70">delete_sweep</span>
                  {{ t('chat.conversation.actions.clear') }}
                </button>
                <button
                  type="button"
                  class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-red-400 hover:bg-red-500/10 hover:text-red-300 transition-colors flex items-center gap-3"
                  v-if="!isDefaultChannel(item.conversation)"
                  @click="handleAction(item.conversation, 'delete')"
                >
                  <span class="material-symbols-outlined text-lg opacity-70">delete</span>
                  {{ t('chat.conversation.actions.deleteChannel') }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div>
        <div class="px-2 mb-2 items-center justify-between group hidden md:flex">
          <h3 class="text-[11px] font-bold text-white/40 uppercase tracking-wider">{{ t('chat.sidebar.directMessages') }}</h3>
          <button class="text-white/20 hover:text-white transition-colors opacity-0 group-hover:opacity-100" type="button">
            <span class="material-symbols-outlined text-[16px]">add</span>
          </button>
        </div>
        <div class="space-y-1">
          <div
            v-for="item in directMessageItems"
            :key="item.conversation.id"
            :class="[
              'w-full px-2 md:px-3 py-2 rounded-xl flex items-center md:items-start gap-3 transition-all group cursor-pointer justify-center md:justify-start',
              item.conversation.id === activeConversationId ? 'bg-white/10 text-white' : 'text-white/60 hover:text-white hover:bg-white/5'
            ]"
            @click="selectConversation(item.conversation.id)"
          >
            <div class="relative">
              <div class="w-9 h-9 rounded-full bg-cover bg-center shadow-md" :style="{ backgroundImage: `url('${item.member.avatar}')` }"></div>
              <div :class="['absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full border-2 border-panel', item.member.status === 'online' ? 'bg-green-500' : item.member.status === 'dnd' ? 'bg-red-500' : 'bg-white/20']"></div>
            </div>
            <div class="hidden md:flex items-start gap-3 min-w-0 flex-1">
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 min-w-0">
                  <span class="text-[13px] font-semibold truncate">{{ item.title }}</span>
                  <span v-if="item.conversation.pinned" class="material-symbols-outlined text-[12px] text-white/40">push_pin</span>
                  <span v-if="item.conversation.muted" class="material-symbols-outlined text-[12px] text-white/40">notifications_off</span>
                </div>
                <div class="text-[11px] text-white/40 truncate">{{ item.preview }}</div>
              </div>
            </div>
            <div class="relative shrink-0 hidden md:block">
              <button
                type="button"
                @click.stop="toggleMenu(item.conversation.id)"
                :class="[
                  'w-7 h-7 rounded-lg flex items-center justify-center transition-colors',
                  openMenuId === item.conversation.id ? 'bg-white/10 text-white' : 'text-white/30 hover:text-white hover:bg-white/10'
                ]"
              >
                <span class="material-symbols-outlined text-[16px]">more_vert</span>
              </button>
              <div
                v-if="openMenuId === item.conversation.id"
                class="absolute right-0 top-full mt-2 w-56 max-h-64 rounded-xl glass-modal bg-panel-strong/95 flex flex-col py-1.5 shadow-2xl overflow-y-auto custom-scrollbar z-50 animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10"
                @click.stop
              >
                <button
                  type="button"
                  class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/5 hover:text-white transition-colors flex items-center gap-3"
                  @click="handleAction(item.conversation, item.conversation.pinned ? 'unpin' : 'pin')"
                >
                  <span class="material-symbols-outlined text-lg opacity-70">push_pin</span>
                  {{ item.conversation.pinned ? t('chat.conversation.actions.unpin') : t('chat.conversation.actions.pin') }}
                </button>
                <button
                  type="button"
                  class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/5 hover:text-white transition-colors flex items-center gap-3"
                  @click="handleAction(item.conversation, item.conversation.muted ? 'unmute' : 'mute')"
                >
                  <span class="material-symbols-outlined text-lg opacity-70">notifications_off</span>
                  {{ item.conversation.muted ? t('chat.conversation.actions.unmute') : t('chat.conversation.actions.mute') }}
                </button>
                <div class="h-px bg-white/10 my-1 mx-2"></div>
                <button
                  type="button"
                  class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-white hover:bg-white/5 hover:text-white transition-colors flex items-center gap-3"
                  @click="handleAction(item.conversation, 'clear')"
                >
                  <span class="material-symbols-outlined text-lg opacity-70">delete_sweep</span>
                  {{ t('chat.conversation.actions.clear') }}
                </button>
                <button
                  type="button"
                  class="relative w-full text-left px-4 py-2.5 text-xs font-bold text-red-400 hover:bg-red-500/10 hover:text-red-300 transition-colors flex items-center gap-3"
                  @click="handleAction(item.conversation, 'delete')"
                >
                  <span class="material-symbols-outlined text-lg opacity-70">delete</span>
                  {{ t('chat.conversation.actions.deleteDirect') }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Conversation, ConversationAction, Member } from '../types';
import { DEFAULT_CHANNEL_ID } from '../data';

const props = defineProps<{
  conversations: Conversation[];
  members: Member[];
  activeConversationId: string;
}>();
const emit = defineEmits<{
  (e: 'select-conversation', conversationId: string): void;
  (e: 'conversation-action', payload: { conversationId: string; action: ConversationAction }): void;
}>();

const { t } = useI18n();

const openMenuId = ref<string | null>(null);
const containerRef = ref<HTMLElement | null>(null);

const memberById = computed(() => new Map(props.members.map((member) => [member.id, member])));

const normalizePreview = (text: string) => text.replace(/\s+/g, ' ').trim();

const getConversationTitle = (conversation: Conversation) => {
  if (conversation.type === 'dm') {
    return memberById.value.get(conversation.targetId)?.name ?? t('members.roles.member');
  }
  return conversation.customName ?? (conversation.nameKey ? t(conversation.nameKey) : conversation.targetId);
};

const isDefaultChannel = (conversation: Conversation) =>
  conversation.type === 'channel' && conversation.targetId === DEFAULT_CHANNEL_ID;

const getLatestMessage = (conversation: Conversation) => {
  let latest: Conversation['messages'][number] | null = null;
  for (const message of conversation.messages) {
    if (!latest || message.createdAt > latest.createdAt) {
      latest = message;
    }
  }
  return latest;
};

const getLastMessageTime = (conversation: Conversation) => getLatestMessage(conversation)?.createdAt ?? 0;

const getLastMessagePreview = (conversation: Conversation) => {
  const latest = getLatestMessage(conversation);
  return latest ? normalizePreview(latest.text) : '';
};

const sortConversations = (items: Conversation[]) =>
  [...items].sort((a, b) => {
    if (a.pinned !== b.pinned) {
      return a.pinned ? -1 : 1;
    }
    const timeA = getLastMessageTime(a);
    const timeB = getLastMessageTime(b);
    if (timeA !== timeB) {
      return timeB - timeA;
    }
    return getConversationTitle(a).localeCompare(getConversationTitle(b));
  });

const channelItems = computed(() =>
  sortConversations(props.conversations.filter((conversation) => conversation.type === 'channel')).map((conversation) => ({
    conversation,
    title: getConversationTitle(conversation),
    preview: getLastMessagePreview(conversation)
  }))
);

const directMessageItems = computed(() =>
  sortConversations(props.conversations.filter((conversation) => conversation.type === 'dm'))
    .map((conversation) => {
      const member = memberById.value.get(conversation.targetId);
      if (!member) return null;
      return {
        conversation,
        member,
        title: getConversationTitle(conversation),
        preview: getLastMessagePreview(conversation)
      };
    })
    .filter((item): item is { conversation: Conversation; member: Member; title: string; preview: string } => Boolean(item))
);

const toggleMenu = (conversationId: string) => {
  openMenuId.value = openMenuId.value === conversationId ? null : conversationId;
};

const selectConversation = (conversationId: string) => {
  openMenuId.value = null;
  emit('select-conversation', conversationId);
};

const handleAction = (conversation: Conversation, action: ConversationAction) => {
  openMenuId.value = null;
  emit('conversation-action', { conversationId: conversation.id, action });
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
