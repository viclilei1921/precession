import { createHashHistory, createRootRoute, createRoute, createRouter, redirect } from '@tanstack/react-router';
import { CipherPage } from '@/features/cipher/page';
import { GrowthPage } from '@/features/growth/page';
import { ImagePage } from '@/features/image/page';
import { JournalEntryPage } from '@/features/journal/entry-page';
import { JournalPage } from '@/features/journal/page';
import { LibraryPage } from '@/features/library/page';
import { ReadPage } from '@/features/library/read-page';
import { ReaderPage } from '@/features/library/reader-page';
import { PlanPage } from '@/features/plan/page';
import { ReviewPage } from '@/features/review/page';
import { YearPage } from '@/features/review/year-page';
import { SettingsPage } from '@/features/settings/page';
import { TodayPage } from '@/features/today/page';
import { VideoPage } from '@/features/video/page';
import { ContentLayout } from '@/layout/content';
import { useAppStore } from '@/store/app';
import { PATH } from './path';

const rootRoute = createRootRoute({
  component: ContentLayout
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.root,
  beforeLoad: () => {
    throw redirect({ to: useAppStore.getState().isMobile() ? PATH.plan : PATH.today });
  }
});

const todayRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.today,
  component: TodayPage
});

type PlanSearch = {
  create?: boolean;
  id?: string;
};

function validatePlanSearch(search: Record<string, unknown>): PlanSearch {
  const next: PlanSearch = {};
  if (search.create === true || search.create === '1' || search.create === 'true') {
    next.create = true;
  }
  if (typeof search.id === 'string' && search.id.length > 0) {
    next.id = search.id;
  }
  return next;
}

const planRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.plan,
  validateSearch: validatePlanSearch,
  component: PlanPage
});

type JournalSearch = {
  create?: boolean;
  kind?: 'diary' | 'spark' | 'writing';
};

function validateJournalSearch(search: Record<string, unknown>): JournalSearch {
  const next: JournalSearch = {};
  if (search.create === true || search.create === '1' || search.create === 'true') {
    next.create = true;
  }
  if (search.kind === 'diary' || search.kind === 'spark' || search.kind === 'writing') {
    next.kind = search.kind;
  }
  return next;
}

const journalRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.journal,
  validateSearch: validateJournalSearch,
  component: JournalPage
});

const journalEntryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.journalEntry,
  component: JournalEntryPage
});

type GrowthSearch = {
  create?: boolean;
  kind?: 'milestone' | 'moment';
};

function validateGrowthSearch(search: Record<string, unknown>): GrowthSearch {
  const next: GrowthSearch = {};
  if (search.create === true || search.create === '1' || search.create === 'true') {
    next.create = true;
  }
  if (search.kind === 'milestone' || search.kind === 'moment') {
    next.kind = search.kind;
  }
  return next;
}

const growthRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.growth,
  validateSearch: validateGrowthSearch,
  component: GrowthPage
});

type LibrarySearch = {
  create?: boolean;
};

function validateLibrarySearch(search: Record<string, unknown>): LibrarySearch {
  if (search.create === true || search.create === '1' || search.create === 'true') {
    return { create: true };
  }
  return {};
}

const libraryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.library,
  validateSearch: validateLibrarySearch,
  component: LibraryPage
});

type BookSearch = {
  task?: string;
};

function validateBookSearch(search: Record<string, unknown>): BookSearch {
  if (typeof search.task === 'string' && search.task.length > 0) {
    return { task: search.task };
  }
  return {};
}

const readerRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.bookReader,
  validateSearch: validateBookSearch,
  component: ReaderPage
});

const readRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.bookRead,
  component: ReadPage
});

const videoRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.video,
  component: VideoPage
});

const imageRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.image,
  component: ImagePage
});

const cipherRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.cipher,
  component: CipherPage
});

const toolboxRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.toolbox,
  beforeLoad: () => {
    throw redirect({ to: PATH.video });
  }
});

const toolboxFileRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.toolboxFile,
  beforeLoad: () => {
    throw redirect({ to: PATH.cipher });
  }
});

const toolboxImageRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.toolboxImage,
  beforeLoad: () => {
    throw redirect({ to: PATH.image });
  }
});

const toolboxVideoRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.toolboxVideo,
  beforeLoad: () => {
    throw redirect({ to: PATH.video });
  }
});

const reviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.review,
  component: ReviewPage
});

const yearRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.yearBook,
  component: YearPage
});

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.settings,
  component: SettingsPage
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  todayRoute,
  planRoute,
  journalRoute,
  journalEntryRoute,
  growthRoute,
  libraryRoute,
  readerRoute,
  readRoute,
  videoRoute,
  imageRoute,
  cipherRoute,
  toolboxRoute,
  toolboxFileRoute,
  toolboxImageRoute,
  toolboxVideoRoute,
  reviewRoute,
  yearRoute,
  settingsRoute
]);

export const router = createRouter({
  routeTree,
  history: createHashHistory()
});

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
