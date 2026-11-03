import { useRouter } from '@tanstack/react-router';
import { useEffect, useLayoutEffect, useRef } from 'react';
import { windowClose } from '@/bridge/window';
import { useAppStore } from '@/store/app';

type BackAction = 'exit' | 'handled' | 'history';

type Layer = {
  close: () => void;
};

type LocationSnap = {
  pathname: string;
  search: Record<string, unknown>;
};

type BackBinding = {
  location: () => LocationSnap;
  back: () => boolean;
  toLibrary: () => void;
};

const layers: Layer[] = [];
let bound: BackBinding | null = null;

declare global {
  interface Window {
    __precessionBack?: () => BackAction;
  }
}

/** 移动端当前这一层。系统返回和界面返回都先收这一层。 */
export function useBackLayer(active: boolean, onClose: () => void) {
  const closeRef = useRef(onClose);
  closeRef.current = onClose;

  useEffect(() => {
    if (!active || !useAppStore.getState().isMobile()) {
      return;
    }
    const layer: Layer = { close: () => closeRef.current() };
    layers.push(layer);
    return () => {
      const index = layers.lastIndexOf(layer);
      if (index >= 0) {
        layers.splice(index, 1);
      }
    };
  }, [active]);
}

/** 退一条历史。没有可退的历史时走 fallback。 */
export function popRoute(fallback: () => void) {
  if (bound?.back()) {
    return;
  }
  fallback();
}

/** 和系统返回同一套判断。界面上的返回在无路可退时关闭窗口。 */
export function performMobileBack() {
  const action = decide();
  if (action === 'exit' || (action === 'history' && !bound?.back())) {
    requestExit();
  }
}

function requestExit() {
  void windowClose().catch(() => undefined);
}

function decide(): BackAction {
  const layer = layers[layers.length - 1];
  if (layer) {
    layer.close();
    return 'handled';
  }
  const location = bound?.location();
  if (!location || isReading(location.pathname) || isModuleRoot(location)) {
    return 'exit';
  }
  if (isBookDetail(location.pathname)) {
    bound?.toLibrary();
    return 'handled';
  }
  return 'history';
}

function isReading(pathname: string) {
  return /^\/library\/[^/]+\/read$/.test(pathname);
}

function isBookDetail(pathname: string) {
  return /^\/library\/[^/]+$/.test(pathname);
}

function isModuleRoot(location: LocationSnap) {
  const { pathname, search } = location;
  if (pathname === '/plan') {
    return search.create !== true && typeof search.id !== 'string';
  }
  if (pathname === '/journal') {
    return search.create !== true;
  }
  if (pathname === '/growth') {
    return search.create !== true && typeof search.sheet !== 'string' && typeof search.edit !== 'string';
  }
  if (pathname === '/library') {
    return true;
  }
  if (pathname === '/settings') {
    return typeof search.sheet !== 'string';
  }
  return false;
}

/** 挂在路由里，把当前地址交给返回判断。 */
export function MobileBackBridge() {
  const router = useRouter();

  useLayoutEffect(() => {
    bound = {
      location: () => ({
        pathname: router.state.location.pathname,
        search: router.state.location.search as Record<string, unknown>
      }),
      back: () => {
        if (!router.history.canGoBack()) {
          return false;
        }
        router.history.back();
        return true;
      },
      toLibrary: () => {
        void router.navigate({ to: '/library', replace: true });
      }
    };
    return () => {
      bound = null;
    };
  }, [router]);

  return null;
}

// 有上一层就退路由。模块首页返回 exit，由 Android 把任务退到后台。
window.__precessionBack = () => {
  const action = decide();
  if (action === 'history') {
    if (bound?.back()) {
      return 'handled';
    }
    return 'exit';
  }
  return action;
};
