<!-- [2026-01-23 00:59] 目的: 统一消息渲染与滚动体验，保证聊天视图在历史加载、输入中与附件展示时一致; 边界: 仅维护展示与本地滚动状态，不负责数据获取或持久化; 设计: 通过按日分组与“贴底”判定减少阅读跳动，并以跳转按钮显式恢复到底部。 -->
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
          'flex gap-5 group -mx-6 px-6 py-2 rounded-2xl transition-[background-color,box-shadow,backdrop-filter] hover:bg-[rgb(var(--color-overlay)_/_0.03)] hover:shadow-[inset_0_0_0_1px_rgb(var(--color-overlay)_/_0.04)] hover:backdrop-blur-[6px]',
          isMe(item.message) ? 'flex-row-reverse' : ''
        ]"
      >
        <div class="mt-1 shrink-0">
          <AvatarBadge
            :avatar="item.message.avatar"
            :label="resolveMessageAuthor(item.message)"
            :class="[
              'w-11 h-11 rounded-[14px] shadow-lg',
              canOpenAvatar(item.message) ? 'cursor-pointer' : 'cursor-default'
            ]"
            @click="handleAvatarClick(item.message)"
          />
        </div>
        <div :class="['flex flex-col flex-1 min-w-0', isMe(item.message) ? 'items-end' : '']">
          <div :class="['flex items-baseline gap-2.5', isMe(item.message) ? 'flex-row-reverse' : '']">
            <span class="text-white font-semibold text-[15px] cursor-pointer hover:underline tracking-tight">{{ resolveMessageAuthor(item.message) }}</span>
            <span class="text-white/30 text-[11px] font-medium">{{ getMessageTime(item.message) }}</span>
          </div>
          <div v-if="isMe(item.message)" class="selectable mt-1 bg-white text-slate-900 message-bubble--me px-5 py-3 rounded-2xl rounded-tr-sm shadow-lg max-w-[80%] text-[15px] leading-relaxed font-medium">
            <template v-for="(part, index) in buildMessageTokens(item.message, false)" :key="index">
              <br v-if="part.type === 'br'" />
              <span v-else>{{ part.value }}</span>
            </template>
          </div>
          <div v-else class="selectable text-white/90 text-[15px] leading-relaxed mt-1 font-light tracking-wide">
            <template v-for="(part, index) in buildMessageTokens(item.message, true)" :key="index">
              <br v-if="part.type === 'br'" />
              <span v-else-if="part.type === 'mention'" class="text-primary bg-primary/10 px-1.5 py-0.5 rounded cursor-pointer hover:bg-primary/20 transition-colors font-medium">{{ part.value }}</span>
              <span v-else>{{ part.value }}</span>
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
// 消息列表组件：负责渲染分组消息、时间分隔与滚动管理。
import { computed, nextTick, onBeforeUnmount, onMounted, ref, toRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Message } from '../types';
import { formatMessageTime, groupMessagesByDay, isMentionToken, splitMentions } from '../utils';
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
  terminalMemberIds?: string[];
}>();
const emit = defineEmits<{
  (e: 'open-roadmap'): void;
  (e: 'load-more'): void;
  (e: 'open-terminal', memberId: string): void;
}>();

const messages = toRef(props, 'messages');
const listRef = ref<HTMLDivElement | null>(null);
const isPinnedToBottom = ref(true);
const terminalMemberIdSet = computed(() => new Set(props.terminalMemberIds ?? []));
let scrollRaf: number | null = null;

const { t, locale } = useI18n();

const groupedItems = computed(() => groupMessagesByDay(messages.value, locale.value));
const TYPEWRITER_INTERVAL_MS = 32;
const TYPEWRITER_CHARS_PER_TICK = 3;
const STREAM_MESSAGE_PREFIX = 'terminal-stream:';
const typewriterDisplay = ref<Record<string, string>>({});
const typewriterState = new Map<string, { target: string; timer: number | null }>();
const isTyping = computed(() => props.isTyping ?? false);
const typingName = computed(() => props.typingName ?? t('members.roles.aiAssistant'));
const typingAvatar = computed(() => props.typingAvatar ?? '');
const hasMore = computed(() => props.hasMore ?? false);
const isLoadingMore = computed(() => props.isLoadingMore ?? false);
const showJumpButton = computed(() => !isPinnedToBottom.value);
const lastMessageSignature = computed(() => {
  const last = messages.value[messages.value.length - 1];
  if (!last) {
    return '';
  }
  if (last.content.type === 'text' && isStreamMessage(last)) {
    const typed = typewriterDisplay.value[last.id];
    if (typed !== undefined) {
      return typed;
    }
  }
  return last.content.type === 'text' ? last.content.text : last.content.key;
});

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
  if (isStreamMessage(message)) {
    const typed = typewriterDisplay.value[message.id];
    if (typed !== undefined) {
      return typed;
    }
  }
  return message.content.text;
};

const canOpenAvatar = (message: Message) =>
  Boolean(message.senderId && terminalMemberIdSet.value.has(message.senderId));

const handleAvatarClick = (message: Message) => {
  if (!message.senderId) {
    return;
  }
  if (!terminalMemberIdSet.value.has(message.senderId)) {
    // [TODO/seekskyworld, 2026-01-27] 非终端好友头像点击逻辑待补充（例如打开资料或私聊）。
    return;
  }
  emit('open-terminal', message.senderId);
};

