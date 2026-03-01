// 监控载荷安全化：规避循环引用、代理对象与超深层级导致的序列化崩溃。
type SafeSerializeOptions = {
  maxDepth?: number;
  maxKeys?: number;
  maxArrayLength?: number;
  maxStringLength?: number;
};

const DEFAULT_OPTIONS: Required<SafeSerializeOptions> = {
  maxDepth: 6,
  maxKeys: 60,
  maxArrayLength: 120,
  maxStringLength: 2000
};

const isPlainObject = (value: unknown): value is Record<string, unknown> => {
  if (!value || typeof value !== 'object') {
    return false;
  }
  const proto = Object.getPrototypeOf(value);
  return proto === Object.prototype || proto === null;
};

const summarizeElement = (value: Element) => ({
  tag: value.tagName,
  id: value.id || null,
  className: typeof (value as HTMLElement).className === 'string' ? (value as HTMLElement).className : null
});

const normalizeString = (value: string, maxLength: number) => {
  if (value.length <= maxLength) {
    return value;
  }
  return `${value.slice(0, maxLength)}…`;
};

const sanitize = (
  value: unknown,
  depth: number,
  options: Required<SafeSerializeOptions>,
  seen: WeakSet<object>
): unknown => {
  if (value === null || value === undefined) {
    return value;
  }
  const valueType = typeof value;
  if (valueType === 'string') {
    return normalizeString(value as string, options.maxStringLength);
  }
  if (valueType === 'number' || valueType === 'boolean') {
    return value;
  }
  if (valueType === 'bigint') {
    return value.toString();
  }
  if (valueType === 'symbol') {
    return value.toString();
  }
  if (valueType === 'function') {
    return '[Function]';
  }
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (value instanceof RegExp) {
    return value.toString();
  }
  if (value instanceof Error) {
    return {
      name: value.name,
      message: value.message,
      stack: value.stack ? normalizeString(value.stack, options.maxStringLength) : null
    };
  }
  if (value instanceof Element) {
    return summarizeElement(value);
  }
  if (valueType === 'object') {
    if (depth >= options.maxDepth) {
      return '[MaxDepth]';
    }
    const obj = value as object;
    if (seen.has(obj)) {
      return '[Circular]';
    }
    seen.add(obj);
    if (Array.isArray(obj)) {
      const items = obj.slice(0, options.maxArrayLength).map((item) =>
        sanitize(item, depth + 1, options, seen)
      );
      if (obj.length > options.maxArrayLength) {
        items.push(`[+${obj.length - options.maxArrayLength} more]`);
      }
      return items;
    }
    if (obj instanceof Map) {
      const entries = Array.from(obj.entries()).slice(0, options.maxArrayLength);
      const mapped = entries.map(([key, entryValue]) => [
        sanitize(key, depth + 1, options, seen),
        sanitize(entryValue, depth + 1, options, seen)
      ]);
      if (obj.size > options.maxArrayLength) {
        mapped.push([`[+${obj.size - options.maxArrayLength} more]`, null]);
      }
      return mapped;
    }
    if (obj instanceof Set) {
      const entries = Array.from(obj.values()).slice(0, options.maxArrayLength);
      const mapped = entries.map((entryValue) => sanitize(entryValue, depth + 1, options, seen));
      if (obj.size > options.maxArrayLength) {
        mapped.push(`[+${obj.size - options.maxArrayLength} more]`);
      }
      return mapped;
    }
    if (isPlainObject(obj)) {
      const output: Record<string, unknown> = {};
      const keys = Object.keys(obj);
      const limitedKeys = keys.slice(0, options.maxKeys);
      for (const key of limitedKeys) {
        try {
          output[key] = sanitize((obj as Record<string, unknown>)[key], depth + 1, options, seen);
        } catch {
          output[key] = '[AccessError]';
        }
      }
      if (keys.length > options.maxKeys) {
        output.__truncatedKeys = keys.length - options.maxKeys;
      }
      return output;
    }
    return String(obj);
  }
  return String(value);
};

export const safeSerialize = (value: unknown, options?: SafeSerializeOptions) => {
  const merged = { ...DEFAULT_OPTIONS, ...options };
  try {
    return sanitize(value, 0, merged, new WeakSet());
  } catch (error) {
    return {
      error: 'safeSerialize failed',
      message: error instanceof Error ? error.message : String(error)
    };
  }
};
