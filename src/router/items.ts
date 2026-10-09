import type { Icon } from '@phosphor-icons/react';
import {
  BookOpenIcon,
  ClockCounterClockwiseIcon,
  FlagIcon,
  GearIcon,
  ListChecksIcon,
  NotebookIcon,
  SunIcon,
  ToolboxIcon
} from '@phosphor-icons/react';
import { PATH } from './path';

/** 一条路由出现在哪些表面上 */
export type Surface = 'sidebar' | 'tab' | 'record' | 'mine';

export type RouteTone = 'plan' | 'journal' | 'growth' | 'library' | 'toolbox';

export type RouteGroup = '记录' | '整理' | '系统';

export type NavPath =
  | typeof PATH.today
  | typeof PATH.plan
  | typeof PATH.journal
  | typeof PATH.growth
  | typeof PATH.library
  | typeof PATH.toolbox
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
};

export const routeItems: RouteItem[] = [
  { id: 'today', label: '今天', to: PATH.today, group: '记录', surfaces: ['sidebar', 'tab'], icon: SunIcon },
  {
    id: 'plan',
    label: '计划',
    to: PATH.plan,
    group: '记录',
    surfaces: ['sidebar', 'record'],
    icon: ListChecksIcon,
    tone: 'plan'
  },
  {
    id: 'journal',
    label: '手记',
    to: PATH.journal,
    group: '记录',
    surfaces: ['sidebar', 'record'],
    icon: NotebookIcon,
    tone: 'journal'
  },
  {
    id: 'growth',
    label: '成长',
    to: PATH.growth,
    group: '记录',
    surfaces: ['sidebar', 'record'],
    icon: FlagIcon,
    tone: 'growth'
  },
  {
    id: 'library',
    label: '书库',
    to: PATH.library,
    group: '记录',
    surfaces: ['sidebar', 'tab'],
    icon: BookOpenIcon,
    tone: 'library'
  },
  {
    id: 'toolbox',
    label: '工具箱',
    to: PATH.toolbox,
    group: '整理',
    surfaces: ['sidebar', 'mine'],
    icon: ToolboxIcon,
    tone: 'toolbox'
  },
  {
    id: 'review',
    label: '回顾',
    to: PATH.review,
    group: '整理',
    surfaces: ['sidebar', 'mine'],
    icon: ClockCounterClockwiseIcon
  },
  { id: 'settings', label: '设置', to: PATH.settings, group: '系统', surfaces: ['sidebar', 'mine'], icon: GearIcon }
];

export const sidebarGroups: RouteGroup[] = ['记录', '整理', '系统'];

export function itemsOn(surface: Surface) {
  return routeItems.filter((item) => item.surfaces.includes(surface));
}
