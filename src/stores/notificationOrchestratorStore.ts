// 通知编排层：统一未读 badge 计算与 Tauri 更新调用，避免跨模块互相依赖。
import { computed, ref, watch } from 'vue';
import { defineStore, storeToRefs } from 'pinia';
import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { useChatStore } from '@/features/chat/chatStore';
import { useSettingsStore } from '@/features/global/settingsStore';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useNavigationStore } from '@/stores/navigationStore';
import { getCurrentWindowLabel } from '@/shared/tauri/windows';
import { renderAvatarPngBytes } from '@/shared/utils/avatarRender';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import { useTerminalMemberStore } from '@/features/terminal/terminalMemberStore';
import {
  updateNotificationState,
  setNotificationActiveWindow,
  type NotificationPreviewItem,
  type NotificationPreviewState
} from '@/shared/tauri/notifications';

const MAIN_WINDOW_LABEL = 'main';
const VIEW_TERMINAL = 'terminal';
const VIEW_WORKSPACE_SELECTION = 'workspace-selection';
const VIEW_NOTIFICATION_PREVIEW = 'notification-preview';
const NOTIFICATION_OPEN_CONVERSATION_EVENT = 'notification-open-conversation';
const NOTIFICATION_OPEN_TERMINAL_EVENT = 'notification-open-terminal';
const PREVIEW_MAX_ITEMS = 6;
const NOTIFICATION_IGNORE_ALL_BOOTSTRAP = '__GOLUTRA_NOTIFICATION_IGNORE_ALL__';
const NOTIFICATION_OPEN_TERMINAL_BOOTSTRAP = '__GOLUTRA_NOTIFICATION_OPEN_TERMINAL__';

type NotificationOpenConversationPayload = { conversationId?: string };
type NotificationOpenTerminalPayload = { conversationId?: string; senderId?: string; workspaceId?: string };

const isMainWindowLabel = (label: string | null) => {
  if (!label) {
    return false;
  }
  return label === MAIN_WINDOW_LABEL || label.startsWith(`${MAIN_WINDOW_LABEL}-`);
};

const resolveWindowView = () => {
  if (typeof window === 'undefined') {
    return null;
  }
  const queryView = new URLSearchParams(window.location.search).get('view');
  const globalView = (window as typeof window & { __GOLUTRA_VIEW__?: string }).__GOLUTRA_VIEW__ ?? null;
  return queryView ?? globalView ?? null;
};

const normalizeUnreadCount = (value: number) => {
  if (!Number.isFinite(value) || value <= 0) {
    return 0;
  }
  return Math.floor(value);
};

const buildPreviewFallback = (): NotificationPreviewState => ({
  title: '',
  totalUnread: 0,
  items: []
});

const resolvePreviewText = (preview: string | undefined, attachment?: { type: string; mimeType?: string }) => {
  if (attachment?.type === 'image') {
    return attachment.mimeType === 'image/gif' ? '[动画表情]' : '[图片]';
  }
  if (attachment?.type) {
    return '[文件]';
  }
  return preview ?? '';
};

