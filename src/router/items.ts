import type { Icon } from '@phosphor-icons/react';
import {
  BookOpenIcon,
  FileTextIcon,
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

/** 侧栏分组 key，对应 nav.json 里的 group.* 字段 */
export type RouteGroup = 'record' | 'toolbox' | 'system';

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
  | typeof PATH.files
  | typeof PATH.settings;

export type RouteItem = {
  /** 路由 id，同时作为 nav.json 里的翻译 key */
  id: string;
  to: NavPath;
  group: RouteGroup;
  surfaces: Surface[];
  icon: Icon;
  tone?: RouteTone;
  fold?: RouteFold;
};

export const routeItems: RouteItem[] = [
  { id: 'today', to: PATH.today, group: 'record', surfaces: ['sidebar'], icon: SunIcon },
  {
    id: 'plan',
    to: PATH.plan,
    group: 'record',
    surfaces: ['sidebar'],
    icon: ListChecksIcon,
    tone: 'plan'
  },
  {
    id: 'journal',
    to: PATH.journal,
    group: 'record',
    surfaces: ['sidebar'],
    icon: NotebookIcon,
    tone: 'journal',
    fold: 'more'
  },
  {
    id: 'growth',
    to: PATH.growth,
    group: 'record',
    surfaces: ['sidebar'],
    icon: FlagIcon,
    tone: 'growth',
    fold: 'more'
  },
  {
    id: 'library',
    to: PATH.library,
    group: 'record',
    surfaces: ['sidebar'],
    icon: BookOpenIcon,
    tone: 'library'
  },
  {
    id: 'cipher',
    to: PATH.cipher,
    group: 'toolbox',
    surfaces: ['sidebar'],
    icon: LockKeyIcon,
    tone: 'toolbox'
  },
  {
    id: 'image',
    to: PATH.image,
    group: 'toolbox',
    surfaces: ['sidebar'],
    icon: ImageIcon,
    tone: 'toolbox'
  },
  {
    id: 'video',
    to: PATH.video,
    group: 'toolbox',
    surfaces: ['sidebar'],
    icon: VideoCameraIcon,
    tone: 'toolbox'
  },
  { id: 'files', to: PATH.files, group: 'system', surfaces: ['sidebar'], icon: FileTextIcon },
  { id: 'settings', to: PATH.settings, group: 'system', surfaces: ['sidebar'], icon: GearIcon }
];

const mobileNavOrder = ['plan', 'journal', 'growth', 'library', 'settings'] as const;

/** 移动端底栏，按固定顺序。今天、文件加密、图片转码、视频处理只出现在桌面侧栏。 */
export function mobileNavItems() {
  return mobileNavOrder.flatMap((id) => {
    const item = routeItems.find((entry) => entry.id === id);
    return item ? [item] : [];
  });
}

export const sidebarGroups: RouteGroup[] = ['record', 'toolbox', 'system'];

export function itemsOn(surface: Surface) {
  return routeItems.filter((item) => item.surfaces.includes(surface));
}
