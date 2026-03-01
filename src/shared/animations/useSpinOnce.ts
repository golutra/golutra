import { onBeforeUnmount, ref } from 'vue';

export const useSpinOnce = (durationMs = 600) => {
  const spinning = ref(false);
  let timer: number | null = null;

  const triggerSpin = () => {
    if (spinning.value) {
      return;
    }
    spinning.value = true;
    if (timer !== null) {
      window.clearTimeout(timer);
    }
    timer = window.setTimeout(() => {
      spinning.value = false;
      timer = null;
    }, durationMs);
  };

  onBeforeUnmount(() => {
    if (timer !== null) {
      window.clearTimeout(timer);
      timer = null;
    }
  });

  return { spinning, triggerSpin };
};
