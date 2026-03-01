// 联系人状态存储：负责联系人列表加载、排序与持久化。
import { defineStore } from 'pinia';
import { ref } from 'vue';

import type { Contact } from './types';
import { loadContacts, saveContacts } from './contactsStorage';

const sortContacts = (items: Contact[]) =>
  [...items].sort((a, b) => {
    if (a.createdAt !== b.createdAt) {
      return b.createdAt - a.createdAt;
    }
    return a.name.localeCompare(b.name);
  });

/**
 * 联系人状态存储。
 * 输入：加载/更新/删除操作。
 * 输出：联系人列表与操作函数。
 */
export const useContactsStore = defineStore('contacts', () => {
  const contacts = ref<Contact[]>([]);
  const isReady = ref(false);
  const contactError = ref<string | null>(null);

  /**
   * 设置联系人列表并持久化。
   * 输入：联系人数组。
   * 输出：无。
   */
  const setContacts = async (next: Contact[]) => {
    const sorted = sortContacts(next);
    contacts.value = sorted;
    try {
      await saveContacts(sorted);
    } catch (error) {
      contactError.value = error instanceof Error ? error.message : String(error);
      console.error('Failed to persist contacts.', error);
    }
  };

  /**
   * 加载联系人列表（仅执行一次）。
   * 输入：无。
   * 输出：无；失败时记录错误。
   */
  const load = async () => {
    if (isReady.value) return;
    contactError.value = null;
    try {
      const stored = await loadContacts();
      contacts.value = sortContacts(stored);
    } catch (error) {
      contactError.value = error instanceof Error ? error.message : String(error);
      console.error('Failed to read contacts.', error);
    } finally {
      isReady.value = true;
    }
  };

  /**
   * 更新或新增联系人。
   * 输入：联系人对象。
   * 输出：无。
   */
  const upsertContact = async (contact: Contact) => {
    const existingIndex = contacts.value.findIndex((item) => item.id === contact.id);
    const next = [...contacts.value];
    if (existingIndex >= 0) {
      next[existingIndex] = contact;
    } else {
      next.push(contact);
    }
    await setContacts(next);
  };

  /**
   * 删除联系人。
   * 输入：联系人 id。
   * 输出：无。
   */
  const removeContact = async (id: string) => {
    await setContacts(contacts.value.filter((contact) => contact.id !== id));
  };

  return {
    contacts,
    contactError,
    isReady,
    load,
    setContacts,
    upsertContact,
    removeContact
  };
});
