// 快捷键配置：集中维护键位方案与文案映射，供设置页与键盘模块复用。
export type KeybindProfile = 'default' | 'vscode' | 'slack';

export type KeybindActionId =
  | 'focus-search'
  | 'new-message'
  | 'toggle-sidebar'
  | 'toggle-mute'
  | 'jump-to-latest'
  | 'open-settings';

export type KeybindProfileBinding = {
  actionId: KeybindActionId;
  labelKey: string;
  keys: string | string[];
};

export type KeybindProfileDefinition = {
  id: KeybindProfile;
  labelKey: string;
  bindings: KeybindProfileBinding[];
};

export const KEYBIND_ACTION_LABELS: Record<KeybindActionId, string> = {
  'focus-search': 'settings.keybindActions.focusSearch',
  'new-message': 'settings.keybindActions.newMessage',
  'toggle-sidebar': 'settings.keybindActions.toggleSidebar',
  'toggle-mute': 'settings.keybindActions.toggleMute',
  'jump-to-latest': 'settings.keybindActions.jumpToLatest',
  'open-settings': 'settings.keybindActions.openSettings'
};

export const KEYBIND_PROFILES: KeybindProfileDefinition[] = [
  {
    id: 'default',
    labelKey: 'settings.keybindProfiles.default',
    bindings: [
      { actionId: 'focus-search', labelKey: KEYBIND_ACTION_LABELS['focus-search'], keys: 'Ctrl + K' },
      { actionId: 'new-message', labelKey: KEYBIND_ACTION_LABELS['new-message'], keys: 'Ctrl + Enter' },
      { actionId: 'toggle-sidebar', labelKey: KEYBIND_ACTION_LABELS['toggle-sidebar'], keys: 'Ctrl + B' },
      { actionId: 'toggle-mute', labelKey: KEYBIND_ACTION_LABELS['toggle-mute'], keys: 'Ctrl + Shift + M' },
      { actionId: 'jump-to-latest', labelKey: KEYBIND_ACTION_LABELS['jump-to-latest'], keys: 'Alt + J' },
      { actionId: 'open-settings', labelKey: KEYBIND_ACTION_LABELS['open-settings'], keys: 'Ctrl + ,' }
    ]
  },
  {
    id: 'vscode',
    labelKey: 'settings.keybindProfiles.vscode',
    bindings: [
      { actionId: 'focus-search', labelKey: KEYBIND_ACTION_LABELS['focus-search'], keys: 'Ctrl + P' },
      { actionId: 'new-message', labelKey: KEYBIND_ACTION_LABELS['new-message'], keys: 'Ctrl + Enter' },
      { actionId: 'toggle-sidebar', labelKey: KEYBIND_ACTION_LABELS['toggle-sidebar'], keys: 'Ctrl + B' },
      { actionId: 'toggle-mute', labelKey: KEYBIND_ACTION_LABELS['toggle-mute'], keys: 'Ctrl + Shift + M' },
      { actionId: 'jump-to-latest', labelKey: KEYBIND_ACTION_LABELS['jump-to-latest'], keys: 'Alt + End' },
      { actionId: 'open-settings', labelKey: KEYBIND_ACTION_LABELS['open-settings'], keys: 'Ctrl + ,' }
    ]
  },
  {
    id: 'slack',
    labelKey: 'settings.keybindProfiles.slack',
    bindings: [
      { actionId: 'focus-search', labelKey: KEYBIND_ACTION_LABELS['focus-search'], keys: 'Ctrl + K' },
      { actionId: 'new-message', labelKey: KEYBIND_ACTION_LABELS['new-message'], keys: 'Ctrl + N' },
      { actionId: 'toggle-sidebar', labelKey: KEYBIND_ACTION_LABELS['toggle-sidebar'], keys: 'Ctrl + Shift + S' },
      { actionId: 'toggle-mute', labelKey: KEYBIND_ACTION_LABELS['toggle-mute'], keys: 'Ctrl + Shift + M' },
      { actionId: 'jump-to-latest', labelKey: KEYBIND_ACTION_LABELS['jump-to-latest'], keys: 'Alt + J' },
      { actionId: 'open-settings', labelKey: KEYBIND_ACTION_LABELS['open-settings'], keys: 'Ctrl + ,' }
    ]
  }
];

const PROFILE_MAP = new Map<KeybindProfile, KeybindProfileDefinition>(
  KEYBIND_PROFILES.map((profile) => [profile.id, profile])
);

export const resolveKeybindKeys = (profile: KeybindProfile, actionId: KeybindActionId): string[] => {
  const definition = PROFILE_MAP.get(profile) ?? KEYBIND_PROFILES[0];
  const binding = definition.bindings.find((item) => item.actionId === actionId);
  if (!binding) {
    return [];
  }
  return Array.isArray(binding.keys) ? binding.keys : [binding.keys];
};
