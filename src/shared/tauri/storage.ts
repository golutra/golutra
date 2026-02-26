import { invoke } from '@tauri-apps/api/core';

export const readAppData = async <T>(relativePath: string): Promise<T | null> =>
  invoke<T | null>('storage_read_app', { relativePath });

export const writeAppData = async (relativePath: string, payload: unknown): Promise<void> =>
  invoke('storage_write_app', { relativePath, payload });

export const readCacheData = async <T>(relativePath: string): Promise<T | null> =>
  invoke<T | null>('storage_read_cache', { relativePath });

export const writeCacheData = async (relativePath: string, payload: unknown): Promise<void> =>
  invoke('storage_write_cache', { relativePath, payload });

export const readWorkspaceData = async <T>(workspacePath: string, relativePath: string): Promise<T | null> =>
  invoke<T | null>('storage_read_workspace', { workspacePath, relativePath });

export const writeWorkspaceData = async (
  workspacePath: string,
  relativePath: string,
  payload: unknown
): Promise<void> => invoke('storage_write_workspace', { workspacePath, relativePath, payload });
