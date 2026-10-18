# MSYS2 MinGW64 编译 FFmpeg 裁剪指南

面向 **Windows 平台**，使用 MSYS2 MinGW64 环境编译 FFmpeg稳定版，最终生成**完全独立、不依赖任何运行库的 ffmpeg.exe**，并可按需精准裁剪功能模块。

macOS 上的对应步骤见 [mac-ffmpeg.md](./mac-ffmpeg.md)。

---

## 一、环境准备

### 1. 终端选择

必须打开 **MSYS2 MinGW x64** 终端（不要使用默认的 MSYS 终端），编译产物为纯原生 Windows 程序，不依赖 `msys-2.0.dll`。

验证终端环境：

```bash
echo $MSYSTEM
# 正确输出：MINGW64
```

### 2. 系统更新与基础工具安装

```bash
# 先更新系统（首次更新可能需要重启终端再执行一次）
pacman -Syu
pacman -Su

# 安装编译基础工具链 + 汇编优化工具
pacman -S base-devel git \
  mingw-w64-x86_64-toolchain \
  mingw-w64-x86_64-nasm \
  mingw-w64-x86_64-yasm

# AV1 编码
pacman -S mingw-w64-x86_64-svt-av1
# H.265 编码
pacman -S mingw-w64-x86_64-x265
# x265.pc 在静态依赖里写了 -lgcc_s。和 --static-libgcc 带入的 libgcc_eh 会重复定义 _Unwind_Resume
sed -i 's/ -lgcc_s//g' /mingw64/lib/pkgconfig/x265.pc
# VP9 编码。MSYS2 的 libvpx 包里仍带 VP8 代码，FFmpeg 只打开 VP9 编码器
pacman -S mingw-w64-x86_64-libvpx
```

AAC 使用 FFmpeg 内置编码器，不安装 FDK-AAC，也不再使用 `--enable-nonfree`。H.264 只解码，不安装 x264。MP3 只解码，不安装 LAME。

### 3. 文字绘制与字幕烧录依赖安装

`drawtext` 和字幕烧录都通过 `fontfile=` 传入字体文件，不启用 fontconfig。zlib、libpng、brotli 会作为 freetype、harfbuzz 的依赖由 pacman 自动安装。

```bash
pacman -S --needed \
  mingw-w64-x86_64-libass \
  mingw-w64-x86_64-freetype \
  mingw-w64-x86_64-fribidi \
  mingw-w64-x86_64-harfbuzz
```

---



## 二、获取 FFmpeg 源码

官方源下载速度慢，推荐使用国内镜像。

### 1. 国内 Git 镜像（推荐，支持版本切换）

```bash
# Gitee 官方镜像，完整保留分支和标签
git clone [https://gitee.com/mirrors/ffmpeg.git](https://gitee.com/mirrors/ffmpeg.git) ffmpeg
cd ffmpeg

# 切换到 e.g. n9.0.2 稳定版
git checkout n9.0.2
```

---



## 三、编译配置



### 1. 裁剪需求说明

配置从 `--disable-everything` 开始，再按名打开本仓库转码实际用到的组件。范围是现有的转码、裁剪、合并、追加，以及即将添加的单音轨选择、单字幕保留和字幕烧录。

- 视频编码：`libx265`、`libsvtav1`、`libvpx-vp9`
- 音频：源音轨解码后再编成内置 AAC。解码包含 AAC、MP3、Opus、Vorbis，以及成片常见的 AC3、E-AC3、DTS、TrueHD、FLAC、PCM
- 字幕：文本字幕可解码。写入 MP4/MOV 时编成 `mov_text`；烧进画面用滤镜 `subtitles`、`ass`
- 封装写出：MP4、MOV、MPEG-TS
- 封装读入：MOV（同时覆盖 MP4）、WebM、Matroska（MKV 的封装器名是 `matroska`，这版只读不写）、MPEG-TS、concat，以及外挂字幕 SRT、ASS、WebVTT
- 协议：`file`、`pipe`
- 移除 GPU 硬件加速、网络协议、图片格式、蓝光、PGS、DVD 字幕
- 不包含 FLV、HLS、MP3 封装，也不编 H.264、VP8、MP3、FDK-AAC
- 最终生成完全静态的独立 exe



### 2. 完整配置命令

在 FFmpeg 源码目录下执行。`--disable-everything` 必须放在组件开关最前面。升级或重装 x265 后，先重复第一节里改 `x265.pc` 的 `sed`，再 `./configure`，否则 `-lgcc_s` 会重新进链接参数。

