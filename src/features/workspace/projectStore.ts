import { computed, ref } from 'vue';
import { acceptHMRUpdate, defineStore, storeToRefs } from 'pinia';
import { readAppData, readWorkspaceData, writeAppData, writeWorkspaceData } from '@/shared/tauri/storage';
import { useWorkspaceStore } from './workspaceStore';
import type { WorkspaceEntry } from './workspaceStore';
import { AI_ASSISTANT_ID, CURRENT_USER_ID } from '@/features/chat/data';
import type { Member, RoadmapTask } from '@/features/chat/types';
import { DEFAULT_AVATAR } from '@/shared/constants/avatars';
import { normalizeAvatar } from '@/shared/utils/avatar';
import { hasTerminalConfig, normalizeTerminalCommand, normalizeTerminalPath, resolveTerminalType } from '@/shared/utils/terminal';
import type { TerminalConnectionStatus } from '@/shared/types/terminal';

export type ProjectSkill = {
  nameKey: string;
  icon: string;
  color: string;
  bg: string;
  ring: string;
  ver: string;
  tags?: string[];
};

type ProjectData = {
  projectId: string;
  version: number;
  members: Member[];
  roadmap: {
    objective: string;
    tasks: RoadmapTask[];
  };
  skills: {
    current: ProjectSkill[];
  };
};

const PROJECT_DATA_PATH = '.golutra/workspace.json';
const projectDataAppPath = (workspaceId: string) => `${workspaceId}/project.json`;
const DEFAULT_OWNER_NAME = 'Owner';
const LEGACY_OWNER_NAME = 'You (Owner)';
const LEGACY_OWNER_ROLE_KEY = 'members.activity.reviewingPRs';
const LEGACY_CURRENT_USER_ID = 'me';
const LEGACY_ASSISTANT_ID = 'ai-assistant';
const ALLOWED_MEMBER_STATUSES = new Set<Member['status']>(['online', 'working', 'dnd', 'offline']);
const ALLOWED_TERMINAL_STATUSES = new Set<TerminalConnectionStatus>([
  'connecting',
  'connected',
  'working',
  'disconnected'
]);

const normalizeMemberStatus = (status: Member['status'] | string | undefined): Member['status'] => {
  if (status === 'away') return 'working';
  if (status && ALLOWED_MEMBER_STATUSES.has(status as Member['status'])) {
    return status as Member['status'];
  }
  return 'online';
};

const normalizeManualStatus = (status: Member['status'] | string | null | undefined): Member['status'] | undefined => {
  if (!status) return undefined;
  if (status === 'away') return 'working';
  if (ALLOWED_MEMBER_STATUSES.has(status as Member['status'])) {
    return status as Member['status'];
  }
  return undefined;
};

const normalizeTerminalStatus = (
  status: TerminalConnectionStatus | Member['status'] | string | null | undefined,
  hasTerminal: boolean
): TerminalConnectionStatus | undefined => {
  if (!hasTerminal) {
    return undefined;
  }
  if (!status) {
    return 'disconnected';
  }
  if (status === 'online') {
    return 'connected';
  }
  if (status === 'offline') {
    return 'disconnected';
  }
  if (ALLOWED_TERMINAL_STATUSES.has(status as TerminalConnectionStatus)) {
    return status as TerminalConnectionStatus;
  }
  return 'disconnected';
};

