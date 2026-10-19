//! 不支持的平台：Android / macOS / 其他

/// 不支持的平台
pub(crate) struct Unsupported;

/// 不支持的平台实现
impl super::Platform for Unsupported {}
