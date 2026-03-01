// 项目数据存储：负责工作区成员/路线图/技能的持久化。
import { computed, ref } from 'vue';
import { acceptHMRUpdate, defineStore, storeToRefs } from 'pinia';
import { readProjectData, writeProjectData } from '@/shared/tauri/projectData';
import { useWorkspaceStore } from './workspaceStore';
import type { WorkspaceEntry } from './workspaceStore';
import { CURRENT_USER_ID } from '@/features/chat/data';
import type { Member, RoadmapTask } from '@/features/chat/types';
import { DEFAULT_AVATAR } from '@/shared/constants/avatars';
import { normalizeAvatar } from '@/shared/utils/avatar';
import { hasTerminalConfig, normalizeTerminalCommand, normalizeTerminalPath, resolveTerminalType } from '@/shared/utils/terminal';
import type { TerminalConnectionStatus } from '@/shared/types/terminal';
import { logDiagnosticsEvent } from '@/shared/monitoring/diagnostics/logger';

export type ProjectSkill = {
  nameKey: string;
  icon: string;
  color: string;
  bg: string;
  ring: string;
  ver: string;
  tags?: string[];
};

export type ProjectTerminalRecentTab = {
  memberId: string;
  closedAt: number;
};

type ProjectData = {
  projectId: string;
  version: number;
  members: Member[];
  memberSequence?: Record<string, number>;
  terminal: {
    recentClosedTabs: ProjectTerminalRecentTab[];
  };
  roadmap: {
    objective: string;
    tasks: RoadmapTask[];
  };
  skills: {
    current: ProjectSkill[];
  };
};

// 默认 owner 名称。
const DEFAULT_OWNER_NAME = 'Owner';
const MEMBER_ID_FALLBACK_PREFIX = 'member';
const MEMBER_ID_FALLBACK_SLICE = 8;
const MAX_RECENT_CLOSED_TABS = 12;
// 仅允许有限状态值，避免被外部数据污染。
const ALLOWED_MEMBER_STATUSES = new Set<Member['status']>(['online', 'working', 'dnd', 'offline']);

// 规范化成员状态并提供默认值。
const normalizeMemberStatus = (status: Member['status'] | string | undefined): Member['status'] => {
  if (status && ALLOWED_MEMBER_STATUSES.has(status as Member['status'])) {
    return status as Member['status'];
  }
  return 'online';
};

// 手动状态允许为空，避免覆盖自动状态逻辑。
const normalizeManualStatus = (status: Member['status'] | string | null | undefined): Member['status'] | undefined => {
  if (!status) return undefined;
  if (ALLOWED_MEMBER_STATUSES.has(status as Member['status'])) {
    return status as Member['status'];
  }
  return undefined;
};

// 终端状态为运行时数据，不从持久化读取。
const normalizeTerminalStatus = (
  _status: TerminalConnectionStatus | Member['status'] | string | null | undefined,
  hasTerminal: boolean
): TerminalConnectionStatus | undefined => {
  if (!hasTerminal) {
    return undefined;
  }
  return 'pending';
};

// 角色展示键缺失时回退为成员默认值。
const resolveRoleKey = (roleKey: Member['roleKey'] | undefined, roleType: Member['roleType'] | string | undefined) => {
  const trimmed = typeof roleKey === 'string' ? roleKey.trim() : '';
  if (trimmed) {
    return trimmed;
  }
  if (roleType === 'owner') return 'members.roles.owner';
  if (roleType === 'admin') return 'members.roles.admin';
  if (roleType === 'assistant') return 'members.roles.aiAssistant';
  return 'members.roles.member';
};

// 名称缺失时使用短 id，避免 UI 空白。
const resolveMemberName = (candidate: string, memberId: string, isCurrentUser: boolean) => {
  const trimmed = candidate.trim();
  if (trimmed) {
    return trimmed;
  }
  if (isCurrentUser) {
    return DEFAULT_OWNER_NAME;
  }
  const prefix = memberId.trim().slice(0, MEMBER_ID_FALLBACK_SLICE);
  if (!prefix) {
    return DEFAULT_OWNER_NAME;
  }
  return `${MEMBER_ID_FALLBACK_PREFIX}-${prefix}`;
};

