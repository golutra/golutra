// 右键菜单类型：描述菜单项、上下文与规则匹配。
export type ContextMenuItem = {
  kind: 'item';
  id: string;
  label?: string;
  labelKey?: string;
  icon?: string;
  shortcut?: string;
  enabled?: boolean;
  danger?: boolean;
  action: (context: ContextMenuContext) => void | Promise<void>;
};

export type ContextMenuSeparator = {
  kind: 'separator';
  id?: string;
};

export type ContextMenuEntry = ContextMenuItem | ContextMenuSeparator;

export type ContextMenuMatchMode = 'override' | 'merge';

export type ContextMenuContext = {
  event: MouseEvent;
  view: string;
  target: EventTarget | null;
  targetElement: HTMLElement | null;
  scope: string | null;
  scopeChain: string[];
  selectionText: string;
  isInput: boolean;
  isEditable: boolean;
  inputElement: HTMLInputElement | HTMLTextAreaElement | null;
  editableElement: HTMLElement | null;
};

export type ContextMenuRule = {
  id: string;
  order?: number;
  mode?: ContextMenuMatchMode;
  stop?: boolean;
  matches: (context: ContextMenuContext) => boolean;
  items: (context: ContextMenuContext) => ContextMenuEntry[];
};
