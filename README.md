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


### android

官方前置：[Tauri Android 前置条件](https://v2.tauri.app/zh-cn/start/prerequisites/#android)。

档案库用了 `bundled-sqlcipher-vendored-openssl`，交叉编译会从源码编 OpenSSL，对 JDK / NDK 工具链更敏感。

| 平台 | 怎么做 |
| --- | --- |
| macOS | 本机即可，按下面「macOS」配置 |
| Windows | 本机交叉编译 OpenSSL 会失败，请在 WSL 里做，见 [docs/wsl-android.md](./docs/wsl-android.md) |

#### macOS

1. 安装 [Android Studio](https://developer.android.com/studio)（或只用 command-line tools），在 SDK Manager 里装好：
   - Android SDK Platform（与 `gen/android` 里 `compileSdk` 对齐，当前为 36）
   - Android SDK Build-Tools
   - NDK（侧边栏 SDK Tools → NDK；记下精确版本号，例如 `30.0.14904198`）
   - Android SDK Platform-Tools（`adb`）

2. 安装 **JDK 21**（不要用 Android Studio 自带的 JBR，若已是 Java 25）。  
   本仓库 Gradle 为 **8.14.x**，不支持 class file major version **69**（Java 25），会出现：
   `Unsupported class file major version 69`。

```bash
brew install openjdk@21
# 按 brew 提示做系统可见的 symlink（可选），并记住下面 JAVA_HOME 路径
```

3. 安装 Rust Android 目标（首次）：

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

4. 写入 `~/.zshrc`（按本机路径改 NDK 版本号；Apple Silicon 上 NDK 预编译目录仍叫 `darwin-x86_64`）：

```bash
# --- Precession Android (macOS) ---
export JAVA_HOME="$(brew --prefix openjdk@21)/libexec/openjdk.jdk/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
# 改成你 SDK Manager 里实际的 NDK 目录名
export NDK_HOME="$ANDROID_HOME/ndk/30.0.14904198"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$PATH"

# NDK 23+ 没有 aarch64-linux-android-ranlib，必须显式指向 llvm-ranlib
# 否则 openssl-src 的 make install_dev 会以 exit 2 失败
NDK_BIN="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export AR_aarch64_linux_android="$NDK_BIN/llvm-ar"
export RANLIB_aarch64_linux_android="$NDK_BIN/llvm-ranlib"
export CC_aarch64_linux_android="$NDK_BIN/aarch64-linux-android24-clang"
export AR_armv7_linux_androideabi="$NDK_BIN/llvm-ar"
export RANLIB_armv7_linux_androideabi="$NDK_BIN/llvm-ranlib"
export CC_armv7_linux_androideabi="$NDK_BIN/armv7a-linux-androideabi24-clang"
export AR_x86_64_linux_android="$NDK_BIN/llvm-ar"
export RANLIB_x86_64_linux_android="$NDK_BIN/llvm-ranlib"
export CC_x86_64_linux_android="$NDK_BIN/x86_64-linux-android24-clang"
export AR_i686_linux_android="$NDK_BIN/llvm-ar"
export RANLIB_i686_linux_android="$NDK_BIN/llvm-ranlib"
export CC_i686_linux_android="$NDK_BIN/i686-linux-android24-clang"
```

`source ~/.zshrc` 后自检：

```bash
java -version          # 应为 21.x，不是 25.x
echo "$JAVA_HOME"
echo "$ANDROID_HOME"
echo "$NDK_HOME"
echo "$RANLIB_aarch64_linux_android"
ls "$RANLIB_aarch64_linux_android"
adb version
```

5. 跑 Android：

```bash
pnpm app:android
# 正式包
pnpm build:android
```

常见问题：

| 现象 | 原因 |
| --- | --- |
| `make install_dev` / OpenSSL exit 2 | 未设 `AR_*` / `RANLIB_*` / `CC_*` |
| `Unsupported class file major version 69` | `JAVA_HOME` 仍指向 Java 25（常见为 Android Studio JBR） |
| `java.lang.System::load` 的 WARNING | 用错新 JDK 时的附带警告；先把 JDK 换成 21 即可，不必单独处理 |
