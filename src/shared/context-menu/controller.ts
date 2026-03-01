// 右键菜单控制器：从事件解析上下文并驱动菜单状态。
import { reactive } from 'vue';
import { resolveContextMenuEntries } from './registry';
import type { ContextMenuContext, ContextMenuEntry } from './types';

type ContextMenuState = {
  open: boolean;
  x: number;
  y: number;
  entries: ContextMenuEntry[];
  context: ContextMenuContext | null;
};

const state = reactive<ContextMenuState>({
  open: false,
  x: 0,
  y: 0,
  entries: [],
  context: null
});

const resolveView = () => {
  if (typeof window === 'undefined') {
    return 'main';
  }
  const view = (window as typeof window & { __GOLUTRA_VIEW__?: string }).__GOLUTRA_VIEW__;
  return view ? view : 'main';
};

const resolveTargetElement = (event: MouseEvent) => {
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
    const scope = current.getAttribute('data-context-scope');
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

export const buildContextMenuContext = (event: MouseEvent): ContextMenuContext => {
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
    scope: scopeChain[0] ?? null,
    scopeChain,
    selectionText,
    isInput,
    isEditable,
    inputElement,
    editableElement
  };
};

export const openContextMenu = (context: ContextMenuContext, entries: ContextMenuEntry[]) => {
  state.open = true;
  state.context = context;
  state.entries = entries;
  state.x = context.event.clientX;
  state.y = context.event.clientY;
};

export const closeContextMenu = () => {
  state.open = false;
  state.entries = [];
  state.context = null;
};

export const handleContextMenuEvent = (event: MouseEvent) => {
  if (event.defaultPrevented) {
    return;
  }
  const context = buildContextMenuContext(event);
  const entries = resolveContextMenuEntries(context);
  if (entries.length === 0) {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  openContextMenu(context, entries);
};

export const contextMenuState = state;
