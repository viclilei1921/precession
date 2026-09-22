# macOS：Tauri 2 + Android + SQLCipher 环境

macOS 上可以同时做桌面端与 Android 开发。档案库使用：

```toml
rusqlite = { version = "0.40", features = ["bundled-sqlcipher-vendored-openssl"] }
```

交叉编译 Android 时会从源码构建 **OpenSSL**（再编进 SQLCipher）。与 Windows 不同：macOS 自带 Unix 风格的 Perl，配合 NDK 的真正 `clang`，这条链可以在本机走通，不必再套一层 Linux 虚拟机。

仍需显式设置 `AR_*` / `RANLIB_*` / `CC_*`：NDK 23+ 不再提供 `aarch64-linux-android-ranlib` 这类旧工具名，只剩 `llvm-ar` / `llvm-ranlib`。不设的话 OpenSSL 往往能 Configure、能编 `.o`，却在 `make install_dev` 以 exit 2 失败。

## 1. 两套目标怎么分工

| 在哪做 | 做什么 |
| --- | --- |
| macOS 桌面 | `pnpm app` / `pnpm build:mac`；日常前端与 Rust 桌面联调 |
| macOS + Android SDK/NDK | `pnpm app:android` / `pnpm build:android`；真机或模拟器 |

桌面端 SQLCipher（Perl / 工具链）见仓库根目录 [README.md](../README.md) 的「数据库 SQLCipher」一节。本文件只讲 **Android 交叉编译**。

## 2. 必备软件

### 2.1 Android Studio（或 Command-line tools）

