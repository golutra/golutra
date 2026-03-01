// 成员选择索引工具：统一 base/custom 的索引语义，便于持久化与回放。
import { BASE_TERMINALS } from '@/shared/constants/terminalCatalog';

// 0 表示内置终端组，1 表示自定义成员组；第二个值为组内索引。
export type MemberSelectionIndex = [0 | 1, number];

export const DEFAULT_MEMBER_INDEX: MemberSelectionIndex = [0, 0];

const isValidGroup = (value: number): value is 0 | 1 => value === 0 || value === 1;

/**
 * 解析外部输入为成员索引。
 * 输入：未知类型值。
 * 输出：合法索引或 null。
 * 边界：非数组、长度不足、非整数或负数均视为非法。
 */
export const parseMemberSelectionIndex = (value: unknown): MemberSelectionIndex | null => {
  if (!Array.isArray(value) || value.length < 2) {
    return null;
  }
  const group = Number(value[0]);
  const index = Number(value[1]);
  if (!Number.isInteger(group) || !Number.isInteger(index)) {
    return null;
  }
  if (!isValidGroup(group) || index < 0) {
    return null;
  }
  return [group, index];
};

/**
 * 将索引限制在有效范围内。
 * 输入：目标索引与自定义成员列表。
 * 输出：合法索引；超界回退到默认值。
 */
export const clampMemberSelectionIndex = (
  value: MemberSelectionIndex,
  customMembers: Array<{ id: string }>
): MemberSelectionIndex => {
  const [group, index] = value;
  if (group === 0) {
    return index >= 0 && index < BASE_TERMINALS.length ? value : DEFAULT_MEMBER_INDEX;
  }
  if (group === 1) {
    return index >= 0 && index < customMembers.length ? value : DEFAULT_MEMBER_INDEX;
  }
  return DEFAULT_MEMBER_INDEX;
};

/**
 * 解析并矫正索引，保证输出稳定可用。
 * 输入：外部值、自定义成员、可选回退值。
 * 输出：合法索引。
 */
export const normalizeMemberSelectionIndex = (
  value: unknown,
  customMembers: Array<{ id: string }>,
  fallback: MemberSelectionIndex = DEFAULT_MEMBER_INDEX
): MemberSelectionIndex => {
  const parsed = parseMemberSelectionIndex(value);
  return clampMemberSelectionIndex(parsed ?? fallback, customMembers);
};

/**
 * 通过索引找到成员 id。
 * 输入：索引与自定义成员列表。
 * 输出：对应 id；不存在则返回 null。
 */
export const resolveMemberIdFromSelectionIndex = (
  value: MemberSelectionIndex,
  customMembers: Array<{ id: string }>
): string | null => {
  const [group, index] = value;
  if (group === 0) {
    return BASE_TERMINALS[index]?.id ?? null;
  }
  if (group === 1) {
    return customMembers[index]?.id ?? null;
  }
  return null;
};

/**
 * 通过成员 id 反查索引位置。
 * 输入：成员 id 与自定义成员列表。
 * 输出：索引或 null。
 */
export const resolveMemberSelectionIndexFromId = (
  id: string,
  customMembers: Array<{ id: string }>
): MemberSelectionIndex | null => {
  const baseIndex = BASE_TERMINALS.findIndex((member) => member.id === id);
  if (baseIndex >= 0) {
    return [0, baseIndex];
  }
  const customIndex = customMembers.findIndex((member) => member.id === id);
  if (customIndex >= 0) {
    return [1, customIndex];
  }
  return null;
};
