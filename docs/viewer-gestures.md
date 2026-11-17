# 图片查看手势方案（待审批）

> 涉及模块：`src/components/stage/`（Stage 舞台 + wheel-paging 翻页条，被「系统看图」「文件查看」「md 媒体查看器」三处共用）、`src-tauri/src/platform/windows.rs`（WebView2 手势配置）。
> 状态：**方案文档，未改任何代码**。批准后按「实施顺序」执行。

---

## 一、问题 1：Windows 触摸板双指捏合缩放失效（Ctrl+滚轮正常）

### 现状链路

```
触摸板捏合
  → Precision Touchpad 驱动翻译为「ctrl+滚轮」手势
  → WebView2 (ICoreWebView2Settings5)
       ├─ IsPinchZoomEnabled = true   (windows.rs:128 已开)
       └─ IsZoomControlEnabled = true (windows.rs:126 已开)
  → 页面 wheel 事件 (event.ctrlKey = true)
  → stage/index.tsx onPinchWheel → zoomBy → 生效
```

链路本身是通的，Ctrl+滚轮有效即为证明（触摸板捏合在 Chromium 里就是 `ctrlKey=true` 的 wheel 事件，两者走同一条代码路径 `onPinchWheel`）。

### 根因分析（按可能性排序，需现场确认）

**根因 A（最可能）：`IsZoomControlEnabled(true)` 劫持了捏合手势。**

WebView2 的 `IsZoomControlEnabled` 文档含义是「启用缩放交互的 UI 控件」，但实测中它会让 WebView2 **直接接管缩放手势**：触摸板捏合映射为 **WebView2 浏览器级 zoom**（`webview.setZoom` 那条通路，表现为整页布局缩放，且经常配合 `zoomFactor` 变化但页面无任何响应——因为你们的页面是 `overflow: hidden` 的固定布局，视觉上就「毫无反应」）。而物理 Ctrl+滚轮走的仍是 wheel 事件，所以「Ctrl+滚轮有效、捏合失效」正好符合这个分裂现象。

`windows.rs` 里两个开关都打开了（126/128 行），本意是「允许捏合到达页面」，实际上 `IsZoomControlEnabled` 恰恰是让捏合**不到达页面**的那个开关。

**修复 A**：`windows.rs:126` 改为 `SetIsZoomControlEnabled(false)`，保留 `IsPinchZoomEnabled(true)`（该开关管的是**触摸屏**双指捏合的合成事件，桌面触摸板场景保留不影响）。

**根因 B（次可能）：捏合 wheel 事件被 React 合成事件层吞掉。**

`stage/index.tsx:293` 把 `onPinchWheel` 挂在 `window` 的 **capture 阶段**，而普通滚轮挂在 `scope`（父容器）的冒泡阶段。如果捏合事件 target 是标题栏/工具条等兄弟节点，事件能到达 window（capture 优先），没问题。但注意 `onPinchWheel` 开头 `if (!wheelZooms(event)) return;` —— 若 WebView2 版本较旧，捏合的 wheel 事件 `ctrlKey` 可能为 `false`（老版本 WebView2 不设置 ctrlKey），会直接 return，等于什么都没发生。

**验证方式**（无需改代码）：在看图页打开 WebView2 DevTools（右键 → 检查，或 F12），Console 里粘贴：

```js
window.addEventListener('wheel', e => console.log('wheel', e.ctrlKey, e.deltaY, e.momentum), { capture: true });
```

然后触摸板捏合，观察输出：
- 无任何输出 → 根因 A（手势被 WebView2 吃了，没到页面）
- 有输出但 `ctrlKey=false` → 根因 B（老内核）
- 有输出且 `ctrlKey=true` 但界面不动 → 前端别的问题，再深查

**修复 B**（若命中）：把 `wheelZooms()` 的判断放宽为「`ctrlKey` 为真 **或** 事件特征像捏合（`Math.abs(deltaY) < 40` 且 `deltaMode === 0` 且连续密集）」——不建议，误触风险高。更好的做法是升级 WebView2 Runtime（2026 年的 Evergreen 已是 Chromium 130+，`momentum` 属性需要 Edge 151+，说明你的 Runtime 很新，大概率不是 B）。

**根因 C（排查兜底）：`onMac()` 判断。** `stage/index.tsx:42-48` 用 UA 判断 mac 才认 metaKey，Windows 上认 ctrlKey，这条没问题，排除。

### 方案结论

1. **主修复**：`windows.rs:126` 的 `SetIsZoomControlEnabled(true)` → `false`。
2. **保留**：`SetIsPinchZoomEnabled(true)` 不动（触摸屏设备受益）。
3. **顺手加保险**：在 `onPinchWheel` 里补一行 `event.stopPropagation()`，确保捏合事件不会被外层（比如 md 编辑器滚动容器）二次消费。
4. 若改完实测捏合仍无效，按「验证方式」打日志回报输出，我再按根因 B/C 分支继续。

---

## 二、问题 2：翻页时机与惯性策略重构

### 需求还原（逐条对应代码）

| # | 需求 | 现状 | 目标 |
|---|------|------|------|
| 1 | 手离开触摸板才翻页 | 已实现（120ms 静默判定兜底 + momentum 结算，wheel-paging.ts:100-105、204-211） | 保持，作为惯性结算的信号源 |
| 2 | 放大后双指（触摸板两指滚动）平移图片，**带惯性** | 已实现（`onWheel` 的 pan 分支，stage/index.tsx:258-266），但 momentum 事件被**整体丢弃**（253-257 行），即**当前刻意无惯性** | momentum 事件继续平移（吃掉惯性尾巴） |
| 3 | 涉及翻页时不带惯性 | 翻页条位移本身就不吃 momentum（moveWheel 直接丢弃 momentum 事件） | 保持，并扩展到「图片顶边后残余推翻页条」的场景 |

