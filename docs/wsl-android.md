# Windows WSL：Tauri 2 + Android + SQLCipher 环境

Windows 桌面端可以正常开发。Android **不能**在 Windows 本机编过，是因为档案库用了：

```toml
rusqlite = { version = "0.40", features = ["bundled-sqlcipher-vendored-openssl"] }
```

这会在交叉编译时从源码编 **OpenSSL**。Windows 上 Perl 路径和 NDK 的 `clang.cmd` 互相打架（`openssl-sys` Configure 失败）。WSL 里是 Unix Perl + Linux NDK 的真正 `clang`，这条链才能走通。

macOS 本机可直接交叉编译 Android，见 [mac-android.md](./mac-android.md)。桌面端 SQLCipher 共性说明见仓库根目录 [README.md](../README.md)。

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

上面的 `debug.keystore` 只用于本机安装。备案和对外发布用下面这把正式密钥，两套证书的 MD5 不同。

### 9.4 发布密钥与备案指纹

备案要填包名、签名证书的 MD5 指纹和公钥。包名是 `com.viclilei.precession`。指纹和公钥都从正式密钥库导出。

密钥库存到仓库外面（WSL 家目录即可）。密钥库文件、密钥库口令、密钥口令、别名 `precession` 四样一起备份。`precession.cer` 只是导出的公钥证书，可以随时再导。不要把 `.jks` 提交进 git。

没有 `openssl` 时先装：`sudo apt install -y openssl`。

生成密钥（RSA 2048，有效期约 27 年）：

```bash
keytool -genkeypair -v \
  -keystore "$HOME/precession-release.jks" \
  -alias precession \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000
```

密钥库口令自己设定，输入时不回显，随后再确认一次。姓名和组织填自己的名字即可。城市和省份用拼音（例如 `Shenzhen`、`Guangdong`），国家代码填 `CN`。问到 `<precession>` 的密钥口令时直接回车，即与密钥库口令相同。

确认行应类似：

```text
CN=viclilei, OU=viclilei, O=viclilei, L=Shenzhen, ST=Guangdong, C=CN是否正确?
```

此时输入 `y`。城市或省份若显示成 `????`，直接回车（默认否），改用拼音重新执行上面的命令。

JDK 21 的 `keytool -list -v` 只打印 SHA1 和 SHA256。MD5 从导出的证书计算。导出时不要加 `-rfc`，否则哈希对不上证书指纹。

```bash
keytool -exportcert -alias precession \
  -keystore "$HOME/precession-release.jks" \
  -file "$HOME/precession.cer"

openssl x509 -inform DER -in "$HOME/precession.cer" -fingerprint -md5 -noout
openssl x509 -inform DER -in "$HOME/precession.cer" -pubkey -noout
```

`MD5 Fingerprint=` 后面去掉冒号，得到 32 位十六进制，填备案的证书 MD5。公钥先整段粘贴（含 `BEGIN PUBLIC KEY` / `END PUBLIC KEY`）。页面若提示格式不对，再去掉首尾两行和换行，只留中间的 Base64。

用这把密钥签名正式包：

```bash
UNSIGNED=src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk

"$ANDROID_HOME/build-tools/36.1.0/apksigner" sign \
  --ks "$HOME/precession-release.jks" \
  --ks-pass pass:你的密钥库口令 \
  --key-pass pass:你的密钥口令 \
  --ks-key-alias precession \
  --out ~/app-universal-release.apk \
  "$UNSIGNED"
```

### 9.5 GitHub Actions 正式签名

CI 的 Android 包由 `.github/workflows/build.yml` 里的 `build-android` 签名。密钥写在 GitHub 仓库的 Actions secrets 里，不进 git。

配置页：<https://github.com/viclilei1921/precession/settings/secrets/actions>

点 **New repository secret**，一次加一条。用上一节已经生成的 `$HOME/precession-release.jks`，别名 `precession`。换一把新密钥后，证书指纹会变，已安装的包无法覆盖升级，备案指纹也对不上。

