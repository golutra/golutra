<template>
  <div v-if="activeTab === 'workspaces'" class="flex h-screen w-full bg-background app-shell relative overflow-hidden">
    <WorkspaceSelection @select-workspace="activeTab = 'chat'" />
  </div>
  <div
    v-else
    class="flex h-screen w-full bg-background app-shell font-sans relative overflow-hidden"
  >
    <SidebarNav :active-tab="activeTab" @change="activeTab = $event" />
    <main class="flex-1 h-full overflow-hidden relative flex flex-col pb-16 md:pb-0">
      <SkillStore v-if="activeTab === 'store'" />
      <PluginMarketplace v-else-if="activeTab === 'plugins'" />
      <Settings v-else-if="activeTab === 'settings'" @logout="activeTab = 'workspaces'" />
      <ChatInterface v-else />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import SidebarNav from '@/shared/components/SidebarNav.vue';
import SkillStore from '@/features/SkillStore.vue';
import PluginMarketplace from '@/features/PluginMarketplace.vue';
import Settings from '@/features/Settings.vue';
import WorkspaceSelection from '@/features/WorkspaceSelection.vue';
import ChatInterface from '@/features/chat/ChatInterface.vue';

type TabId = 'workspaces' | 'chat' | 'store' | 'plugins' | 'settings';

const activeTab = ref<TabId>('workspaces');
</script>
