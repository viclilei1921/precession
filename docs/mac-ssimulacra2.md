# macOS 编译 ssimulacra2

面向 **Apple Silicon（arm64）**，从 libjxl **v0.12.0** 编出 `ssimulacra2`。这一版和仓库里的 `cjxl-aarch64-apple-darwin` 相同。不编 Intel。

`avifdec` 见 [mac-avifdec.md](./mac-avifdec.md)。侧车只依赖 macOS 系统库。`cjxl` 还会链系统自带的 `libz`，这是允许的。不要用 Homebrew 的 `libhwy.dylib`、brotli 或 `libjpeg.dylib`，换一台机器就无法运行。

Apple 链接器做不出「一个文件、零动态库」的全静态程序。不要加 `-DJPEGXL_STATIC=ON`，它会带上 `-static`，在 macOS 上链接失败。

---

## 一、环境准备

`uname -m` 必须是 `arm64`。若输出 `x86_64`，当前终端跑在 Rosetta 下。

```bash
uname -m
xcode-select -p
```

编译工具：

```bash
brew install cmake ninja pkg-config git
```

确认编译器是 Xcode 的 clang：

```bash
xcrun --find clang
```

源码放在 `~/git/libjxl`。

---

## 二、取源码

主仓库用 Gitee。`deps.sh` 在 git 仓库里只会 `git submodule update`，子模块地址仍指向 GitHub，所以按下面的提交把依赖放进 `third_party/`。这些提交与 libjxl v0.12.0 的 `deps.sh` 一致。

```bash
cd ~/git
git clone -b v0.12.0 --depth 1 https://gitee.com/mirrors_libsdl-org/libjxl.git
cd libjxl

git clone --depth 1 -b 1.2.0 https://gitee.com/mirrors_google/highway.git third_party/highway
git clone --depth 1 -b v1.2.0 https://gitee.com/mirrors/brotli.git third_party/brotli
git clone --depth 1 -b v1.3.1 https://gitee.com/mirrors/zlib.git third_party/zlib
git clone --depth 1 -b v1.6.47 https://gitee.com/mirrors/libpng.git third_party/libpng
git clone --depth 1 -b 2.1.5.1 https://gitee.com/mirrors/libjpeg-turbo.git third_party/libjpeg-turbo

mkdir third_party/skcms
git -C third_party/skcms init
git -C third_party/skcms remote add origin https://gitee.com/mirrors_google/skcms.git
git -C third_party/skcms fetch --depth 1 origin 96d9171c94b937a1b5f0293de7309ac16311b722
git -C third_party/skcms checkout FETCH_HEAD
```

`third_party/` 里如果已经有同名空目录，先删掉再克隆。highway 的标签是 `1.2.0`，没有 `v` 前缀。

目录缺了 `skcms/skcms.h` 时，CMake 会停在 `Please provide skcms`。highway 或 brotli 的源码不在时，CMake 会改用 Homebrew 里的动态库。

---

## 三、静态 libjpeg

libjxl 用 `find_package(JPEG)`。不指定静态库时，它会链上 Homebrew 的 `libjpeg.dylib`。

libjpeg-turbo 2.1.5.1 的 `cmake_minimum_required` 是 2.8.12。CMake 4 需要加上兼容开关。

```bash
cd ~/git/libjxl
cmake -S third_party/libjpeg-turbo -B "$HOME/git/jpeg-static" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$HOME/git/jpeg-static" \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
  -DENABLE_SHARED=OFF -DENABLE_STATIC=ON -DWITH_TURBOJPEG=OFF
cmake --build "$HOME/git/jpeg-static" --target install
```

安装结果里应有 `$HOME/git/jpeg-static/lib/libjpeg.a` 和 `$HOME/git/jpeg-static/include/jpeglib.h`。

---

## 四、配置并编译

曾经配置过、并且日志里出现过 `Found HWY: /opt/homebrew` 时，先删掉 `build`。缓存会记住那次找到的动态库。

`ssimulacra2` 在开发工具里，要打开 `JPEGXL_ENABLE_DEVTOOLS`。`SJPEG` 和 OpenEXR 关掉，避免再去拉依赖或链上 Homebrew。

```bash
cd ~/git/libjxl
rm -rf build
cmake -S . -B build -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_TESTING=OFF \
  -DBUILD_SHARED_LIBS=OFF \
  -DJPEGXL_ENABLE_DEVTOOLS=ON \
  -DJPEGXL_ENABLE_TOOLS=ON \
  -DJPEGXL_ENABLE_SJPEG=OFF \
  -DJPEGXL_ENABLE_OPENEXR=OFF \
  -DCMAKE_DISABLE_FIND_PACKAGE_GIF=ON \
  -DJPEGXL_BUNDLE_LIBPNG=ON \
  -DJPEG_INCLUDE_DIR="$HOME/git/jpeg-static/include" \
  -DJPEG_LIBRARY="$HOME/git/jpeg-static/lib/libjpeg.a"
cmake --build build --target ssimulacra2
```

成品在 `~/git/libjxl/build/tools/ssimulacra2`。

---

## 五、验证

```bash
file ~/git/libjxl/build/tools/ssimulacra2
otool -L ~/git/libjxl/build/tools/ssimulacra2
```

`file` 应显示 `Mach-O 64-bit executable arm64`。`otool -L` 里可以有 `/usr/lib/libz.1.dylib`、`libc++.1.dylib`、`libSystem.B.dylib`。出现 `/opt/homebrew` 或 `/usr/local/opt` 时，确认 `third_party/highway` 和 `third_party/brotli` 里有源码，`JPEG_LIBRARY` 指向 `libjpeg.a`，然后删掉 `build` 再配置。

程序调用是 `ssimulacra2 原图 转码结果`。标准输出一个浮点数。JPEG XL 可以直接作为第二个参数，不必先解成 PNG。

---

## 六、放到本仓库

Tauri 侧车文件名固定为 `ssimulacra2-aarch64-apple-darwin`。

```bash
cd /path/to/precession
cp ~/git/libjxl/build/tools/ssimulacra2 src-tauri/binaries/ssimulacra2-aarch64-apple-darwin
chmod +x src-tauri/binaries/ssimulacra2-aarch64-apple-darwin
```
