# macOS 编译 FFmpeg 裁剪指南

面向 **Apple Silicon（arm64）**，用 Xcode 命令行工具编译 FFmpeg 稳定版。不编 Intel。裁剪目标和 Windows 指南一致，见 [window-ffmpeg.md](./window-ffmpeg.md)。

第三方库（libass、freetype、x265、SVT-AV1 等）全部静态链进 `ffmpeg`。成品只依赖 macOS 系统库（`libSystem`、`libc++`），可以拷到其他 Apple Silicon Mac 上运行，不需要安装 Homebrew。

Apple 链接器做不出 MinGW 那种「一个文件、零动态库」的全静态程序。不要加 `-static`。

---

## 一、环境准备

### 1. 终端与架构

只在 Apple Silicon 本机编译，产物是 arm64。`uname -m` 必须是 `arm64`。若输出 `x86_64`，当前终端跑在 Rosetta 下，退出该终端后用原生终端再编。

```bash
uname -m
# arm64

xcode-select -p
# 应指向 Xcode 或 Command Line Tools
```

尚未安装命令行工具时：

```bash
xcode-select --install
```

### 2. 编译工具

依赖库从源码静态安装，不要用 `brew install ffmpeg libass x265 svt-av1`。那些瓶是动态库，链进去之后成品会依赖 `/opt/homebrew/lib` 或 `/usr/local/lib`，换一台机器就无法运行。

```bash
brew install cmake meson ninja pkg-config git
```

确认用的是 Xcode 的 clang，而不是 Homebrew 的 llvm：

```bash
xcrun --find clang
# 期望：/Applications/Xcode.app/... 或 /Library/Developer/CommandLineTools/...
```

### 3. 安装前缀

依赖装到 `~/ffmpeg-deps`，FFmpeg 本身装到 `~/ffmpeg-custom`。不要装进 `/usr/local` 或 Homebrew 前缀，避免和系统包混在一起。

新开一个终端后，先重新执行这一段，再继续后面的步骤：

```bash
export PREFIX="$HOME/ffmpeg-deps"
export PATH="$PREFIX/bin:$PATH"
export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig"
export PKG_CONFIG_LIBDIR="$PREFIX/lib/pkgconfig"
export MACOSX_DEPLOYMENT_TARGET=11.0
export CC=clang
export CXX=clang++
export CFLAGS="-mmacosx-version-min=11.0"
export CXXFLAGS="-mmacosx-version-min=11.0"
export LDFLAGS="-mmacosx-version-min=11.0"
export JOBS="$(sysctl -n hw.ncpu)"

mkdir -p "$PREFIX" "$HOME/src/ffmpeg-build"
```

`PKG_CONFIG_LIBDIR` 把搜索范围限制在 `$PREFIX`，避免 pkg-config 捡到 Homebrew 的动态库。

---

## 二、静态编译依赖

以下版本与 FFmpeg `n9.0.2` 配套。某个发布包地址失效时，换该库的当前稳定版，仍安装到同一个 `$PREFIX`。

| 库 | 版本 | 作用 |
| --- | --- | --- |
| zlib / libpng / brotli | 1.3.1 / 1.6.47 / 1.1.0 | freetype 读字体 |
| freetype | 2.14.1 | `drawtext`、字幕渲染 |
| fribidi / harfbuzz | 1.0.16 / 10.1.0 | 双向文字与整形；libass 0.17 需要 harfbuzz |
| libass | 0.17.4 | 字幕烧录。关闭 fontconfig 和 CoreText，字体由 `fontfile=` 传入 |
| SVT-AV1 | 4.2.0 | AV1 软件编码 |
| x265 | 4.2 | H.265。同一个静态库包含 8 bit 和 10 bit，不编 12 bit |
| libvpx | 1.15.2 | 只编 VP9。AAC 用 FFmpeg 内置编码器，不编译 FDK-AAC |

