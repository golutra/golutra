import { invoke } from '@tauri-apps/api/core';

export type AvatarAsset = {
  id: string;
  filename: string;
  createdAt: number;
};

export type AvatarContent = {
  bytes: number[];
  mime: string;
};

export const listAvatarAssets = async (): Promise<AvatarAsset[]> => invoke<AvatarAsset[]>('avatar_list');

export const storeAvatarAsset = async (bytes: number[], extension?: string): Promise<AvatarAsset> =>
  invoke<AvatarAsset>('avatar_store', { bytes, extension });

export const deleteAvatarAsset = async (id: string): Promise<boolean> =>
  invoke<boolean>('avatar_delete', { id });

export const resolveAvatarPath = async (id: string): Promise<string> =>
  invoke<string>('avatar_resolve_path', { id });

export const readAvatarAsset = async (id: string): Promise<AvatarContent> =>
  invoke<AvatarContent>('avatar_read', { id });