export const useNotificationOrchestratorStore = defineStore('notification-orchestrator', () => {
  const chatStore = useChatStore();
  const settingsStore = useSettingsStore();
  const workspaceStore = useWorkspaceStore();
  const projectStore = useProjectStore();
  const terminalMemberStore = useTerminalMemberStore();
  const navigationStore = useNavigationStore();
  const { totalUnreadCount, isReady, conversations } = storeToRefs(chatStore);
  const {
    markAllConversationsRead,
    markConversationRead,
    getConversationTitle,
    ensureConversationLatestMessage,
    currentUser
  } = chatStore;
  const { settings } = storeToRefs(settingsStore);
  const { currentWorkspace } = storeToRefs(workspaceStore);
  const { members } = storeToRefs(projectStore);
  const { openMemberTerminal } = terminalMemberStore;
  const { setActiveTab } = navigationStore;

  const resolvedView = resolveWindowView();
  const windowLabel = ref<string | null>(getCurrentWindowLabel());
  const canPublish = computed(
    () =>
      isTauri() &&
      isMainWindowLabel(windowLabel.value) &&
      resolvedView !== VIEW_TERMINAL &&
      resolvedView !== VIEW_WORKSPACE_SELECTION &&
      resolvedView !== VIEW_NOTIFICATION_PREVIEW
  );

  const lastSentSignature = ref<string | null>(null);
  const lastAvatarKey = ref<string | null>(null);
  const lastAvatarBytes = ref<number[] | null>(null);
  const pendingOpenConversationId = ref<string | null>(null);
  const pendingIgnoreAll = ref(false);
  const pendingOpenTerminalPayloads = ref<NotificationOpenTerminalPayload[]>([]);
  const terminalOpenChains = new Map<string, Promise<void>>();
  let listenersReady = false;
  let focusListenerReady = false;

  const resolveOwnerName = () => {
    const displayName = settings.value.account.displayName.trim();
    if (displayName) {
      return displayName;
    }
    return currentUser?.value?.name ?? 'Owner';
  };

  const resolveUnreadConversations = () => {
    return conversations.value
      .filter((conversation) => (conversation.unreadCount ?? 0) > 0)
      .sort((left, right) => (right.lastMessageAt ?? 0) - (left.lastMessageAt ?? 0))
      .slice(0, PREVIEW_MAX_ITEMS);
  };

  const ensurePreviewMessages = async (conversationIds: string[]) => {
    if (conversationIds.length === 0) {
      return;
    }
    await Promise.all(
      conversationIds.map((conversationId) => ensureConversationLatestMessage(conversationId))
    );
  };

  const resolveWindowLabel = () => {
    const label = getCurrentWindowLabel();
    if (label) {
      return label;
    }
    if (resolvedView === null && isTauri()) {
      return MAIN_WINDOW_LABEL;
    }
    return null;
  };

  const resolveIgnoreAllBootstrap = () => {
    if (typeof window === 'undefined') {
      return;
    }
    const pending = (window as typeof window & Record<string, unknown>)[NOTIFICATION_IGNORE_ALL_BOOTSTRAP];
    if (!pending) {
      return;
    }
    pendingIgnoreAll.value = true;
    try {
      delete (window as typeof window & Record<string, unknown>)[NOTIFICATION_IGNORE_ALL_BOOTSTRAP];
    } catch {
      // ignore
    }
  };

  const resolveOpenTerminalBootstrap = () => {
    if (typeof window === 'undefined') {
      return;
    }
    const payloads = (window as typeof window & Record<string, unknown>)[NOTIFICATION_OPEN_TERMINAL_BOOTSTRAP];
    if (!Array.isArray(payloads) || payloads.length === 0) {
      return;
    }
    pendingOpenTerminalPayloads.value = payloads
      .filter((item): item is NotificationOpenTerminalPayload => Boolean(item && typeof item === 'object'))
      .map((item) => ({
        workspaceId: typeof item.workspaceId === 'string' ? item.workspaceId : undefined,
        conversationId: typeof item.conversationId === 'string' ? item.conversationId : undefined,
        senderId: typeof item.senderId === 'string' ? item.senderId : undefined
      }))
      .filter((item) => item.workspaceId && item.conversationId && item.senderId);
    try {
      delete (window as typeof window & Record<string, unknown>)[NOTIFICATION_OPEN_TERMINAL_BOOTSTRAP];
    } catch {
      // ignore
    }
  };

  const pushNotificationState = async (count: number) => {
    if (!windowLabel.value) {
      windowLabel.value = resolveWindowLabel();
    }
    if (!canPublish.value || !windowLabel.value) {
      lastSentSignature.value = null;
      lastAvatarKey.value = null;
      lastAvatarBytes.value = null;
      return;
    }

    const preview = buildPreviewFallback();
    preview.title = resolveOwnerName();
    preview.totalUnread = count;
    const unreadConversations = count > 0 ? resolveUnreadConversations() : [];
    await ensurePreviewMessages(unreadConversations.map((conversation) => conversation.id));
    const workspaceId = currentWorkspace.value?.id ?? null;
    const workspaceName = currentWorkspace.value?.name ?? null;
    const workspacePath = currentWorkspace.value?.path ?? null;
    const previewItems: NotificationPreviewItem[] = unreadConversations.map((conversation) => {
      const senderId = conversation.lastMessageSenderId ?? null;
      const sender = senderId
        ? members.value.find((candidate) => candidate.id === senderId)
        : undefined;
      return {
        workspaceId,
        workspaceName,
        workspacePath,
        conversationId: conversation.id,
        conversationName: getConversationTitle(conversation),
        conversationUnread: conversation.unreadCount ?? 0,
        conversationType: conversation.type,
        memberCount:
          conversation.type === 'channel' && conversation.memberIds.length > 0
            ? conversation.memberIds.length
            : null,
        senderId,
        senderName: conversation.lastMessageSenderName ?? null,
        senderAvatar: conversation.lastMessageSenderAvatar ?? null,
        senderCanOpenTerminal: Boolean(
          sender && hasTerminalConfig(sender.terminalType, sender.terminalCommand)
        ),
        preview: resolvePreviewText(
          conversation.lastMessagePreview,
          conversation.lastMessageAttachment ?? undefined
        ),
        lastMessageAt: conversation.lastMessageAt ?? 0
      };
    });
    preview.items = previewItems;

    const avatarKey = preview.items[0]?.senderAvatar ?? '';
    let avatarBytes = lastAvatarBytes.value;
    if (avatarKey) {
      const shouldRender = avatarKey !== lastAvatarKey.value || !lastAvatarBytes.value;
      if (shouldRender) {
        try {
          avatarBytes = await renderAvatarPngBytes(avatarKey);
          if (import.meta.env.DEV) {
            console.info('[notifications] tray avatar bytes', {
              avatarKey,
              byteLength: avatarBytes?.length ?? 0
            });
          }
          if (avatarBytes) {
            lastAvatarKey.value = avatarKey;
            lastAvatarBytes.value = avatarBytes;
          } else {
            avatarBytes = null;
            lastAvatarKey.value = null;
            lastAvatarBytes.value = null;
          }
        } catch (error) {
          avatarBytes = null;
          lastAvatarKey.value = null;
          lastAvatarBytes.value = null;
          console.error('Failed to render tray avatar.', error);
        }
      }
    } else {
      avatarBytes = null;
      lastAvatarKey.value = null;
      lastAvatarBytes.value = null;
    }

    const signature = [
      preview.title,
      preview.totalUnread,
      preview.items
        .map((item) =>
          [
            item.workspaceId ?? '',
            item.conversationId ?? '',
            item.conversationName ?? '',
            item.conversationUnread,
            item.conversationType ?? '',
            item.memberCount ?? 0,
            item.senderId ?? '',
            item.senderName ?? '',
            item.senderAvatar ?? '',
            item.senderCanOpenTerminal ?? false,
            item.preview,
            item.lastMessageAt ?? 0
          ].join(',')
        )
        .join(';')
    ].join('|');
    if (signature === lastSentSignature.value) {
      return;
    }
    lastSentSignature.value = signature;
    try {
      await updateNotificationState({
        windowLabel: windowLabel.value,
        unreadCount: count,
        ...preview,
        avatarPng: avatarBytes
      });
    } catch (error) {
      lastSentSignature.value = null;
      console.error('Failed to update notification badge.', error);
    }
  };

  const activatePublisherWindow = async (reason: string) => {
    if (!windowLabel.value) {
      windowLabel.value = resolveWindowLabel();
    }
    if (!canPublish.value || !windowLabel.value) {
      return;
    }
    try {
      const activated = await setNotificationActiveWindow(windowLabel.value);
      if (!activated) {
        return;
      }
      lastSentSignature.value = null;
      lastAvatarKey.value = null;
      lastAvatarBytes.value = null;
      const badgeCount =
        isReady.value && currentWorkspace.value
          ? normalizeUnreadCount(totalUnreadCount.value)
          : 0;
      await pushNotificationState(badgeCount);
      if (import.meta.env.DEV) {
        console.info('[notifications] active publisher window', {
          windowLabel: windowLabel.value,
          reason
        });
      }
    } catch (error) {
      console.error('Failed to set active notification window.', error);
    }
  };

  const applyOpenTerminalPayloads = async (payloads: NotificationOpenTerminalPayload[]) => {
    if (!payloads.length) {
      return;
    }
    for (const payload of payloads) {
      const workspaceId = payload.workspaceId?.trim() ?? '';
      const conversationId = payload.conversationId?.trim() ?? '';
      const senderId = payload.senderId?.trim() ?? '';
      if (!workspaceId || !conversationId || !senderId) {
        continue;
      }
      if (workspaceId !== currentWorkspace.value?.id) {
        continue;
      }
      pendingOpenConversationId.value = conversationId;
      setActiveTab('chat');
      await markConversationRead(conversationId);
      const sender = members.value.find((candidate) => candidate.id === senderId);
      if (sender && hasTerminalConfig(sender.terminalType, sender.terminalCommand)) {
        await openMemberTerminal(sender);
      }
    }
  };

  const enqueueOpenTerminal = (payloads: NotificationOpenTerminalPayload[]) => {
    const workspaceId = currentWorkspace.value?.id ?? 'unknown';
    const chain = terminalOpenChains.get(workspaceId) ?? Promise.resolve();
    const task = chain.then(
      () => applyOpenTerminalPayloads(payloads),
      () => applyOpenTerminalPayloads(payloads)
    );
    terminalOpenChains.set(workspaceId, task);
    return task.finally(() => {
      if (terminalOpenChains.get(workspaceId) === task) {
        terminalOpenChains.delete(workspaceId);
      }
    });
  };

  watch(
    [totalUnreadCount, isReady, currentWorkspace, conversations, () => settings.value.notifications],
    ([count, ready, workspace]) => {
      const badgeCount = ready && workspace ? normalizeUnreadCount(count) : 0;
      void pushNotificationState(badgeCount);
    },
    { immediate: true }
  );

  watch(
    isReady,
    (ready) => {
      if (!ready || !pendingIgnoreAll.value) {
        return;
      }
      pendingIgnoreAll.value = false;
      void markAllConversationsRead();
    },
    { immediate: true }
  );

  watch(
    [isReady, currentWorkspace, () => pendingOpenTerminalPayloads.value],
    ([ready, workspace, payloads]) => {
      if (!ready || !workspace || payloads.length === 0) {
        return;
      }
      pendingOpenTerminalPayloads.value = [];
      void enqueueOpenTerminal(payloads);
    },
    { immediate: true }
  );

  const ensureListeners = async () => {
    if (listenersReady || !isTauri()) {
      return;
    }
    if (!windowLabel.value) {
      windowLabel.value = resolveWindowLabel();
    }
    if (!isMainWindowLabel(windowLabel.value)) {
      return;
    }
    listenersReady = true;
    await listen(
      'notification-ignore-all',
      () => {
        void markAllConversationsRead();
      },
      { target: windowLabel.value }
    );
    await listen<NotificationOpenConversationPayload>(
      NOTIFICATION_OPEN_CONVERSATION_EVENT,
      (event) => {
        const conversationId = event.payload?.conversationId?.trim() ?? '';
        if (!conversationId) {
          return;
        }
        pendingOpenConversationId.value = conversationId;
        setActiveTab('chat');
      },
      { target: windowLabel.value }
    );
    await listen<NotificationOpenTerminalPayload>(
      NOTIFICATION_OPEN_TERMINAL_EVENT,
      async (event) => {
        const conversationId = event.payload?.conversationId?.trim() ?? '';
        const senderId = event.payload?.senderId?.trim() ?? '';
        const workspaceId = event.payload?.workspaceId?.trim() ?? '';
        if (!conversationId || !senderId) {
          return;
        }
        if (workspaceId && workspaceId !== currentWorkspace.value?.id) {
          return;
        }
        void enqueueOpenTerminal([{ conversationId, senderId, workspaceId }]);
      },
      { target: windowLabel.value }
    );
  };

  const ensureFocusListener = () => {
    if (focusListenerReady || typeof window === 'undefined') {
      return;
    }
    focusListenerReady = true;
    window.addEventListener('focus', () => {
      void activatePublisherWindow('window-focus');
    });
  };

  watch(windowLabel, () => {
    void ensureListeners();
  });
  void ensureListeners();
  ensureFocusListener();
  void activatePublisherWindow('init');
  resolveIgnoreAllBootstrap();
  resolveOpenTerminalBootstrap();

  return {
    lastSentSignature,
    pendingOpenConversationId,
    clearPendingOpenConversation: () => {
      pendingOpenConversationId.value = null;
    }
  };
});
