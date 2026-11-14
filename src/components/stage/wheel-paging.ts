/**
 * 滚轮翻页的手势判断：一次滚动（间隔不超过 IDLE_MS 的连续 wheel 事件）
 * 最多翻一页，滚动惯性连发不会连翻好几张。
 *
 * - 事件间隔超过 IDLE_MS 视为新的一次滚动，重新累计
 * - 累计滚动量达到 STEP_DELTA 才翻，轻微的触摸板碰触不会误翻
 * - 翻过一页后本次手势内不再翻，直到滚动停下
 */
const IDLE_MS = 300;
const STEP_DELTA = 50;

export function createWheelPager(step: (direction: 1 | -1) => void) {
  const state = { lastAt: 0, accum: 0, done: false };
  return (event: WheelEvent) => {
    if (event.ctrlKey || event.metaKey || event.shiftKey) {
      return;
    }
    // 横向滚动不翻页，只认明显纵向的滚动
    if (Math.abs(event.deltaY) <= Math.abs(event.deltaX)) {
      return;
    }
    const now = performance.now();
    if (now - state.lastAt > IDLE_MS) {
      state.accum = 0;
      state.done = false;
    }
    state.lastAt = now;
    // 本次滚动已经翻过页，惯性里剩下的事件全部忽略
    if (state.done) {
      return;
    }
    state.accum += event.deltaY;
    if (Math.abs(state.accum) < STEP_DELTA) {
      return;
    }
    state.done = true;
    step(event.deltaY > 0 ? 1 : -1);
  };
}
