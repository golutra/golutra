import { BASE_TERMINALS } from '@/shared/constants/terminalCatalog';

export type MemberSelectionIndex = [0 | 1, number];

export const DEFAULT_MEMBER_INDEX: MemberSelectionIndex = [0, 0];

const isValidGroup = (value: number): value is 0 | 1 => value === 0 || value === 1;

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

export const normalizeMemberSelectionIndex = (
  value: unknown,
  customMembers: Array<{ id: string }>,
  fallback: MemberSelectionIndex = DEFAULT_MEMBER_INDEX
): MemberSelectionIndex => {
  const parsed = parseMemberSelectionIndex(value);
  return clampMemberSelectionIndex(parsed ?? fallback, customMembers);
};

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