// 标准化成员数据，补齐头像与终端字段并保证一致性。
const normalizeMembers = (members: Member[]) =>
  members.map((member) => {
    const resolvedId = member.id;
    const rawName = typeof member.name === 'string' ? member.name : '';
    const resolvedName = resolveMemberName(rawName, resolvedId, resolvedId === CURRENT_USER_ID);
    const baseName = resolvedName || resolvedId;
    const terminalCommand = normalizeTerminalCommand(member.terminalCommand);
    const terminalPath = normalizeTerminalPath(member.terminalPath);
    const terminalType = resolveTerminalType(member.terminalType, terminalCommand);
    const hasTerminal = hasTerminalConfig(terminalType, terminalCommand);
    const rawStatus = normalizeMemberStatus(member.status);
    const rawManualStatus = normalizeManualStatus(member.manualStatus);
    const status = hasTerminal && rawStatus === 'working' ? 'online' : rawStatus;
    const manualStatus = hasTerminal && rawManualStatus === 'working' ? 'online' : rawManualStatus;
    const terminalStatus = normalizeTerminalStatus(member.terminalStatus, hasTerminal);
    // 默认启用终端会话，除非显式关闭。
    const autoStartTerminal =
      typeof member.autoStartTerminal === 'boolean'
        ? member.autoStartTerminal
        : hasTerminal;
    if (resolvedId !== CURRENT_USER_ID) {
      const avatar = normalizeAvatar(member.avatar, baseName);
      if (
        status === member.status &&
        manualStatus === member.manualStatus &&
        terminalStatus === member.terminalStatus &&
        avatar === member.avatar &&
        resolvedId === member.id &&
        autoStartTerminal === member.autoStartTerminal &&
        terminalType === member.terminalType &&
        terminalCommand === member.terminalCommand &&
        terminalPath === member.terminalPath
      ) {
        return member;
      }
      return {
        ...member,
        id: resolvedId,
        name: resolvedName,
        status,
        manualStatus,
        terminalStatus,
        avatar,
        autoStartTerminal,
        terminalType,
        terminalCommand,
        terminalPath
      };
    }
    const roleKey = resolveRoleKey(member.roleKey, member.roleType);
    const normalizedName = resolvedName;
    return {
      ...member,
      id: resolvedId,
      name: normalizedName,
      roleKey,
      status,
      manualStatus,
      terminalStatus,
      avatar: normalizeAvatar(member.avatar, normalizedName || baseName),
      autoStartTerminal,
      terminalType,
      terminalCommand,
      terminalPath
    };
  });

// 构造初始项目数据，保证最小可用状态。
const buildDefaultProjectData = (workspaceName: string, projectId: string): ProjectData => ({
  projectId,
  version: 1,
  memberSequence: {},
  members: [
    {
      id: CURRENT_USER_ID,
      name: 'Owner',
      role: '',
      roleKey: 'members.roles.owner',
      roleType: 'owner',
      avatar: DEFAULT_AVATAR,
      status: 'online'
    }
  ],
  terminal: {
    recentClosedTabs: []
  },
  roadmap: {
    objective: '',
    tasks: []
  },
  skills: {
    current: []
  }
});

// 过滤无效序号，避免前端写入异常结构。
const normalizeMemberSequence = (candidate: unknown): Record<string, number> => {
  if (!candidate || typeof candidate !== 'object' || Array.isArray(candidate)) {
    return {};
  }
  const entries = Object.entries(candidate as Record<string, unknown>);
  const normalized: Record<string, number> = {};
  entries.forEach(([key, value]) => {
    if (typeof key !== 'string' || !key.trim()) {
      return;
    }
    if (typeof value === 'number' && Number.isFinite(value) && value > 0) {
      normalized[key] = Math.floor(value);
    }
  });
  return normalized;
};

const normalizeRecentClosedTabs = (candidate: unknown): ProjectTerminalRecentTab[] => {
  if (!Array.isArray(candidate)) {
    return [];
  }
  const normalized: ProjectTerminalRecentTab[] = [];
  const memberSet = new Set<string>();
  for (const entry of candidate) {
    let memberId = '';
    let closedAt = 0;
    if (typeof entry === 'string') {
      memberId = entry.trim();
    } else if (entry && typeof entry === 'object') {
      const rawMemberId = (entry as { memberId?: unknown }).memberId;
      const rawClosedAt = (entry as { closedAt?: unknown }).closedAt;
      memberId = typeof rawMemberId === 'string' ? rawMemberId.trim() : '';
      closedAt = typeof rawClosedAt === 'number' && Number.isFinite(rawClosedAt) ? rawClosedAt : 0;
    }
    if (!memberId || memberSet.has(memberId)) {
      continue;
    }
    memberSet.add(memberId);
    normalized.push({ memberId, closedAt });
    if (normalized.length >= MAX_RECENT_CLOSED_TABS) {
      break;
    }
  }
  return normalized;
};