每段命令都假定已经执行过第一节的环境变量，并且从 `~/src/ffmpeg-build` 开始。

### 1. zlib、libpng、brotli

```bash
cd "$HOME/src/ffmpeg-build"

curl -fL -o zlib-1.3.1.tar.gz https://zlib.net/zlib-1.3.1.tar.gz
tar xf zlib-1.3.1.tar.gz
cd zlib-1.3.1
./configure --prefix="$PREFIX" --static
make -j"$JOBS" && make install
cd ..

curl -fL -o libpng-1.6.47.tar.xz https://download.sourceforge.net/libpng/libpng-1.6.47.tar.xz
tar xf libpng-1.6.47.tar.xz
cd libpng-1.6.47
./configure --prefix="$PREFIX" --enable-static --disable-shared --with-zlib-prefix="$PREFIX"
make -j"$JOBS" && make install
cd ..

curl -fL -o brotli-1.1.0.tar.gz https://github.com/google/brotli/archive/refs/tags/v1.1.0.tar.gz
tar xf brotli-1.1.0.tar.gz
cmake -S brotli-1.1.0 -B brotli-1.1.0/build \
  -DCMAKE_INSTALL_PREFIX="$PREFIX" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_OSX_ARCHITECTURES=arm64 \
  -DBUILD_SHARED_LIBS=OFF \
  -DCMAKE_POSITION_INDEPENDENT_CODE=ON
cmake --build brotli-1.1.0/build -j"$JOBS"
cmake --install brotli-1.1.0/build
```

### 2. freetype、fribidi、harfbuzz、libass

freetype 先不链接 harfbuzz，避免两者循环依赖。libass 关闭 CoreText 和 fontconfig，字体由调用方通过 `fontfile=` 传入（本仓库 macOS 使用苹方等系统字体文件）。

```bash
cd "$HOME/src/ffmpeg-build"

curl -fL -o freetype-2.14.1.tar.xz https://download.savannah.gnu.org/releases/freetype/freetype-2.14.1.tar.xz
tar xf freetype-2.14.1.tar.xz
cd freetype-2.14.1
./configure --prefix="$PREFIX" --enable-static --disable-shared \
  --without-harfbuzz --with-png=yes --with-brotli=yes --with-zlib=yes --without-bzip2
make -j"$JOBS" && make install
cd ..

curl -fL -o fribidi-1.0.16.tar.xz https://github.com/fribidi/fribidi/releases/download/v1.0.16/fribidi-1.0.16.tar.xz
tar xf fribidi-1.0.16.tar.xz
cd fribidi-1.0.16
./configure --prefix="$PREFIX" --enable-static --disable-shared
make -j"$JOBS" && make install
cd ..

curl -fL -o harfbuzz-10.1.0.tar.xz https://github.com/harfbuzz/harfbuzz/releases/download/10.1.0/harfbuzz-10.1.0.tar.xz
tar xf harfbuzz-10.1.0.tar.xz
meson setup harfbuzz-10.1.0/build harfbuzz-10.1.0 \
  --prefix="$PREFIX" \
  --default-library=static \
  --buildtype=release \
  -Dtests=disabled \
  -Ddocs=disabled \
  -Dbenchmark=disabled \
  -Dutilities=disabled \
  -Dcairo=disabled \
  -Dicu=disabled \
  -Dglib=disabled \
  -Dgobject=disabled \
  -Dintrospection=disabled \
  -Dcoretext=disabled \
  -Dfreetype=enabled
ninja -C harfbuzz-10.1.0/build
ninja -C harfbuzz-10.1.0/build install

curl -fL -o libass-0.17.4.tar.xz https://github.com/libass/libass/releases/download/0.17.4/libass-0.17.4.tar.xz
tar xf libass-0.17.4.tar.xz
cd libass-0.17.4
./configure --prefix="$PREFIX" --enable-static --disable-shared \
  --disable-fontconfig --disable-coretext --disable-require-system-font-provider
make -j"$JOBS" && make install
cd ..
```

