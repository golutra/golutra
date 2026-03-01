// 右键菜单规则注册表：按优先级解析最终菜单项。
import type { ContextMenuContext, ContextMenuEntry, ContextMenuRule } from './types';

const rules = new Map<string, ContextMenuRule>();

export const registerContextMenuRule = (rule: ContextMenuRule) => {
  rules.set(rule.id, rule);
};

export const unregisterContextMenuRule = (id: string) => {
  rules.delete(id);
};

const normalizeEntries = (entries: ContextMenuEntry[]) => {
  const normalized: ContextMenuEntry[] = [];
  let lastWasSeparator = true;
  for (const entry of entries) {
    if (entry.kind === 'separator') {
      if (lastWasSeparator) {
        continue;
      }
      normalized.push(entry);
      lastWasSeparator = true;
      continue;
    }
    normalized.push(entry);
    lastWasSeparator = false;
  }
  while (normalized.length > 0 && normalized[normalized.length - 1]?.kind === 'separator') {
    normalized.pop();
  }
  return normalized;
};

export const resolveContextMenuEntries = (context: ContextMenuContext): ContextMenuEntry[] => {
  const sorted = Array.from(rules.values()).sort((a, b) => (b.order ?? 0) - (a.order ?? 0));
  let entries: ContextMenuEntry[] = [];
  for (const rule of sorted) {
    if (!rule.matches(context)) {
      continue;
    }
    const nextEntries = rule.items(context);
    if (nextEntries.length === 0) {
      continue;
    }
    const mode = rule.mode ?? 'override';
    entries = mode === 'merge' ? entries.concat(nextEntries) : nextEntries;
    const shouldStop = rule.stop ?? mode !== 'merge';
    if (shouldStop) {
      break;
    }
  }
  return normalizeEntries(entries);
};
