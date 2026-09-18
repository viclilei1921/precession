# Precession

基于 Tauri2 + Vite + React的跨平台应用。
定位：记录生活。通过计划、手记、成长、书库来进行记录。

## 参考文档

- [Tauri 2 中文](https://v2.tauri.app/zh-cn/start/create-project/)
- [React](https://react.dev/)

## 环境与运行

### node

node环境以及版本管理，使用[fnm](https://www.lilei1921.cn/notes/node-version-tool-fnm)。

包管理使用 [pnpm](https://pnpm.io/)（可用 `corepack enable` 启用，或 `npm i -g pnpm` 安装）

```bash
# 安装依赖
pnpm i
# 检测包更新和更新包
# pnpm-workspace.yaml 中配置 minimumReleaseAge来确定最新为多少时间前的包。0值为最新
pnpm up:c
pnpm up:i
# 运行项目(桌面开发模式)
pnpm app
#打包win和mac
pnpm build:win
pnpm build:mac
```
更新图标
```bash
# icon更换
pnpm run tauri icon ./app-icon.png
```

格式化工具[biome](https://biomejs.dev/zh-cn/)，配置文件：[biome.json](./biome.json)

```bash
pnpm add --save-dev --save-exact @biomejs/biome
# 配置文件地址
# biome.json
# 使用vscode编辑器，注意配置setting.json
# 检测格式以及修复
pnpm lint
pnpm lint:fix
```

rust格式化，使用cargo fmt。

```bash
pnpm fmt
pnpm fmt:fix
```

格式化并修复前后端

```bash
pnpm format
```

### rust

```shell
# windows安装
winget install --id Rustlang.Rustup

# 安装 cargo-edit 工具（仅首次需要）
cargo install cargo-edit
# 显示所有可升级依赖及其版本对比
cargo upgrade --dry-run
# 升级所有依赖并直接重写 Cargo.toml
cargo upgrade
# 升级到最新的兼容版本（不跨主版本，类似 cargo update，但会修改 Cargo.toml）
cargo upgrade --compatible
# 强制升级到最新的主版本（默认行为）
cargo upgrade --breaking
```

### windows

Tauri 使用 Microsoft C++ 生成工具进行开发以及 Microsoft Edge WebView2。这两者都是在 Windows 上进行开发所必需的。

[下载Microsoft C++](https://visualstudio.microsoft.com/zh-hans/visual-cpp-build-tools/)，安装过程中，选中“使用 C++ 的桌面开发”选项，细节查看[Tauri2文档](https://v2.tauri.app/zh-cn/start/prerequisites/#microsoft-c-%E7%94%9F%E6%88%90%E5%B7%A5%E5%85%B7)。
