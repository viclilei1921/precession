//! 不支持的平台：Android / macOS / 其他

/// 不支持的平台
///
/// 不是 Android、macOS、Windows 时被选为 `Platform` 的实现，方法保持空操作。
///
/// # 示例
/// 见 [`Platform`](super::Platform)
pub(crate) struct Unsupported;

/// 不支持的平台实现
impl super::Platform for Unsupported {}
