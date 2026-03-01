<template>
  <div class="notification-preview-root">
    <div class="notification-preview-card" @mouseenter="handleHover(true)" @mouseleave="handleHover(false)">
      <header class="notification-preview-header">
        <div class="notification-preview-title">
          <span class="notification-preview-title__text">{{ titleText }}</span>
        </div>
        <span v-if="totalUnread > 0" class="notification-preview-count">
          {{ totalUnread }}
        </span>
      </header>

      <div class="notification-preview-body">
        <div class="notification-preview-list">
          <div v-for="item in items" :key="itemKey(item)" class="notification-preview-item">
            <AvatarBadge
              v-if="item.senderAvatar"
              :avatar="item.senderAvatar"
              :label="item.senderName || item.conversationName"
              class="notification-preview-avatar"
            />
            <div v-else class="notification-preview-avatar notification-preview-avatar--fallback"></div>

            <div class="notification-preview-detail">
              <div class="notification-preview-meta-row">
                <div class="notification-preview-meta-left">
                  <span v-if="resolveTagLabel(item)" class="notification-preview-tag">
                    {{ resolveTagLabel(item) }}
                  </span>
                  <span class="notification-preview-meta-text">
                    {{ resolveMetaText(item) }}
                  </span>
                </div>
                <span
                  v-if="item.conversationUnread > 0"
                  class="notification-preview-count notification-preview-count--secondary notification-preview-count--meta"
                >
                  {{ formatUnread(item.conversationUnread) }}
                </span>
              </div>
              <div class="notification-preview-main-row">
                <div
                  class="notification-preview-content"
                  :class="{ 'notification-preview-content--inactive': !canOpenConversation(item) }"
                  @click="handleOpenConversation(item)"
                >
                  <div class="notification-preview-text">
                    <span class="notification-preview-preview">
                      {{ item.preview }}
                    </span>
                  </div>
                </div>
                <div class="notification-preview-item-actions">
                  <button
                    v-if="canOpenTerminal(item)"
                    type="button"
                    class="notification-preview-action notification-preview-action--compact"
                    @click.stop="handleOpenTerminal(item)"
                  >
                    打开终端
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <footer class="notification-preview-footer">
        <button
          v-if="canOpenAnyTerminal"
          type="button"
          class="notification-preview-action"
          @click="handleOpenAllTerminals"
        >
          打开全部终端
        </button>
        <div class="notification-preview-actions">
          <button type="button" class="notification-preview-action" @click="handleIgnoreAll">忽略全部</button>
          <button type="button" class="notification-preview-action" @click="handleViewAll">查看全部</button>
        </div>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
// 通知悬浮窗视图：展示托盘 hover 预览并提供忽略/查看入口。
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import AvatarBadge from '@/shared/components/AvatarBadge.vue';
import { CURRENT_USER_ID } from '@/features/chat/data';
import {
  getNotificationState,
  requestIgnoreAll,
  requestOpenConversation,
  requestOpenTerminal,
  requestViewAll,
  setNotificationPreviewHovered,
  type NotificationPreviewItem,
  type NotificationPreviewState
} from '@/shared/tauri/notifications';

const preview = ref<NotificationPreviewState>({
  title: '',
  totalUnread: 0,
  items: []
});
let unlistenPreview: (() => void) | null = null;

const titleText = computed(() => preview.value.title?.trim() || 'Owner');
const items = computed(() => preview.value.items ?? []);
const totalUnread = computed(() => preview.value.totalUnread ?? 0);
const canOpenAnyTerminal = computed(() => items.value.some((item) => Boolean(item.senderCanOpenTerminal)));
const formatUnread = (value: number) => (value > 99 ? '99+' : String(value));

const resolveTagLabel = (item: NotificationPreviewItem) => {
  if (item.conversationType === 'channel') {
    return '群聊';
  }
  if (item.conversationType === 'dm') {
    return '私聊';
  }
  return '';
};

