# 前端

界面是 Tauri 2 里的 React 应用。本地数据经 `src/bridge` 进出，服务端状态用 TanStack Query，跨组件 UI 状态用 Zustand，路由是代码注册的 hash history。

看界面用 `pnpm app`，Android 用 `pnpm app:android`。不要执行 `pnpm dev`：它只起 Vite，没有 Tauri 窗口。

## 目录

| 路径                              | 职责                                           |
| --------------------------------- | ---------------------------------------------- |
| `src/features/<领域>/page.tsx`    | 页面。承接路由，可以有交互和 `page.module.css` |
| `src/features/<领域>/components/` | 只属于该页面的组件                             |
| `src/components/`                 | 可复用的界面组件，不依赖某个业务页             |
| `src/layout/`                     | 壳：解锁、标题栏、侧栏、内容区                 |
| `src/query/<领域>/`               | `queryOptions` 和写完后的 `refresh*`           |
| `src/store/`                      | Zustand。现有 `useAppStore`、`useSearchStore`  |
| `src/router/`                     | 路由表、`PATH`、侧栏项                         |
| `src/bridge/`                     | 唯一允许导入 `@tauri-apps/*` 的目录            |
| `src/style/`                      | `token.css`、`reset.css`、`index.css`          |
| `src/hooks/`、`src/utils/`        | 跨页面的钩子和纯函数                           |

新组件目录用 kebab-case，例如 `date-field`、`plan-check`。`titleBar` 保持现状。

页面可以写样式和交互。分层是为了把壳、通用件和业务页分开，不是禁止页面拥有自己的逻辑。

## DOM

删掉无用包裹。判断标准：去掉这一层之后，功能、样式、语义都不变。

- 组件只返回一个元素时，不要在外面再包一层没有样式、事件、语义的 `div`。
- 返回多个节点时用 `<>...</>`，不要用 `div` 把它们凑成一个根。
- 类名合并到目标元素上。不要为了再加一个 class，或多一个变体，外面再套一层。
- `map` 出来的 `li` 直接放进 `ul` / `ol`。
- 能用语义标签就用语义标签：`button`、`ul` / `ol`、`header`、`main`、`section`、`article`、`nav`、`aside`。

这些嵌套留下：卡片内部的标题、正文、底部；独立的定位、滚动、flex / grid 子容器；外层统一监听事件；弹层自己的遮罩和面板。

不设「基础组件最多 3 层、业务组件最多 5 层」这种硬指标。

```tsx
function UserCard({ user, isLoading, className }) {
  return (
    <article className={clsx(styles.card, className)} data-loading={isLoading ? 'true' : undefined}>
      <h3 className={styles.name}>{user.name}</h3>
      <p className={styles.desc}>{user.desc}</p>
    </article>
  );
}
```

## class 与 data-*

静态外观、尺寸、渲染时就确定的变体用 class。会随状态来回切换的 UI 用 `data-*`。

CSS 文件里的类名用 kebab-case，TSX 里写 `styles.camelCase`。`vite.config.ts` 的 `localsConvention: 'camelCase'` 负责对应。已经写成 `.cardTitle`、`.groupLabel` 的类名保持不动，新代码用中划线。

布尔状态打开时写 `"true"`，关闭时省略属性（传 `undefined`）。选择器写 `[data-on="true"]`，不要为了关着的状态再写一份 `"false"`。

```tsx
<button type="button" className={styles.segment} data-on={view === 'todo' ? 'true' : undefined}>
  待办
</button>;
```

```css
.segment[data-on="true"] {
  font-weight: 700;
  background: var(--card);
}
```

`data-*` 也用来放封闭枚举，让样式去选设计 token，例如 `data-tone="plan"`。宿主属性保持原样：`data-platform`、`data-touch-ui`、`data-tauri-drag-region`。用户 id、价格这类业务数据不放进 `data-*`。

`disabled`、`checked`、`readonly` 用原生属性和 `:disabled` 等伪类。

