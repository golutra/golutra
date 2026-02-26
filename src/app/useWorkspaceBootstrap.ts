import { ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useProjectStore } from '@/features/workspace/projectStore';
import { useChatStore } from '@/features/chat/chatStore';

export const useWorkspaceBootstrap = () => {
  const workspaceStore = useWorkspaceStore();
  const projectStore = useProjectStore();
  const chatStore = useChatStore();
  const { currentWorkspace } = storeToRefs(workspaceStore);
  const appReady = ref(false);
  let initSequence = 0;

  const initWorkspace = async () => {
    const requestId = ++initSequence;
    appReady.value = false;
    projectStore.reset();
    chatStore.reset();

    if (!currentWorkspace.value) {
      appReady.value = true;
      return;
    }

    await projectStore.hydrate();
    await chatStore.loadSession();

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

  return { appReady, initWorkspace };
};
