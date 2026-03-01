// 右键菜单组合式：封装状态与行为。
import { readonly } from 'vue';
import { closeContextMenu, contextMenuState, handleContextMenuEvent } from './controller';
import type { ContextMenuItem } from './types';

const runAction = async (item: ContextMenuItem) => {
  const context = contextMenuState.context;
  if (!context) {
    closeContextMenu();
    return;
  }
  await item.action(context);
  closeContextMenu();
};

export const useContextMenu = () => ({
  state: readonly(contextMenuState),
  closeMenu: closeContextMenu,
  handleContextMenuEvent,
  runAction
});
