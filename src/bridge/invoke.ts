import { invoke } from '@tauri-apps/api/core';

/**
 * 调用 Rust 命令。后端把错误序列化成字符串，这里统一转成 Error。
 */
export async function invokeCommand<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error: unknown) {
    if (error instanceof Error) {
      throw error;
    }
    if (typeof error === 'string' && error.length > 0) {
      throw new Error(error);
    }
    throw new Error('操作失败');
  }
}
