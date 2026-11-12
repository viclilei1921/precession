type Listener = (path: string) => void;

let current: string | null = null;
const listeners = new Set<Listener>();

/** 记下一张要打开的图片，并通知已经在看的页面 */
export function rememberViewerPath(path: string) {
  if (!path) {
    return;
  }
  current = path;
  for (const listener of listeners) {
    listener(path);
  }
}

/** 改名后换掉记住的路径，不重新打开文件夹 */
export function retargetViewerPath(from: string, to: string) {
  if (current === from) {
    current = to;
  }
}

/** 当前要打开的路径。还没有时为空 */
export function peekViewerPath() {
  return current;
}

/** 退出查看。不会把刚才那张再打开一次 */
export function clearViewerPath() {
  current = null;
}

/** 路径变化时通知。返回取消订阅的函数 */
export function subscribeViewerPath(listener: Listener) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}
