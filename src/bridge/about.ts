import { invokeCommand } from './invoke';

/** 设置页版本 */
export type AppVersions = {
  /** 应用版本 */
  app: string;
  /** WebView 版本 */
  webview: string;
  /** FFmpeg 版本 */
  ffmpeg: string;
  /** cjxl 版本 */
  cjxl: string;
  /** avifenc 版本 */
  avifenc: string;
};

/** 读取应用、WebView 和侧车版本 */
export function appVersions() {
  return invokeCommand<AppVersions>('app_versions');
}
