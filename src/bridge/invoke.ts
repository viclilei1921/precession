import { invoke } from '@tauri-apps/api/core';

/**
 * 调用 Rust 命令
 * @param cmd - 命令名称
 * @param args - 参数
 * @returns - 返回值
 */
export function invokeCommand<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args).then(
    (value) => value,
    (error) => Promise.reject(error)
  );
}
