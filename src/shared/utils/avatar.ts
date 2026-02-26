import { convertFileSrc, isTauri } from '@tauri-apps/api/core';
import {
  AVATAR_PRESETS,
  CSS_AVATAR_PREFIX,
  DEFAULT_AVATAR,
  DEFAULT_AVATAR_ID,
  type AvatarPreset
} from '@/shared/constants/avatars';
import { readAvatarAsset, resolveAvatarPath } from '@/shared/tauri/avatars';

const toSafeString = (value: unknown) => (typeof value === 'string' ? value.trim() : '');

export const LOCAL_AVATAR_PREFIX = 'local:';

export const isCssAvatar = (avatar?: string | null) => toSafeString(avatar).startsWith(CSS_AVATAR_PREFIX);
export const isLocalAvatar = (avatar?: string | null) => toSafeString(avatar).startsWith(LOCAL_AVATAR_PREFIX);

export const getCssAvatarId = (avatar?: string | null) => {
  const value = toSafeString(avatar);
  if (!value.startsWith(CSS_AVATAR_PREFIX)) return null;
  const id = value.slice(CSS_AVATAR_PREFIX.length);
  return id || null;
};

export const getLocalAvatarId = (avatar?: string | null) => {
  const value = toSafeString(avatar);
  if (!value.startsWith(LOCAL_AVATAR_PREFIX)) return null;
  const id = value.slice(LOCAL_AVATAR_PREFIX.length);
  return id || null;
};

export const toCssAvatar = (id: string) => `${CSS_AVATAR_PREFIX}${id}`;
export const toLocalAvatar = (id: string) => `${LOCAL_AVATAR_PREFIX}${id}`;

export const resolveAvatarPreset = (avatar?: string | null): AvatarPreset => {
  const id = getCssAvatarId(avatar) ?? DEFAULT_AVATAR_ID;
  return AVATAR_PRESETS.find((preset) => preset.id === id) ?? AVATAR_PRESETS[0];
};

export const getAvatarVars = (avatar?: string | null) => resolveAvatarPreset(avatar).vars;

export const ensureAvatar = (avatar?: string | null) => {
  const value = toSafeString(avatar);
  return value || DEFAULT_AVATAR;
};

const localAvatarUrlCache = new Map<string, string>();
const localAvatarRequests = new Map<string, Promise<string>>();

const revokeCachedUrl = (url?: string) => {
  if (!url || !url.startsWith('blob:')) return;
  URL.revokeObjectURL(url);
};

export const clearAvatarUrlCache = (id?: string) => {
  if (id) {
    revokeCachedUrl(localAvatarUrlCache.get(id));
    localAvatarUrlCache.delete(id);
    localAvatarRequests.delete(id);
    return;
  }
  localAvatarUrlCache.forEach((url) => revokeCachedUrl(url));
  localAvatarUrlCache.clear();
  localAvatarRequests.clear();
};

const resolveLocalAvatarUrl = async (id: string) => {
  if (localAvatarUrlCache.has(id)) {
    return localAvatarUrlCache.get(id) ?? '';
  }
  if (localAvatarRequests.has(id)) {
    return localAvatarRequests.get(id) ?? '';
  }
  const request = (async () => {
    if (!isTauri()) return '';
    try {
      const payload = await readAvatarAsset(id);
      const buffer = Uint8Array.from(payload.bytes);
      const url = URL.createObjectURL(
        new Blob([buffer], { type: payload.mime || 'image/png' })
      );
      localAvatarUrlCache.set(id, url);
      return url;
    } catch {
      try {
        const path = await resolveAvatarPath(id);
        const url = convertFileSrc(path);
        localAvatarUrlCache.set(id, url);
        return url;
      } catch {
        return '';
      }
    } finally {
      localAvatarRequests.delete(id);
    }
  })();
  localAvatarRequests.set(id, request);
  return request;
};

export const resolveAvatarUrl = async (avatar?: string | null) => {
  const value = ensureAvatar(avatar);
  if (isCssAvatar(value)) return value;
  const localId = getLocalAvatarId(value);
  if (localId) {
    return resolveLocalAvatarUrl(localId);
  }
  if (isTauri() && (/^[a-zA-Z]:[\\/]/.test(value) || value.startsWith('\\\\') || value.startsWith('file://'))) {
    return convertFileSrc(value);
  }
  return value;
};

export const isRemoteDefaultAvatar = (avatar?: string | null) => {
  const value = toSafeString(avatar);
  if (!value) return false;
  return value.includes('picsum.photos') || value.includes('ui-avatars.com');
};

const hashSeed = (seed: string) => {
  let hash = 0;
  for (let i = 0; i < seed.length; i += 1) {
    hash = (hash << 5) - hash + seed.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
};

export const pickAvatarPresetId = (seed: string) => {
  if (!seed) return DEFAULT_AVATAR_ID;
  const index = hashSeed(seed) % AVATAR_PRESETS.length;
  return AVATAR_PRESETS[index]?.id ?? DEFAULT_AVATAR_ID;
};

export const buildSeededAvatar = (seed: string) => toCssAvatar(pickAvatarPresetId(seed));

export const normalizeAvatar = (candidate: unknown, seed: string) => {
  const value = toSafeString(candidate);
  if (!value || isRemoteDefaultAvatar(value)) {
    return buildSeededAvatar(seed);
  }
  return value;
};
