import { createHashHistory, createRootRoute, createRoute, createRouter, redirect } from '@tanstack/react-router';
import { GrowthPage } from '@/features/growth/page';
import { JournalEntryPage } from '@/features/journal/entry-page';
import { JournalPage } from '@/features/journal/page';
import { LibraryPage } from '@/features/library/page';
import { ReaderPage } from '@/features/library/reader-page';
import { PlanPage } from '@/features/plan/page';
import { ReviewPage } from '@/features/review/page';
import { YearPage } from '@/features/review/year-page';
import { SettingsPage } from '@/features/settings/page';
import { TodayPage } from '@/features/today/page';
import { ToolboxPage } from '@/features/toolbox/page';
import { Layout } from '@/layout';
import { DeviceUnlock } from '@/session/device-unlock';
import { PATH } from './path';

const rootRoute = createRootRoute({
  component: Layout
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.root,
  beforeLoad: () => {
    throw redirect({ to: PATH.today });
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

const readerRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.bookReader,
  component: ReaderPage
});

const toolboxRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.toolbox,
  component: ToolboxPage
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

function SettingsRoute() {
  return <SettingsPage deviceUnlock={<DeviceUnlock />} />;
}

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.settings,
  component: SettingsRoute
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
  toolboxRoute,
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
