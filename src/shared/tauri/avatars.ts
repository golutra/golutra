// Tauri 头像资源封装：统一资产列表、存储与读取的调用入口。
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

/**
 * 获取头像资源列表。
 * 输入：无。
 * 输出：资源数组。
 */
export const listAvatarAssets = async (): Promise<AvatarAsset[]> => invoke<AvatarAsset[]>('avatar_list');

/**
 * 存储头像资源。
 * 输入：字节数组与可选扩展名。
 * 输出：创建后的资源信息。
 */
export const storeAvatarAsset = async (bytes: number[], extension?: string): Promise<AvatarAsset> =>
  invoke<AvatarAsset>('avatar_store', { bytes, extension });

/**
 * 删除头像资源。
 * 输入：资源 id。
 * 输出：是否删除成功。
 */
export const deleteAvatarAsset = async (id: string): Promise<boolean> =>
  invoke<boolean>('avatar_delete', { id });

/**
 * 解析头像资源的文件路径。
 * 输入：资源 id。
 * 输出：本地路径字符串。
 */
export const resolveAvatarPath = async (id: string): Promise<string> =>
  invoke<string>('avatar_resolve_path', { id });

/**
 * 读取头像资源字节内容。
 * 输入：资源 id。
 * 输出：字节数组与 MIME。
 */
export const readAvatarAsset = async (id: string): Promise<AvatarContent> =>
  invoke<AvatarContent>('avatar_read', { id });
