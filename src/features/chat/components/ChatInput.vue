<template>
  <div class="p-6 pb-8">
    <div v-if="quickPrompts.length" class="mb-3 flex items-center gap-2 overflow-x-auto no-scrollbar">
      <button
        v-for="prompt in quickPrompts"
        :key="prompt"
        type="button"
        class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10 text-[12px] text-white/70 hover:text-white hover:bg-white/10 transition-colors whitespace-nowrap"
        @click="applyPrompt(prompt)"
      >
        {{ prompt }}
      </button>
    </div>
    <form
      ref="formRef"
      class="relative bg-white/5 backdrop-blur-md rounded-2xl shadow-lg flex items-end p-2 gap-3 border border-white/5 focus-within:border-primary/50 focus-within:ring-1 focus-within:ring-primary/25 focus-within:bg-white/[0.07] transition-all duration-300 group"
      @submit.prevent="emitSend"
    >
      <button type="button" class="w-10 h-10 rounded-xl hover:bg-white/10 text-white/70 flex items-center justify-center shrink-0 transition-colors mb-0.5">
        <span class="material-symbols-outlined text-[22px]">add_circle</span>
      </button>
      <div ref="scrollRef" class="flex-1 min-h-[44px] max-h-40 overflow-y-auto py-2.5" @scroll="scheduleMentionAnchorUpdate">
        <textarea
          ref="inputRef"
          :value="modelValue"
          class="w-full bg-transparent border-none p-0 text-white placeholder-white/30 focus:ring-0 outline-none focus:outline-none focus-visible:outline-none focus-visible:ring-0 text-[15px] font-light resize-none min-h-[24px]"
          :placeholder="placeholder"
          :maxlength="maxLength"
          spellcheck="false"
          rows="1"
          @input="handleInput"
          @compositionstart="isComposing = true"
          @compositionend="isComposing = false"
          @click="updateCursor"
          @keydown="handleKeydown"
          @keyup="updateCursor"
        ></textarea>
      </div>
      <div
        v-if="showMentionSuggestions"
        ref="mentionDropdownRef"
        class="absolute w-64 max-h-64 rounded-xl glass-modal bg-panel-strong/95 border border-white/10 shadow-2xl overflow-y-auto custom-scrollbar z-20 animate-in fade-in zoom-in-95 duration-200 ring-1 ring-white/10"
        :style="mentionDropdownStyle"
      >
        <button
          v-for="(member, index) in mentionSuggestions"
          :key="member.id"
          type="button"
          :class="[
            'w-full px-3 py-2 rounded-lg flex items-center gap-3 text-left transition-colors',
            index === activeMentionIndex
              ? 'bg-white/[0.12] ring-1 ring-white/10 hover:bg-white/[0.16]'
              : 'hover:bg-white/10 hover:ring-1 hover:ring-white/10'
          ]"
          @click="applyMention(member)"
        >
          <AvatarBadge
            :avatar="member.avatar"
            :label="member.name"
            class="w-8 h-8 rounded-full shadow-md"
          />
          <div class="min-w-0">
            <div class="text-xs font-semibold text-white truncate">@{{ member.name }}</div>
            <div class="text-[10px] text-white/40 truncate">{{ getMemberRole(member) }}</div>
          </div>
        </button>
      </div>
      <div class="flex items-center gap-1 shrink-0 mb-0.5">
        <button type="button" class="w-10 h-10 rounded-xl hover:bg-white/10 text-white/70 hover:text-white transition-colors flex items-center justify-center">
          <span class="material-symbols-outlined text-[22px]">sentiment_satisfied</span>
        </button>
        <div class="w-[1px] h-6 bg-white/10 mx-1"></div>
        <button
          v-if="isGenerating"
          type="button"
          class="h-10 px-4 bg-red-500/10 text-red-300 border border-red-500/30 text-sm font-semibold rounded-xl flex items-center gap-2 transition-all hover:bg-red-500/20 active:scale-95"
          @click="emit('stop')"
        >
          <span class="material-symbols-outlined text-[18px]">stop_circle</span>
          {{ t('chat.input.stop') }}
        </button>
        <button
          v-else
          type="submit"
          :disabled="!modelValue.trim()"
          :class="[
            'h-10 px-5 bg-primary hover:bg-primary-hover text-on-primary text-sm font-bold rounded-xl shadow-glow flex items-center gap-2 transition-all active:scale-95 transform',
            !modelValue.trim() ? 'opacity-50 cursor-not-allowed' : ''
          ]"
        >
          {{ t('chat.input.send') }}
        </button>
      </div>
    </form>
    <div class="mt-2 flex items-center justify-between px-2 text-[11px] text-white/30 font-medium tracking-wide">
      <span>{{ t('chat.input.hint') }}</span>
      <span>{{ modelValue.length }}/{{ maxLength }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, toRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Member, MessageMentionsPayload } from '../types';