const resolveMetaText = (item: NotificationPreviewItem) => {
  const sender = item.senderName?.trim() || item.conversationName?.trim() || '成员';
  if (item.conversationType === 'channel') {
    const groupName = item.conversationName?.trim() || '群聊';
    const count = item.memberCount && item.memberCount > 0 ? ` (${item.memberCount})` : '';
    if (sender && sender !== groupName) {
      return `${groupName}${count} · ${sender}`;
    }
    return `${groupName}${count}`;
  }
  return sender;
};

const handleHover = (hovered: boolean) => {
  void setNotificationPreviewHovered(hovered);
};

const handleIgnoreAll = () => {
  void requestIgnoreAll(CURRENT_USER_ID);
};

const handleViewAll = () => {
  void requestViewAll(CURRENT_USER_ID, items.value);
};

const itemKey = (item: NotificationPreviewItem) =>
  `${item.workspaceId ?? 'workspace'}:${item.conversationId ?? 'conversation'}:${item.senderId ?? 'sender'}:${item.lastMessageAt ?? 0}`;

const canOpenConversation = (item: NotificationPreviewItem) =>
  Boolean(item.workspaceId?.trim()) && Boolean(item.conversationId?.trim());

const canOpenTerminal = (item: NotificationPreviewItem) =>
  Boolean(item.senderCanOpenTerminal) &&
  Boolean(item.senderId?.trim()) &&
  Boolean(item.workspaceId?.trim()) &&
  Boolean(item.conversationId?.trim());

const handleOpenTerminal = (item: NotificationPreviewItem) => {
  const workspaceId = item.workspaceId?.trim() ?? '';
  const conversationId = item.conversationId?.trim() ?? '';
  const senderId = item.senderId?.trim() ?? '';
  if (!workspaceId || !conversationId || !senderId) {
    return;
  }
  void requestOpenTerminal({ workspaceId, conversationId, senderId });
};

const handleOpenAllTerminals = async () => {
  if (!items.value.length) {
    return;
  }
  const ordered = [...items.value].sort(
    (left, right) => (left.lastMessageAt ?? 0) - (right.lastMessageAt ?? 0)
  );
  for (const item of ordered) {
    if (!canOpenTerminal(item)) {
      continue;
    }
    await requestOpenTerminal({
      workspaceId: item.workspaceId?.trim() ?? '',
      conversationId: item.conversationId?.trim() ?? '',
      senderId: item.senderId?.trim() ?? ''
    });
  }
};

const handleOpenConversation = (item: NotificationPreviewItem) => {
  const workspaceId = item.workspaceId?.trim() ?? '';
  const conversationId = item.conversationId?.trim() ?? '';
  if (!workspaceId || !conversationId) {
    return;
  }
  void requestOpenConversation({ workspaceId, conversationId });
};

const setRootClass = (enabled: boolean) => {
  if (typeof document === 'undefined') {
    return;
  }
  const rootClass = 'notification-preview-body';
  document.documentElement.classList.toggle(rootClass, enabled);
  document.body.classList.toggle(rootClass, enabled);
  const appRoot = document.getElementById('app');
  if (appRoot) {
    appRoot.classList.toggle(rootClass, enabled);
  }
};

onMounted(async () => {
  setRootClass(true);
  try {
    preview.value = await getNotificationState();
  } catch (error) {
    console.error('Failed to load notification preview state.', error);
  }
  try {
    unlistenPreview = await listen<NotificationPreviewState>('notification-preview-updated', (event) => {
      preview.value = event.payload;
    });
  } catch (error) {
    console.error('Failed to listen to notification preview updates.', error);
  }
});

onBeforeUnmount(() => {
  if (unlistenPreview) {
    unlistenPreview();
    unlistenPreview = null;
  }
  setRootClass(false);
  void setNotificationPreviewHovered(false);
});
</script>

<style scoped>
.notification-preview-root {
  display: flex;
  width: 100%;
  height: 100%;
  padding: 0;
  box-sizing: border-box;
  background: transparent;
  border-radius: 16px;
  overflow: hidden;
  pointer-events: none;
}

:global(html.notification-preview-body),
:global(body.notification-preview-body),
:global(#app.notification-preview-body) {
  border-radius: 16px;
  overflow: hidden;
  background: transparent;
}

