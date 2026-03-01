// 键盘快捷键控制器：构建上下文并匹配组合键后执行动作。
import { resolveKeybindItems } from './registry';
import type { KeyCombo, KeyComboInput, KeybindContext, KeybindItem } from './types';

const resolveView = () => {
  if (typeof window === 'undefined') {
    return 'main';
  }
  const view = (window as typeof window & { __GOLUTRA_VIEW__?: string }).__GOLUTRA_VIEW__;
  return view ? view : 'main';
};

const resolveTargetElement = (event: KeyboardEvent) => {
  if (typeof event.composedPath === 'function') {
    const path = event.composedPath();
    for (const entry of path) {
      if (entry instanceof HTMLElement) {
        return entry;
      }
    }
  }
  return event.target instanceof HTMLElement ? event.target : null;
};

const resolveScopeChain = (element: HTMLElement | null) => {
  const chain: string[] = [];
  let current: HTMLElement | null = element;
  while (current) {
    const scope = current.getAttribute('data-key-scope');
    if (scope) {
      chain.push(scope);
    }
    current = current.parentElement;
  }
  return chain;
};

const getInputSelectionText = (input: HTMLInputElement | HTMLTextAreaElement | null) => {
  if (!input) {
    return '';
  }
  const start = input.selectionStart;
  const end = input.selectionEnd;
  if (start === null || end === null || end <= start) {
    return '';
  }
  return input.value.slice(start, end);
};

const resolveEditableElement = (target: HTMLElement | null) => {
  if (!target) {
    return null;
  }
  const editable = target.closest('[contenteditable="true"], [contenteditable=""]') as HTMLElement | null;
  if (editable) {
    return editable;
  }
  return target.isContentEditable ? target : null;
};

export const buildKeybindContext = (event: KeyboardEvent): KeybindContext => {
  const targetElement = resolveTargetElement(event);
  const inputElement =
    (targetElement?.closest('input, textarea') as HTMLInputElement | HTMLTextAreaElement | null) ?? null;
  const editableElement = resolveEditableElement(targetElement);
  const isInput = Boolean(inputElement);
  const isEditable = isInput || Boolean(editableElement);
  const selectionText = isInput ? getInputSelectionText(inputElement) : window.getSelection()?.toString() ?? '';
  const scopeChain = resolveScopeChain(targetElement);

  return {
    event,
    view: resolveView(),
    target: event.target,
    targetElement,
    activeElement: typeof document === 'undefined' ? null : document.activeElement,
    scope: scopeChain[0] ?? null,
    scopeChain,
    selectionText,
    isInput,
    isEditable,
    inputElement,
    editableElement
  };
};

const normalizeKey = (raw: string) => {
  if (raw === ' ') return 'space';
  const trimmed = raw.trim();
  if (!trimmed) return '';
  if (trimmed.length === 1) return trimmed.toLowerCase();
  const lower = trimmed.toLowerCase();
  const alias: Record<string, string> = {
    esc: 'escape',
    escape: 'escape',
    return: 'enter',
    enter: 'enter',
    space: 'space',
    spacebar: 'space',
    del: 'delete',
    delete: 'delete',
    up: 'arrowup',
    arrowup: 'arrowup',
    down: 'arrowdown',
    arrowdown: 'arrowdown',
    left: 'arrowleft',
    arrowleft: 'arrowleft',
    right: 'arrowright',
    arrowright: 'arrowright'
  };
  return alias[lower] ?? lower;
};

// 解析组合键字符串，必要时拆成 Cmd/Ctrl 两套组合键。
const parseComboString = (input: string): KeyCombo[] => {
  const tokens = input.split('+').map((part) => part.trim()).filter(Boolean);
  if (tokens.length === 0) {
    return [];
  }
  const modifiers = {
    ctrl: false,
    alt: false,
    shift: false,
    meta: false
  };
  let useCmdOrCtrl = false;
  const keyParts: string[] = [];

  tokens.forEach((token) => {
    const lower = token.toLowerCase();
    if (['ctrl', 'control'].includes(lower)) {
      modifiers.ctrl = true;
      return;
    }
    if (['alt', 'option'].includes(lower)) {
      modifiers.alt = true;
      return;
    }
    if (lower === 'shift') {
      modifiers.shift = true;
      return;
    }
    if (['cmd', 'command', 'meta', 'super'].includes(lower)) {
      modifiers.meta = true;
      return;
    }
    if (['cmdorctrl', 'commandorcontrol', 'mod'].includes(lower)) {
      useCmdOrCtrl = true;
      return;
    }
    keyParts.push(token);
  });

  const key = normalizeKey(keyParts.join('+'));
  if (!key) {
    return [];
  }
  const base: KeyCombo = {
    key,
    ctrl: modifiers.ctrl,
    alt: modifiers.alt,
    shift: modifiers.shift,
    meta: modifiers.meta
  };
  if (!useCmdOrCtrl) {
    return [base];
  }
  return [
    { ...base, ctrl: true, meta: false },
    { ...base, ctrl: false, meta: true }
  ];
};

