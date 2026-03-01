// 键盘快捷键类型定义：约束组合键、上下文与规则结构，便于跨模块扩展与统一解析。
export type KeyCombo = {
  key: string;
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
  code?: string;
};

export type KeyComboInput =
  | string
  | {
      key: string;
      ctrl?: boolean;
      alt?: boolean;
      shift?: boolean;
      meta?: boolean;
      code?: string;
    };

export type KeybindContext = {
  event: KeyboardEvent;
  view: string;
  target: EventTarget | null;
  targetElement: HTMLElement | null;
  activeElement: Element | null;
  scope: string | null;
  scopeChain: string[];
  selectionText: string;
  isInput: boolean;
  isEditable: boolean;
  inputElement: HTMLInputElement | HTMLTextAreaElement | null;
  editableElement: HTMLElement | null;
};

export type KeybindItem = {
  id: string;
  keys: KeyComboInput | KeyComboInput[];
  action: (context: KeybindContext) => void | Promise<void>;
  order?: number;
  enabled?: boolean | ((context: KeybindContext) => boolean);
  when?: (context: KeybindContext) => boolean;
  allowInEditable?: boolean;
  allowWhenComposing?: boolean;
  allowRepeat?: boolean;
  preventDefault?: boolean;
  stopPropagation?: boolean;
  stop?: boolean;
  scope?: string | string[];
};

export type KeybindMatchMode = 'merge' | 'override';

export type KeybindRule = {
  id: string;
  order?: number;
  mode?: KeybindMatchMode;
  stop?: boolean;
  matches: (context: KeybindContext) => boolean;
  items: (context: KeybindContext) => KeybindItem[];
};