import AvatarBadge from '@/shared/components/AvatarBadge.vue';

const props = defineProps<{
  modelValue: string;
  maxLength?: number;
  isGenerating?: boolean;
  quickPrompts?: string[];
  placeholder?: string;
  members?: Member[];
}>();
const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'send', payload: MessageMentionsPayload): void;
  (e: 'stop'): void;
}>();

const { t } = useI18n();
const formRef = ref<HTMLFormElement | null>(null);
const scrollRef = ref<HTMLDivElement | null>(null);
const inputRef = ref<HTMLTextAreaElement | null>(null);
const mentionDropdownRef = ref<HTMLDivElement | null>(null);
const maxLength = computed(() => props.maxLength ?? 1200);
const isGenerating = computed(() => props.isGenerating ?? false);
const quickPrompts = computed(() => props.quickPrompts ?? []);
const placeholder = computed(() => props.placeholder ?? t('chat.input.placeholder', { channel: '' }));
const isComposing = ref(false);
const members = computed(() => props.members ?? []);
const cursorIndex = ref(0);
const activeMentionIndex = ref(0);
const mentionAnchor = ref({ left: 0, top: 0 });
const mentionTokens = ref<Array<{ id: string; name: string }>>([]);
const mentionAllPattern = /(^|\s)@all(\s|$)/i;

const handleKeydown = (event: KeyboardEvent) => {
  if (showMentionSuggestions.value) {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      if (!mentionSuggestions.value.length) return;
      const direction = event.key === 'ArrowDown' ? 1 : -1;
      activeMentionIndex.value =
        (activeMentionIndex.value + direction + mentionSuggestions.value.length) % mentionSuggestions.value.length;
      return;
    }
    if (event.key === 'Enter' || event.key === 'Tab') {
      event.preventDefault();
      const selected = mentionSuggestions.value[activeMentionIndex.value];
      if (selected) {
        applyMention(selected);
      }
      return;
    }
  }
  if (event.key === 'Enter' && !event.shiftKey && !isComposing.value) {
    event.preventDefault();
    if (isGenerating.value) return;
    emitSend();
  }
};

const modelValue = toRef(props, 'modelValue');

const resizeInput = () => {
  if (!inputRef.value) return;
  inputRef.value.style.height = 'auto';
  inputRef.value.style.height = `${Math.min(inputRef.value.scrollHeight, 160)}px`;
};

const focusInput = () => {
  if (!inputRef.value) return;
  inputRef.value.focus();
  const length = inputRef.value.value.length;
  inputRef.value.setSelectionRange(length, length);
};

const applyPrompt = (prompt: string) => {
  const nextValue = modelValue.value.trim()
    ? `${modelValue.value.trim()}\n${prompt}`
    : prompt;
  emit('update:modelValue', nextValue);
  nextTick(() => {
    inputRef.value?.focus();
  });
};

const updateCursor = () => {
  if (!inputRef.value) return;
  cursorIndex.value = inputRef.value.selectionStart ?? inputRef.value.value.length;
  scheduleMentionAnchorUpdate();
};

const buildSendPayload = (): MessageMentionsPayload => ({
  mentionIds: mentionTokens.value.map((token) => token.id),
  mentionAll: mentionAllPattern.test(modelValue.value)
});

const emitSend = () => {
  emit('send', buildSendPayload());
};

const syncMentionTokens = (value: string) => {
  if (!mentionTokens.value.length) {
    return;
  }
  const lower = value.toLowerCase();
  mentionTokens.value = mentionTokens.value.filter((token) => lower.includes(`@${token.name.toLowerCase()}`));
};

const handleInput = (event: Event) => {
  const target = event.target as HTMLTextAreaElement;
  emit('update:modelValue', target.value);
  cursorIndex.value = target.selectionStart ?? target.value.length;
  scheduleMentionAnchorUpdate();
  syncMentionTokens(target.value);
};

const mentionState = computed(() => {
  if (isComposing.value) return null;
  const text = modelValue.value;
  const cursor = cursorIndex.value;
  const prefix = text.slice(0, cursor);
  const match = prefix.match(/(^|\s)@([^\s@]*)$/);
  if (!match) return null;
  const query = match[2] ?? '';
  const startIndex = prefix.length - query.length - 1;
  return { query, startIndex };
});

const mentionSuggestions = computed(() => {
  if (!mentionState.value) return [];
  const query = mentionState.value.query.trim().toLowerCase();
  const options = members.value.filter((member) => member.name.toLowerCase().startsWith(query));
  return options.slice(0, 6);
});

const showMentionSuggestions = computed(() => mentionState.value !== null && mentionSuggestions.value.length > 0);