### 3. SVT-AV1、x265、libvpx

```bash
cd "$HOME/src/ffmpeg-build"

git clone --depth 1 --branch v4.2.0 https://gitlab.com/AOMediaCodec/SVT-AV1.git
cmake -S SVT-AV1 -B SVT-AV1/build \
  -DCMAKE_INSTALL_PREFIX="$PREFIX" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_OSX_ARCHITECTURES=arm64 \
  -DBUILD_SHARED_LIBS=OFF \
  -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
  -DBUILD_APPS=OFF \
  -DBUILD_TESTING=OFF \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build SVT-AV1/build -j"$JOBS"
cmake --install SVT-AV1/build

git clone --depth 1 --branch 4.2 https://bitbucket.org/multicoreware/x265_git.git x265

# 10 bit 只做中间库，不安装。关掉 C 接口后符号进 x265_10bit，避免和 8 bit 重名。
cmake -S x265/source -B x265/build-10 \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_OSX_ARCHITECTURES=arm64 \
  -DENABLE_SHARED=OFF \
  -DENABLE_CLI=OFF \
  -DENABLE_PIC=ON \
  -DHIGH_BIT_DEPTH=ON \
  -DEXPORT_C_API=OFF \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build x265/build-10 -j"$JOBS"

# 8 bit 打开 LINKED_10BIT，x265_api_get(10) 转到上面那份。
mkdir -p x265/build
ln -sfn ../build-10/libx265.a x265/build/libx265_main10.a
cmake -S x265/source -B x265/build \
  -DCMAKE_INSTALL_PREFIX="$PREFIX" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_OSX_ARCHITECTURES=arm64 \
  -DENABLE_SHARED=OFF \
  -DENABLE_CLI=OFF \
  -DENABLE_PIC=ON \
  -DEXTRA_LIB="x265_main10.a" \
  -DEXTRA_LINK_FLAGS="-L." \
  -DLINKED_10BIT=ON \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build x265/build -j"$JOBS"

# CMake 不会把 10 bit 目标文件打进 8 bit 的归档。合成后再安装，FFmpeg 只链 -lx265。
libtool -static -o x265/build/libx265-combined.a \
  x265/build/libx265.a x265/build/libx265_main10.a
cmake --install x265/build
cp x265/build/libx265-combined.a "$PREFIX/lib/libx265.a"
ranlib "$PREFIX/lib/libx265.a"

curl -fL -o libvpx-1.15.2.tar.gz https://github.com/webmproject/libvpx/archive/refs/tags/v1.15.2.tar.gz
tar xf libvpx-1.15.2.tar.gz
cd libvpx-1.15.2
./configure --prefix="$PREFIX" --enable-static --disable-shared --enable-pic \
  --disable-examples --disable-unit-tests --disable-vp8 \
  --target=arm64-darwin20-gcc
make -j"$JOBS" && make install
cd ..
```

x265 先编 10 bit、再编 8 bit，最后合成一个 `libx265.a`。10 bit 那份不要 `cmake --install`，它的 `x265_config.h` 会盖掉 8 bit 的头文件。不编 12 bit。

macOS 的 C++ 标准库是 `libc++`。x265 的 pkg-config 有时写成 `-lstdc++`，链接会失败。装完后改掉：

```bash
sed -i '' 's/-lstdc++/-lc++/g' "$PREFIX/lib/pkgconfig/x265.pc"
```

### 4. 确认静态库都在

```bash
ls "$PREFIX/lib/libass.a" \
   "$PREFIX/lib/libfreetype.a" \
   "$PREFIX/lib/libfribidi.a" \
   "$PREFIX/lib/libharfbuzz.a" \
   "$PREFIX/lib/libSvtAv1Enc.a" \
   "$PREFIX/lib/libx265.a" \
   "$PREFIX/lib/libvpx.a"

pkg-config --static --libs libass freetype2 fribidi harfbuzz SvtAv1Enc x265 vpx
```