```bash
./configure \
  --disable-everything \
  --disable-ffplay \
  --disable-ffprobe \
  --disable-doc \
  --disable-network \
  --disable-dxva2 \
  --disable-d3d11va \
  --disable-d3d12va \
  --disable-nvenc \
  --disable-nvdec \
  --disable-cuvid \
  --disable-amf \
  --disable-ffnvcodec \
  --disable-cuda-llvm \
  --disable-vulkan \
  --disable-vaapi \
  --disable-mediafoundation \
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
  --enable-decoder=h264,hevc,vp8,vp9,av1,aac,opus,vorbis,mp3,ac3,eac3,dca,truehd,flac,pcm_s16le,pcm_s24le,pcm_s32le,ass,ssa,subrip,webvtt,movtext,text \
  --enable-parser=h264,hevc,vp8,vp9,av1,aac,aac_latm,opus,vorbis,mpegaudio,ac3,dca,mlp,flac \
  --enable-bsf=h264_mp4toannexb,hevc_mp4toannexb,aac_adtstoasc,extract_extradata,vp9_superframe,av1_metadata,null \
  --enable-filter=fps,scale,setsar,pad,drawtext,aresample,aformat,concat,format,subtitles,ass \
  --enable-static \
  --disable-shared \
  --pkg-config-flags="--static" \
  --extra-ldflags="-static -static-libgcc -static-libstdc++" \
  --extra-libs="-lharfbuzz -lstdc++ -lpthread" \
  --enable-small \
  --prefix=/usr/local/ffmpeg-custom
```

`./configure` 的组件名来自源码符号，和 `ffmpeg -encoders` 打印出来的名字不一样。VP9 编码器写 `libvpx_vp9`，MP4 文本字幕的编解码器写 `movtext`。编进程序之后，命令行里看到的仍是 `libvpx-vp9` 和 `mov_text`。名字写成打印名时，同一行里的 `libx265`、`libsvtav1`、`aac` 仍能匹配，configure 不会报错，只会把 VP9 和 `movtext` 丢掉；接着 `--enable-libvpx` 会因为没有任何 libvpx 编解码器而失败，提示 `libvpx enabled but no supported decoders found`。

### 3. 核心配置项说明


| 分类    | 参数                                                   | 作用                                                                                                            |
| ----- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| 白名单   | `--disable-everything` 后再 `--enable-*`               | 只编译点名的程序、封装、编解码、滤镜和协议                                                                                         |
| 程序裁剪  | `--disable-ffplay/ffprobe/doc`                       | 禁用播放器、分析工具和文档                                                                                                 |
| 网络    | `--disable-network`                                  | 禁用网络协议，只打开 `file`、`pipe`                                                                                      |
| 硬件加速  | `--disable-dxva2/d3d11va/nvenc/amf/vaapi/...`        | 禁用 Windows 显卡硬件加速，以及 MinGW 会自动打开的 VAAPI。Intel QSV 没有 `--disable-qsv`，只有显式 `--enable-libmfx` 或 `--enable-libvpl` 才会编入，这里两者都不开 |
| 文字与字幕 | `--enable-libfreetype/libharfbuzz/libfribidi/libass` | `drawtext` 需要 freetype 和 harfbuzz。字幕烧录的 `subtitles`、`ass` 需要 libass。字体用 `fontfile=`                           |
| 软件编码  | `--enable-libx265/libsvtav1/libvpx` `--enable-gpl`   | H.265、AV1、VP9。configure 里写 `libvpx_vp9`、`movtext`。AAC 和 `mov_text` 是内置编码器，不需要额外库，也不使用 `--enable-nonfree` |
| 静态编译  | `--enable-static --disable-shared`                   | 编译静态库，不生成动态库                                                                                                  |
| 链接优化  | `--pkg-config-flags="--static"`                      | 让 pkg-config 以静态模式查找依赖                                                                                        |
| 链接优化  | `--extra-libs="-lharfbuzz -lstdc++ -lpthread"`       | 补上 C++ 符号。MSYS2 的 freetype 静态库会调用 `hb_shape`，pkg-config 却把 `-lharfbuzz` 放在 `-lfreetype` 前面，需要在末尾再链一次 harfbuzz |
| 体积优化  | `--enable-small`                                     | 优化最终二进制体积，牺牲极少量运行速度                                                                                           |
| 安装路径  | `--prefix=/usr/local/ffmpeg-custom`                  | 指定最终安装目录                                                                                                      |


---



## 四、编译与安装



### 1. 清理旧缓存（修改配置后必须执行）

```bash
make distclean
```



### 2. 执行编译

```bash
# 并行编译，自动使用全部 CPU 核心
make -j$(nproc)
```

- 编译耗时：主流 8 核 CPU 约 5~10 分钟。
- 如需查看详细错误，执行 `make -j1` 单线程编译。



### 3. 安装到指定目录

```bash
make install
```

安装完成后，最终的 `ffmpeg.exe` 位于：

```
/usr/local/ffmpeg-custom/bin/ffmpeg.exe
```

---



## 五、验证裁剪结果

进入安装目录，执行以下命令确认功能符合预期：

