// 应用入口：统一完成主题预设、状态管理与国际化注入，保证首屏渲染前就绪。
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './app/App.vue';
import { i18n } from '@/i18n';
import { useSettingsStore } from '@/features/global/settingsStore';
import { initializeMonitoring } from '@/shared/monitoring/diagnostics/logger';
import './styles/global.css';

const bootstrapApp = async () => {
  const pinia = createPinia();
  const settingsStore = useSettingsStore(pinia);
  await settingsStore.hydrate();
  initializeMonitoring();
  const app = createApp(App);
  app.use(pinia).use(i18n);
  app.mount('#app');
};

void bootstrapApp();

