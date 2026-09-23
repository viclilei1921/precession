# Precession

基于 **Tauri 2** + **Vite** + **React** 的跨平台应用。

定位：记录生活——通过计划、手记、成长、书库进行记录。

## 环境

通用前置条件见 [Tauri 2 前置要求](https://v2.tauri.app/zh-cn/start/prerequisites/)。下文只列本仓库额外约定。

本项目因使用 SQLCipher（见「数据库 SQLCipher」），编译期还需 **C 工具链 + Perl**；Android 交叉编译另有独立文档。

### Node 版本管理（fnm）

官方前置要求已包含 Node。推荐使用版本管理工具 [fnm](https://github.com/schniz/fnm)，安装说明见 [fnm 安装文档](https://www.lilei1921.cn/notes/node-version-tool-fnm)。

### pnpm

包管理使用 [pnpm](https://pnpm.io/)。推荐通过 Corepack 启用：

```bash
corepack enable
corepack prepare pnpm@latest --activate
```

## 常用命令

```bash
# 开发
pnpm app
pnpm app:android

# 打包
pnpm build:win
pnpm build:mac
pnpm build:android

# 前后端格式化并修复
pnpm format

# 根据 app-icon.png 生成各平台图标
pnpm tauri icon ./app-icon.png
```



## 前端



### 格式化（Biome）

未使用 ESLint。配置与用法见 [Biome 文档](https://biomejs.dev/zh-cn/)，仓库配置文件为 [biome.json](./biome.json)。

```bash
# 使用 VS Code / Cursor 时，请配置工作区 settings.json（本仓库已有 .vscode/settings.json）
pnpm lint       # 检查
pnpm lint:fix  # 检查并写入修复
```



### 依赖

```bash
pnpm i          # 安装依赖
pnpm up:c       # 查看可更新包（outdated）
pnpm up:i       # 按 latest 升级依赖

# pnpm-workspace.yaml 中的 minimumReleaseAge 控制「多久之前发布的包才视为可用」；0 表示不限制
```



### React

使用 React 开发，参考 [React 文档](https://zh-hans.react.dev/)。

### 样式

使用原生 CSS Modules。样式文件中可用 kebab-case 类名，在 React 中以小驼峰引用；Vite 中已做相应配置。

## 后端



### 格式化（rustfmt）

为缩小与前端代码风格的差异，对 rustfmt 做了少量调整，见 [src-tauri/.rustfmt.toml](./src-tauri/.rustfmt.toml)。

```bash
cargo fmt --check
cargo fmt

# 仓库脚本（在项目根目录）
pnpm fmt
pnpm fmt:fix
```



### 依赖（Cargo）

除原生 `cargo` 外，推荐安装 [cargo-edit](https://github.com/killercup/cargo-edit) 以便升级 `Cargo.toml` 中的版本号：

```bash
# 仅首次需要
cargo install cargo-edit

# 仅刷新 Cargo.lock（不改 Cargo.toml 版本约束）
cargo update

# 在兼容版本范围内升级（推荐流程）
cargo upgrade -n    # 预览
cargo upgrade       # 写入 Cargo.toml
cargo update
cargo check

# 跨大版本：务必逐个 crate，不要一次全部 --incompatible
cargo upgrade -n --incompatible
cargo upgrade -n --incompatible -p some_crate
cargo upgrade --incompatible -p some_crate
cargo update -p some_crate
cargo check
```



### 编译缓存

Rust 编译缓存体积较大，可按需清理：

```bash
# 清理本项目编译产物，保留 ~/.cargo 依赖缓存（常用）
cargo clean
cargo clean --profile debug
cargo clean --profile release

# 查看 / 清理全局 ~/.cargo 缓存（需安装 cargo-cache）
cargo cache
cargo cache --autoclean
```



## 数据库 SQLCipher

应用会存储敏感数据，因此本地库使用 SQLite 的加密分支 [SQLCipher](https://github.com/sqlcipher/sqlcipher)，而不是明文 SQLite。

### 编译依赖

`[src-tauri/Cargo.toml](./src-tauri/Cargo.toml)` 中：

```toml
rusqlite = { version = "0.40", features = ["bundled-sqlcipher-vendored-openssl"] }
```

该 feature 会在 **编译期从源码构建 OpenSSL**，再编进 SQLCipher。因此真正需要的是：

- **C 工具链**（Windows：MSVC / Build Tools；macOS：Xcode CLT；Linux：`build-essential`）
- **Perl**（OpenSSL 的 `./Configure` 是 Perl 脚本）
- **make** 等基础构建工具（通常随上述工具链一起提供）

一般 **不必** 先单独安装一套「系统 OpenSSL 库」再链接；缺的是能跑通 `openssl-src` 的构建环境。各平台细节见下文。

### 运行时模型（简要）

应用数据目录下大致为：


| 文件                | 作用                              |
| ----------------- | ------------------------------- |
| `data.sqlite`     | SQLCipher 加密库；没有正确 DEK 时内容不可读   |
| `key.header.json` | 密钥头：盐、Argon2id 参数、被 KEK 包装的 DEK |
| `device.wrap`     | 设备槽密文。系统密钥库才能解开；没有它时每次都要输入档案密码 |


流程概要：

1. 用户密码经 **Argon2id** 派生 **KEK**，用 **XChaCha20-Poly1305** 包装随机 **DEK**（32 字节），写入 `key.header.json`。
2. 开库时解开 DEK，执行 `PRAGMA key = "x'<64 hex>'"`（原始密钥，不再让 SQLCipher 做口令派生）。
3. 启用设备槽后，同一把 DEK 再由本机系统密钥库包一份，写入 `device.wrap`。未锁定时打开应用会直接进入。点锁定后写入 `user.lock`，重启仍保持锁定，解锁需要 Windows Hello、Touch ID 或生物识别。拷走数据库目录仍然无法解密。见 [docs/device-unlock.md](./docs/device-unlock.md)。
4. 业务表通过 `DbState::with_conn` 访问；如何新增业务模块见 [docs/db-feature.md](./docs/db-feature.md)。



### 平台一览


| 场景                | 做法                                                                            |
| ----------------- | ----------------------------------------------------------------------------- |
| Windows 桌面        | 下文「Windows 桌面」                                                                |
| macOS 桌面          | 下文「macOS 桌面」                                                                  |
| Linux / WSL 桌面    | 下文「Linux / WSL」；apt 明细可参考 [docs/wsl-android.md](./docs/wsl-android.md) 第 4 节  |
| Windows → Android | **不能**在 Windows 本机交叉编译；必须在 WSL，见 [docs/wsl-android.md](./docs/wsl-android.md) |
| macOS → Android   | 见 [docs/mac-android.md](./docs/mac-android.md)                                |




### Windows 桌面

1. 安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)（勾选「使用 C++ 的桌面开发」），满足 Tauri / 原生 crate 的 MSVC 要求。
2. 安装 [Strawberry Perl](https://strawberryperl.com/)（Unix 路径风格的 Perl；OpenSSL Configure 依赖它）。安装后确认 `perl` 在 PATH 中。
3. 自检后编译桌面端：

```powershell
perl -v
cl      # 或在「x64 Native Tools」终端中确认 MSVC 可用
pnpm app
```

若 `openssl-src` 报找不到 Perl / 无法 Configure，优先检查 PATH 是否指向 Strawberry 的 `perl.exe`，以及是否在正确的 MSVC 开发者环境中执行 `cargo` / `pnpm app`。

### macOS 桌面

```bash
xcode-select --install          # 若尚未安装 Command Line Tools
brew install perl pkg-config    # 建议；系统 Perl 通常也可用

which perl make clang
pnpm app
```

Android 交叉编译另需 JDK 21、Android SDK/NDK 及 `AR_*` / `RANLIB_*` / `CC_*`，见 [docs/mac-android.md](./docs/mac-android.md)。

### Linux / WSL

桌面端至少需要：

```bash
sudo apt update
sudo apt install -y build-essential pkg-config perl
```

在 WSL 中编 Android、装完整 SDK/NDK 时，请直接按 [docs/wsl-android.md](./docs/wsl-android.md) 操作（含 `openjdk-21-jdk`、cmdline-tools、环境变量等）。

### Android 交叉编译

- **Windows**：本机交叉编译 OpenSSL 会失败（Perl / NDK `clang.cmd` 路径冲突）。请在 WSL 中完成，见 [docs/wsl-android.md](./docs/wsl-android.md)。
- **macOS**：本机即可，见 [docs/mac-android.md](./docs/mac-android.md)。

```bash
pnpm app:android
pnpm build:android
```

