//! 图片转码。编码参数移动端也要能反序列化；桥接和侧车只在桌面编译。

pub(crate) mod params;

#[cfg(desktop)]
mod bridge;
#[cfg(desktop)]
mod executor;
#[cfg(desktop)]
#[allow(dead_code)]
mod utils;

#[cfg(desktop)]
pub(crate) use executor::{run_convert_avif, run_convert_jxl};
