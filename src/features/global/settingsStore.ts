import { computed, ref } from 'vue';
import { acceptHMRUpdate, defineStore } from 'pinia';
import { i18n, LOCALE_STORAGE_KEY, setDocumentLang, type AppLocale } from '@/i18n';
import { DEFAULT_AVATAR } from '@/shared/constants/avatars';
import { normalizeAvatar } from '@/shared/utils/avatar';
import {
  DEFAULT_MEMBER_INDEX,
  clampMemberSelectionIndex,
  parseMemberSelectionIndex,
  resolveMemberSelectionIndexFromId,
  type MemberSelectionIndex
} from '@/shared/utils/memberSelection';
import { LEGACY_TIME_ZONE_ALIASES, TIME_ZONE_IDS, type TimeZoneId } from '@/shared/constants/timeZones';
import { isTerminalType, type TerminalType } from '@/shared/types/terminal';

export type AccountStatus = 'online' | 'working' | 'dnd' | 'offline';

export type LocaleOption = {
  id: AppLocale;
  labelKey: string;
  flag: string;
};

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
  weeklyDigest: boolean;
  quietHoursEnabled: boolean;
  quietHoursStart: string;
  quietHoursEnd: string;
};

export type KeybindProfile = 'default' | 'vscode' | 'slack';

export type KeybindSettings = {
  enabled: boolean;
  showHints: boolean;
  profile: KeybindProfile;
};

export type CustomMember = {
  id: string;
  name: string;
  command: string;
};

export type TerminalPathMap = Partial<Record<TerminalType, string>>;

export type MemberSettings = {
  defaultMemberIndex: MemberSelectionIndex;
  customMembers: CustomMember[];
  terminalPaths: TerminalPathMap;
};

export type SettingsState = {
  account: AccountSettings;
  notifications: NotificationSettings;
  keybinds: KeybindSettings;
  members: MemberSettings;
};

const SETTINGS_STORAGE_KEY = 'nexus-settings';

const DEFAULT_SETTINGS: SettingsState = {
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
    weeklyDigest: true,
    quietHoursEnabled: false,
    quietHoursStart: '22:00',
    quietHoursEnd: '07:00'
  },
  keybinds: {
    enabled: true,
    showHints: true,
    profile: 'default'
  },
  members: {
    defaultMemberIndex: DEFAULT_MEMBER_INDEX,
    customMembers: [],
    terminalPaths: {}
  }
};

const ALLOWED_STATUSES = new Set<AccountStatus>(['online', 'working', 'dnd', 'offline']);
const LEGACY_STATUS_ALIASES: Record<string, AccountStatus> = {
  away: 'working'
};
const ALLOWED_KEYBINDS = new Set<KeybindProfile>(['default', 'vscode', 'slack']);
const CUSTOM_MEMBER_PREFIX = 'custom-cli-';
const DEFAULT_OWNER_NAME = 'Owner';

const isTimeValue = (value: string) => /^\d{2}:\d{2}$/.test(value);
const safeString = (value: unknown, fallback = '') => (typeof value === 'string' ? value : fallback);

export const cloneSettings = (value: SettingsState): SettingsState => JSON.parse(JSON.stringify(value)) as SettingsState;

const normalizeCustomMember = (candidate: Partial<CustomMember>, fallbackId: string): CustomMember | null => {
  const id = safeString(candidate.id).trim() || fallbackId;
  const name = safeString(candidate.name).trim();
  const command = safeString(candidate.command).trim();
  if (!id || (!name && !command)) {
    return null;
  }
  return { id, name, command };
};

const buildCustomMembers = (candidate: SettingsState): CustomMember[] => {
  const rawList = Array.isArray(candidate.members.customMembers) ? candidate.members.customMembers : [];
  const normalized: CustomMember[] = [];

  rawList.forEach((member, index) => {
    const next = normalizeCustomMember(member, `${CUSTOM_MEMBER_PREFIX}${index + 1}`);
    if (next) {
      normalized.push(next);
    }
  });

  if ((candidate.members as { customMember?: Partial<CustomMember> }).customMember) {
    const legacy = normalizeCustomMember(
      (candidate.members as { customMember?: Partial<CustomMember> }).customMember ?? {},
      `${CUSTOM_MEMBER_PREFIX}${normalized.length + 1}`
    );
    if (legacy) {
      normalized.push(legacy);
    }
  }

  const seen = new Set<string>();
  return normalized.filter((member) => {
    if (seen.has(member.id)) {
      return false;
    }
    seen.add(member.id);
    return true;
  });
};

