export type ImportedSkillFolder = {
  id: string;
  name: string;
  path: string;
  addedAt: number;
};

export type GlobalData = {
  version: number;
  installedSkills: number[];
  installedPlugins: number[];
  importedSkillFolders: ImportedSkillFolder[];
};

export type SkillFolderSource = {
  name: string;
  path: string;
};

export const buildDefaultGlobalData = (): GlobalData => ({
  version: 1,
  installedSkills: [],
  installedPlugins: [],
  importedSkillFolders: []
});

const normalizeFolderName = (name: string, path: string) =>
  name.trim() || path.split(/[\\/]/).filter(Boolean).pop() || path;

export const normalizeImportedSkillFolders = (candidate?: unknown): ImportedSkillFolder[] => {
  if (!Array.isArray(candidate)) {
    return [];
  }
  const folders: ImportedSkillFolder[] = [];
  const seenPaths = new Set<string>();
  for (const entry of candidate) {
    if (!entry || typeof entry !== 'object') {
      continue;
    }
    const record = entry as Partial<ImportedSkillFolder>;
    const path = typeof record.path === 'string' ? record.path.trim() : '';
    if (!path || seenPaths.has(path)) {
      continue;
    }
    const name = normalizeFolderName(typeof record.name === 'string' ? record.name : '', path);
    const id = typeof record.id === 'string' && record.id.trim() ? record.id.trim() : path;
    const addedAt =
      typeof record.addedAt === 'number' && Number.isFinite(record.addedAt)
        ? record.addedAt
        : 0;
    folders.push({ id, name, path, addedAt });
    seenPaths.add(path);
  }
  return folders;
};

export const normalizeGlobalData = (candidate?: Partial<GlobalData>): GlobalData => {
  const defaults = buildDefaultGlobalData();
  const version = Number(candidate?.version);
  const installedSkills = Array.isArray(candidate?.installedSkills)
    ? candidate?.installedSkills
    : defaults.installedSkills;
  const installedPlugins = Array.isArray(candidate?.installedPlugins)
    ? candidate?.installedPlugins
    : defaults.installedPlugins;
  const importedSkillFolders = normalizeImportedSkillFolders(candidate?.importedSkillFolders);
  return {
    version: Number.isFinite(version) && version > 0 ? version : defaults.version,
    installedSkills,
    installedPlugins,
    importedSkillFolders
  };
};

const normalizeSkillFolderSources = (candidate?: SkillFolderSource[] | null): SkillFolderSource[] => {
  if (!Array.isArray(candidate)) {
    return [];
  }
  const folders: SkillFolderSource[] = [];
  const seenPaths = new Set<string>();
  for (const entry of candidate) {
    if (!entry || typeof entry !== 'object') {
      continue;
    }
    const path = typeof entry.path === 'string' ? entry.path.trim() : '';
    if (!path || seenPaths.has(path)) {
      continue;
    }
    const name = normalizeFolderName(typeof entry.name === 'string' ? entry.name : '', path);
    folders.push({ name, path });
    seenPaths.add(path);
  }
  return folders;
};

export const reconcileImportedSkillFolders = (
  storedCandidate: unknown,
  actualCandidate: SkillFolderSource[] | null | undefined,
  now = Date.now()
): ImportedSkillFolder[] => {
  const stored = normalizeImportedSkillFolders(storedCandidate);
  const actual = normalizeSkillFolderSources(actualCandidate);
  if (actual.length === 0 && !Array.isArray(actualCandidate)) {
    return stored;
  }

  const storedByPath = new Map(stored.map((folder) => [folder.path, folder] as const));
  const storedByName = new Map(stored.map((folder) => [folder.name.toLowerCase(), folder] as const));

  return actual.map((folder) => {
    const existing =
      storedByPath.get(folder.path) ?? storedByName.get(folder.name.toLowerCase()) ?? null;
    const addedAt = existing && existing.addedAt > 0 ? existing.addedAt : now;
    return {
      id: folder.path,
      name: folder.name,
      path: folder.path,
      addedAt
    };
  });
};

export const sameImportedSkillFolders = (
  left: ImportedSkillFolder[],
  right: ImportedSkillFolder[]
): boolean => {
  if (left.length !== right.length) {
    return false;
  }
  return left.every((folder, index) => {
    const other = right[index];
    return (
      folder.id === other.id &&
      folder.name === other.name &&
      folder.path === other.path &&
      folder.addedAt === other.addedAt
    );
  });
};
