// 键盘快捷键组合式封装：向外暴露事件处理入口。
import { handleKeydownEvent } from './controller';

export const useKeyboard = () => ({
  handleKeydownEvent
});