const parseMemberSequenceFromName = (name: string) => {
  const match = name.trim().match(/^(.*)-(\d+)$/);
  if (!match) {
    return null;
  }
  const base = match[1].trim();
  if (!base) {
    return null;
  }
  const index = Number.parseInt(match[2], 10);
  if (!Number.isFinite(index) || index <= 0) {
    return null;
  }
  return { base, nextIndex: index + 1 };
};

// 归一化项目数据，补齐缺省字段。
const normalizeProjectData = (data: Partial<ProjectData>, workspaceName: string, projectId: string): ProjectData => {
  const resolvedProjectId =
    typeof data.projectId === 'string' && data.projectId.trim() ? data.projectId.trim() : projectId;
  const defaults = buildDefaultProjectData(workspaceName, resolvedProjectId);
  return {
    projectId: defaults.projectId,
    version: typeof data.version === 'number' ? data.version : defaults.version,
    members: normalizeMembers(Array.isArray(data.members) && data.members.length > 0 ? data.members : defaults.members),
    memberSequence: normalizeMemberSequence(data.memberSequence),
    terminal: {
      recentClosedTabs: normalizeRecentClosedTabs(data.terminal?.recentClosedTabs)
    },
    roadmap: {
      objective:
        typeof data.roadmap?.objective === 'string' && data.roadmap.objective
          ? data.roadmap.objective
          : defaults.roadmap.objective,
      tasks: Array.isArray(data.roadmap?.tasks) && data.roadmap.tasks.length > 0 ? data.roadmap.tasks : defaults.roadmap.tasks
    },
    skills: {
      current: Array.isArray(data.skills?.current) && data.skills?.current.length > 0 ? data.skills.current : defaults.skills.current
    }
  };
};

const formatError = (error: unknown) => (error instanceof Error ? error.message : String(error));

// 防止并发加载覆盖结果的序号标记。
let loadSequence = 0;

/**
 * 项目数据存储。
 * 输入：外部调用 hydrate/reset 与成员/路线图修改。
 * 输出：当前项目数据与派生视图。
 */