```bash
cd /usr/local/ffmpeg-custom/bin

# 1. 查看版本和编译配置
./ffmpeg -version

# 2. 查看已启用的封装格式
./ffmpeg -formats

# 3. 验证网络已禁用（仅保留 file、pipe）
./ffmpeg -protocols

# 4. 验证硬件加速已全部移除
./ffmpeg -hwaccels

# 5. 文字绘制、字幕烧录和转码滤镜。用字段匹配，避免版本信息里的 libswscale 被 scale 误中
./ffmpeg -filters | awk '$2 ~ /^(drawtext|subtitles|ass|fps|scale|concat)$/'

# 6. 软件编码器。这里的 libvpx-vp9、mov_text 是命令行名字
./ffmpeg -encoders | grep -E "libx265|libsvtav1|libvpx-vp9| aac |mov_text"
./ffmpeg -decoders | grep mov_text
./ffmpeg -decoders | awk '$2 ~ /^(ac3|eac3|dca|truehd|flac|pcm_s16le)$/'

# 7. 硬件编码器应为空
./ffmpeg -encoders | awk '$2 ~ /_(nvenc|amf|qsv|cuvid)$/'

# 8. 图片解码/编码应为空。-codecs 会把已关闭的条目也列出来，不能用来判断
./ffmpeg -decoders | awk '$2 ~ /^(png|apng|gif|mjpeg|mjpegb|ljpeg|jpegls|jpeg2000|smvjpeg)$/'
./ffmpeg -encoders | awk '$2 ~ /^(png|apng|gif|mjpeg|mjpegb|ljpeg|jpegls|jpeg2000|smvjpeg)$/'

# 9. PGS、DVD 字幕和蓝光字幕应为空
./ffmpeg -decoders | awk '$2 ~ /^(pgssub|dvdsub|dvbsub|hdmv_pgs_subtitle)$/'
```

正常结果：

- `-formats`：MP4、MOV 可写，读 MP4 走 `mov` 那一行。MPEG-TS 读写都有。Matroska、WebM、concat、SRT、ASS、WebVTT 只读。还会多出只写的 `adts` 和 `latm`，这是 MPEG-TS 封装 AAC 时自动带上的。
- `-hwaccels`：只有标题 `Hardware acceleration methods:`，下面没有方法名。出现 `vaapi` 说明少了 `--disable-vaapi`。
- `-filters`：要有 `ass`、`drawtext`、`fps`、`scale`、`subtitles`、`concat`。说明文字是 `(null)`，因为开了 `--enable-small`。`drawtext` 前面的 `T` 表示可以按时间轴开关。
- `-encoders`：五行都要有，名字是 `libsvtav1`、`libx265`、`libvpx-vp9`、`aac`、`mov_text`。行尾的 `D` 是直接渲染能力。描述可能是 `(codec ...)` 或 `(null)`。`-decoders` 里也要有 `mov_text`，以及 `ac3`、`eac3`、`dca`、`truehd`、`flac`、`pcm_s16le`。
- 第 7、8、9 条没有输出。

---



## 六、拷贝到 Windows 系统目录



### 1. 命令行拷贝

MSYS2 中 Windows 盘符映射为 `/盘符/` 格式：

```bash
# 示例：拷贝 ffmpeg.exe 到 Windows 桌面
cp /usr/local/ffmpeg-custom/bin/ffmpeg.exe /c/Users/你的Windows用户名/Desktop/

# 示例：拷贝整个目录到 D 盘
cp -r /usr/local/ffmpeg-custom/bin /d/ffmpeg/
```

> 路径包含空格、中文时，需用双引号包裹路径。



### 2. 资源管理器直接访问

MSYS2 默认安装在 `C:\msys64`，直接在 Windows 资源管理器地址栏输入：

```
C:\msys64\usr\local\ffmpeg-custom\bin
```

即可看到编译好的文件，和普通文件夹操作完全一致。

也可以在终端直接打开资源管理器：

```bash
explorer /usr/local/ffmpeg-custom/bin/
```

---



## 七、补充说明

1. **重新编译**：修改配置后必须先执行 `make distclean` 清理旧产物，再重新 configure 和 make。
2. **静态编译优势**：最终生成的 `ffmpeg.exe` 是完全独立的程序，不依赖任何 MinGW DLL，可直接拷贝到任意 Windows 电脑运行。
3. **软件编码器**：H.265（`libx265`）、AV1（`libsvtav1`）、VP9（`libvpx-vp9`）和内置 AAC。不使用 `--enable-nonfree`。macOS 使用同一组编码器，见 [mac-ffmpeg.md](./mac-ffmpeg.md)。macOS 从源码编译 libvpx 时会加上 `--disable-vp8`；Windows 使用 MSYS2 整包，VP8 代码仍在库内，FFmpeg 不打开 VP8 编码器。

