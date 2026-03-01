// Tauri 存储封装：统一 App/Cache/Workspace 三类目录的读写入口。
import { invoke } from '@tauri-apps/api/core';

/**
 * 读取应用级数据。
 * 输入：相对路径。
 * 输出：解析后的数据或 null。
 */
export const readAppData = async <T>(relativePath: string): Promise<T | null> =>
  invoke<T | null>('storage_read_app', { relativePath });

/**
 * 写入应用级数据。
 * 输入：相对路径与待写入 payload。
 * 输出：无；错误由 Tauri 调用抛出。
 */
export const writeAppData = async (relativePath: string, payload: unknown): Promise<void> =>
  invoke('storage_write_app', { relativePath, payload });

/**
 * 读取缓存数据。
 * 输入：相对路径。
 * 输出：解析后的数据或 null。
 */
export const readCacheData = async <T>(relativePath: string): Promise<T | null> =>
  invoke<T | null>('storage_read_cache', { relativePath });

/**
 * 写入缓存数据。
 * 输入：相对路径与 payload。
 * 输出：无；错误由 Tauri 调用抛出。
 */
export const writeCacheData = async (relativePath: string, payload: unknown): Promise<void> =>
  invoke('storage_write_cache', { relativePath, payload });

/**
 * 写入缓存文本并返回绝对路径。
 * 输入：相对路径与文本内容。
 * 输出：绝对路径字符串。
 */
export const writeCacheText = async (relativePath: string, contents: string): Promise<string> =>
  invoke<string>('storage_write_cache_text', { relativePath, contents });

/**
 * 读取工作区数据。
 * 输入：工作区路径与相对路径。
 * 输出：解析后的数据或 null。
 */
export const readWorkspaceData = async <T>(workspacePath: string, relativePath: string): Promise<T | null> =>
  invoke<T | null>('storage_read_workspace', { workspacePath, relativePath });

/**
 * 写入工作区数据。
 * 输入：工作区路径、相对路径与 payload。
 * 输出：无；错误由 Tauri 调用抛出。
 */
export const writeWorkspaceData = async (
  workspacePath: string,
  relativePath: string,
  payload: unknown
): Promise<void> => invoke('storage_write_workspace', { workspacePath, relativePath, payload });
