# Windows WSL：Tauri 2 + Android + SQLCipher 环境

Windows 桌面端可以正常开发。Android 不能在 Windows 本机编过，是因为档案库用了：

```toml
rusqlite = { version = "0.37", features = ["bundled-sqlcipher-vendored-openssl"] }
```

这会在交叉编译时从源码编 **OpenSSL**。Windows 上 Perl 路径和 NDK 的 `clang.cmd` 互相打架（`openssl-sys` Configure 失败）。WSL 里是 Unix Perl + Linux NDK 的真正 `clang`，这条链才能走通。

WSL 没有图形界面没关系。SDK 用命令行即可。App 界面跑在手机或 Windows 上的模拟器里，不跑在 Ubuntu 窗口里。

## 1. 两套环境怎么分工


| 在哪做        | 做什么                                                       |
| ---------- | --------------------------------------------------------- |
| Windows    | 桌面 `pnpm app` / `pnpm build:win`；可选：用 Android Studio 开模拟器 |
| WSL Ubuntu | **只做 Android 编译和** `adb` **安装**                           |


## 3. Windows侧

### 3.1 WSL2

确认是 WSL2：

```powershell
wsl -l -v
```

`VERSION` 必须是 `2`。本机 `networkingMode=mirrored` 时，WSL 里的 `adb` 能直接看到 Windows 上的手机/模拟器，**不必**设 `ADB_SERVER_SOCKET`。

### 3.2 Android Studio（可选）

WSL 编包不需要** Studio。只有要在 Windows 上点选创建/启动模拟器时才需要。真机调试同样不依赖 Studio。

## 4. WSL内：Ubuntu包

### 4.1 必须装的 apt 包

一条装齐：

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  pkg-config \
  perl \
  python3 \
  unzip \
  zip \
  curl \
  ca-certificates \
  git \
  openjdk-21-jdk
```


| 包                 | 为什么要装                                                                                                                                             |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `build-essential` | 元包，带上 `gcc` / `g++` / `make` / `libc6-dev`。Cargo 的 `build.rs` 在**宿主机**上跑 C 编译器；`openssl-src` 编 OpenSSL 时也会调 `make`。没有 `make`，SQLCipher 过不了。       |
| `pkg-config`      | 给 Rust 原生 crate 查头文件和 `.pc`。`openssl-sys`、部分系统库探测会用到。                                                                                             |
| `perl`            | **SQLCipher 的关键依赖。** OpenSSL 的 `./Configure` 是 Perl 脚本。WSL 的 `/usr/bin/perl` 是 Unix Perl，路径用正斜杠，这正是 Windows Strawberry（`MSWin32`）编 Android 失败的原因。 |
| `python3`         | NDK / Android SDK 部分包装脚本、以及 OpenSSL 3 构建会调 Python。不装可能在 NDK clang 包装器或 `sdkmanager` 外围脚本上报缺解释器。                                                   |
| `unzip`           | 解压 Google 的 `commandlinetools-linux-*.zip`。                                                                                                       |
| `zip`             | 打 APK、部分 Gradle / 构建脚本会调 `zip`。                                                                                                                   |
| `curl`            | 装 `rustup`、`fnm`、下载 cmdline-tools。                                                                                                                |
| `ca-certificates` | `curl` / `sdkmanager` 走 HTTPS。精简镜像有时缺证书。                                                                                                          |
| `git`             | `rustup` 默认工具链、Cargo git 依赖、从 `/mnt/d` 同步代码。                                                                                                      |
| `openjdk-21-jdk`  | Gradle 8.14 + AGP 8.11 跑在 JVM 上。只要 JRE 没有 `javac`，有些任务会失败。AGP 8.11 最低 JDK 17，这里与 Windows 已用的 21 对齐。                                               |


`build-essential` 已经包含 `make`，不必再单独 `apt install make`。`perl` 要单独写，因为它不在 `build-essential` 里。

## 5. Rust

Tauri要求的四个Android target都要加（dev可能编模拟器x86_64，真机是aarch64）：

```bash
rustup target add \
  aarch64-linux-android \
  armv7-linux-androideabi \
  i686-linux-android \
  x86_64-linux-android