`pkg-config` 的输出里不应出现 `/opt/homebrew` 或 `.dylib`。

---

## 三、获取 FFmpeg 源码

官方源下载速度慢，推荐使用国内镜像。

```bash
cd "$HOME/src/ffmpeg-build"
git clone https://gitee.com/mirrors/ffmpeg.git ffmpeg
cd ffmpeg
git checkout n9.0.2
```

---

## 四、编译配置

### 1. 裁剪需求说明

与 Windows 指南相同，从 `--disable-everything` 开始，再按名打开组件：

- 视频编码：`libx265`、`libsvtav1`、`libvpx-vp9`
- 音频编码：内置 `aac`。多条音轨只保留一条时用 `-map` 选出，再编成 AAC
- 字幕：文本字幕可解码。写入 MP4/MOV 时编成 `mov_text`；烧进画面用滤镜 `subtitles`、`ass`
- 封装写出：MP4、MOV、MPEG-TS
- 封装读入：MOV（同时覆盖 MP4）、WebM、Matroska（MKV 只读不写）、MPEG-TS、concat，以及外挂字幕 SRT、ASS、WebVTT
- 协议：`file`、`pipe`
- 移除 GPU 硬件加速（macOS 上是 VideoToolbox、AudioToolbox、AVFoundation、CoreImage、Metal）、网络协议、图片格式、蓝光、PGS、DVD 字幕
- 不包含 FLV、HLS、MP3 封装，也不编 H.264、VP8、MP3、FDK-AAC

### 2. 完整配置命令

在 FFmpeg 源码目录下执行。若当前 shell 没有第一节的环境变量，先回到那一节重新导出。`--disable-everything` 必须放在组件开关最前面。

```bash
cd "$HOME/src/ffmpeg-build/ffmpeg"

./configure \
  --prefix="$HOME/ffmpeg-custom" \
  --arch=arm64 \
  --cc=clang \
  --disable-everything \
  --disable-ffplay \
  --disable-ffprobe \
  --disable-doc \
  --disable-network \
  --disable-sdl2 \
  --disable-videotoolbox \
  --disable-audiotoolbox \
  --disable-avfoundation \
  --disable-coreimage \
  --disable-appkit \
  --disable-metal \
  --disable-securetransport \
  --disable-opengl \
  --disable-opencl \
  --disable-vulkan \
  --disable-vaapi \
  --enable-ffmpeg \
  --enable-avutil \
  --enable-avcodec \
  --enable-avformat \
  --enable-avfilter \
  --enable-swscale \
  --enable-swresample \
  --enable-gpl \
  --enable-libx265 \
  --enable-libsvtav1 \
  --enable-libvpx \
  --enable-libfreetype \
  --enable-libharfbuzz \
  --enable-libfribidi \
  --enable-libass \
  --enable-protocol=file,pipe \
  --enable-muxer=mp4,mov,mpegts \
  --enable-demuxer=mov,webm,matroska,mpegts,concat,srt,ass,webvtt \
  --enable-encoder=libx265,libsvtav1,libvpx_vp9,aac,movtext \
  --enable-decoder=h264,hevc,vp8,vp9,av1,aac,opus,vorbis,mp3,ass,ssa,subrip,webvtt,movtext,text \
  --enable-parser=h264,hevc,vp8,vp9,av1,aac,aac_latm,opus,vorbis,mpegaudio \
  --enable-bsf=h264_mp4toannexb,hevc_mp4toannexb,aac_adtstoasc,extract_extradata,vp9_superframe,av1_metadata,null \
  --enable-filter=fps,scale,setsar,pad,drawtext,aresample,aformat,concat,format,subtitles,ass \
  --enable-static \
  --disable-shared \
  --pkg-config-flags="--static" \
  --extra-cflags="-I${PREFIX}/include" \
  --extra-ldflags="-L${PREFIX}/lib" \
  --extra-libs="-lc++" \
  --enable-small
```

