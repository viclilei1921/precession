//! 视频转码。读信息和真正调用 ffmpeg 只在桌面；时间片段类型移动端也要能反序列化。

pub(crate) mod models;

#[cfg(desktop)]
mod append;
#[cfg(desktop)]
mod convert;
#[cfg(desktop)]
mod cut;
#[cfg(desktop)]
mod merge;
#[cfg(desktop)]
mod utils;
#[cfg(desktop)]
mod video_info;

#[cfg(desktop)]
pub(crate) use append::run_append_smart;
#[cfg(desktop)]
pub(crate) use convert::run_convert;
#[cfg(desktop)]
pub(crate) use cut::run_cut;
#[cfg(desktop)]
pub(crate) use merge::run_merge_smart;
#[cfg(desktop)]
pub(crate) use video_info::video_info;
