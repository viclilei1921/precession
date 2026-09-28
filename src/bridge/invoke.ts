import { invoke } from '@tauri-apps/api/core';

/**
 * 调用 Rust 命令。后端把错误序列化成字符串，这里统一转成 Error。
 */
export function invokeCommand<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args).catch((error: unknown) => {
    if (error instanceof Error) {
      return Promise.reject(error);
    }
    if (typeof error === 'string' && error.length > 0) {
      return Promise.reject(new Error(error));
    }
    return Promise.reject(new Error('操作失败'));
  });
}
