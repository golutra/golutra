import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './app/App.vue';
import { i18n } from '@/i18n';
import { applyInitialTheme, useThemeStore } from '@/features/global/themeStore';
import './styles/global.css';

applyInitialTheme();

const pinia = createPinia();
const app = createApp(App);

app.use(pinia).use(i18n);

useThemeStore(pinia).initializeTheme();

app.mount('#app');
