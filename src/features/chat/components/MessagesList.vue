<template>
  <div ref="listRef" class="flex-1 overflow-y-auto px-8 py-6 space-y-8 custom-scrollbar flex flex-col">
    <div v-if="hasMore" class="flex justify-center">
      <button
        type="button"
        class="px-4 py-2 rounded-full text-[12px] font-semibold tracking-wide border border-white/10 bg-panel/40 text-white/70 hover:bg-panel/70 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="isLoadingMore"
        @click="emit('load-more')"
      >
        {{ isLoadingMore ? t('chat.messages.loadingHistory') : t('chat.messages.loadHistory') }}
      </button>
    </div>
    <template v-for="item in groupedItems" :key="item.id">
      <div v-if="item.type === 'separator'" class="relative flex py-2 items-center justify-center">
        <div class="absolute inset-0 flex items-center">
          <div class="w-full border-t border-white/5"></div>
        </div>
        <span class="relative px-4 bg-panel-strong/30 rounded-full text-white/30 text-[11px] font-semibold backdrop-blur-md border border-white/5">{{ item.label }}</span>
      </div>

      <div
        v-else
        :class="[
          'flex gap-5 group hover:bg-white/[0.02] -mx-6 px-6 py-2 rounded-2xl transition-colors',
          isMe(item.message) ? 'flex-row-reverse' : ''
        ]"
      >
        <div class="mt-1 shrink-0 cursor-pointer">
          <AvatarBadge
            :avatar="item.message.avatar"
            :label="resolveMessageAuthor(item.message)"
            class="w-11 h-11 rounded-[14px] shadow-lg"
          />
        </div>
        <div :class="['flex flex-col flex-1 min-w-0', isMe(item.message) ? 'items-end' : '']">
          <div :class="['flex items-baseline gap-2.5', isMe(item.message) ? 'flex-row-reverse' : '']">
            <span class="text-white font-semibold text-[15px] cursor-pointer hover:underline tracking-tight">{{ resolveMessageAuthor(item.message) }}</span>
            <span class="text-white/30 text-[11px] font-medium">{{ getMessageTime(item.message) }}</span>
          </div>
          <div v-if="isMe(item.message)" class="selectable mt-1 bg-white text-slate-900 message-bubble--me px-5 py-3 rounded-2xl rounded-tr-sm shadow-lg max-w-[80%] text-[15px] leading-relaxed font-medium">
            {{ resolveMessageText(item.message) }}
          </div>
          <div v-else class="selectable text-white/90 text-[15px] leading-relaxed mt-1 font-light tracking-wide">
            <template v-for="(part, index) in splitMentions(resolveMessageText(item.message))" :key="index">
              <span v-if="part.startsWith('@')" class="text-primary bg-primary/10 px-1.5 py-0.5 rounded cursor-pointer hover:bg-primary/20 transition-colors font-medium">{{ part }}</span>
              <span v-else>{{ part }}</span>
            </template>
          </div>

          <div v-if="isMe(item.message) && item.message.status" class="mt-1 text-[11px] text-white/30 font-medium">
            <span v-if="item.message.status === 'sending'">{{ t('chat.messages.status.sending') }}</span>
            <span v-else-if="item.message.status === 'failed'">{{ t('chat.messages.status.failed') }}</span>
          </div>

          <div v-if="item.message.attachment && item.message.attachment.type === 'image'" class="mt-3 bg-white/5 rounded-xl max-w-sm overflow-hidden border border-white/5 group/image relative cursor-pointer hover:border-primary/30 transition-all hover:shadow-lg">
            <div
              class="h-44 w-full bg-cover bg-center"
              :style="{ backgroundImage: `url('${item.message.attachment.thumbnailPath ?? item.message.attachment.filePath}')` }"
            ></div>
            <div class="p-3 bg-white/[0.02] backdrop-blur-sm">
              <div class="text-[13px] font-medium text-white truncate">{{ item.message.attachment.fileName }}</div>
              <div class="text-[11px] text-white/40 mt-0.5">{{ formatFileSize(item.message.attachment.fileSize) }}</div>
            </div>
            <div class="absolute inset-0 bg-primary/20 opacity-0 group-hover/image:opacity-100 flex items-center justify-center transition-opacity backdrop-blur-[2px]">
              <span class="material-symbols-outlined text-white text-3xl drop-shadow-lg">download</span>
            </div>
          </div>

          <div
            v-if="item.message.attachment && item.message.attachment.type === 'roadmap'"
            class="mt-3 inline-flex items-center gap-3 p-3 rounded-xl bg-panel-soft border border-white/10 hover:border-primary/30 transition-colors cursor-pointer group/attachment"
            @click="emit('open-roadmap')"
          >
            <div class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-400">
              <span class="material-symbols-outlined">map</span>
            </div>
            <div>
              <div class="text-sm font-bold text-white group-hover/attachment:text-primary transition-colors">{{ item.message.attachment.title }}</div>
              <div class="text-xs text-white/40">{{ t('chat.messages.roadmapHint') }}</div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <div v-if="isTyping" class="flex items-center gap-4 py-2 px-6 -mx-6 rounded-2xl">
      <AvatarBadge
        v-if="typingAvatar"
        :avatar="typingAvatar"
        :label="typingName"
        class="w-11 h-11 rounded-[14px] shadow-lg"
      />
      <div v-else class="w-11 h-11 rounded-[14px] bg-panel/60 shadow-lg flex items-center justify-center">
        <span class="material-symbols-outlined text-white/40 text-[22px]">smart_toy</span>
      </div>
      <div class="flex flex-col">
        <div class="flex items-center gap-1 text-white/40 text-[12px]">
          <span class="animate-pulse">•</span>
          <span class="animate-pulse">•</span>
          <span class="animate-pulse">•</span>
          <span class="ml-2">{{ t('chat.messages.typing', { name: typingName }) }}</span>
        </div>
      </div>
    </div>

    <button
      v-if="showJumpButton"
      type="button"
      class="sticky bottom-6 self-end mr-2 px-4 py-2 rounded-full bg-panel/80 border border-white/10 text-white/70 hover:text-white hover:bg-panel-strong/80 transition-all shadow-lg backdrop-blur"
      @click="handleJumpToLatest"
    >
      <span class="material-symbols-outlined text-[16px] mr-1 align-middle">south</span>
      <span class="text-[12px] font-medium">{{ t('chat.messages.jumpToLatest') }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, toRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Message } from '../types';