export const useProjectStore = defineStore('project', () => {
  const workspaceStore = useWorkspaceStore();
  const { currentWorkspace, workspaceReadOnly } = storeToRefs(workspaceStore);

  const defaultState = () => ({
    projectData: null as ProjectData | null,
    loadingProject: false,
    projectError: null as string | null
  });

  const projectData = ref<ProjectData | null>(defaultState().projectData);
  const loadingProject = ref(defaultState().loadingProject);
  const projectError = ref<string | null>(defaultState().projectError);

  /**
   * 重置项目状态。
   * 输入：无。
   * 输出：清空数据并递增加载序号。
   */
  const reset = () => {
    const next = defaultState();
    projectData.value = next.projectData;
    loadingProject.value = next.loadingProject;
    projectError.value = next.projectError;
    loadSequence += 1;
  };

  /**
   * 读取并初始化项目数据。
   * 输入：无（依赖当前工作区）。
   * 输出：更新 projectData；失败时仅记录错误。
   */
  const hydrate = async () => {
    const workspace = currentWorkspace.value;
    if (!workspace) {
      projectData.value = null;
      return;
    }
    if (loadingProject.value) {
      return;
    }
    const requestId = ++loadSequence;
    const readOnly = workspaceReadOnly.value;
    loadingProject.value = true;
    projectError.value = null;

    try {
      let stored: ProjectData | null = null;
      try {
        const result = await readProjectData<ProjectData>(workspace.id, workspace.path);
        stored = result.data ?? null;
        if (result.warning) {
          projectError.value = result.warning;
          console.warn('Project data read warning.', result.warning);
        }
      } catch (error) {
        projectError.value = formatError(error);
        console.error('Failed to load project data.', error);
      }

      if (requestId !== loadSequence || workspace.id !== currentWorkspace.value?.id) {
        return;
      }

      let normalized = normalizeProjectData(stored ?? {}, workspace.name, workspace.id);
      projectData.value = normalized;

      if (!stored) {
        await persistProjectData(workspace, readOnly);
      }
    } finally {
      if (requestId === loadSequence) {
        loadingProject.value = false;
      }
    }
  };

  // 终端状态为运行时数据，持久化时需要剥离。
  const buildPersistPayload = (data: ProjectData): ProjectData => ({
    ...data,
    members: data.members.map((member) => {
      const { terminalStatus: _terminalStatus, ...rest } = member;
      void _terminalStatus;
      return rest;
    })
  });

  /**
   * 持久化项目数据。
   * 输入：工作区与只读标记。
   * 输出：无；失败时记录错误，由后端负责回退策略。
   */
  const persistProjectData = async (workspace: WorkspaceEntry, readOnly: boolean) => {
    if (!projectData.value) return;
    const payload = buildPersistPayload(projectData.value);
    try {
      const result = await writeProjectData(workspace.id, workspace.path, readOnly, payload);
      if (result.warning) {
        projectError.value = result.warning;
        console.warn('Project data write warning.', result.warning);
      }
    } catch (error) {
      projectError.value = formatError(error);
      console.error('Failed to persist project data.', error);
    }
  };

  const members = computed(() => projectData.value?.members ?? []);
  const roadmap = computed(() => projectData.value?.roadmap ?? { objective: '', tasks: [] });
  const currentSkills = computed(() => projectData.value?.skills.current ?? []);

  const setMembers = async (nextMembers: Member[]) => {
    const workspace = currentWorkspace.value;
    if (!projectData.value || !workspace) return;
    const readOnly = workspaceReadOnly.value;
    projectData.value = { ...projectData.value, members: nextMembers };
    await persistProjectData(workspace, readOnly);
  };

  const applyMembers = (nextMembers: Member[]) => {
    if (!projectData.value) return;
    const existingById = new Map(projectData.value.members.map((member) => [member.id, member]));
    const merged = nextMembers.map((member) => {
      const existing = existingById.get(member.id);
      if (!existing) {
        return member;
      }
      return {
        ...member,
        terminalStatus: existing.terminalStatus ?? member.terminalStatus
      };
    });
    projectData.value = { ...projectData.value, members: merged };
  };

  const addMember = async (member: Member) => {
    await logDiagnosticsEvent('add-member', {
      memberId: member.id,
      name: member.name,
      roleType: member.roleType,
      terminalType: member.terminalType,
      terminalCommand: member.terminalCommand
    });
    await setMembers([...(projectData.value?.members ?? []), member]);
  };

  const updateMember = async (id: string, updates: Partial<Member>, options?: { persist?: boolean }) => {
    if (!projectData.value) {
      return;
    }
    const nextMembers = projectData.value.members.map((member) => (member.id === id ? { ...member, ...updates } : member));
    if (options?.persist === false) {
      projectData.value = { ...projectData.value, members: nextMembers };
      return;
    }
    await setMembers(nextMembers);
  };

  const removeMember = async (id: string) => {
    if (!projectData.value) return;
    const nextMembers = projectData.value.members.filter((member) => member.id !== id);
    await setMembers(nextMembers);
  };

  const updateRoadmap = async (nextRoadmap: ProjectData['roadmap']) => {
    const workspace = currentWorkspace.value;
    if (!projectData.value || !workspace) return;
    const readOnly = workspaceReadOnly.value;
    projectData.value = { ...projectData.value, roadmap: nextRoadmap };
    await persistProjectData(workspace, readOnly);
  };

  const updateMemberSequence = async (names: string[]) => {
    const workspace = currentWorkspace.value;
    if (!projectData.value || !workspace || names.length === 0) {
      return;
    }
    const readOnly = workspaceReadOnly.value;
    const currentSequence = projectData.value.memberSequence ?? {};
    const nextSequence = { ...currentSequence };
    let changed = false;
    names.forEach((name) => {
      const parsed = parseMemberSequenceFromName(name);
      if (!parsed) {
        return;
      }
      const existing = nextSequence[parsed.base];
      if (typeof existing !== 'number' || existing < parsed.nextIndex) {
        nextSequence[parsed.base] = parsed.nextIndex;
        changed = true;
      }
    });
    if (!changed) {
      return;
    }
    projectData.value = { ...projectData.value, memberSequence: nextSequence };
    await persistProjectData(workspace, readOnly);
  };

  const updateCurrentSkills = async (nextSkills: ProjectSkill[]) => {
    const workspace = currentWorkspace.value;
    if (!projectData.value || !workspace) return;
    const readOnly = workspaceReadOnly.value;
    projectData.value = { ...projectData.value, skills: { current: nextSkills } };
    await persistProjectData(workspace, readOnly);
  };

  return {
    projectData,
    loadingProject,
    projectError,
    members,
    roadmap,
    currentSkills,
    hydrate,
    reset,
    applyMembers,
    addMember,
    updateMember,
    removeMember,
    updateRoadmap,
    updateMemberSequence,
    updateCurrentSkills
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useProjectStore, import.meta.hot));
}