const normalizeComboInput = (input: KeyComboInput): KeyCombo[] => {
  if (typeof input === 'string') {
    return parseComboString(input);
  }
  const normalized: KeyCombo = {
    key: normalizeKey(input.key),
    ctrl: Boolean(input.ctrl),
    alt: Boolean(input.alt),
    shift: Boolean(input.shift),
    meta: Boolean(input.meta),
    code: input.code
  };
  return normalized.key ? [normalized] : [];
};

const normalizeComboInputs = (keys: KeyComboInput | KeyComboInput[]): KeyCombo[] => {
  const list = Array.isArray(keys) ? keys : [keys];
  return list.flatMap((entry) => normalizeComboInput(entry));
};

const isMacOS = () => typeof navigator !== 'undefined' && /Mac/i.test(navigator.platform);

const isComboMatch = (event: KeyboardEvent, combo: KeyCombo) => {
  const key = normalizeKey(event.key);
  if (combo.code && combo.code !== event.code) {
    return false;
  }
  if (combo.key !== key) {
    return false;
  }
  // 兼容 macOS：当使用 Command 触发 Ctrl 组合键时，允许忽略额外的 Meta 校验。
  const macMetaAsCtrl = isMacOS() && combo.ctrl && !combo.meta && event.metaKey;
  const matchCtrl = combo.ctrl ? event.ctrlKey || macMetaAsCtrl : !event.ctrlKey;
  const matchMeta = combo.meta ? event.metaKey : !event.metaKey || macMetaAsCtrl;
  if (!matchCtrl || !matchMeta) {
    return false;
  }
  if (combo.alt !== event.altKey) {
    return false;
  }
  if (combo.shift !== event.shiftKey) {
    return false;
  }
  return true;
};

const matchesScope = (item: KeybindItem, context: KeybindContext) => {
  if (!item.scope) {
    return true;
  }
  const scopes = Array.isArray(item.scope) ? item.scope : [item.scope];
  return scopes.some((scope) => context.scopeChain.includes(scope));
};

const isItemEnabled = (item: KeybindItem, context: KeybindContext) => {
  if (typeof item.enabled === 'boolean') {
    return item.enabled;
  }
  if (typeof item.enabled === 'function') {
    return item.enabled(context);
  }
  return true;
};

const shouldHandleEditable = (item: KeybindItem, context: KeybindContext) => {
  if (!context.isEditable) {
    return true;
  }
  return item.allowInEditable === true;
};

const runKeybindAction = async (item: KeybindItem, context: KeybindContext) => {
  try {
    await item.action(context);
  } catch (error) {
    console.error('Keybind action failed.', error);
  }
};

export const handleKeydownEvent = (event: KeyboardEvent) => {
  if (event.defaultPrevented) {
    return;
  }
  const context = buildKeybindContext(event);
  const items = resolveKeybindItems(context);
  if (items.length === 0) {
    return;
  }
  for (const item of items) {
    if (event.isComposing && !item.allowWhenComposing) {
      continue;
    }
    if (!matchesScope(item, context)) {
      continue;
    }
    if (!isItemEnabled(item, context)) {
      continue;
    }
    if (item.when && !item.when(context)) {
      continue;
    }
    if (!shouldHandleEditable(item, context)) {
      continue;
    }
    if (event.repeat && !item.allowRepeat) {
      continue;
    }
    const combos = normalizeComboInputs(item.keys);
    if (combos.length === 0) {
      continue;
    }
    if (!combos.some((combo) => isComboMatch(event, combo))) {
      continue;
    }
    if (item.preventDefault ?? true) {
      event.preventDefault();
    }
    if (item.stopPropagation ?? true) {
      event.stopPropagation();
    }
    void runKeybindAction(item, context);
    if (item.stop !== false) {
      break;
    }
  }
};
