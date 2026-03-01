// 默认右键菜单规则：提供基础编辑操作。
import { isTauri } from '@tauri-apps/api/core';
import { readText as readClipboardText, writeText as writeClipboardText } from '@tauri-apps/plugin-clipboard-manager';
import { registerContextMenuRule } from './registry';
import type { ContextMenuContext, ContextMenuEntry } from './types';

const focusEditableTarget = (context: ContextMenuContext) => {
  if (context.inputElement) {
    context.inputElement.focus();
    return;
  }
  context.editableElement?.focus();
};

const execCommand = (command: string, value?: string) => {
  try {
    return document.execCommand(command, false, value);
  } catch {
    return false;
  }
};

const insertTextIntoInput = (input: HTMLInputElement | HTMLTextAreaElement, text: string) => {
  const start = input.selectionStart ?? input.value.length;
  const end = input.selectionEnd ?? start;
  const nextValue = `${input.value.slice(0, start)}${text}${input.value.slice(end)}`;
  input.value = nextValue;
  const nextPos = start + text.length;
  input.setSelectionRange(nextPos, nextPos);
  input.dispatchEvent(new Event('input', { bubbles: true }));
};

const writeClipboard = async (text: string) => {
  if (!text) {
    return false;
  }
  if (isTauri()) {
    try {
      await writeClipboardText(text);
      return true;
    } catch {
      // ignore
    }
  }
  if (navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      // ignore
    }
  }
  return false;
};

const readClipboard = async () => {
  if (isTauri()) {
    try {
      return await readClipboardText();
    } catch {
      // ignore
    }
  }
  if (navigator.clipboard?.readText) {
    try {
      return await navigator.clipboard.readText();
    } catch {
      // ignore
    }
  }
  return '';
};

const handleCopy = async (context: ContextMenuContext) => {
  focusEditableTarget(context);
  if (execCommand('copy')) {
    return;
  }
  await writeClipboard(context.selectionText);
};

const handleCut = async (context: ContextMenuContext) => {
  focusEditableTarget(context);
  if (execCommand('cut')) {
    return;
  }
  if (context.selectionText) {
    await writeClipboard(context.selectionText);
  }
};

const handlePaste = async (context: ContextMenuContext) => {
  focusEditableTarget(context);
  if (execCommand('paste')) {
    return;
  }
  const text = await readClipboard();
  if (!text) {
    return;
  }
  if (context.inputElement) {
    insertTextIntoInput(context.inputElement, text);
    return;
  }
  if (context.editableElement) {
    execCommand('insertText', text);
  }
};

const handleSelectAll = (context: ContextMenuContext) => {
  focusEditableTarget(context);
  execCommand('selectAll');
};

export const registerDefaultContextMenuRules = () => {
  registerContextMenuRule({
    id: 'context-menu-default',
    order: 0,
    mode: 'override',
    matches: () => true,
    items: (context): ContextMenuEntry[] => {
      const hasSelection = context.selectionText.length > 0;
      const isEditable =
        context.isEditable && !(context.inputElement?.readOnly ?? false) && !(context.inputElement?.disabled ?? false);

      return [
        {
          kind: 'item',
          id: 'copy',
          labelKey: 'contextMenu.copy',
          icon: 'content_copy',
          enabled: hasSelection,
          action: handleCopy
        },
        {
          kind: 'item',
          id: 'cut',
          labelKey: 'contextMenu.cut',
          icon: 'content_cut',
          enabled: isEditable && hasSelection,
          action: handleCut
        },
        {
          kind: 'item',
          id: 'paste',
          labelKey: 'contextMenu.paste',
          icon: 'content_paste',
          enabled: isEditable,
          action: handlePaste
        },
        { kind: 'separator' },
        {
          kind: 'item',
          id: 'select-all',
          labelKey: 'contextMenu.selectAll',
          icon: 'select_all',
          enabled: isEditable,
          action: handleSelectAll
        }
      ];
    }
  });
};