组件名和 Windows 相同：configure 里写 `libvpx_vp9`、`movtext`。`ffmpeg -encoders` 打印出来的是 `libvpx-vp9`、`mov_text`。写成打印名时，configure 会悄悄丢掉这两项，随后因 `--enable-libvpx` 没有任何编解码器而失败。

### 3. 与 Windows 配置的对应关系

| 分类 | macOS 参数 | 作用 |
| --- | --- | --- |
| 白名单 | `--disable-everything` 后再 `--enable-*` | 与 Windows 相同的封装、编解码、滤镜和协议 |
| 程序裁剪 | `--disable-ffplay/ffprobe/doc/sdl2` | 不编播放器、分析工具、文档和 SDL |
| 网络 | `--disable-network` `--disable-securetransport` | 只保留 `file`、`pipe`，并关掉 Apple TLS |
| 硬件加速 | `--disable-videotoolbox/audiotoolbox/avfoundation/coreimage/appkit/metal/opengl/opencl/vulkan/vaapi` | 对应 Windows 指南里的 DXVA、D3D、NVENC、AMF、VAAPI。产品不再使用 GPU 编码。Intel QSV 需另开 `--enable-libmfx` 或 `--enable-libvpl`，这里不开 |
| 文字与字幕 | `--enable-libfreetype/libharfbuzz/libfribidi/libass` | `drawtext` 需要 freetype 和 harfbuzz。`subtitles`、`ass` 需要 libass。字体用 `fontfile=` |
| 软件编码 | `--enable-libx265/libsvtav1/libvpx` `--enable-gpl` | H.265（8 bit 与 10 bit）、AV1、VP9。configure 里写 `libvpx_vp9`、`movtext`。AAC 和 `mov_text` 是内置编码器，不使用 `--enable-nonfree` |
| 静态链接 | `--enable-static --disable-shared` `--pkg-config-flags="--static"` | 第三方库打进二进制 |
| C++ 运行库 | `--extra-libs="-lc++"` | 静态链接 x265、harfbuzz 时解析 C++ 符号 |
| 体积 | `--enable-small` | 缩小二进制，换极少一点速度 |
| 架构 | `--arch=arm64` | 只生成 Apple Silicon 二进制 |
| 安装路径 | `--prefix="$HOME/ffmpeg-custom"` | 不写入系统目录 |

---

## 五、编译与安装

### 1. 清理旧缓存（修改配置后必须执行）

```bash
cd "$HOME/src/ffmpeg-build/ffmpeg"
make distclean
```

`distclean` 之后要重新执行上一节的 `./configure`。

### 2. 执行编译

macOS 没有 `nproc`，用 `sysctl` 取核心数。

```bash
make -j"$(sysctl -n hw.ncpu)"
```

依赖库加上 FFmpeg，8 核大约 15–30 分钟。要看完整报错时用 `make -j1`。

### 3. 安装

```bash
make install
```

安装完成后，二进制位于：

```text
~/ffmpeg-custom/bin/ffmpeg
```

---

## 六、验证裁剪结果

