// 头像处理工具：统一编码规则、默认回退与本地资源解析，避免 UI 组件分散处理。
import { convertFileSrc, isTauri } from '@tauri-apps/api/core';
import {
  AVATAR_PRESETS,
  CSS_AVATAR_PREFIX,
  DEFAULT_AVATAR,
  DEFAULT_AVATAR_ID,
  type AvatarPreset
} from '@/shared/constants/avatars';
import { readAvatarAsset, resolveAvatarPath } from '@/shared/tauri/avatars';

// 统一将输入规整为可比较字符串，避免 undefined/null 造成分支膨胀。
const toSafeString = (value: unknown) => (typeof value === 'string' ? value.trim() : '');

// 本地头像标识前缀，与 CSS 头像前缀区分来源。
export const LOCAL_AVATAR_PREFIX = 'local:';

/**
 * 判断是否为 CSS 头像标识。
 * 输入：头像字符串。
 * 输出：是否为 CSS 头像。
 */
export const isCssAvatar = (avatar?: string | null) => toSafeString(avatar).startsWith(CSS_AVATAR_PREFIX);
/**
 * 判断是否为本地头像标识。
 * 输入：头像字符串。
 * 输出：是否为本地头像。
 */
export const isLocalAvatar = (avatar?: string | null) => toSafeString(avatar).startsWith(LOCAL_AVATAR_PREFIX);

/**
 * 解析 CSS 头像 id。
 * 输入：头像字符串。
 * 输出：id 或 null。
 */
export const getCssAvatarId = (avatar?: string | null) => {
  const value = toSafeString(avatar);
  if (!value.startsWith(CSS_AVATAR_PREFIX)) return null;
  const id = value.slice(CSS_AVATAR_PREFIX.length);
  return id || null;
};

/**
 * 解析本地头像 id。
 * 输入：头像字符串。
 * 输出：id 或 null。
 */
export const getLocalAvatarId = (avatar?: string | null) => {
  const value = toSafeString(avatar);
  if (!value.startsWith(LOCAL_AVATAR_PREFIX)) return null;
  const id = value.slice(LOCAL_AVATAR_PREFIX.length);
  return id || null;
};

/**
 * 拼接 CSS 头像标识。
 * 输入：预设 id。
 * 输出：可被识别的 CSS 头像字符串。
 */
export const toCssAvatar = (id: string) => `${CSS_AVATAR_PREFIX}${id}`;
/**
 * 拼接本地头像标识。
 * 输入：本地资源 id。
 * 输出：可被识别的本地头像字符串。
 */
export const toLocalAvatar = (id: string) => `${LOCAL_AVATAR_PREFIX}${id}`;

/**
 * 解析头像预设。
 * 输入：头像字符串。
 * 输出：有效预设；无法解析时回退到默认预设。
 */
export const resolveAvatarPreset = (avatar?: string | null): AvatarPreset => {
  const id = getCssAvatarId(avatar) ?? DEFAULT_AVATAR_ID;
  return AVATAR_PRESETS.find((preset) => preset.id === id) ?? AVATAR_PRESETS[0];
};

/**
 * 获取头像预设的 CSS 变量集合。
 * 输入：头像字符串。
 * 输出：变量对象。
 */
export const getAvatarVars = (avatar?: string | null) => resolveAvatarPreset(avatar).vars;

/**
 * 确保头像值存在。
 * 输入：头像字符串。
 * 输出：有效头像字符串；空值回退到默认头像。
 */
export const ensureAvatar = (avatar?: string | null) => {
  const value = toSafeString(avatar);
  return value || DEFAULT_AVATAR;
};

// 缓存本地头像 URL，减少文件读写与 Blob 创建成本。
const localAvatarUrlCache = new Map<string, string>();
// 同一资源并发请求时合并 promise，避免重复 IO。
const localAvatarRequests = new Map<string, Promise<string>>();

const revokeCachedUrl = (url?: string) => {
  if (!url || !url.startsWith('blob:')) return;
  URL.revokeObjectURL(url);
};

/**
 * 清理本地头像 URL 缓存。
 * 输入：可选 id，传入时仅清理指定项。
 * 输出：无。
 * 边界：仅回收 blob URL，文件 URL 不需要 revoke。
 */
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

// 在 Tauri 环境下解析本地头像 URL，优先读取字节流，失败时回退到文件路径。
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

/**
 * 解析可用的头像 URL。
 * 输入：头像字符串。
 * 输出：可用于 img/src 的 URL 或 CSS 头像标识。
 * 错误语义：解析失败时返回空字符串或原始值，调用方需自行容错。
 */
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

/**
 * 判断是否为远程默认头像。
 * 输入：头像字符串。
 * 输出：是否匹配远程默认来源。
 */
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

/**
 * 根据种子挑选头像预设 id。
 * 输入：种子字符串。
 * 输出：预设 id；空种子回退到默认。
 */
export const pickAvatarPresetId = (seed: string) => {
  if (!seed) return DEFAULT_AVATAR_ID;
  const index = hashSeed(seed) % AVATAR_PRESETS.length;
  return AVATAR_PRESETS[index]?.id ?? DEFAULT_AVATAR_ID;
};

/**
 * 基于种子生成 CSS 头像标识。
 * 输入：种子字符串。
 * 输出：CSS 头像字符串。
 */
export const buildSeededAvatar = (seed: string) => toCssAvatar(pickAvatarPresetId(seed));

/**
 * 归一化头像输入，兼容外部默认头像与空值。
 * 输入：候选头像与种子。
 * 输出：稳定可用的头像字符串。
 */
export const normalizeAvatar = (candidate: unknown, seed: string) => {
  const value = toSafeString(candidate);
  if (!value || isRemoteDefaultAvatar(value)) {
    return buildSeededAvatar(seed);
  }
  return value;
};
