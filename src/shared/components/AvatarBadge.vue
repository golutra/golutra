<template>
  <div
    class="avatar-badge"
    :class="isCss ? 'avatar-badge--css' : 'avatar-badge--image'"
    :style="avatarVars"
    role="img"
    :aria-label="label"
  >
    <img v-if="!isCss" :src="resolvedAvatar" :alt="label" class="avatar-badge__image" />
    <span v-else class="avatar-badge__ring" aria-hidden="true"></span>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { DEFAULT_AVATAR } from '@/shared/constants/avatars';
import { ensureAvatar, getAvatarVars, isCssAvatar, isLocalAvatar, resolveAvatarUrl } from '@/shared/utils/avatar';

const props = defineProps<{ avatar?: string; label?: string }>();

const resolvedAvatar = ref(DEFAULT_AVATAR);
const isCss = computed(() => isCssAvatar(resolvedAvatar.value));
const avatarVars = computed(() => (isCss.value ? getAvatarVars(resolvedAvatar.value) : undefined));
const label = computed(() => props.label ?? '');
let loadSequence = 0;

const updateAvatar = async () => {
  const current = ++loadSequence;
  const candidate = ensureAvatar(props.avatar);
  if (isCssAvatar(candidate)) {
    resolvedAvatar.value = candidate;
    return;
  }
  if (isLocalAvatar(candidate)) {
    resolvedAvatar.value = DEFAULT_AVATAR;
    const url = await resolveAvatarUrl(candidate);
    if (current === loadSequence) {
      resolvedAvatar.value = url || DEFAULT_AVATAR;
    }
    return;
  }
  resolvedAvatar.value = candidate;
};

watch(
  () => props.avatar,
  () => {
    void updateAvatar();
  },
  { immediate: true }
);
</script>

<style scoped>
.avatar-badge {
  position: relative;
  display: grid;
  place-items: center;
  overflow: hidden;
  background: rgb(var(--color-panel));
}

.avatar-badge__image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-badge--css {
  background: var(--avatar-bg, linear-gradient(135deg, #0b1220 0%, #1f2937 100%));
}

.avatar-badge--css::before,
.avatar-badge--css::after {
  content: '';
  position: absolute;
  display: block;
  border-radius: var(--avatar-spot-radius, 42% 58% 50% 50%);
  filter: blur(0.2px);
}

.avatar-badge--css::before {
  width: var(--avatar-spot-size, 72%);
  height: var(--avatar-spot-size, 72%);
  background: var(--avatar-spot, rgba(255, 255, 255, 0.25));
  transform: translate(var(--avatar-spot-x, -10%), var(--avatar-spot-y, -10%))
    rotate(var(--avatar-spot-rotate, 0deg));
}

.avatar-badge--css::after {
  width: var(--avatar-spot-2-size, 48%);
  height: var(--avatar-spot-2-size, 48%);
  background: var(--avatar-spot-2, rgba(255, 255, 255, 0.2));
  border-radius: var(--avatar-spot-2-radius, 55% 45% 60% 40%);
  transform: translate(var(--avatar-spot-2-x, 18%), var(--avatar-spot-2-y, 18%))
    rotate(var(--avatar-spot-2-rotate, 0deg));
}

.avatar-badge__ring {
  position: absolute;
  inset: 18%;
  border-radius: inherit;
  border: 1px solid var(--avatar-ring, rgba(255, 255, 255, 0.35));
  box-shadow: 0 0 12px var(--avatar-glow, rgba(255, 255, 255, 0.2));
  opacity: 0.8;
  mix-blend-mode: screen;
}
</style>
