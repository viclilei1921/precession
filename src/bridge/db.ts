import { invokeCommand } from './invoke';

/**
 * 数据库状态
 */
export type DbStatus = {
  /** 数据库是否存在 */
  exists: boolean;
  /** 数据库是否已解锁 */
  unlocked: boolean;
};

/**
 * 获取数据库状态
 */
export function dbStatus() {
  return invokeCommand<DbStatus>('db_status');
}

/**
 * 创建数据库
 * @param password - 密码
 * @param passwordConfirm - 确认密码
 */
export function dbCreate(password: string, passwordConfirm: string) {
  return invokeCommand<void>('db_create', { password, passwordConfirm });
}

/**
 * 解锁数据库
 * @param password - 密码
 */
export function dbUnlock(password: string) {
  return invokeCommand<void>('db_unlock', { password });
}

/**
 * 锁定数据库
 */
export function dbLock() {
  return invokeCommand<void>('db_lock');
}