const mentionDropdownStyle = computed(() => ({
  left: `${mentionAnchor.value.left}px`,
  top: `${mentionAnchor.value.top}px`
}));

const getMemberRole = (member: Member) => {
  if (member.roleKey) return t(member.roleKey);
  if (member.role) return member.role;
  return t(`members.roles.${member.roleType}`);
};

const applyMention = (member: Member) => {
  if (!mentionState.value) return;
  if (!mentionTokens.value.some((token) => token.id === member.id)) {
    mentionTokens.value.push({ id: member.id, name: member.name });
  }
  const mention = `@${member.name}`;
  const before = modelValue.value.slice(0, mentionState.value.startIndex);
  const after = modelValue.value.slice(cursorIndex.value);
  const trimmedAfter = after.replace(/^[\s\n]+/, '');
  const nextValue = `${before}${mention} ${trimmedAfter}`;
  emit('update:modelValue', nextValue);
  nextTick(() => {
    if (!inputRef.value) return;
    const nextCursor = before.length + mention.length + 1;
    inputRef.value.focus();
    inputRef.value.setSelectionRange(nextCursor, nextCursor);
    cursorIndex.value = nextCursor;
    updateMentionAnchor();
  });
};

const registerMention = (member: Member) => {
  if (!mentionTokens.value.some((token) => token.id === member.id)) {
    mentionTokens.value.push({ id: member.id, name: member.name });
  }
};

const scheduleMentionAnchorUpdate = () => {
  if (!showMentionSuggestions.value) return;
  nextTick(() => {
    updateMentionAnchor();
  });
};

const updateMentionAnchor = () => {
  if (!inputRef.value || !formRef.value || !mentionState.value) return;
  const anchorIndex = Math.min(Math.max(mentionState.value.startIndex, 0), inputRef.value.value.length);
  const caret = getCaretCoordinates(inputRef.value, anchorIndex);
  if (!caret) return;
  const inputRect = inputRef.value.getBoundingClientRect();
  const formRect = formRef.value.getBoundingClientRect();
  const scrollOffsetY = scrollRef.value?.scrollTop ?? 0;
  const scrollOffsetX = inputRef.value.scrollLeft ?? 0;
  const absLeft = inputRect.left + caret.left - scrollOffsetX;
  const absTop = inputRect.top + caret.top - scrollOffsetY;
  const dropdownWidth = mentionDropdownRef.value?.offsetWidth ?? 256;
  const dropdownHeight = mentionDropdownRef.value?.offsetHeight ?? 256;
  const viewportPadding = 8;
  const clampedLeft = Math.max(viewportPadding, Math.min(absLeft, window.innerWidth - dropdownWidth - viewportPadding));
  const clampedTop = Math.max(viewportPadding, Math.min(absTop - dropdownHeight, window.innerHeight - dropdownHeight - viewportPadding));
  mentionAnchor.value = {
    left: clampedLeft - formRect.left,
    top: clampedTop - formRect.top
  };
};

const getCaretCoordinates = (input: HTMLTextAreaElement, position: number) => {
  const style = window.getComputedStyle(input);
  const mirror = document.createElement('div');
  mirror.style.position = 'absolute';
  mirror.style.visibility = 'hidden';
  mirror.style.whiteSpace = 'pre-wrap';
  mirror.style.wordWrap = 'break-word';
  mirror.style.boxSizing = 'border-box';
  mirror.style.padding = style.padding;
  mirror.style.border = style.border;
  mirror.style.fontFamily = style.fontFamily;
  mirror.style.fontSize = style.fontSize;
  mirror.style.lineHeight = style.lineHeight;
  mirror.style.letterSpacing = style.letterSpacing;
  mirror.style.width = `${input.clientWidth}px`;
  mirror.textContent = input.value.slice(0, position);
  const marker = document.createElement('span');
  marker.textContent = '\u200b';
  mirror.appendChild(marker);
  document.body.appendChild(mirror);
  const coords = { left: marker.offsetLeft, top: marker.offsetTop, height: marker.offsetHeight };
  document.body.removeChild(mirror);
  return coords;
};

watch(modelValue, async () => {
  if (!modelValue.value.trim()) {
    mentionTokens.value = [];
  } else {
    syncMentionTokens(modelValue.value);
  }
  await nextTick();
  resizeInput();
  scheduleMentionAnchorUpdate();
});

watch(mentionSuggestions, () => {
  activeMentionIndex.value = 0;
  scheduleMentionAnchorUpdate();
});

watch(showMentionSuggestions, (visible) => {
  if (visible) {
    scheduleMentionAnchorUpdate();
  }
});

onMounted(() => {
  resizeInput();
});

defineExpose({ focus: focusInput, registerMention });
</script>
