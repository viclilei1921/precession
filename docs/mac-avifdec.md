# macOS 编译 avifdec

面向 **Apple Silicon（arm64）**，编译与仓库里 `avifenc-aarch64-apple-darwin` 同一版的 `avifdec`。现有侧车是 libavif **1.4.2**（dav1d 1.5.3、aom 3.14.1）。不编 Intel。

画质比对文档见 [mac-ssimulacra2.md](./mac-ssimulacra2.md)。侧车只依赖 macOS 系统库（`libSystem`、`libc++`），可以拷到其他 Apple Silicon Mac 上运行。不要用 `brew install libavif` 的动态库，那些瓶链着 `/opt/homebrew`。

Apple 链接器做不出「一个文件、零动态库」的全静态程序。不要加 `-static`。

---

## 一、环境准备

`uname -m` 必须是 `arm64`。若输出 `x86_64`，当前终端跑在 Rosetta 下。

```bash
uname -m
xcode-select -p
```

编译工具：

```bash
brew install cmake meson ninja pkg-config git
```

dav1d 用 Meson 编。确认编译器是 Xcode 的 clang：

```bash
xcrun --find clang
```

源码放在 `~/git/libavif`。下文命令都从这里的上一级或仓库根目录执行，以每段开头的 `cd` 为准。

---

## 二、取源码

主仓库用 Gitee。`aom` v3.14.1 没有可用的 Gitee 镜像，只有这一份从 googlesource 浅克隆。其余依赖先放进 `ext/`，CMake 看到目录就不会再下载。

```bash
cd ~/git
git clone -b v1.4.2 --depth 1 https://gitee.com/mirrors_libsdl-org/libavif.git
cd libavif

git clone --depth 1 -b 1.5.3 https://gitee.com/mirrors_videolan/dav1d.git ext/dav1d
git clone --depth 1 -b v3.14.1 https://aomedia.googlesource.com/aom.git ext/aom
git clone --depth 1 -b v1.3.2 https://gitee.com/mirrors/zlib.git ext/zlib
git clone --depth 1 -b v1.6.58 https://gitee.com/mirrors/libpng.git ext/libpng
git clone --depth 1 -b v1.6.0 https://gitee.com/mirrors/libwebp.git ext/libwebp
git clone --depth 1 -b 3.1.4.1 https://gitee.com/mirrors/libjpeg-turbo.git ext/libjpeg-turbo

mkdir ext/libyuv
git -C ext/libyuv init
git -C ext/libyuv remote add origin https://gitcode.com/gh_mirrors/li/libyuv.git
git -C ext/libyuv fetch --depth 1 origin 644251f252a84bf8ce91ff0aca86a9b16b069ab8
git -C ext/libyuv checkout FETCH_HEAD
```

`ext/` 里如果已经有同名空目录，先删掉再克隆。libyuv 这一笔是 libavif 1.4.2 钉死的提交。

libjpeg-turbo 要先编成静态库。libavif 只在下面这个路径找到成品时才跳过下载：

```bash
cmake -S ext/libjpeg-turbo -B ext/libjpeg-turbo/build.libavif \
  -DCMAKE_BUILD_TYPE=Release \
  -DENABLE_SHARED=OFF -DENABLE_STATIC=ON -DWITH_TURBOJPEG=OFF
cmake --build ext/libjpeg-turbo/build.libavif --target jpeg-static
```

完成后应存在 `ext/libjpeg-turbo/build.libavif/libjpeg.a`。

---

## 三、配置并编译

libavif 1.4.2 把本地 aom 的变量写成了 `FETCHCONTENT_SOURCE_DIR_AOM`，而 `FetchContent` 的名字是 `libaom`，实际只认 `FETCHCONTENT_SOURCE_DIR_LIBAOM`。漏掉这一项时，CMake 会忽略 `ext/aom`，停在 `configuring aom` 并从 googlesource 重新克隆。

```bash
cd ~/git/libavif
rm -rf build
cmake -S . -B build \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_SHARED_LIBS=OFF \
  -DAVIF_CODEC_AOM=LOCAL \
  -DAVIF_CODEC_DAV1D=LOCAL \
  -DAVIF_LIBYUV=LOCAL \
  -DAVIF_LIBSHARPYUV=LOCAL \
  -DAVIF_JPEG=LOCAL \
  -DAVIF_ZLIBPNG=LOCAL \
  -DAVIF_BUILD_APPS=ON \
  -DFETCHCONTENT_SOURCE_DIR_LIBAOM="$PWD/ext/aom"
cmake --build build --target avifdec
```

配置日志里 aom、libyuv、libwebp、zlib、libpng 都应出现 `ext/... found`。aom 的配置要跑很多编译器探测，几分钟没有新行是正常的，结束时会打印 `configuring aom - complete`。

成品在 `~/git/libavif/build/avifdec`。

---

## 四、验证

```bash
file ~/git/libavif/build/avifdec
otool -L ~/git/libavif/build/avifdec
~/git/libavif/build/avifdec --version
```

`file` 应显示 `Mach-O 64-bit executable arm64`。`otool -L` 里只应有 `/usr/lib/` 和 `/System/Library/` 下的系统库，例如 `libSystem.B.dylib`、`libc++.1.dylib`。出现 `/opt/homebrew` 或 `/usr/local/opt` 说明某个依赖被动态链接了，删掉 `build` 后确认第二节的 `ext/` 都在，再重新配置。

`--version` 应报 `1.4.2`，并带 dav1d 与 aom。程序调用的是 `avifdec -d 8 输入.avif 输出.png`；10-bit 和 12-bit 用 `-d 16`。

---

## 五、放到本仓库

Tauri 侧车文件名固定为 `avifdec-aarch64-apple-darwin`。

```bash
cd /path/to/precession
cp ~/git/libavif/build/avifdec src-tauri/binaries/avifdec-aarch64-apple-darwin
chmod +x src-tauri/binaries/avifdec-aarch64-apple-darwin
```
