import { createHashHistory, createRootRoute, createRoute, createRouter, redirect } from '@tanstack/react-router';
import { GrowthPage } from '@/features/growth/page';
import { JournalEditorPage } from '@/features/journal/editor';
import { JournalPage } from '@/features/journal/page';
import { LibraryPage } from '@/features/library/page';
import { ReaderPage } from '@/features/library/reader';
import { PlanPage } from '@/features/plan/page';
import { ReviewPage } from '@/features/review/page';
import { YearPage } from '@/features/review/year';
import { SettingsPage } from '@/features/settings/page';
import { TodayPage } from '@/features/today/page';
import { ToolboxPage } from '@/features/toolbox/page';
import { Shell } from './shell/Shell';

const rootRoute = createRootRoute({
  component: Shell
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  beforeLoad: () => {
    throw redirect({ to: '/today' });
  }
});

const todayRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/today',
  component: TodayPage
});

const planRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/plan',
  component: PlanPage
});

const journalRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/journal',
  component: JournalPage
});

const journalEditorRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/journal/$entryId',
  component: JournalEditorPage
});

const growthRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/growth',
  component: GrowthPage
});

const libraryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/library',
  component: LibraryPage
});

const readerRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/library/$bookId',
  component: ReaderPage
});

const toolboxRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/toolbox',
  component: ToolboxPage
});

const reviewRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/review',
  component: ReviewPage
});

const yearRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/review/year',
  component: YearPage
});

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/settings',
  component: SettingsPage
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  todayRoute,
  planRoute,
  journalRoute,
  journalEditorRoute,
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
