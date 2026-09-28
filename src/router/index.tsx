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
import { Layout } from './layout';
import { PATH } from './path';

const rootRoute = createRootRoute({
  component: Layout
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.home,
  beforeLoad: () => {
    throw redirect({ to: PATH.today });
  }
});

const todayRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.today,
  component: TodayPage
});

const planRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.plan,
  component: PlanPage
});

const journalRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.journal,
  component: JournalPage
});

const journalEditorRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.journalEditor,
  component: JournalEditorPage
});

const growthRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.growth,
  component: GrowthPage
});

const libraryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.library,
  component: LibraryPage
});

const readerRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: PATH.reader,
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
  path: PATH.year,
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
