const MOBILE_WIDTH = 720;

type TauriInternals = {
  plugins?: Record<string, unknown>;
};

function isAndroid() {
  const internals = (window as Window & { __TAURI_INTERNALS__?: TauriInternals }).__TAURI_INTERNALS__;
  return Boolean(internals?.plugins && 'device-slot' in internals.plugins);
}

/** 写入 data-platform 和 data-touch-ui，供布局和全局样式读取。 */
export function applyPlatform() {
  const mobile = isAndroid() || window.innerWidth < MOBILE_WIDTH;
  document.documentElement.dataset.platform = mobile ? 'mobile' : 'desktop';
  document.documentElement.dataset.touchUi = mobile ? 'true' : 'false';
}

export function watchPlatform() {
  applyPlatform();
  window.addEventListener('resize', applyPlatform);
}

export function isMobilePlatform() {
  return document.documentElement.dataset.platform === 'mobile';
}