```bash
cd "$HOME/ffmpeg-custom/bin"

# 1. 确认是 arm64，且没有链到 Homebrew
file ./ffmpeg
otool -L ./ffmpeg

# 2. 版本和编译配置
./ffmpeg -version

# 3. 封装格式
./ffmpeg -formats

# 4. 网络已禁用（只剩 file、pipe）
./ffmpeg -protocols

# 5. 硬件加速已移除
./ffmpeg -hwaccels

# 6. 文字绘制、字幕烧录和转码滤镜。用字段匹配，避免版本信息里的 libswscale 被 scale 误中
./ffmpeg -filters | awk '$2 ~ /^(drawtext|subtitles|ass|fps|scale|concat)$/'

# 7. 软件编码器。这里的 libvpx-vp9、mov_text 是命令行名字
./ffmpeg -encoders | grep -E "libx265|libsvtav1|libvpx-vp9| aac |mov_text"
./ffmpeg -decoders | grep mov_text
# libx265 应同时接受 8 bit 和 10 bit。没有 yuv420p10le 说明合成库没装上
./ffmpeg -h encoder=libx265 | awk '/Supported pixel formats:/ {print}'

# 8. 硬件编码器应为空
./ffmpeg -encoders | awk '$2 ~ /_(nvenc|amf|qsv|videotoolbox|cuvid)$/'

# 9. 已启用的图片解码/编码应为空。-codecs 会把已关闭的条目也列出来，不能用来判断
./ffmpeg -decoders | awk '$2 ~ /^(png|apng|gif|mjpeg|mjpegb|ljpeg|jpegls|jpeg2000|smvjpeg)$/'
./ffmpeg -encoders | awk '$2 ~ /^(png|apng|gif|mjpeg|mjpegb|ljpeg|jpegls|jpeg2000|smvjpeg)$/'

# 10. PGS、DVD 字幕和蓝光字幕应为空
./ffmpeg -decoders | awk '$2 ~ /^(pgssub|dvdsub|dvbsub|hdmv_pgs_subtitle)$/'
```

`file` 应显示 `Mach-O 64-bit executable arm64`。`otool -L` 里应只有 `/usr/lib/` 和 `/System/Library/` 下的系统库，例如 `libSystem.B.dylib`、`libc++.1.dylib`。出现 `/opt/homebrew` 或 `/usr/local/opt` 说明某个依赖被动态链接了，回到第二节检查 `PKG_CONFIG_LIBDIR`，然后 `make distclean` 再配置。

裁剪结果与 Windows 相同。`-formats` 里除点名的封装外，还会多出只写的 `adts` 和 `latm`。`-hwaccels` 只有标题、没有方法名。滤镜说明和编码器描述会是 `(null)` 或 `(codec ...)`，因为开了 `--enable-small`。编码器 grep 要同时看到 `libsvtav1`、`libx265`、`libvpx-vp9`、`aac`、`mov_text`，解码器里也要有 `mov_text`。`libx265` 的像素格式里要同时有 `yuv420p` 和 `yuv420p10le`。第 8、9、10 条没有输出。

---

## 七、放到本仓库

Tauri 侧车文件名固定为 `ffmpeg-aarch64-apple-darwin`，没有 `.exe` 后缀。

```bash
cd /path/to/precession
cp "$HOME/ffmpeg-custom/bin/ffmpeg" src-tauri/binaries/ffmpeg-aarch64-apple-darwin
chmod +x src-tauri/binaries/ffmpeg-aarch64-apple-darwin
```

产品不再走 GPU 编码，保持 `./configure` 里的 `--disable-videotoolbox`。VideoToolbox 不会编进这版侧车。

---

## 八、补充说明

1. **重新编译**：改配置后先 `make distclean`，再重新 `./configure` 和 `make`。只改依赖库时，同样要重新配置 FFmpeg，否则会继续用旧的检测结果。
2. **链接结果**：`otool -L` 只出现系统库时，二进制就可以拷走。不要添加 `--extra-ldflags="-static"`，macOS 上这会让链接直接失败。
3. **CMake 4**：若 x265 或 SVT-AV1 报 `cmake_minimum_required` 过旧，保留命令里的 `-DCMAKE_POLICY_VERSION_MINIMUM=3.5`。
4. **软件编码器**：与 Windows 相同，包含 `libx265`、`libsvtav1`、`libvpx-vp9` 和内置 AAC。不使用 `--enable-nonfree`。`libx265` 是 8 bit 加 10 bit，不含 12 bit。libvpx 在第二节用 `--disable-vp8` 编译，只提供 VP9。
5. **新终端**：`PREFIX`、`PKG_CONFIG_PATH`、`PKG_CONFIG_LIBDIR` 只在当前 shell 有效。编译中断后重开终端，先执行第一节第 3 步再继续。