`clsx` 只在合并多个 class 时使用，顺序是基础类、变体类、外部 `className`。单个类直接 `className={styles.card}`。开关状态不要塞进 `clsx`。

布尔 prop 已有 `wide`、`collapsed` 这类名字，不要求改成 `is` / `has`。新 prop 如果光看名字分不清是不是布尔，再用 `is` / `has`；对应的 `data-*` 去掉前缀，例如 `isLoading` 对应 `data-loading`。

## CSS

颜色、间距、圆角、字号、图标尺寸用 `src/style/token.css` 的变量。组件里不要重复写死一套相同的色值或间距。

CSS Module 已经做了作用域隔离。不要为了防重名去套一层无意义的后代选择器。后代选择器只留给真实的父子联动，例如收起侧栏时隐藏标签。不要把标签选择器当作主选择器，哈希不会加到标签上。

`:global` 用于两处：按平台改布局，以及必须覆盖的第三方样式。选择器要包在组件类名外面。

```css
:global(html[data-platform="mobile"]) .split {
  grid-template-columns: 1fr;
}
```

移动端和桌面的差异来自启动时的平台：脚本里用 `useAppStore((state) => state.isMobile())`，样式里用 `html[data-platform="mobile"]` 和 `html[data-touch-ui="true"]`。不要用媒体查询去猜测这是不是手机。

只有运行时算出来的数值才写内联 `style`，例如进度条宽度。固定外观写 class。

`button` 写 `type="button"`。图标用 Phosphor，尺寸用 class，默认 `weight="regular"`。

格式以 Biome 为准，见仓库根目录的 `biome.json`。

## 状态

| 状态                                   | 放哪里                    |
| -------------------------------------- | ------------------------- |
| 只在当前组件里用的 UI                  | `useState` / `useReducer` |
| 侧栏、主题、平台、数据库锁、标题栏搜索 | Zustand                   |
| 接口返回的列表和详情                   | TanStack Query            |

订阅 store 时用 selector，避免整棵 store 触发渲染：

```tsx
const collapsed = useAppStore((state) => state.sidebarCollapsed);
```

在效果里一次性读写、不需要订阅时，用 `useSearchStore.getState()`。按领域拆 store，不要合成一个全局大 store。同步动作用 store 自己的 action。读库、写库走 `src/bridge`，不把异步请求写进 store。

查询定义放在 `src/query/<领域>/query.ts`，用 `queryOptions`。`queryKey` 写成 `['模块', '接口', ...参数]`，例如 `['plan', 'list']`、`['journal', 'entry', id]`。写成功后调用同目录的 `refresh*`，按模块前缀失效，并在需要时连带刷新 `timeline`。

页面里直接 `useQuery(planListQuery)`。同一套读写被多个组件使用时，再抽钩子，例如 `useMembers`、`useCompletePlan`。加载和错误从 query 或 mutation 上取，页面用文案展示。本仓库没有 toast 库。

接口数据不要再抄一份进 Zustand。

## 路由

路由表在 `src/router/index.tsx`，`createHashHistory()`。Tauri 页面走本地文件，用 hash，不用文件路由。路径常量在 `src/router/path.ts`。页面在路由文件里同步引入，不做按页懒加载，也不做全局骨架屏。

需要刷新或从别处打开后还在的意图，放进 search params，并用 `validateSearch` 收成明确类型。现有字段是 `create`、`id`、`kind`。不要使用 `any`。

待办、收集箱、日历这种离开页面就可以丢掉的视图，留在组件的 `useState`。标题栏搜索用 `usePageSearch` 和 `useSearchStore`，不进 URL。

站内跳转用 `Link` 或 `useNavigate`。

## 组件

`src/components` 里的新组件把 `className` 接到根节点，调用方就不用为了补一个类再包一层 `div`。不回头给每个旧组件补齐 `style` 和任意 `data-*`。

props 如果要穿过很多层才能到底，把共享的那部分放进现有 store 或查询，而不是继续加参数。本仓库页面不深，这是书写时的提醒，不是层数配额。
