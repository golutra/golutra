// 工作区启动编排：切换工作区时串联重置与拉取，避免旧状态污染新会话。
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useChatStore } from '@/features/chat/chatStore';
import { useTerminalMemberStore } from '@/features/terminal/terminalMemberStore';

/**
 * 负责工作区切换后的初始化流程。
 * 输入：无（依赖当前工作区响应式状态）。
 * 输出：appReady 标记与可手动触发的 initWorkspace。
 * 边界：当工作区为空时直接进入 ready；并发切换以序号保护后到覆盖先到。
 */
export const useWorkspaceBootstrap = () => {
  const workspaceStore = useWorkspaceStore();
  const projectStore = useProjectStore();
  const chatStore = useChatStore();
  const terminalMemberStore = useTerminalMemberStore();
  const { currentWorkspace } = storeToRefs(workspaceStore);
  const appReady = ref(false);
  // 用递增序号避免并发初始化的结果互相覆盖。
  let initSequence = 0;
  let focusListener: (() => void) | null = null;

  const initWorkspace = async () => {
    const requestId = ++initSequence;
    appReady.value = false;
    // 先清理旧状态，再拉取新工作区数据，避免 UI 显示混叠。
    projectStore.reset();
    chatStore.reset();

    if (!currentWorkspace.value) {
      appReady.value = true;
      return;
    }

    await projectStore.hydrate();
    if (requestId !== initSequence) {
      return;
    }
    await chatStore.loadSession();
    if (requestId !== initSequence) {
      return;
    }
    await terminalMemberStore.syncWorkspaceStatuses({ force: true, reason: 'workspace-bootstrap' });

    if (requestId !== initSequence) {
      return;
    }

    appReady.value = true;
  };

  watch(
    () => currentWorkspace.value?.id,
    () => {
      void initWorkspace();
    },
    { immediate: true }
  );

  onMounted(() => {
    if (typeof window === 'undefined') {
      return;
    }
    const handler = () => {
      void terminalMemberStore.syncWorkspaceStatuses({ reason: 'window-focus' });
    };
    focusListener = handler;
    // 主窗口重新获得焦点时刷新终端状态，避免离线显示滞后。
    window.addEventListener('focus', handler);
  });

  onBeforeUnmount(() => {
    if (!focusListener || typeof window === 'undefined') {
      return;
    }
    window.removeEventListener('focus', focusListener);
    focusListener = null;
  });

  return { appReady, initWorkspace };
};
