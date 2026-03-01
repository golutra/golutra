// 全局设置管理：负责账户/通知/快捷键/成员偏好与应用级持久化。
import { computed, ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';
import { applyLocale, LOCALE_CHANGED_EVENT, messages, type AppLocale } from '@/i18n';
import { isTauri } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { DEFAULT_AVATAR } from '@/shared/constants/avatars';
import { normalizeAvatar } from '@/shared/utils/avatar';
import { readAppData, writeAppData } from '@/shared/tauri/storage';
import { isAppTheme, syncTheme, THEME_CHANGED_EVENT, type AppTheme } from '@/features/global/theme';
import { KEYBIND_PROFILES, type KeybindProfile } from '@/shared/keyboard/profiles';
import {
  DEFAULT_MEMBER_INDEX,
  clampMemberSelectionIndex,
  parseMemberSelectionIndex,
  type MemberSelectionIndex
} from '@/shared/utils/memberSelection';
import { TIME_ZONE_IDS, type TimeZoneId } from '@/shared/constants/timeZones';
import { isTerminalType, type TerminalType } from '@/shared/types/terminal';

export type AccountStatus = 'online' | 'working' | 'dnd' | 'offline';

export type LocaleOption = {
  id: AppLocale;
  labelKey: string;
  flag: string;
};

// 可选语言列表，flag 仅用于展示。
export const localeOptions: LocaleOption[] = [
  { id: 'en-US', labelKey: 'language.enUS', flag: '\uD83C\uDDFA\uD83C\uDDF8' },
  { id: 'zh-CN', labelKey: 'language.zhCN', flag: '\uD83C\uDDE8\uD83C\uDDF3' }
];

export type AccountSettings = {
  displayName: string;
  email: string;
  title: string;
  avatar: string;
  timezone: TimeZoneId;
  status: AccountStatus;
  statusMessage: string;
};

export type NotificationSettings = {
  desktop: boolean;
  sound: boolean;
  mentionsOnly: boolean;
  previews: boolean;
  quietHoursEnabled: boolean;
  quietHoursStart: string;
  quietHoursEnd: string;
};

export type KeybindSettings = {
  enabled: boolean;
  showHints: boolean;
  profile: KeybindProfile;
};

export type ChatSettings = {
  streamOutput: boolean;
};

export type AppearanceSettings = {
  theme: AppTheme;
};

export type CustomMember = {
  id: string;
  name: string;
  command: string;
};

export type CustomTerminal = {
  name: string;
  path: string;
};

export type TerminalPathMap = Partial<Record<TerminalType, string>>;

export type MemberSettings = {
  defaultMemberIndex: MemberSelectionIndex;
  customMembers: CustomMember[];
  customTerminals: CustomTerminal[];
  terminalPaths: TerminalPathMap;
  defaultTerminalName: string;
  defaultTerminalPath: string;
};

export type SettingsState = {
  appearance: AppearanceSettings;
  locale: AppLocale;
  account: AccountSettings;
  notifications: NotificationSettings;
  keybinds: KeybindSettings;
  chat: ChatSettings;
  members: MemberSettings;
};

// 应用级设置文件，统一由 app data 持久化。
const GLOBAL_SETTINGS_PATH = 'global-settings.json';

// 默认设置用于初始化与异常回退。
const DEFAULT_SETTINGS: SettingsState = {
  appearance: {
    theme: 'dark'
  },
  locale: 'en-US',
  account: {
    displayName: '',
    email: '',
    title: '',
    avatar: DEFAULT_AVATAR,
    timezone: 'utc',
    status: 'online',
    statusMessage: ''
  },
  notifications: {
    desktop: true,
    sound: false,
    mentionsOnly: false,
    previews: true,
    quietHoursEnabled: false,
    quietHoursStart: '22:00',
    quietHoursEnd: '07:00'
  },
  keybinds: {
    enabled: true,
    showHints: true,
    profile: 'default'
  },
  chat: {
    streamOutput: true
  },
  members: {
    defaultMemberIndex: DEFAULT_MEMBER_INDEX,
    customMembers: [],
    customTerminals: [],
    terminalPaths: {},
    defaultTerminalName: '',
    defaultTerminalPath: ''
  }
};

const ALLOWED_STATUSES = new Set<AccountStatus>(['online', 'working', 'dnd', 'offline']);
const ALLOWED_KEYBINDS = new Set<KeybindProfile>(KEYBIND_PROFILES.map((profile) => profile.id));
const ALLOWED_LOCALES = new Set<AppLocale>(Object.keys(messages) as AppLocale[]);
const ALLOWED_THEMES = new Set<AppTheme>(['dark', 'light', 'system']);
// 为历史自定义成员生成稳定 id 前缀。
const CUSTOM_MEMBER_PREFIX = 'custom-cli-';
const DEFAULT_OWNER_NAME = 'Owner';

const isTimeValue = (value: string) => /^\d{2}:\d{2}$/.test(value);
const safeString = (value: unknown, fallback = '') => (typeof value === 'string' ? value : fallback);

// 规范化语言设置，未知值回退默认语言。
const normalizeLocale = (value: unknown): AppLocale => {
  if (typeof value === 'string' && ALLOWED_LOCALES.has(value as AppLocale)) {
    return value as AppLocale;
  }
  return DEFAULT_SETTINGS.locale;
};

// 规范化主题设置，未知值回退默认主题。
const normalizeTheme = (value: unknown): AppTheme => {
  if (isAppTheme(value) || ALLOWED_THEMES.has(value as AppTheme)) {
    return value as AppTheme;
  }
  return DEFAULT_SETTINGS.appearance.theme;
};

/**
 * 深拷贝设置对象，避免引用共享导致副作用。
 * 输入：SettingsState。
 * 输出：全量复制的新对象。
 */
export const cloneSettings = (value: SettingsState): SettingsState => JSON.parse(JSON.stringify(value)) as SettingsState;

// 归一化自定义成员条目，保证 id/name/command 的最小可用性。
const normalizeCustomMember = (candidate: Partial<CustomMember>, fallbackId: string): CustomMember | null => {
  const id = safeString(candidate.id).trim() || fallbackId;
  const name = safeString(candidate.name).trim();
  const command = safeString(candidate.command).trim();
  if (!id || (!name && !command)) {
    return null;
  }
  return { id, name, command };
};

// 构建自定义成员列表并去重。
const buildCustomMembers = (candidate: SettingsState): CustomMember[] => {
  const rawList = Array.isArray(candidate.members.customMembers) ? candidate.members.customMembers : [];
  const normalized: CustomMember[] = [];

  rawList.forEach((member, index) => {
    const next = normalizeCustomMember(member, `${CUSTOM_MEMBER_PREFIX}${index + 1}`);
    if (next) {
      normalized.push(next);
    }
  });

  const seen = new Set<string>();
  return normalized.filter((member) => {
    if (seen.has(member.id)) {
      return false;
    }
    seen.add(member.id);
    return true;
  });
};

// 规范化状态字段并保证落在允许范围内。
const normalizeAccountStatus = (value: unknown): AccountStatus => {
  if (ALLOWED_STATUSES.has(value as AccountStatus)) {
    return value as AccountStatus;
  }
  return DEFAULT_SETTINGS.account.status;
};

// 规范化时区 id，未知值回退默认时区。
const normalizeTimeZone = (value: unknown): TimeZoneId => {
  const raw = safeString(value).trim();
  if (TIME_ZONE_IDS.has(raw as TimeZoneId)) {
    return raw as TimeZoneId;
  }
  return DEFAULT_SETTINGS.account.timezone;
};

// 解析默认成员索引。
const resolveDefaultMemberIndex = (candidate: SettingsState, customMembers: CustomMember[]): MemberSelectionIndex => {
  const memberPayload = candidate.members as { defaultMemberIndex?: unknown };
  const parsedIndex = parseMemberSelectionIndex(memberPayload.defaultMemberIndex);
  if (parsedIndex) {
    const clamped = clampMemberSelectionIndex(parsedIndex, customMembers);
    return clamped;
  }
  return DEFAULT_MEMBER_INDEX;
};

// 过滤无效终端路径，仅保留合法类型键值。
const normalizeTerminalPaths = (value: unknown): TerminalPathMap => {
  if (!value || typeof value !== 'object') {
    return {};
  }
  const entries = Object.entries(value as Record<string, unknown>);
  const next: TerminalPathMap = {};
  entries.forEach(([key, raw]) => {
    if (!isTerminalType(key)) {
      return;
    }
    const trimmed = safeString(raw).trim();
    if (trimmed) {
      next[key] = trimmed;
    }
  });
  return next;
};

const normalizeCustomTerminals = (value: unknown): CustomTerminal[] => {
  if (!Array.isArray(value)) {
    return [];
  }
  const seen = new Set<string>();
  const result: CustomTerminal[] = [];
  value.forEach((entry) => {
    if (!entry || typeof entry !== 'object') {
      return;
    }
    const payload = entry as Partial<CustomTerminal>;
    const path = safeString(payload.path).trim();
    if (!path) {
      return;
    }
    const key = path.toLowerCase();
    if (seen.has(key)) {
      return;
    }
    seen.add(key);
    result.push({
      name: safeString(payload.name).trim(),
      path
    });
  });
  return result;
};

// 核心归一化逻辑，统一处理缺省值、旧字段与格式清洗。
const normalizeSettings = (candidate: SettingsState): SettingsState => {
  const appearance = {
    theme: normalizeTheme(candidate.appearance.theme)
  };
  const locale = normalizeLocale(candidate.locale);
  const displayName = safeString(candidate.account.displayName).trim();
  const account = {
    displayName,
    email: safeString(candidate.account.email).trim().toLowerCase(),
    title: safeString(candidate.account.title).trim(),
    avatar: normalizeAvatar(candidate.account.avatar, displayName || DEFAULT_OWNER_NAME),
    timezone: normalizeTimeZone(candidate.account.timezone),
    status: normalizeAccountStatus(candidate.account.status),
    statusMessage: safeString(candidate.account.statusMessage).trim()
  };

  const notifications = {
    desktop: Boolean(candidate.notifications.desktop),
    sound: Boolean(candidate.notifications.sound),
    mentionsOnly: Boolean(candidate.notifications.mentionsOnly),
    previews: Boolean(candidate.notifications.previews),
    quietHoursEnabled: Boolean(candidate.notifications.quietHoursEnabled),
    quietHoursStart: isTimeValue(safeString(candidate.notifications.quietHoursStart))
      ? safeString(candidate.notifications.quietHoursStart)
      : DEFAULT_SETTINGS.notifications.quietHoursStart,
    quietHoursEnd: isTimeValue(safeString(candidate.notifications.quietHoursEnd))
      ? safeString(candidate.notifications.quietHoursEnd)
      : DEFAULT_SETTINGS.notifications.quietHoursEnd
  };

  const keybinds = {
    enabled: Boolean(candidate.keybinds.enabled),
    showHints: Boolean(candidate.keybinds.showHints),
    profile: ALLOWED_KEYBINDS.has(candidate.keybinds.profile) ? candidate.keybinds.profile : DEFAULT_SETTINGS.keybinds.profile
  };
  const chat = {
    streamOutput: Boolean(candidate.chat?.streamOutput ?? DEFAULT_SETTINGS.chat.streamOutput)
  };

  const customMembers = buildCustomMembers(candidate);
  const defaultMemberIndex = resolveDefaultMemberIndex(candidate, customMembers);
  const defaultTerminalName = safeString(candidate.members?.defaultTerminalName).trim();
  const defaultTerminalPath = safeString(candidate.members?.defaultTerminalPath).trim();

  return {
    appearance,
    locale,
    account,
    notifications,
    keybinds,
    chat,
    members: {
      defaultMemberIndex,
      customMembers,
      customTerminals: normalizeCustomTerminals(candidate.members?.customTerminals),
      terminalPaths: normalizeTerminalPaths(candidate.members?.terminalPaths),
      defaultTerminalName,
      defaultTerminalPath
    }
  };
};

const buildSettings = (candidate?: Partial<SettingsState>): SettingsState => {
  const merged: SettingsState = {
    appearance: {
      ...DEFAULT_SETTINGS.appearance,
      ...candidate?.appearance,
      theme: (candidate?.appearance?.theme ?? DEFAULT_SETTINGS.appearance.theme) as AppTheme
    },
    locale: (candidate?.locale ?? DEFAULT_SETTINGS.locale) as AppLocale,
    account: { ...DEFAULT_SETTINGS.account, ...candidate?.account },
    notifications: { ...DEFAULT_SETTINGS.notifications, ...candidate?.notifications },
    keybinds: { ...DEFAULT_SETTINGS.keybinds, ...candidate?.keybinds },
    chat: { ...DEFAULT_SETTINGS.chat, ...candidate?.chat },
    members: { ...DEFAULT_SETTINGS.members, ...candidate?.members }
  };
  return normalizeSettings(merged);
};

const persistSettings = async (next: SettingsState) => {
  if (!isTauri()) {
    return;
  }
  await writeAppData(GLOBAL_SETTINGS_PATH, next);
};

const applySettingsEffects = (
  next: SettingsState,
  previous: SettingsState | null,
  options?: { emitEvents?: boolean; force?: boolean }
) => {
  const emitEvents = options?.emitEvents ?? true;
  const force = options?.force ?? false;
  if (force || !previous || previous.appearance.theme !== next.appearance.theme) {
    syncTheme(next.appearance.theme);
    if (emitEvents && isTauri()) {
      void emit(THEME_CHANGED_EVENT, { theme: next.appearance.theme });
    }
  }
  if (force || !previous || previous.locale !== next.locale) {
    applyLocale(next.locale);
    if (emitEvents && isTauri()) {
      void emit(LOCALE_CHANGED_EVENT, { locale: next.locale });
    }
  }
};

/**
 * 设置状态存储。
 * 输入：保存/重置/修改局部设置。
 * 输出：当前设置与操作函数。
 */
export const useSettingsStore = defineStore('settings', () => {
  const settingsRef = ref<SettingsState>(cloneSettings(DEFAULT_SETTINGS));
  const loadingSettings = ref(false);
  const loadedSettings = ref(false);
  const settingsError = ref<string | null>(null);
  const locale = computed(() => settingsRef.value.locale);
  const theme = computed(() => settingsRef.value.appearance.theme);

  const formatError = (error: unknown) => (error instanceof Error ? error.message : String(error));

  /**
   * 读取并初始化设置，仅执行一次。
   * 输入：无。
   * 输出：更新设置并应用主题/语言。
   */
  const hydrate = async () => {
    if (loadingSettings.value || loadedSettings.value) return;
    loadingSettings.value = true;
    settingsError.value = null;
    try {
      if (!isTauri()) {
        const normalized = cloneSettings(DEFAULT_SETTINGS);
        const previous = settingsRef.value;
        settingsRef.value = normalized;
        applySettingsEffects(normalized, previous, { emitEvents: false, force: true });
        loadedSettings.value = true;
        return;
      }
      const stored = await readAppData<Partial<SettingsState>>(GLOBAL_SETTINGS_PATH);
      const normalized = buildSettings(stored ?? undefined);
      const previous = settingsRef.value;
      settingsRef.value = normalized;
      applySettingsEffects(normalized, previous, { emitEvents: false, force: true });
      const shouldPersist = !stored || typeof stored !== 'object' || !('appearance' in stored) || !('locale' in stored);
      if (shouldPersist) {
        await persistSettings(normalized);
      }
      loadedSettings.value = true;
    } catch (error) {
      settingsError.value = formatError(error);
      console.error('Failed to hydrate settings.', error);
    } finally {
      loadingSettings.value = false;
    }
  };

  /**
   * 保存完整设置并持久化。
   * 输入：SettingsState。
   * 输出：归一化后的设置。
   */
  const saveSettings = (next: SettingsState) => {
    const previous = settingsRef.value;
    const normalized = normalizeSettings(next);
    settingsRef.value = normalized;
    void persistSettings(normalized);
    applySettingsEffects(normalized, previous, { emitEvents: true });
    return normalized;
  };

  const setAccountDisplayName = (displayName: string) => {
    const nextName = displayName.trim();
    if (settingsRef.value.account.displayName === nextName) {
      return settingsRef.value;
    }
    const next: SettingsState = {
      ...settingsRef.value,
      account: {
        ...settingsRef.value.account,
        displayName: nextName
      }
    };
    return saveSettings(next);
  };

  const setAccountStatus = (status: AccountStatus) => {
    if (settingsRef.value.account.status === status) {
      return settingsRef.value;
    }
    const next: SettingsState = {
      ...settingsRef.value,
      account: {
        ...settingsRef.value.account,
        status
      }
    };
    return saveSettings(next);
  };

  /**
   * 恢复默认设置并持久化。
   * 输入：无。
   * 输出：默认设置对象。
   */
  const resetSettings = () => {
    const previous = settingsRef.value;
    const next = cloneSettings(DEFAULT_SETTINGS);
    settingsRef.value = next;
    void persistSettings(next);
    applySettingsEffects(next, previous, { emitEvents: true, force: true });
    return next;
  };

  /**
   * 切换语言并同步到 i18n/document.lang。
   * 输入：AppLocale。
   * 输出：无。
   */
  const setLocale = (next: AppLocale) => {
    const normalized = normalizeLocale(next);
    if (settingsRef.value.locale === normalized) {
      applyLocale(normalized);
      return;
    }
    const updated: SettingsState = {
      ...settingsRef.value,
      locale: normalized
    };
    saveSettings(updated);
  };

  /**
   * 切换主题并同步到 DOM。
   * 输入：AppTheme。
   * 输出：无。
   */
  const setTheme = (next: AppTheme) => {
    const normalized = normalizeTheme(next);
    if (settingsRef.value.appearance.theme === normalized) {
      syncTheme(normalized);
      return;
    }
    const updated: SettingsState = {
      ...settingsRef.value,
      appearance: {
        ...settingsRef.value.appearance,
        theme: normalized
      }
    };
    saveSettings(updated);
  };

  const applyExternalLocale = (next: AppLocale) => {
    const normalized = normalizeLocale(next);
    if (settingsRef.value.locale === normalized) {
      applyLocale(normalized);
      return;
    }
    const previous = settingsRef.value;
    const updated: SettingsState = {
      ...settingsRef.value,
      locale: normalized
    };
    settingsRef.value = updated;
    applySettingsEffects(updated, previous, { emitEvents: false });
  };

  const applyExternalTheme = (next: AppTheme) => {
    const normalized = normalizeTheme(next);
    if (settingsRef.value.appearance.theme === normalized) {
      syncTheme(normalized);
      return;
    }
    const previous = settingsRef.value;
    const updated: SettingsState = {
      ...settingsRef.value,
      appearance: {
        ...settingsRef.value.appearance,
        theme: normalized
      }
    };
    settingsRef.value = updated;
    applySettingsEffects(updated, previous, { emitEvents: false });
  };

  return {
    settings: computed(() => settingsRef.value),
    saveSettings,
    resetSettings,
    setAccountDisplayName,
    setAccountStatus,
    locale,
    setLocale,
    theme,
    setTheme,
    hydrate,
    applyExternalLocale,
    applyExternalTheme,
    loadingSettings,
    settingsError
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useSettingsStore, import.meta.hot));
}
