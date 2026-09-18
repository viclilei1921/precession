/// 头文件版本
pub const HEADER_VERSION: u32 = 1;

/// 密钥派生算法
pub const KDF_ALGO: &str = "argon2id";

/// 密钥包装算法
pub const WRAP_ALGO: &str = "xchacha20poly1305";

/// Argon2id 版本
pub const ARGON2_VERSION: u32 = 0x13;

/// DEK 长度
pub const DEK_LEN: usize = 32;

/// KEK 长度
pub const KEK_LEN: usize = 32;

/// 盐长度
pub const SALT_LEN: usize = 16;

/// 非对称加密 nonce 长度
pub const NONCE_LEN: usize = 24;

/// 附加数据
pub const AAD: &[u8] = b"precession.db.dek.v1";

/// 最小内存 KiB
pub const MIN_M_KIB: u32 = 8;

/// 最大内存 KiB
pub const MAX_M_KIB: u32 = 1024 * 1024;
