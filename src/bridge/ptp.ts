import type { UnlistenFn } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';

/** Rust 侧 Raw Input 触摸板手指接触事件 */
export const PTP_FINGERS_EVENT = 'ptp:fingers';

export type PtpFingersHandler = (fingers: number) => void;

/**
 * 监听触摸板手指接触数量（Windows PTP，Rust 侧 Raw Input）。
 *
 * 确定性信号：>0 表示手指还在板上（翻页手势不应静默结算），
 * 0 表示手指已离开（立即结算）。非 PTP 触摸板 / 非 Windows 永远收不到
 * 事件，调用方据此自动回退静默计时兜底。调用方在卸载时执行返回的函数。
 */
export function onPtpFingers(handler: PtpFingersHandler): Promise<UnlistenFn> {
  return listen<number>(PTP_FINGERS_EVENT, (event) => {
    if (typeof event.payload === 'number') {
      handler(event.payload);
    }
  });
}
