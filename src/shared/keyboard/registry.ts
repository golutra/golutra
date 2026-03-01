// 键盘快捷键规则注册表：按优先级解析并合并可用的快捷键条目。
import type { KeybindContext, KeybindItem, KeybindRule } from './types';

const rules = new Map<string, KeybindRule>();

export const registerKeybindRule = (rule: KeybindRule) => {
  rules.set(rule.id, rule);
};

export const unregisterKeybindRule = (id: string) => {
  rules.delete(id);
};

const sortItems = (items: KeybindItem[]) =>
  items.slice().sort((a, b) => (b.order ?? 0) - (a.order ?? 0));

export const resolveKeybindItems = (context: KeybindContext): KeybindItem[] => {
  const sortedRules = Array.from(rules.values()).sort((a, b) => (b.order ?? 0) - (a.order ?? 0));
  let items: KeybindItem[] = [];
  for (const rule of sortedRules) {
    if (!rule.matches(context)) {
      continue;
    }
    const nextItems = sortItems(rule.items(context));
    if (nextItems.length === 0) {
      continue;
    }
    const mode = rule.mode ?? 'merge';
    items = mode === 'merge' ? items.concat(nextItems) : nextItems;
    const shouldStop = rule.stop ?? mode !== 'merge';
    if (shouldStop) {
      break;
    }
  }
  return items;
};
