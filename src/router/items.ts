import type { Icon } from '@phosphor-icons/react';
import {
  BookOpenIcon,
  ClockCounterClockwiseIcon,
  FlagIcon,
  GearIcon,
  ImageIcon,
  ListChecksIcon,
  LockKeyIcon,
  NotebookIcon,
  SunIcon,
  VideoCameraIcon
} from '@phosphor-icons/react';
import { PATH } from './path';

/** 一条路由出现在哪些表面上 */
export type Surface = 'sidebar';

export type RouteTone = 'plan' | 'journal' | 'growth' | 'library' | 'toolbox';

export type RouteGroup = '记录' | '工具箱' | '系统';

/** 桌面侧栏里默认收进「更多」 */
export type RouteFold = 'more';

export type NavPath =
  | typeof PATH.today
  | typeof PATH.plan
  | typeof PATH.journal
  | typeof PATH.growth
  | typeof PATH.library
  | typeof PATH.cipher
  | typeof PATH.image
  | typeof PATH.video
  | typeof PATH.review
  | typeof PATH.settings;

export type RouteItem = {
  id: string;
  label: string;
  to: NavPath;
  group: RouteGroup;
  surfaces: Surface[];
  icon: Icon;
  tone?: RouteTone;
  fold?: RouteFold;
};

export const routeItems: RouteItem[] = [
  { id: 'today', label: '今天', to: PATH.today, group: '记录', surfaces: ['sidebar'], icon: SunIcon },
  {
    id: 'plan',
    label: '计划',
    to: PATH.plan,
    group: '记录',
    surfaces: ['sidebar'],
    icon: ListChecksIcon,
    tone: 'plan'
  },
  {
    id: 'journal',
    label: '手记',
    to: PATH.journal,
    group: '记录',
    surfaces: ['sidebar'],
    icon: NotebookIcon,
    tone: 'journal',
    fold: 'more'
  },
  {
    id: 'growth',
    label: '成长',
    to: PATH.growth,
    group: '记录',
    surfaces: ['sidebar'],
    icon: FlagIcon,
    tone: 'growth',
    fold: 'more'
  },
  {
    id: 'library',
    label: '书库',
    to: PATH.library,
    group: '记录',
    surfaces: ['sidebar'],
    icon: BookOpenIcon,
    tone: 'library'
  },
  {
    id: 'review',
    label: '回顾',
    to: PATH.review,
    group: '记录',
    surfaces: ['sidebar'],
    icon: ClockCounterClockwiseIcon,
    fold: 'more'
  },
  {
    id: 'cipher',
    label: '文件加密',
    to: PATH.cipher,
    group: '工具箱',
    surfaces: ['sidebar'],
    icon: LockKeyIcon,
    tone: 'toolbox'
  },
  {
    id: 'image',
    label: '图片转码',
    to: PATH.image,
    group: '工具箱',
    surfaces: ['sidebar'],
    icon: ImageIcon,
    tone: 'toolbox'
  },
  {
    id: 'video',
    label: '视频处理',
    to: PATH.video,
    group: '工具箱',
    surfaces: ['sidebar'],
    icon: VideoCameraIcon,
    tone: 'toolbox'
  },
  { id: 'settings', label: '设置', to: PATH.settings, group: '系统', surfaces: ['sidebar'], icon: GearIcon }
];

const mobileNavOrder = ['plan', 'journal', 'growth', 'library', 'settings'] as const;

/** 移动端底栏，按固定顺序。今天、文件加密、图片转码、视频处理、回顾只出现在桌面侧栏。 */
export function mobileNavItems() {
  return mobileNavOrder.flatMap((id) => {
    const item = routeItems.find((entry) => entry.id === id);
    return item ? [item] : [];
  });
}

export const sidebarGroups: RouteGroup[] = ['记录', '工具箱', '系统'];

export function itemsOn(surface: Surface) {
  return routeItems.filter((item) => item.surfaces.includes(surface));
}
