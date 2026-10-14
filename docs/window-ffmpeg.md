# MSYS2 MinGW64 编译 FFmpeg 裁剪指南
面向 **Windows 平台**，使用 MSYS2 MinGW64 环境编译 FFmpeg稳定版，最终生成**完全独立、不依赖任何运行库的 ffmpeg.exe**，并可按需精准裁剪功能模块。

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
  
# 安装 SVT-AV1 系统开发库
pacman -S mingw-w64-x86_64-svt-av1
# H.264 编码
pacman -S mingw-w64-x86_64-x264
# H.265 编码
pacman -S mingw-w64-x86_64-x265
# FDK-AAC 高质量音频编码
pacman -S mingw-w64-x86_64-fdk-aac
# MP3 编码
pacman -S mingw-w64-x86_64-lame
# VP9 编解码
pacman -S mingw-w64-x86_64-libvpx
```

### 3. 字幕与文字绘制依赖安装
如果需要字幕烧录、视频文字绘制功能，必须安装以下依赖库：
```bash
pacman -S --needed \
  mingw-w64-x86_64-libass \
  mingw-w64-x86_64-freetype \
  mingw-w64-x86_64-fribidi \
  mingw-w64-x86_64-harfbuzz \
  mingw-w64-x86_64-expat \
  mingw-w64-x86_64-brotli \
  mingw-w64-x86_64-libpng \
  mingw-w64-x86_64-zlib
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
本项目配置的裁剪目标：
- ✅ 保留所有主流音视频封装格式（MP4/MKV/MOV/TS/FLV/MP3/HLS 等）
- ✅ 保留字幕处理、文字绘制功能
- ✅ 保留所有主流音视频软件编解码
- ❌ 移除蓝光相关功能
- ❌ 移除所有图片格式编解码
- ❌ 移除所有网络协议
- ❌ 移除所有 GPU 硬件加速
- ✅ 最终生成完全静态的独立 exe

### 2. 完整配置命令
在 FFmpeg 源码目录下执行：
```bash
./configure \
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
  --disable-mediafoundation \
  --disable-muxer=bluray,bdpg,image2,image2pipe,gif,png,jpeg2000,bmp,tiff,webp,tga,pgm,ppm,pnm,pcx,sgi,sunrast,xbm,xface,exr \
  --disable-demuxer=bluray,bdpg,bdsup,bdsub,hdmv,hdmv_es,image2,image2pipe,gif,png,jpeg2000,bmp,tiff,webp,tga,pgm,ppm,pnm,pcx,sgi,sunrast,xbm,xface,exr,jp2 \
  --disable-encoder=mjpeg,jpegls,png,png_qsv,png_amf,jpeg2000,bmp,tiff,gif,webp,tga,pgm,ppm,pnm,pcx,sgi,sunrast,xbm,xface,exr \
  --disable-decoder=mjpeg,jpegls,png,jpeg2000,bmp,tiff,gif,webp,tga,pgm,ppm,pnm,pcx,sgi,sunrast,xbm,xface,exr \
  --disable-parser=bdpg,bdsup,bdsub,mjpeg,png,jpeg2000 \
  --enable-libass \
  --enable-libfreetype \
  --enable-libfribidi \
  --enable-libsvtav1 \
  --enable-gpl \
  --enable-static \
  --disable-shared \
  --pkg-config-flags="--static" \
  --extra-ldflags="-static -static-libgcc -static-libstdc++" \
  --extra-libs="-lstdc++ -lpthread" \
  --enable-small \
  --prefix=/usr/local/ffmpeg-custom

```

### 3. 核心配置项说明
| 分类 | 参数 | 作用 |
|------|------|------|
| 程序裁剪 | `--disable-ffplay/ffprobe/doc` | 禁用播放器、分析工具和文档，减少体积和编译时间 |
| 网络 | `--disable-network` | 一键禁用所有网络协议，仅保留本地文件读写 |
| 硬件加速 | `--disable-dxva2/d3d11va/nvenc/...` | 禁用所有 Windows 显卡硬件加速接口 |
| 蓝光移除 | 禁用 bluray 系列 muxer/demuxer/parser | 彻底移除蓝光光盘相关功能 |
| 图片移除 | 禁用所有图片编解码+容器+解析器 | 移除 JPEG/PNG/GIF 等全量图片格式 |
| 字幕文字 | `--enable-libass/libfreetype/libfribidi` | 支持字幕烧录、视频文字绘制 |
| 静态编译 | `--enable-static --disable-shared` | 编译静态库，不生成动态库 |
| 链接优化 | `--pkg-config-flags="--static"` | 让 pkg-config 以静态模式查找依赖 |
| 链接优化 | `--extra-libs="-lstdc++ -lpthread"` | 修复静态链接时 C++ 符号缺失问题 |
| 体积优化 | `--enable-small` | 优化最终二进制体积，牺牲极少量运行速度 |
| 安装路径 | `--prefix=/usr/local/ffmpeg-custom` | 指定最终安装目录 |

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

# 3. 验证网络已禁用（仅保留 file、pipe 等本地协议）
./ffmpeg -protocols

# 4. 验证硬件加速已全部移除
./ffmpeg -hwaccels

# 5. 验证字幕和文字绘制功能正常
./ffmpeg -filters | grep -E "subtitles|ass|drawtext"

# 6. 验证图片、蓝光已彻底移除（无任何输出即为成功）
./ffmpeg -codecs | grep -iE "jpeg|png|gif|bluray"
```

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
3. **功能扩展**：如需添加 x265、fdk-aac 等第三方编码库，先通过 pacman 安装对应开发包，再在 configure 中添加 `--enable-libx265`、`--enable-nonfree --enable-libfdk-aac` 等参数即可。