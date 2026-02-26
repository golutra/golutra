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

export const useContactsStore = defineStore('contacts', () => {
  const contacts = ref<Contact[]>([]);
  const isReady = ref(false);
  const contactError = ref<string | null>(null);

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