const normalizeMembers = (members: Member[]) =>
  members.map((member) => {
    const resolvedId =
      member.id === LEGACY_CURRENT_USER_ID
        ? CURRENT_USER_ID
        : member.id === LEGACY_ASSISTANT_ID
          ? AI_ASSISTANT_ID
          : member.id;
    const name = typeof member.name === 'string' ? member.name.trim() : '';
    const baseName = name || resolvedId;
    const terminalCommand = normalizeTerminalCommand(member.terminalCommand);
    const terminalPath = normalizeTerminalPath(member.terminalPath);
    const terminalType = resolveTerminalType(member.terminalType, terminalCommand);
    const hasTerminal = hasTerminalConfig(terminalType, terminalCommand);
    const rawStatus = normalizeMemberStatus(member.status);
    const rawManualStatus = normalizeManualStatus(member.manualStatus);
    const status = hasTerminal && rawStatus === 'working' ? 'online' : rawStatus;
    const manualStatus = hasTerminal && rawManualStatus === 'working' ? 'online' : rawManualStatus;
    const terminalStatus = normalizeTerminalStatus(member.terminalStatus, hasTerminal);
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
    const roleKey =
      !member.roleKey || member.roleKey === LEGACY_OWNER_ROLE_KEY ? 'members.roles.owner' : member.roleKey;
    const normalizedName = !name || name === LEGACY_OWNER_NAME ? DEFAULT_OWNER_NAME : member.name;
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

const applyProjectPrefix = (name: string, projectName: string) => {
  const trimmedName = name.trim();
  const trimmedProject = projectName.trim();
  if (!trimmedProject) {
    return trimmedName;
  }
  if (!trimmedName) {
    return trimmedProject;
  }
  const prefix = `${trimmedProject.toLowerCase()}-`;
  if (trimmedName.toLowerCase().startsWith(prefix)) {
    return trimmedName;
  }
  return `${trimmedProject}-${trimmedName}`;
};

const prefixTerminalMemberNames = (members: Member[], projectName: string) => {
  const trimmedProject = projectName.trim();
  if (!trimmedProject) {
    return { members, changed: false };
  }
  const used = new Set(members.map((member) => member.name.toLowerCase()));
  let changed = false;
  const next = [...members];
  for (let i = 0; i < next.length; i += 1) {
    const member = next[i];
    if (!hasTerminalConfig(member.terminalType, member.terminalCommand)) {
      continue;
    }
    const baseName = member.name?.trim() || member.id;
    const desired = applyProjectPrefix(baseName, trimmedProject);
    if (desired.toLowerCase() === member.name.toLowerCase()) {
      continue;
    }
    used.delete(member.name.toLowerCase());
    let candidate = desired;
    let counter = 1;
    while (used.has(candidate.toLowerCase())) {
      counter += 1;
      candidate = `${desired}-${counter}`;
    }
    used.add(candidate.toLowerCase());
    next[i] = { ...member, name: candidate };
    changed = true;
  }
  return { members: next, changed };
};

const buildDefaultProjectData = (workspaceName: string, projectId: string): ProjectData => ({
  projectId,
  version: 1,
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
  roadmap: {
    objective: '',
    tasks: []
  },
  skills: {
    current: []
  }
});

const normalizeProjectData = (data: Partial<ProjectData>, workspaceName: string, projectId: string): ProjectData => {
  const resolvedProjectId =
    typeof data.projectId === 'string' && data.projectId.trim() ? data.projectId.trim() : projectId;
  const defaults = buildDefaultProjectData(workspaceName, resolvedProjectId);
  return {
    projectId: defaults.projectId,
    version: typeof data.version === 'number' ? data.version : defaults.version,
    members: normalizeMembers(Array.isArray(data.members) && data.members.length > 0 ? data.members : defaults.members),
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

let loadSequence = 0;

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

  const reset = () => {
    const next = defaultState();
    projectData.value = next.projectData;
    loadingProject.value = next.loadingProject;
    projectError.value = next.projectError;
    loadSequence += 1;
  };

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
        stored = await readWorkspaceData<ProjectData>(workspace.path, PROJECT_DATA_PATH);
        if (!stored) {
          stored = await readAppData<ProjectData>(projectDataAppPath(workspace.id));
        }
      } catch (error) {
        projectError.value = formatError(error);
        console.error('Failed to load project data.', error);
      }

      if (requestId !== loadSequence || workspace.id !== currentWorkspace.value?.id) {
        return;
      }

      let normalized = normalizeProjectData(stored ?? {}, workspace.name, workspace.id);
      const prefixed = prefixTerminalMemberNames(normalized.members, workspace.name);
      if (prefixed.changed) {
        normalized = { ...normalized, members: prefixed.members };
      }
      projectData.value = normalized;

      if (!stored) {
        try {
          if (!readOnly) {
            await writeWorkspaceData(workspace.path, PROJECT_DATA_PATH, normalized);
          }
        } catch (error) {
          projectError.value = formatError(error);
          console.error('Failed to initialize project data.', error);
          try {
            await writeAppData(projectDataAppPath(workspace.id), normalized);
          } catch (fallbackError) {
            projectError.value = formatError(fallbackError);
            console.error('Failed to initialize project data in app storage.', fallbackError);
          }
        }
        if (readOnly) {
          try {
            await writeAppData(projectDataAppPath(workspace.id), normalized);
          } catch (error) {
            projectError.value = formatError(error);
            console.error('Failed to initialize project data in app storage.', error);
          }
        }
      } else if (prefixed.changed) {
        await persistProjectData(workspace, readOnly);
      }
    } finally {
      if (requestId === loadSequence) {
        loadingProject.value = false;
      }
    }
  };

  const persistProjectData = async (workspace: WorkspaceEntry, readOnly: boolean) => {
    if (!projectData.value || readOnly) {
      if (!projectData.value) return;
      try {
        await writeAppData(projectDataAppPath(workspace.id), projectData.value);
      } catch (error) {
        projectError.value = formatError(error);
        console.error('Failed to persist project data to app storage.', error);
      }
      return;
    }
    try {
      await writeWorkspaceData(workspace.path, PROJECT_DATA_PATH, projectData.value);
    } catch (error) {
      projectError.value = formatError(error);
      console.error('Failed to persist project data.', error);
      try {
        await writeAppData(projectDataAppPath(workspace.id), projectData.value);
      } catch (fallbackError) {
        projectError.value = formatError(fallbackError);
        console.error('Failed to persist project data to app storage.', fallbackError);
      }
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

  const addMember = async (member: Member) => {
    await setMembers([...(projectData.value?.members ?? []), member]);
  };

  const updateMember = async (id: string, updates: Partial<Member>, options?: { persist?: boolean }) => {
    if (!projectData.value) return;
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
    addMember,
    updateMember,
    removeMember,
    updateRoadmap,
    updateCurrentSkills
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useProjectStore, import.meta.hot));
}
