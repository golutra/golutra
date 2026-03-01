// 轻量提示管理：只维护当前会话内的 UI toast，避免持久化带来的语义歧义。
import { ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';

export type ToastTone = 'info' | 'success' | 'error';

export type Toast = {
  id: string;
  message: string;
  tone: ToastTone;
};

// 限制可见 toast 数量，防止高频错误遮挡主操作区。
const MAX_TOASTS = 4;
// 默认停留时间，兼顾可读性与界面打扰。
const DEFAULT_DURATION_MS = 3200;

// 仅用于 UI 内部标识，不做安全或持久化用途。
const buildToastId = () => `${Date.now()}-${Math.random().toString(16).slice(2)}`;

/**
 * UI 级提示存储。
 * 输入：消息文本与可选语气/时长。
 * 输出：toast 列表与操作函数；push 返回生成的 toast id。
 * 边界：duration <= 0 表示不自动关闭；列表最多保留最近 MAX_TOASTS 条。
 */
export const useToastStore = defineStore('toast', () => {
  const toasts = ref<Toast[]>([]);

  const removeToast = (id: string) => {
    toasts.value = toasts.value.filter((toast) => toast.id !== id);
  };

  /**
   * 追加提示并返回其 id，用于外部主动关闭或追踪。
   */
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
