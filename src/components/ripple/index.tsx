import type { KeyboardEvent, PointerEvent, ReactNode } from 'react';
import { useState } from 'react';
import styles from './index.module.css';

/** 一次按下产生的波纹。坐标和尺寸相对宿主左上角。 */
type Ripple = {
  id: number;
  x: number;
  y: number;
  size: number;
};

/** 跨实例递增，避免同一宿主里连续两次点击共用 key。 */
let rippleId = 0;

/**
 * 以触点为圆心，半径取到宿主最远一角，保证波纹能盖住整个宿主。
 * 返回的是圆心左上角的定位，直径为半径的两倍。
 * @param rect - 宿主的矩形区域
 * @param clientX - 触点的 x 坐标
 * @param clientY - 触点的 y 坐标
 * @returns 波纹的坐标和尺寸
 */
function rippleAt(rect: DOMRect, clientX: number, clientY: number): Ripple {
  const x = clientX - rect.left;
  const y = clientY - rect.top;
  const radius = Math.hypot(Math.max(x, rect.width - x), Math.max(y, rect.height - y));
  return { id: rippleId++, x: x - radius, y: y - radius, size: radius * 2 };
}

/**
 * 按下时在宿主里展开一圈水波纹。
 *
 * `rippleHost` 加到接收按下的元素上，`nodes` 作为它的子节点。
 * 指针按下走 `onPointerDown`，从触点展开；键盘激活走 `onKeyDown`，从中心展开。
 * 系统开启「减少动态效果」时不生成波纹。
 */
export function useRipple() {
  const [ripples, setRipples] = useState<Ripple[]>([]);

  /** 添加波纹 */
  function addRipple(rect: DOMRect, clientX: number, clientY: number) {
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      // 系统开启「减少动态效果」时不生成波纹
      return;
    }

    // 添加波纹
    setRipples((list) => [...list, rippleAt(rect, clientX, clientY)]);
  }

  /** 主键按下。右键和中键不产生波纹。 */
  function onPointerDown(event: PointerEvent<HTMLElement>) {
    if (event.button !== 0) {
      return;
    }

    const rect = event.currentTarget.getBoundingClientRect();
    addRipple(rect, event.clientX, event.clientY);
  }

  /**
   * 键盘激活控件时调用，波纹从宿主中心展开。
   *
   * Enter 和空格是按钮的激活键。链接上的空格用来滚动页面，不视为激活。
   * 按住不放产生的重复事件忽略，避免一串波纹叠在一起。
   */
  function onKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.currentTarget instanceof HTMLAnchorElement && event.key === ' ') {
      return;
    }
    if (event.repeat || (event.key !== 'Enter' && event.key !== ' ')) {
      return;
    }
    const rect = event.currentTarget.getBoundingClientRect();
    addRipple(rect, rect.left + rect.width / 2, rect.top + rect.height / 2);
  }

  /** 动画结束即卸下，避免已播完的节点留在列表里。 */
  const nodes: ReactNode = ripples.map((ripple) => (
    <span
      key={ripple.id}
      className={styles.ripple}
      style={{ left: ripple.x, top: ripple.y, width: ripple.size, height: ripple.size }}
      onAnimationEnd={() => setRipples((list) => list.filter((item) => item.id !== ripple.id))}
    />
  ));

  return { onPointerDown, onKeyDown, nodes };
}

/** 波纹宿主。加在接收按下的元素上，波纹绝对定位并被裁在它内部。 */
export const rippleHost = styles.host;
