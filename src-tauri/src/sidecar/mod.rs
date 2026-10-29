//! 侧车进程。桌面才跟踪子进程；其他平台这些入口什么都不做。

#[cfg(desktop)]
pub mod cache;
#[cfg_attr(not(desktop), allow(dead_code))]
mod error;
#[cfg_attr(not(desktop), allow(dead_code))]
mod platform;
#[cfg_attr(not(desktop), allow(dead_code))]
mod slot;

pub use error::{SidecarError, SidecarResult};
pub use slot::{ProcessSlot, clear_current_child, secure_spawn_step};
