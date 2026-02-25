import { createApp } from 'vue';
import App from './app/App.vue';
import { i18n } from './i18n';
import { initializeTheme } from './shared/composables/useTheme';
import './styles/global.css';

initializeTheme();

createApp(App).use(i18n).mount('#app');