即现状是「一刀切丢弃所有 momentum」，目标改为**分场景**：平移吃惯性、翻页不吃。

### 设计原则

```
手势进行中（手指在板上）：
  wheel(delta) → 1:1 立即位移（平移或翻页条，行为不变）

手势结束后（手指离开，momentum 事件开始流入）：
  momentum wheel(delta)
    ├─ 当前模式 = 翻页条（图未放大/缩回 fit）   → 丢弃 + 结算现有手势（现状，不变）
    ├─ 当前模式 = 图片平移中，且图片未到边界    → 继续平移，且衰减系数 < 1（惯性）
    └─ 图片已顶到边界                          → 丢弃 momentum，防惯性顶穿翻页
```

关键点：**惯性必须收敛到边界**，不能滑过头——图片顶边那一瞬间，剩余惯性要被 clampPan 收住，同时不允许残余位移溢出成翻页（保持现有 `pull.push` 只在非 momentum 事件里生效的逻辑，stage/index.tsx:268-273）。

### 惯性模型选择

在 `wheel-paging.ts` 旁新增一个平移惯性模型，不侵入现有 StripPager：

```
PanMomentum（新增模块 stage/pan-momentum.ts）：
  - feed(delta): momentum 事件到来时累加速度
  - tick(): requestAnimationFrame 循环，速度按 0.92/帧 衰减（60fps 基准，约 350ms 内衰减到停止）
  - 位移应用前过 clampPan：撞到边界时速度直接清零（硬停），不做橡皮筋回弹
    （做图片查看器时「撞墙即停」比「撞墙回弹」更符合直觉，Photoshop/ACDSee 均如此）
  - 任意新的真实 wheel / pointerdown 事件到来 → 立即掐灭惯性
```

### 需要改动的文件与要点

1. **`stage/index.tsx`**
   - `onWheel` 的 momentum 分支：从「无条件丢弃」改为「翻页模式丢弃 + 结算；平移模式喂给 PanMomentum」。
   - 新增 `onPointerDown` 时 `panMomentum.stop()`（手一放上去惯性就该停）。
   - `clampPan` 撞边时通知 PanMomentum 清速度（边界即惯性终点）。

2. **`stage/pan-momentum.ts`（新增）**
   - 纯逻辑模块，无 React 依赖，风格对齐 `wheel-paging.ts`（自包含、dispose、prefers-reduced-motion 时直接不启用惯性）。
   - 衰减参数与 `wheel-paging.ts` 的手感参数（`SMOOTH_MS`、`TAIL_MS`）保持同量级，方便统一调优。

3. **`stage/wheel-paging.ts`**
   - 基本不动。唯一小改：`push()` 目前接受任何来源的残余位移推翻页条，需加约束——**momentum 推动平移产生的残余（比如惯性把图顶到边界后理论上不会再有余量，因为速度已清零）不再走 push**。实际上由于边界清速度，这条天然满足，代码里加一行注释说明即可，不改行为。

4. **`windows.rs`**
   - 见问题 1：`SetIsZoomControlEnabled(false)` + 保留 `SetIsPinchZoomEnabled(true)`。

### 不改动 / 明确排除的

- 触摸屏双指捏合（`gesturestart/gesturechange`，stage/index.tsx:276-289）：现有实现已满足需求，不动。
- 手机端翻页（media-viewer.tsx 的左右滑动翻页）：与本需求无关，不动。
- `momentum` 属性缺失的旧内核回退路径（120ms 静默判定）：保留，但旧内核下「平移惯性」退化为无惯性（静默判定只够翻页结算用，不够重建速度）。文档中标注为已知限制。

### 手感参数（初值，实施后可在真机上调）

| 参数 | 值 | 说明 |
|------|-----|------|
| 衰减系数 | 0.92/帧 | 60fps 下 ~350ms 收敛 |
| 最大初速 | 2400 px/s | 防一次猛滑飞出几屏 |
| 撞边行为 | 速度清零 | 硬停，不回弹 |
| 翻页触发 | 不变 | 保持现有 50% 页高 + 500ms 动画 |
| reduced-motion | 惯性禁用 | 平移立即停 |

---

## 三、实施顺序（批准后执行）

1. `windows.rs` 捏合开关修复（问题 1 主修复，1 行改动）。
2. 新增 `stage/pan-momentum.ts` 惯性模块。
3. `stage/index.tsx` 接入：momentum 分流 + pointerdown 掐灭 + 边界清速度。
4. `pnpm typecheck && pnpm lint` 通过后交付。
5. 真机验证清单：
   - 捏合缩放生效，锚点跟随指针。
   - 捏合中松手 → 缩放立刻停（缩放不吃惯性，避免 overshoot 回弹怪异）。
   - 放大后两指滚动 → 平移跟手 + 松手后滑行约 0.3s 收敛，撞边硬停。
   - 平移撞到边界继续滚 → 翻页条被推出，但松手前不翻页，松手/静默后才结算。
   - fit 模式两指滚动 → 翻页条跟手，松手结算，惯性尾巴不翻页。
   - 触摸屏捏合、手机端滑动翻页回归正常。

## 四、风险

- WebView2 Evergreen 版本差异：`momentum` 属性需 Edge/WebView2 151+（约 2026 年中），老机器上平移惯性退化为无惯性——可接受。
- `IsZoomControlEnabled(false)` 的副作用：浏览器级 zoom UI（缩放控件浮条）彻底关闭。你们页面是自绘 UI + 固定布局，本来就不依赖浏览器 zoom，无实际影响。
