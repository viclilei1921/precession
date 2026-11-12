//! 本地图片查看。列出同一层的图片，改文件名，把 JPEG XL 解成可以显示的 PNG，并在 Windows 上登记打开方式。

mod associate;
pub mod commands;
mod decode;
mod error;
mod folder;
mod open;

pub use open::{PendingOpen, note_open};
