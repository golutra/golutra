// [2026-01-23 11:12] 目的: 复用好友邀请创建逻辑与菜单状态，统一好友页与默认频道入口; 边界: 仅处理邀请成员/联系人与终端联动，不负责布局与导航; 设计: 使用 composable 聚合状态与动作以减少重复代码。
import { ref } from 'vue';
import { storeToRefs } from 'pinia';

import { useProjectStore } from '@/features/workspace/projectStore';
import { useWorkspaceStore } from '@/features/workspace/workspaceStore';
import { useTerminalOrchestratorStore } from '@/stores/terminalOrchestratorStore';
import { useToastStore } from '@/stores/toastStore';
import { buildSeededAvatar } from '@/shared/utils/avatar';
import { hasTerminalConfig } from '@/shared/utils/terminal';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';
import { inviteProjectMembers } from '@/shared/tauri/projectData';
import type { TerminalType } from '@/shared/types/terminal';
import { generateUlid } from './chatBridge';
import { CURRENT_USER_ID } from './data';
import { useChatStore } from './chatStore';
import { useContactsStore } from './contactsStore';
import type { Contact, Member } from './types';

export type InviteModalType = 'admin' | 'assistant' | 'member' | null;
export type InviteRole = 'assistant' | 'member';
export type InviteModel = {
  id: string;
  label: string;
  command: string;
  terminalType: TerminalType;
  instances: number;
  unlimitedAccess: boolean;
  sandboxed: boolean;
};

type InviteRoleConfig = {
  shouldStartSession: (member: Member) => boolean;
};

export const useFriendInvites = () => {
  const projectStore = useProjectStore();
  const workspaceStore = useWorkspaceStore();
  const chatStore = useChatStore();
  const terminalOrchestratorStore = useTerminalOrchestratorStore();
  const toastStore = useToastStore();
  const contactsStore = useContactsStore();

  const { members } = storeToRefs(projectStore);
  const { currentWorkspace } = storeToRefs(workspaceStore);
  const { conversations, currentUser } = storeToRefs(chatStore);
  const { contacts } = storeToRefs(contactsStore);

  const { applyMembers, updateMember, updateMemberSequence } = projectStore;
  const { setConversationMembers } = chatStore;
  const { ensureMemberSession } = terminalOrchestratorStore;
  const { pushToast } = toastStore;
  const { upsertContact } = contactsStore;

  const showInviteMenu = ref(false);
  const activeModalType = ref<InviteModalType>(null);

  const toggleInviteMenu = () => {
    showInviteMenu.value = !showInviteMenu.value;
  };

  const handleInviteSelect = (type: Exclude<InviteModalType, null>) => {
    showInviteMenu.value = false;
    activeModalType.value = type;
  };

  const ensureUniqueContactName = (name: string, roster: Array<{ name: string }>) => {
    const trimmed = name.trim();
    if (!trimmed) return '';
    const lowerNames = new Set(roster.map((entry) => entry.name.toLowerCase()));
    if (!lowerNames.has(trimmed.toLowerCase())) {
      return trimmed;
    }
    let counter = 1;
    let candidate = `${trimmed}-${counter}`;
    while (lowerNames.has(candidate.toLowerCase())) {
      counter += 1;
      candidate = `${trimmed}-${counter}`;
    }
    return candidate;
  };

  const createMemberId = async () => {
    try {
      return await generateUlid();
    } catch (error) {
      console.error('Failed to generate member id.', error);
      pushToast('Failed to generate member id.', { tone: 'error' });
      return null;
    }
  };

  const ROLE_CONFIGS: Record<InviteRole, InviteRoleConfig> = {
    assistant: {
      shouldStartSession: (member) => hasTerminalConfig(member.terminalType, member.terminalCommand)
    },
    member: {
      shouldStartSession: () => true
    }
  };

  const normalizeInviteInstances = (value: number) => {
    if (!Number.isFinite(value)) {
      return 1;
    }
    return Math.max(1, Math.round(value));
  };

  const syncDefaultChannelMembers = async () => {
    const defaultConversation = conversations.value.find((item) => item.isDefault);
    if (!defaultConversation) return;
    const currentUserId = currentUser.value?.id ?? CURRENT_USER_ID;
    const memberIds = Array.from(new Set([currentUserId, ...members.value.map((member) => member.id)]));
    await setConversationMembers(defaultConversation.id, memberIds);
  };

  const handleAdminInvite = async (payload: { identifier: string }) => {
    const trimmed = payload.identifier.trim();
    if (!trimmed) {
      return;
    }
    const id = await createMemberId();
    if (!id) {
      return;
    }
    const name = ensureUniqueContactName(trimmed, [
      ...members.value.map((member) => ({ name: member.name })),
      ...contacts.value.map((contact) => ({ name: contact.name }))
    ]);
    if (!name) {
      return;
    }
    const contact: Contact = {
      id,
      name,
      avatar: buildSeededAvatar(`admin:${name}`),
      roleType: 'admin',
      status: 'offline',
      createdAt: Date.now()
    };
    await upsertContact(contact);
    activeModalType.value = null;
  };

  const handleInvite = async (model: InviteModel, type: InviteRole) => {
    const workspace = currentWorkspace.value;
    if (!workspace) {
      pushToast('请先打开工作区', { tone: 'warning' });
      return;
    }
    const instanceCount = normalizeInviteInstances(model.instances);
    const access = { unlimitedAccess: model.unlimitedAccess, sandboxed: model.sandboxed };
    const command = model.command?.trim() ?? '';
    const config = ROLE_CONFIGS[type];
    await logDiagnosticsEvent('handle-invite', {
      role: type,
      terminalType: model.terminalType,
      terminalCommand: command,
      instances: instanceCount,
      unlimitedAccess: access.unlimitedAccess,
      sandboxed: access.sandboxed
    });
    let result: { members: Member[]; createdMembers: Member[]; warning?: string | null };
    try {
      result = await inviteProjectMembers<Member>(workspace.id, {
        roleType: type,
        command: command || undefined,
        terminalType: model.terminalType,
        instanceCount,
        unlimitedAccess: access.unlimitedAccess,
        sandboxed: access.sandboxed
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      pushToast(message || '邀请失败', { tone: 'error' });
      return;
    }
    if (result.warning) {
      console.warn('Project invite warning.', result.warning);
    }
    if (result.members.length > 0) {
      applyMembers(result.members);
    }
    if (result.createdMembers.length > 0) {
      const names = result.createdMembers.map((member) => member.name);
      await updateMemberSequence(names);
    }
    for (const [index, member] of result.createdMembers.entries()) {
      await logDiagnosticsEvent('build-member', {
        memberId: member.id,
        name: member.name,
        roleType: member.roleType,
        terminalType: member.terminalType,
        terminalCommand: member.terminalCommand,
        index: index + 1,
        total: result.createdMembers.length
      });
    }
    await syncDefaultChannelMembers();
    activeModalType.value = null;
    for (const member of result.createdMembers) {
      if (!config.shouldStartSession(member)) {
        continue;
      }
      const inviteMeta = {
        memberName: member.name,
        defaultCommand: command,
        instanceCount,
        unlimitedAccess: access.unlimitedAccess,
        sandboxed: access.sandboxed
      };
      const session = await ensureMemberSession(member, { openTab: false, inviteMeta });
      if (!session) {
        await updateMember(member.id, { status: 'offline' });
      }
    }
  };

  return {
    showInviteMenu,
    activeModalType,
    toggleInviteMenu,
    handleInviteSelect,
    handleAdminInvite,
    handleInvite,
    syncDefaultChannelMembers
  };
};