const buildMessageTokens = (message: Message, highlightMentions: boolean) => {
  const text = resolveMessageText(message) ?? '';
  const lines = text.split('\n');
  const tokens: Array<{ type: 'text' | 'mention' | 'br'; value?: string }> = [];
  lines.forEach((line, index) => {
    if (line) {
      if (highlightMentions) {
        for (const part of splitMentions(line)) {
          if (isMentionToken(part)) {
            tokens.push({ type: 'mention', value: part });
          } else {
            tokens.push({ type: 'text', value: part });
          }
        }
      } else {
        tokens.push({ type: 'text', value: line });
      }
    }
    if (index < lines.length - 1) {
      tokens.push({ type: 'br' });
    }
  });
  return tokens;
};

const isStreamMessage = (message: Message) =>
  message.content.type === 'text' && message.id.startsWith(STREAM_MESSAGE_PREFIX);

const stopTypewriter = (messageId: string) => {
  const state = typewriterState.get(messageId);
  if (state?.timer !== null) {
    window.clearInterval(state.timer);
  }
  typewriterState.delete(messageId);
  if (typewriterDisplay.value[messageId] !== undefined) {
    const next = { ...typewriterDisplay.value };
    delete next[messageId];
    typewriterDisplay.value = next;
  }
};

const ensureTypewriter = (messageId: string, target: string) => {
  const trimmedTarget = target ?? '';
  if (!trimmedTarget) {
    stopTypewriter(messageId);
    return;
  }
  const state = typewriterState.get(messageId) ?? { target: trimmedTarget, timer: null };
  const current = typewriterDisplay.value[messageId] ?? '';
  state.target = trimmedTarget;
  typewriterState.set(messageId, state);
  if (!trimmedTarget.startsWith(current)) {
    typewriterDisplay.value = { ...typewriterDisplay.value, [messageId]: '' };
  }
  if (state.timer !== null) {
    return;
  }
  const timerId = window.setInterval(() => {
    const active = typewriterState.get(messageId);
    if (!active || active.timer !== timerId) {
      window.clearInterval(timerId);
      return;
    }
    const targetText = active.target;
    const shown = typewriterDisplay.value[messageId] ?? '';
    if (!targetText) {
      window.clearInterval(timerId);
      stopTypewriter(messageId);
      return;
    }
    if (shown.length >= targetText.length) {
      window.clearInterval(timerId);
      active.timer = null;
      typewriterState.set(messageId, active);
      return;
    }
    const nextLength = Math.min(targetText.length, shown.length + TYPEWRITER_CHARS_PER_TICK);
    const nextText = targetText.slice(0, nextLength);
    typewriterDisplay.value = { ...typewriterDisplay.value, [messageId]: nextText };
    if (nextText.length >= targetText.length) {
      window.clearInterval(timerId);
      active.timer = null;
      typewriterState.set(messageId, active);
    }
  }, TYPEWRITER_INTERVAL_MS);
  state.timer = timerId;
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
  const target = Math.max(0, listRef.value.scrollHeight - listRef.value.clientHeight);
  listRef.value.scrollTop = target;
};

const scheduleScrollToBottom = () => {
  if (!listRef.value || !isPinnedToBottom.value) return;
  if (scrollRaf !== null) return;
  // 打字机刷新频繁，滚动合并到一帧避免列表抖动。
  scrollRaf = window.requestAnimationFrame(() => {
    scrollRaf = null;
    if (!listRef.value) return;
    const target = Math.max(0, listRef.value.scrollHeight - listRef.value.clientHeight);
    if (Math.abs(listRef.value.scrollTop - target) < 1) {
      return;
    }
    listRef.value.scrollTop = target;
  });
};

const handleJumpToLatest = () => {
  scrollToBottom();
  isPinnedToBottom.value = true;
};

defineExpose({
  jumpToLatest: handleJumpToLatest
});

const streamSnapshot = computed(() =>
  messages.value
    .filter((message) => isStreamMessage(message))
    .map((message) => ({ id: message.id, text: message.content.text }))
);

watch(
  streamSnapshot,
  (next) => {
    const activeIds = new Set(next.map((item) => item.id));
    for (const item of next) {
      ensureTypewriter(item.id, item.text);
    }
    for (const messageId of Array.from(typewriterState.keys())) {
      if (!activeIds.has(messageId)) {
        stopTypewriter(messageId);
      }
    }
  },
  { deep: true }
);

watch(
  [() => messages.value.length, () => lastMessageSignature.value, isTyping],
  async () => {
    await nextTick();
    scheduleScrollToBottom();
  }
);

onMounted(() => {
  updatePinnedState();
  listRef.value?.addEventListener('scroll', updatePinnedState, { passive: true });
  scrollToBottom();
});

onBeforeUnmount(() => {
  listRef.value?.removeEventListener('scroll', updatePinnedState);
  if (scrollRaf !== null) {
    window.cancelAnimationFrame(scrollRaf);
    scrollRaf = null;
  }
  for (const messageId of Array.from(typewriterState.keys())) {
    stopTypewriter(messageId);
  }
});
</script>