```


| target                                        | 为什么                                           |
| --------------------------------------------- | --------------------------------------------- |
| `aarch64-linux-android`                       | 现在的手机；`pnpm run build:android` 也是这个。          |
| `armv7-linux-androideabi`                     | 32 位 ARM。`tauri android dev` 默认 universal 会编。 |
| `i686-linux-android` / `x86_64-linux-android` | 模拟器常见 ABI。                                    |


## 6. Node + pnpm

本仓库 `.node-version` 是 `lts-latest`，一样用 [fnm](https://www.lilei1921.cn/notes/node-version-tool-fnm)。

```bash
# 如果连接不上用下面地址
# curl -fsSL https://raw.githubusercontent.com/Schniz/fnm/master/.ci/install.sh  | bash
curl -fsSL https://fnm.vercel.app/install | bash
# 环境变量.bashrc配置查看上面文档
source ~/.bashrc
fnm install --lts
fnm use lts-latest
node -v
corepack enable
corepack prepare pnpm@latest --activate
pnpm -v
```

## 7. Android SDK / NDK（Linux 命令行，不要 Studio）

无界面 Linux 的标准做法：只装 [Command line tools only（Linux）](https://developer.android.com/studio#command-line-tools-only)。

### 7.1 `$ANDROID_HOME` 是什么、解压后怎么放

`$ANDROID_HOME` 是 **WSL 里 Android SDK 根目录的环境变量**。本文件把它固定为 Linux 家目录下的：

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
```

那么实际地址就是：

```text
/home/<你的 Ubuntu 用户名>/Android/Sdk
```

Google 的 zip **不能解压完就当 SDK 用**。zip 里面只有一层 `cmdline-tools/`，`sdkmanager` 却要求中间必须多一个 `latest` 目录：

```text
$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager
```

解压前 zip 内部是：

```text
cmdline-tools/
  bin/sdkmanager
  lib/
  NOTICE.txt
  source.properties
```

你要放到的最终样子：

```text
/home/<用户>/Android/Sdk/                  ← 这就是 $ANDROID_HOME
└── cmdline-tools/
    └── latest/                            ← 必须叫 latest，不要省略
        ├── bin/
        │   └── sdkmanager
        ├── lib/
        ├── NOTICE.txt
        └── source.properties
```

按下面做就会放到正确位置：

```bash
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64
export ANDROID_HOME="$HOME/Android/Sdk"
mkdir -p "$ANDROID_HOME/cmdline-tools"
```

放好后必须能直接执行：

```bash
ls "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
# 应输出：/home/<用户>/Android/Sdk/cmdline-tools/latest/bin/sdkmanager

"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --sdk_root="$ANDROID_HOME" --list | head
```

`--sdk_root="$ANDROID_HOME"` 的意思是：以后 `platform-tools`、`platforms`、`ndk` 都装进这个同一棵目录树，不要装到别的地方。

`JAVA_HOME` 此时必须已指向 JDK 21，否则 `sdkmanager`（Java 程序）起不来。Ubuntu 26.04 上路径一般是 `/usr/lib/jvm/java-21-openjdk-amd64`。若不同：

```bash
ls /usr/lib/jvm
```

### 7.2 sdkmanager 要装的包

```bash
yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --sdk_root="$ANDROID_HOME" --licenses

"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --sdk_root="$ANDROID_HOME" \
  "platform-tools" \
  "platforms;android-36" \
  "build-tools;36.1.0" \
  "ndk;29.0.13846066"
```

某包名报不存在时，先查再装：

```bash
"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --sdk_root="$ANDROID_HOME" --list | grep -E "build-tools|ndk;29|platforms;android-36"
```


| SDK 包                     | 为什么要装                                                                                                                                                                                           |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cmdline-tools`（上一步的 zip） | 提供 `sdkmanager`。没有它后面的包装不了。                                                                                                                                                                     |
| `platform-tools`          | `adb`。装 APK、看设备、端口转发、Chrome inspect 都靠它。                                                                                                                                                        |
| `platforms;android-36`    | 对应 `compileSdk = 36` / `targetSdk = 36`。缺了 Gradle 会报找不到 `android-36`。                                                                                                                           |
| `build-tools;36.1.0`      | `aapt2`、`d8`、`zipalign`、`apksigner`。打 APK 和签名用。                                                                                                                                                 |
| `ndk;29.0.13846066`       | **SQLCipher/OpenSSL 和 Rust Android 的编译器。** 提供 `linux-x86_64` 的 `clang` / `llvm-ar`，以及 `aarch64-linux-android24-clang` 这类包装器。`minSdk = 24`，所以是 API 24 的 triple。Tauri / cargo-mobile2 用它当 linker。 |


装完自检（都应存在）：

```bash
ls "$ANDROID_HOME/platform-tools/adb"
ls "$ANDROID_HOME/platforms/android-36"
ls "$ANDROID_HOME/build-tools/36.1.0/apksigner"
ls "$ANDROID_HOME/ndk/29.0.13846066/toolchains/llvm/prebuilt/linux-x86_64/bin/clang"
ls "$ANDROID_HOME/ndk/29.0.13846066/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang"
```

## 8. 环境变量（写入 `~/.bashrc`）

```bash
# --- Precession Android (WSL) ---
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export NDK_HOME="$ANDROID_HOME/ndk/29.0.13846066"
export PATH="$HOME/.cargo/bin:$JAVA_HOME/bin:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"

