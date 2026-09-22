import { invokeCommand } from './invoke';

/**
 * 数据库状态
 */
export type DbStatus = {
  /** 数据库是否存在 */
  exists: boolean;
  /** 数据库是否已解锁 */
  unlocked: boolean;
  /** 是否已启用设备槽。为真时打开应用可以走系统验证。 */
  deviceUnlock: boolean;
  /** 用户主动锁定。重启后仍然为真，解锁需要系统验证。 */
  userLocked: boolean;
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

/**
 * 用档案密码启用这台设备上的系统验证解锁
 * @param password - 档案密码
 */
export function dbEnableDeviceUnlock(password: string) {
  return invokeCommand<void>('db_enable_device_unlock', { password });
}

/**
 * 用设备槽解锁
 */
export function dbUnlockDevice() {
  return invokeCommand<void>('db_unlock_device');
}

/**
 * 关闭这台设备上的免密解锁
 */
export function dbDisableDeviceUnlock() {
  return invokeCommand<void>('db_disable_device_unlock');
}
