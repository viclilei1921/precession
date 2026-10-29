//! 图片转码。编码参数移动端也要能反序列化；桥接和侧车只在桌面。
//!
//! AVIF 和 JXL 各走自己的编码入口。前端不传参数时，后端 `Default` 就是点默认就导出。
//! AVIF 默认质量 60、透明质量 60、速度 6、8-bit、自动分块。
//! JXL 默认距离 1、effort 7、解码速度 0。
//!
//! 这次不做：把编码器原始参数字符串放进界面。`codec`、`cicp`、`advanced`、`extra_hints` 仍可反序列化。

pub(crate) mod models;

#[cfg(desktop)]
mod avif;
#[cfg(desktop)]
mod bridge;
#[cfg(desktop)]
mod jxl;
#[cfg(desktop)]
mod utils;

#[cfg(desktop)]
pub(crate) use avif::run_convert_avif;
#[cfg(desktop)]
pub(crate) use jxl::run_convert_jxl;