| Name | Secret |
| --- | --- |
| `ANDROID_KEY_ALIAS` | `precession` |
| `ANDROID_KEYSTORE_PASSWORD` | 生成密钥库时设的口令 |
| `ANDROID_KEY_PASSWORD` | 密钥口令。生成时直接回车的话，与密钥库口令相同 |
| `ANDROID_KEYSTORE_BASE64` | 密钥库文件的 Base64，一整段，不要换行，不要带 `BEGIN` / `END` |

取出 Base64：

```bash
base64 -w 0 "$HOME/precession-release.jks"
```

记不清口令时，能列出证书即说明口令正确。把刚输入的那串分别填进两个口令 Secret：

```bash
keytool -list -keystore "$HOME/precession-release.jks" -alias precession
```

四条要一起配。只配了 `ANDROID_KEYSTORE_BASE64`、口令或别名缺了，签名步骤会失败，日志是「已提供 ANDROID_KEYSTORE_BASE64，但密码或别名为空」。四条都空时，CI 会临时生成密钥，APK 能装，后续版本无法覆盖安装。`GITHUB_TOKEN` 由 Actions 自动注入，不用建。工作流里的 macOS、Windows 签名目前是注释，不用配。

手动确认：Actions → **Cross-Platform Build** → **Run workflow**。打开 `build-android` 的 **Sign APK**。成功时这一步是绿的，日志里有：

```text
使用仓库密钥签名
```

产物在该次运行 Summary 底部，名字是 `precession-aarch64-linux-android`。口令错误、别名不存在，或 Base64 不是这把 `.jks` 时，`apksigner` 会报密钥库密码错误或格式不对。

### 9.6 从 APK 核对包结构

下载的 Artifact 是 zip，先解压得到里面的 `.apk`。手动触发时文件名类似 `precession-0.1.0-<7位提交号>-aarch64.apk`。推送 `v*` tag 时，版本段是 tag 名，例如 `precession-v0.1.0-aarch64.apk`。

这次 CI 执行的是 `tauri android build --target aarch64 --apk`。正确的包同时满足：

- 包名 `com.viclilei.precession`，`minSdk` 24，`targetSdk` 36
- 原生代码只有 `arm64-v8a`
- 存在 `lib/arm64-v8a/libprecession_lib.so`。`tauri.android.conf.json` 把 `externalBin` 置空，包里没有 `ffmpeg`、`cjxl`、`avifenc`
- 签名证书的 SHA-256 与 `$HOME/precession-release.jks` 里别名 `precession` 的证书一致

```bash
APK=~/precession-0.1.0-xxxxxxx-aarch64.apk

"$ANDROID_HOME/build-tools/36.1.0/aapt" dump badging "$APK" | grep -E "package:|sdkVersion:|targetSdkVersion:|native-code:"
unzip -l "$APK" | grep -E "lib/|AndroidManifest.xml|classes.*dex"
"$ANDROID_HOME/build-tools/36.1.0/apksigner" verify --print-certs "$APK"
keytool -list -v -keystore "$HOME/precession-release.jks" -alias precession
```

`aapt` 里应看到包名和 `native-code: 'arm64-v8a'`。`apksigner` 打印 `Verifies`，并且证书 SHA-256 与 `keytool` 那一行相同，说明包结构和正式签名都对。能否在手机上打开，仍要安装后看。

本机 `pnpm run build:android` 打的是 universal 包（见 9.3），可以带多个 ABI。上面的「只有 arm64-v8a」针对这次 CI 产物。

### 9.7 卡住时

Gradle 下载超时：把 `src-tauri/gen/android/gradle/wrapper/gradle-wrapper.properties` 里的 `distributionUrl` 换成：

`https\://mirrors.cloud.tencent.com/gradle/gradle-8.14.3-bin.zip`

调试页用 Windows Chrome 的 `chrome://inspect`。