# NDK 23+ 不再提供 aarch64-linux-android-ranlib，必须显式指向 llvm-ranlib
# 否则 openssl-src 的 make install_dev 会以 exit 2 失败
NDK_BIN="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
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

`source ~/.bashrc` 后检查：

```bash
echo "$JAVA_HOME"
echo "$ANDROID_HOME"
echo "$NDK_HOME"
which java perl make adb rustc node pnpm
java -version
perl -v | head -2
adb version
rustc -V
node -v
pnpm -v
```


| 变量                           | 为什么                                                                                                                                              |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `JAVA_HOME`                  | Gradle / `sdkmanager` 找 JDK。只把 `java` 放进 PATH 不够，AGP 会读这个变量。                                                                                     |
| `ANDROID_HOME`               | Tauri CLI、Gradle、`sdkmanager` 的 SDK 根目录。必须指向 **Linux** SDK。                                                                                      |
| `ANDROID_SDK_ROOT`           | 部分旧脚本仍读这个名字，和 `ANDROID_HOME` 设成同一处。                                                                                                              |
| `NDK_HOME`                   | Tauri / cargo-mobile2 用来拼 clang 路径。必须是 `.../ndk/<精确版本>`，不要只写到 `ndk/`。                                                                            |
| `AR_*` / `RANLIB_*` / `CC_*` | NDK 23 起没有 `aarch64-linux-android-ranlib` 这种旧名字，只剩 `llvm-ar` / `llvm-ranlib`。不设的话 OpenSSL 能 Configure、能编 `.o`，却在 `make install_dev` 以 exit 2 失败。 |


## 9. 日常怎么跑

代码在 WSL 的 Linux 目录里。Windows 桌面仍用windows环境跑 `pnpm app`。

### 9.1 真机（USB 插 Windows）

1. 手机打开 USB 调试，数据线插电脑。
2. WSL 里 `adb devices -l`，状态为 `device` 即可。
3. `pnpm run app:android`

### 9.2 模拟器（窗口开在 Windows）

WSL 没有界面，模拟器只能在 Windows 开。

```powershell
# 已有的虚拟机名字
& "$env:ANDROID_HOME\emulator\emulator.exe" -list-avds

# 启动（名字换成上一步的输出，窗口不要关）
& "$env:ANDROID_HOME\emulator\emulator.exe" -avd Pixel_9a
```

没有 AVD：Android Studio → Device Manager → Create Device，镜像选 **x86_64**。  
`emulator.exe` 没有：SDK Manager 勾选 Android Emulator。  
`ANDROID_HOME` 空：`$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"`。

等到模拟器进桌面，另开 PowerShell 或在 WSL：`adb devices -l` 出现 `emulator-5554 ... device` 后，同样 `pnpm run app:android`。

### 9.3 只打包

```bash
pnpm run build:android
```

release 未签名包在：

`src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`

签名（WSL，`ANDROID_HOME` 指向 Linux 那份 SDK）：

```bash
UNSIGNED=src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk

"$ANDROID_HOME/build-tools/36.1.0/apksigner" sign \
  --ks "$HOME/.android/debug.keystore" \
  --ks-pass pass:android \
  --key-pass pass:android \
  --out ~/app-universal-release.apk \
  "$UNSIGNED"
```

没有 `~/.android/debug.keystore` 时，可用 Windows 上那份：`/mnt/c/Users/<用户>/.android/debug.keystore`。

安装：

```bash
adb devices -l
adb -s <序列号> install -r ~/app-universal-release.apk
```

### 9.4 卡住时

Gradle 下载超时：把 `src-tauri/gen/android/gradle/wrapper/gradle-wrapper.properties` 里的 `distributionUrl` 换成：

`https\://mirrors.cloud.tencent.com/gradle/gradle-8.14.3-bin.zip`

调试页用 Windows Chrome 的 `chrome://inspect`。