.notification-preview-card {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  padding: 14px;
  border-radius: 16px;
  background-clip: padding-box;
  -webkit-clip-path: inset(0 round 16px);
  clip-path: inset(0 round 16px);
  border: 1px solid rgb(var(--color-border) / 0.12);
  background: linear-gradient(160deg, rgb(var(--color-panel) / 0.92), rgb(var(--color-background) / 0.96));
  box-shadow: var(--shadow-modal);
  backdrop-filter: blur(18px);
  -webkit-backdrop-filter: blur(18px);
  overflow: hidden;
  gap: 12px;
  pointer-events: auto;
}

.notification-preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 22px;
}

.notification-preview-title {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.notification-preview-title__text {
  font-size: 13px;
  font-weight: 600;
  color: rgb(var(--color-text) / 0.9);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 200px;
}

.notification-preview-title__dot {
  font-size: 12px;
  color: rgb(var(--color-text) / 0.4);
}

.notification-preview-title__sub {
  font-size: 12px;
  font-weight: 600;
  color: rgb(var(--color-text) / 0.75);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 160px;
}

.notification-preview-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgb(209 101 91 / 0.95);
  color: #fff !important;
  line-height: 1;
  min-height: 18px;
}

.notification-preview-count--secondary {
  background: rgb(209 101 91 / 0.95);
  color: #fff !important;
}

.notification-preview-count--meta {
  align-self: flex-end;
}

.notification-preview-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
}

.notification-preview-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow: hidden;
}

.notification-preview-item {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 10px;
  height: 80px;
}

.notification-preview-avatar {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  flex-shrink: 0;
}

.notification-preview-avatar--fallback {
  background: rgb(var(--color-panel-strong));
  border: 1px solid rgb(var(--color-border) / 0.12);
}

.notification-preview-detail {
  display: grid;
  grid-template-rows: 18px 1fr;
  gap: 6px;
  min-width: 0;
}

.notification-preview-meta-row {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  min-width: 0;
}

.notification-preview-meta-left {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  flex: 1;
}

.notification-preview-meta-text {
  font-size: 11px;
  font-weight: 600;
  color: rgb(var(--color-text) / 0.7);
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.notification-preview-main-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
  align-items: flex-end;
  min-width: 0;
}

.notification-preview-content {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  min-width: 0;
  box-sizing: border-box;
  height: 48px;
  overflow: hidden;
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 999px;
  border: 1px solid rgb(var(--color-border) / 0.12);
  background: rgb(var(--color-panel-soft) / 0.6);
  transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
  align-self: flex-end;
}

.notification-preview-content--inactive {
  cursor: default;
  pointer-events: none;
  opacity: 0.7;
}

.notification-preview-content:hover {
  background: rgb(var(--color-overlay) / 0.18);
  color: rgb(var(--color-text) / 0.9);
  border-color: rgb(var(--color-border) / 0.2);
}

.notification-preview-tag {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 999px;
  background: rgb(var(--color-overlay) / 0.16);
  color: rgb(var(--color-text) / 0.7);
  line-height: 1.2;
  white-space: nowrap;
}

.notification-preview-text {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  font-size: 13px;
  color: rgb(var(--color-text) / 0.85);
  line-height: 1.2;
  white-space: normal;
  height: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.notification-preview-preview {
  display: block;
  max-height: 100%;
  min-width: 0;
  color: rgb(var(--color-text) / 0.8);
  overflow: hidden;
}

.notification-preview-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 28px;
}

.notification-preview-item-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: center;
  gap: 6px;
  align-self: center;
}

.notification-preview-actions {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.notification-preview-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  border: 1px solid rgb(var(--color-border) / 0.12);
  background: rgb(var(--color-panel-soft) / 0.6);
  color: rgb(var(--color-text) / 0.7);
  font-size: 12px;
  font-weight: 600;
  line-height: 1;
  height: 28px;
  padding: 6px 10px;
  border-radius: 999px;
  transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
}

.notification-preview-action--compact {
  height: 24px;
  padding: 4px 8px;
  font-size: 11px;
}

.notification-preview-action:hover {
  background: rgb(var(--color-overlay) / 0.18);
  color: rgb(var(--color-text) / 0.9);
  border-color: rgb(var(--color-border) / 0.2);
}

</style>
