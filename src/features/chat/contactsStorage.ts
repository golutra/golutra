// 联系人存储：负责联系人列表的持久化与数据清洗。
import { isTauri } from '@tauri-apps/api/core';

import { readAppData, writeAppData } from '@/shared/tauri/storage';
import { DEFAULT_AVATAR } from '@/shared/constants/avatars';
import type { Contact, MemberRole, MemberStatus } from './types';

// 联系人数据文件名，与应用级存储约定保持一致。
const CONTACTS_PATH = 'contacts.json';
const ALLOWED_ROLES = new Set<MemberRole>(['owner', 'admin', 'assistant', 'member']);
const ALLOWED_STATUSES = new Set<MemberStatus>(['online', 'working', 'dnd', 'offline']);

const normalizeRole = (value: unknown): MemberRole => {
  if (typeof value === 'string' && ALLOWED_ROLES.has(value as MemberRole)) {
    return value as MemberRole;
  }
  return 'admin';
};

const normalizeStatus = (value: unknown): MemberStatus => {
  if (typeof value === 'string' && ALLOWED_STATUSES.has(value as MemberStatus)) {
    return value as MemberStatus;
  }
  return 'offline';
};

const normalizeContact = (value: unknown): Contact | null => {
  if (!value || typeof value !== 'object') return null;
  const candidate = value as Partial<Contact>;
  const id = typeof candidate.id === 'string' ? candidate.id.trim() : '';
  const name = typeof candidate.name === 'string' ? candidate.name.trim() : '';
  if (!id || !name) return null;
  const avatar = typeof candidate.avatar === 'string' && candidate.avatar.trim() ? candidate.avatar : DEFAULT_AVATAR;
  const createdAt = typeof candidate.createdAt === 'number' ? candidate.createdAt : Date.now();
  return {
    id,
    name,
    avatar,
    roleType: normalizeRole(candidate.roleType),
    status: normalizeStatus(candidate.status),
    createdAt
  };
};

const normalizeContacts = (value: unknown): Contact[] => {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const out: Contact[] = [];
  for (const item of value) {
    const contact = normalizeContact(item);
    if (!contact || seen.has(contact.id)) continue;
    seen.add(contact.id);
    out.push(contact);
  }
  return out;
};

/**
 * 读取联系人列表。
 * 输入：无。
 * 输出：联系人数组；非 Tauri 环境返回空数组。
 */
export const loadContacts = async (): Promise<Contact[]> => {
  if (!isTauri()) {
    return [];
  }
  try {
    const stored = await readAppData<Contact[] | null>(CONTACTS_PATH);
    if (stored) {
      return normalizeContacts(stored);
    }
  } catch (error) {
    console.error('Failed to load contacts.', error);
  }
  return [];
};

/**
 * 保存联系人列表。
 * 输入：联系人数组。
 * 输出：无；非 Tauri 环境直接返回。
 */
export const saveContacts = async (contacts: Contact[]): Promise<void> => {
  if (!isTauri()) {
    return;
  }
  await writeAppData(CONTACTS_PATH, contacts);
};