import { formatMessageTime, groupMessagesByDay, splitMentions } from '../utils';
import AvatarBadge from '@/shared/components/AvatarBadge.vue';

const props = defineProps<{
  messages: Message[];
  currentUserId: string;
  currentUserName: string;
  isTyping?: boolean;
  typingName?: string;
  typingAvatar?: string;
  hasMore?: boolean;
  isLoadingMore?: boolean;
}>();
const emit = defineEmits<{ (e: 'open-roadmap'): void; (e: 'load-more'): void }>();

const messages = toRef(props, 'messages');
const listRef = ref<HTMLDivElement | null>(null);
const isPinnedToBottom = ref(true);

const { t, locale } = useI18n();

const groupedItems = computed(() => groupMessagesByDay(messages.value, locale.value));
const isTyping = computed(() => props.isTyping ?? false);
const typingName = computed(() => props.typingName ?? t('members.roles.aiAssistant'));
const typingAvatar = computed(() => props.typingAvatar ?? '');
const hasMore = computed(() => props.hasMore ?? false);
const isLoadingMore = computed(() => props.isLoadingMore ?? false);
const showJumpButton = computed(() => !isPinnedToBottom.value);

const isMe = (msg: Message) => {
  if (msg.senderId) {
    return msg.senderId === props.currentUserId;
  }
  return msg.user === props.currentUserName;
};

const resolveMessageAuthor = (message: Message) => {
  if (message.senderId && message.senderId === props.currentUserId) {
    return props.currentUserName || message.user;
  }
  if (message.userKey) {
    return t(message.userKey, message.userArgs ?? {});
  }
  return message.user;
};

const resolveMessageText = (message: Message) => {
  if (message.content.type === 'system') {
    return t(message.content.key, message.content.args ?? {});
  }
  return message.content.text;
};

const getMessageTime = (message: Message) => formatMessageTime(message.createdAt, locale.value);

const formatFileSize = (bytes: number) => {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return '0 B';
  }
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  const precision = size < 10 && unitIndex > 0 ? 1 : 0;
  return `${size.toFixed(precision)} ${units[unitIndex]}`;
};

const updatePinnedState = () => {
  if (!listRef.value) return;
  const threshold = 120;
  const distanceFromBottom = listRef.value.scrollHeight - listRef.value.scrollTop - listRef.value.clientHeight;
  isPinnedToBottom.value = distanceFromBottom < threshold;
};

const scrollToBottom = () => {
  if (!listRef.value) return;
  listRef.value.scrollTop = listRef.value.scrollHeight;
};

const handleJumpToLatest = () => {
  scrollToBottom();
  isPinnedToBottom.value = true;
};

watch(
  [() => messages.value.length, isTyping],
  async () => {
    await nextTick();
    if (isPinnedToBottom.value) {
      scrollToBottom();
    }
  }
);

onMounted(() => {
  updatePinnedState();
  listRef.value?.addEventListener('scroll', updatePinnedState, { passive: true });
  scrollToBottom();
});

onBeforeUnmount(() => {
  listRef.value?.removeEventListener('scroll', updatePinnedState);
});
</script>