安装 [Android Studio](https://developer.android.com/studio)，在 SDK Manager 中安装：

- Android SDK Platform（与 `gen/android` 里 `compileSdk` 对齐，当前为 **36**）
- Android SDK Build-Tools
- NDK（SDK Tools → NDK；记下精确版本号，例如 `30.0.14904198`）
- Android SDK Platform-Tools（`adb`）

SDK 默认根目录：

```text
$HOME/Library/Android/sdk
```

也可以只用 [Command line tools only（macOS）](https://developer.android.com/studio#command-line-tools-only)，目录布局与 WSL 文档类似，需保证存在：

```text
$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager
```

多数情况用 Android Studio 勾选上述组件即可，不必强迫纯命令行。

### 2.2 JDK 21

本仓库 Gradle 为 **8.14.x**，不支持 class file major version **69**（Java 25）。**不要**把 `JAVA_HOME` 指到 Android Studio 自带的 JBR（若已是 Java 25），否则会出现：

```text
Unsupported class file major version 69
```

用 Homebrew 安装 JDK 21：

```bash
brew install openjdk@21
# 按 brew 提示做系统可见的 symlink（可选），并记住下面 JAVA_HOME 路径
```

### 2.3 Homebrew 辅助包

桌面与 Android 共用时建议装齐（桌面 SQLCipher 编 OpenSSL 也需要 Perl）：

```bash
brew install perl pkg-config
```

| 包 | 为什么要装 |
| --- | --- |
| `perl` | OpenSSL 的 `./Configure` 是 Perl 脚本；macOS 系统 Perl 一般可用，Homebrew 版更易与 PATH 对齐 |
| `pkg-config` | 部分原生 crate 探测头文件 / `.pc` |

Xcode Command Line Tools（`clang` / `make`）必须已安装：`xcode-select --install`。

## 3. Rust Android 目标

Tauri 要求的四个 Android target 都要加（dev 可能编模拟器 x86_64，真机多为 aarch64）：

```bash
rustup target add \
  aarch64-linux-android \
  armv7-linux-androideabi \
  i686-linux-android \
  x86_64-linux-android
```

| target | 为什么 |
| --- | --- |
| `aarch64-linux-android` | 当前主流手机；`pnpm build:android` 也是这个 |
| `armv7-linux-androideabi` | 32 位 ARM；`tauri android dev` 默认 universal 会编 |
| `i686-linux-android` / `x86_64-linux-android` | 模拟器常见 ABI |

## 4. Node + pnpm

与桌面共用同一套工具链即可，见 [README.md](../README.md) 的「环境」一节。本仓库 `.node-version` 为 `lts-latest`，推荐 [fnm](https://www.lilei1921.cn/notes/node-version-tool-fnm)：

```bash
fnm install --lts
fnm use lts-latest
corepack enable
corepack prepare pnpm@latest --activate
node -v && pnpm -v
```

## 5. SDK / NDK 路径约定

默认：

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/<精确版本号>"
```

装完自检（路径按本机 NDK 版本改）：

```bash
ls "$ANDROID_HOME/platform-tools/adb"
ls "$ANDROID_HOME/platforms/android-36"
ls "$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/clang"
ls "$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android24-clang"
```

**注意：** 即便是 Apple Silicon，NDK 预编译工具目录名仍叫 `darwin-x86_64`，不要改成 `darwin-aarch64`。

`minSdk = 24`，因此 clang 包装器使用 `*-android24-clang` 这类 triple。

## 6. 环境变量（写入 `~/.zshrc`）

按本机 SDK Manager 里实际的 NDK 目录名修改 `NDK_HOME`：

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
which java perl make adb rustc node pnpm
adb version
rustc -V
node -v
pnpm -v
```

| 变量 | 为什么 |
| --- | --- |
| `JAVA_HOME` | Gradle / AGP 找 JDK；只把 `java` 放进 PATH 不够 |
| `ANDROID_HOME` / `ANDROID_SDK_ROOT` | Tauri CLI、Gradle 的 SDK 根目录；二者设成同一处 |
| `NDK_HOME` | 拼 clang 路径；必须是 `.../ndk/<精确版本>`，不要只写到 `ndk/` |
| `AR_*` / `RANLIB_*` / `CC_*` | 供 `openssl-src` 交叉编译；缺了会在 `make install_dev` 失败 |

## 7. 日常怎么跑

在仓库根目录：

```bash
# 开发（连真机或已启动的模拟器）
pnpm app:android

# 正式包（aarch64）
pnpm build:android
```

### 7.1 真机（USB）

1. 手机打开开发者选项与 USB 调试，数据线连接 Mac。
2. `adb devices -l`，状态为 `device`。
3. `pnpm app:android`。

### 7.2 模拟器

Android Studio → Device Manager → 创建/启动 AVD（镜像可用 arm64 或 x86_64，视本机芯片而定）。进桌面后：

```bash
adb devices -l
pnpm app:android
```

### 7.3 产物位置

release 未签名包通常在：

```text
src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk
```

（若只编了单一 ABI，路径可能在对应 flavor 下，以 Gradle 输出为准。）

签名可用 SDK 自带的 `apksigner`（Build-Tools 版本按本机安装调整）：

```bash
UNSIGNED=src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk

"$ANDROID_HOME/build-tools/36.1.0/apksigner" sign \
  --ks "$HOME/.android/debug.keystore" \
  --ks-pass pass:android \
  --key-pass pass:android \
  --out ~/app-universal-release.apk \
  "$UNSIGNED"

adb install -r ~/app-universal-release.apk
```

## 8. 常见问题

| 现象 | 原因 / 处理 |
| --- | --- |
| `make install_dev` / OpenSSL exit 2 | 未设 `AR_*` / `RANLIB_*` / `CC_*`，或 `NDK_BIN` 指错（应为 `darwin-x86_64`） |
| `Unsupported class file major version 69` | `JAVA_HOME` 仍指向 Java 25（常见为 Android Studio JBR）；改用 `openjdk@21` |
| `java.lang.System::load` 的 WARNING | 用错新 JDK 时的附带警告；先换成 JDK 21 即可 |
| `sdkmanager` / Gradle 找不到 `android-36` | SDK Manager 未装 `platforms;android-36` |
| Configure / Perl 相关失败 | 确认 `which perl` 可用；必要时 `brew install perl` |

## 与 Windows WSL 文档的差异

| 项 | macOS（本文） | Windows → Android |
| --- | --- | --- |
| 编译场所 | 本机 macOS | **必须**在 WSL，见 [wsl-android.md](./wsl-android.md) |
| NDK 预编译目录 | `darwin-x86_64` | `linux-x86_64` |
| JDK | `brew install openjdk@21` | WSL 内 `openjdk-21-jdk` |
| SDK 管理 | Android Studio 即可 | 常用 Linux cmdline-tools |
