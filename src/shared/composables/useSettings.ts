import { computed, ref } from 'vue';

export type AccountStatus = 'online' | 'away' | 'dnd';

export type AccountSettings = {
  displayName: string;
  email: string;
  title: string;
  timezone: string;
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

export type MemberSettings = {
  defaultMemberId: string;
  customMembers: CustomMember[];
};

export type LanguageSettings = {
  spellCheck: boolean;
};

export type SettingsState = {
  account: AccountSettings;
  notifications: NotificationSettings;
  keybinds: KeybindSettings;
  members: MemberSettings;
  language: LanguageSettings;
};

const SETTINGS_STORAGE_KEY = 'nexus-settings';

const DEFAULT_SETTINGS: SettingsState = {
  account: {
    displayName: '',
    email: '',
    title: '',
    timezone: 'UTC',
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
    defaultMemberId: 'gemini-cli',
    customMembers: []
  },
  language: {
    spellCheck: true
  }
};

const BASE_MEMBER_IDS = ['gemini-cli', 'codex', 'claude-code'];
const ALLOWED_MEMBER_IDS = new Set(BASE_MEMBER_IDS);
const ALLOWED_STATUSES = new Set<AccountStatus>(['online', 'away', 'dnd']);
const ALLOWED_KEYBINDS = new Set<KeybindProfile>(['default', 'vscode', 'slack']);
const CUSTOM_MEMBER_PREFIX = 'custom-cli-';

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

const normalizeSettings = (candidate: SettingsState): SettingsState => {
  const account = {
    displayName: safeString(candidate.account.displayName).trim(),
    email: safeString(candidate.account.email).trim().toLowerCase(),
    title: safeString(candidate.account.title).trim(),
    timezone: safeString(candidate.account.timezone).trim() || DEFAULT_SETTINGS.account.timezone,
    status: ALLOWED_STATUSES.has(candidate.account.status) ? candidate.account.status : DEFAULT_SETTINGS.account.status,
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
  const allowedMemberIds = new Set([...ALLOWED_MEMBER_IDS, ...customMembers.map((member) => member.id)]);
  const requestedMemberId =
    candidate.members.defaultMemberId === 'custom-cli' && customMembers.length > 0 ? customMembers[0].id : candidate.members.defaultMemberId;
  const defaultMemberId = allowedMemberIds.has(requestedMemberId) ? requestedMemberId : DEFAULT_SETTINGS.members.defaultMemberId;

  return {
    account,
    notifications,
    keybinds,
    members: {
      defaultMemberId,
      customMembers
    },
    language: {
      spellCheck: Boolean(candidate.language.spellCheck)
    }
  };
};

const buildSettings = (candidate?: Partial<SettingsState>): SettingsState => {
  const merged: SettingsState = {
    account: { ...DEFAULT_SETTINGS.account, ...candidate?.account },
    notifications: { ...DEFAULT_SETTINGS.notifications, ...candidate?.notifications },
    keybinds: { ...DEFAULT_SETTINGS.keybinds, ...candidate?.keybinds },
    members: { ...DEFAULT_SETTINGS.members, ...candidate?.members },
    language: { ...DEFAULT_SETTINGS.language, ...candidate?.language }
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

const settingsRef = ref<SettingsState>(loadSettings());

const persistSettings = (next: SettingsState) => {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(next));
};

export const useSettings = () => {
  const saveSettings = (next: SettingsState) => {
    const normalized = normalizeSettings(next);
    settingsRef.value = normalized;
    persistSettings(normalized);
    return normalized;
  };

  const resetSettings = () => {
    const next = cloneSettings(DEFAULT_SETTINGS);
    settingsRef.value = next;
    persistSettings(next);
    return next;
  };

  return {
    settings: computed(() => settingsRef.value),
    saveSettings,
    resetSettings
  };
};
