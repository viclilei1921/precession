import { invokeCommand } from './invoke';

/**
 * 示例表一行
 */
export type DemoItem = {
  id: string;
  title: string;
  body: string;
  createdAt: number;
  updatedAt: number;
};

/**
 * 列出示例表全部行
 */
export function demoList() {
  return invokeCommand<DemoItem[]>('demo_list');
}

/**
 * 按 id 读取一行
 */
export function demoGet(id: string) {
  return invokeCommand<DemoItem>('demo_get', { id });
}

/**
 * 新建一行
 */
export function demoCreate(title: string, body: string) {
  return invokeCommand<DemoItem>('demo_create', { title, body });
}

/**
 * 更新一行
 */
export function demoUpdate(id: string, title: string, body: string) {
  return invokeCommand<DemoItem>('demo_update', { id, title, body });
}

/**
 * 删除一行
 */
export function demoDelete(id: string) {
  return invokeCommand<void>('demo_delete', { id });
}