const normalizeAccountStatus = (value: unknown): AccountStatus => {
  if (typeof value === 'string' && value in LEGACY_STATUS_ALIASES) {
    return LEGACY_STATUS_ALIASES[value];
  }
  if (ALLOWED_STATUSES.has(value as AccountStatus)) {
    return value as AccountStatus;
  }
  return DEFAULT_SETTINGS.account.status;
};

const normalizeTimeZone = (value: unknown): TimeZoneId => {
  const raw = safeString(value).trim();
  if (TIME_ZONE_IDS.has(raw as TimeZoneId)) {
    return raw as TimeZoneId;
  }
  const legacy = LEGACY_TIME_ZONE_ALIASES[raw];
  if (legacy) {
    return legacy;
  }
  return DEFAULT_SETTINGS.account.timezone;
};

const resolveDefaultMemberIndex = (candidate: SettingsState, customMembers: CustomMember[]): MemberSelectionIndex => {
  const memberPayload = candidate.members as { defaultMemberIndex?: unknown; defaultMemberId?: unknown };
  const legacyId = safeString(memberPayload.defaultMemberId).trim();
  const parsedIndex = parseMemberSelectionIndex(memberPayload.defaultMemberIndex);
  if (parsedIndex) {
    const clamped = clampMemberSelectionIndex(parsedIndex, customMembers);
    const isDefault =
      clamped[0] === DEFAULT_MEMBER_INDEX[0] &&
      clamped[1] === DEFAULT_MEMBER_INDEX[1];
    if (!legacyId || !isDefault) {
      return clamped;
    }
  }
  if (legacyId === 'custom-cli' && customMembers.length > 0) {
    return clampMemberSelectionIndex([1, 0], customMembers);
  }
  if (legacyId) {
    const mapped = resolveMemberSelectionIndexFromId(legacyId, customMembers);
    if (mapped) {
      return clampMemberSelectionIndex(mapped, customMembers);
    }
  }
  return DEFAULT_MEMBER_INDEX;
};

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

const normalizeSettings = (candidate: SettingsState): SettingsState => {
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
    weeklyDigest: Boolean(candidate.notifications.weeklyDigest),
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

  const customMembers = buildCustomMembers(candidate);
  const defaultMemberIndex = resolveDefaultMemberIndex(candidate, customMembers);

  return {
    account,
    notifications,
    keybinds,
    members: {
      defaultMemberIndex,
      customMembers,
      terminalPaths: normalizeTerminalPaths(candidate.members?.terminalPaths)
    }
  };
};

const buildSettings = (candidate?: Partial<SettingsState>): SettingsState => {
  const merged: SettingsState = {
    account: { ...DEFAULT_SETTINGS.account, ...candidate?.account },
    notifications: { ...DEFAULT_SETTINGS.notifications, ...candidate?.notifications },
    keybinds: { ...DEFAULT_SETTINGS.keybinds, ...candidate?.keybinds },
    members: { ...DEFAULT_SETTINGS.members, ...candidate?.members }
  };
  return normalizeSettings(merged);
};

const loadSettings = (): SettingsState => {
  if (typeof window === 'undefined') {
    return cloneSettings(DEFAULT_SETTINGS);
  }
  const raw = window.localStorage.getItem(SETTINGS_STORAGE_KEY);
  if (!raw) {
    return cloneSettings(DEFAULT_SETTINGS);
  }
  try {
    const parsed = JSON.parse(raw) as Partial<SettingsState>;
    return buildSettings(parsed);
  } catch {
    return cloneSettings(DEFAULT_SETTINGS);
  }
};

const persistSettings = (next: SettingsState) => {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(next));
};

export const useSettingsStore = defineStore('settings', () => {
  const settingsRef = ref<SettingsState>(loadSettings());
  const locale = computed(() => i18n.global.locale.value as AppLocale);

  const saveSettings = (next: SettingsState) => {
    const normalized = normalizeSettings(next);
    settingsRef.value = normalized;
    persistSettings(normalized);
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

  const resetSettings = () => {
    const next = cloneSettings(DEFAULT_SETTINGS);
    settingsRef.value = next;
    persistSettings(next);
    return next;
  };

  const setLocale = (next: AppLocale) => {
    if (i18n.global.locale.value === next) {
      return;
    }
    i18n.global.locale.value = next;
    if (typeof window !== 'undefined') {
      window.localStorage.setItem(LOCALE_STORAGE_KEY, next);
    }
    setDocumentLang(next);
  };

  return {
    settings: computed(() => settingsRef.value),
    saveSettings,
    resetSettings,
    setAccountDisplayName,
    setAccountStatus,
    locale,
    setLocale
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useSettingsStore, import.meta.hot));
}
