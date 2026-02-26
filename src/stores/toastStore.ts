import { ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';

export type ToastTone = 'info' | 'success' | 'error';

export type Toast = {
  id: string;
  message: string;
  tone: ToastTone;
};

const MAX_TOASTS = 4;
const DEFAULT_DURATION_MS = 3200;

const buildToastId = () => `${Date.now()}-${Math.random().toString(16).slice(2)}`;

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<Toast[]>([]);

  const removeToast = (id: string) => {
    toasts.value = toasts.value.filter((toast) => toast.id !== id);
  };

  const pushToast = (message: string, options?: { tone?: ToastTone; duration?: number }) => {
    const id = buildToastId();
    const toast: Toast = {
      id,
      message,
      tone: options?.tone ?? 'info'
    };
    toasts.value = [...toasts.value, toast].slice(-MAX_TOASTS);

    const duration = options?.duration ?? DEFAULT_DURATION_MS;
    if (duration > 0 && typeof window !== 'undefined') {
      window.setTimeout(() => removeToast(id), duration);
    }
    return id;
  };

  return {
    toasts,
    pushToast,
    removeToast
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useToastStore, import.meta.hot));
}
