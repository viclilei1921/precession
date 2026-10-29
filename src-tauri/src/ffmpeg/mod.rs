//! 视频转码。读信息和真正调用 ffmpeg 只在桌面；时间片段类型移动端也要能反序列化。
//!
//! 转码、裁剪、合并、追加共用一套 `VideoEncode`。前端不传时，后端 `Default` 就是点默认就导出。
//! 侧车能力以 `docs/mac-ffmpeg.md`、`docs/window-ffmpeg.md` 为准：
//! 视频编码器只有 `libsvtav1`、`libx265`、`libvpx-vp9`，纯软件，没有 GPU，也不能编 H.264。
//! 音频只有内置 AAC，只能选轨道、声道、采样率和码率。
//! 可写封装只有 mp4、mov、mpegts。WebM 只有 demuxer，能读不能写。
//! 画面滤镜有 `fps`、`scale`、`setsar`、`pad`、`drawtext`、`format`、`concat`、`subtitles`、`ass`，没有 `crop`，铺满裁切做不到。
//! 音频对齐用 `aresample`、`aformat`。没有 fontconfig，标题用 `fontfile=`，字幕用 `fontsdir=`。
//! 无网络、无图片、无位图字幕。标题和字幕只烧文字，PGS、DVD 字幕不可用。
//!
//! 这次不做：写出 WebM、保留全部音轨、每段各自选不同音轨序号、固定码率、两遍编码、
//! 编码器原始参数字符串、VP9 10-bit、位图字幕、图片水印（还要 `overlay`）。

pub(crate) mod commands;
pub(crate) mod models;

mod platform;

#[cfg(desktop)]
mod append;
#[cfg(desktop)]
mod convert;
#[cfg(desktop)]
mod cut;
#[cfg(desktop)]
mod graph;
#[cfg(desktop)]
mod info;
#[cfg(desktop)]
mod merge;
#[cfg(desktop)]
mod utils;

#[cfg(desktop)]
pub(crate) use append::run_append_smart;
#[cfg(desktop)]
pub(crate) use convert::run_convert;
#[cfg(desktop)]
pub(crate) use cut::run_cut;
#[cfg(desktop)]
pub(crate) use info::video_info;
#[cfg(desktop)]
pub(crate) use merge::run_merge_smart;
