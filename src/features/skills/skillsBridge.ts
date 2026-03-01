// Skills IPC bridge: import skill folders through the UI gateway.
import { invoke, isTauri } from '@tauri-apps/api/core';

export type SkillFolderImportResult = {
  folderName: string;
  destPath: string;
};

export type ProjectSkillLink = {
  name: string;
  linkPath: string;
  targetPath: string;
};

export const importSkillFolder = async (): Promise<SkillFolderImportResult | null> => {
  if (!isTauri()) {
    return null;
  }
  return invoke<SkillFolderImportResult | null>('skills_import_folder');
};

export const removeSkillFolder = async (path: string): Promise<boolean> => {
  if (!isTauri()) {
    return false;
  }
  return invoke<boolean>('skills_remove_folder', { path });
};

export const openSkillFolder = async (path: string): Promise<void> => {
  if (!isTauri()) {
    return;
  }
  await invoke('skills_open_folder', { path });
};

export const listProjectSkillLinks = async (workspacePath: string): Promise<ProjectSkillLink[]> => {
  if (!isTauri()) {
    return [];
  }
  return invoke<ProjectSkillLink[]>('project_skills_list', { workspacePath });
};

export const linkProjectSkill = async (
  workspacePath: string,
  sourcePath: string
): Promise<ProjectSkillLink> => {
  if (!isTauri()) {
    throw new Error('project skill linking is only available in the desktop app');
  }
  return invoke<ProjectSkillLink>('project_skills_link', { workspacePath, sourcePath });
};

export const unlinkProjectSkill = async (workspacePath: string, linkName: string): Promise<boolean> => {
  if (!isTauri()) {
    throw new Error('project skill unlinking is only available in the desktop app');
  }
  return invoke<boolean>('project_skills_unlink', { workspacePath, linkName });
};
