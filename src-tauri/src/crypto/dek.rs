use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand::TryRng;
use rand::rngs::SysRng;
use zeroize::{Zeroize, Zeroizing};

use super::constants::{AAD, DEK_LEN, KEK_LEN, MAX_M_KIB, MIN_M_KIB, NONCE_LEN, SALT_LEN};
use super::constants::{ARGON2_VERSION, HEADER_VERSION, KDF_ALGO, WRAP_ALGO};
use super::error::CryptoError;
use super::header::{KdfHeader, KeyHeader, WrapHeader};

/// Argon2id 成本。m 是内存 KiB，t 是迭代次数，p 是并行度（不要超过物理核数）。
#[derive(Clone, Copy, Debug)]
pub struct KdfParams {
  pub m_kib: u32,
  pub t: u32,
  pub p: u32,
}

/// 返回当前平台的 Argon2id 参数
///
/// Android 和 iOS 使用 32 MiB、3 次迭代、并行度 2。其他平台使用 64 MiB、3 次迭代、并行度 4。
///
/// # 返回值
/// `m_kib`、`t`、`p` 三个成本参数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let params = platform_kdf_params();
/// assert_eq!(params.t, 3);
/// assert!(params.m_kib == 32 * 1024 || params.m_kib == 64 * 1024);
/// ```
pub fn platform_kdf_params() -> KdfParams {
  #[cfg(any(target_os = "android", target_os = "ios"))]
  {
    KdfParams { m_kib: 32 * 1024, t: 3, p: 2 }
  }
  #[cfg(not(any(target_os = "android", target_os = "ios")))]
  {
    KdfParams { m_kib: 64 * 1024, t: 3, p: 4 }
  }
}

/// 生成 32 字节随机 DEK
///
/// # 返回值
/// 成功时返回会在丢弃时清零的 DEK
///
/// # 错误
/// 系统随机源失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let dek = random_dek()?;
/// ```
pub fn random_dek() -> Result<Zeroizing<[u8; DEK_LEN]>, CryptoError> {
  random_bytes()
}

/// 用密码和 KDF 参数把 DEK 包进密钥头
///
/// # 参数
/// - `password`: 用户密码
/// - `dek`: 32 字节 DEK
/// - `kdf`: Argon2id 的内存、迭代和并行度
///
/// # 返回值
/// 带盐、nonce 和密文的 `KeyHeader`
///
/// # 错误
/// 生成盐、派生 KEK 或包装 DEK 失败时返回该 `CryptoError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let header = seal_dek(password, &dek, kdf)?;
/// ```
pub fn seal_dek(password: &str, dek: &[u8; DEK_LEN], kdf: KdfParams) -> Result<KeyHeader, CryptoError> {
  // 生成盐
  let salt = random_bytes::<SALT_LEN>()?;
  // 派生 KEK
  let kek = derive_kek(password, salt.as_ref(), kdf.m_kib, kdf.t, kdf.p)?;
  // 包装 DEK
  let (nonce, ct) = wrap_dek(&kek, dek)?;

  Ok(KeyHeader {
    v: HEADER_VERSION,
    kdf: KdfHeader {
      algo: KDF_ALGO.to_string(),
      version: ARGON2_VERSION,
      m: kdf.m_kib,
      t: kdf.t,
      p: kdf.p,
      salt: BASE64.encode(salt.as_ref()),
    },
    wrap: WrapHeader { algo: WRAP_ALGO.to_string(), nonce: BASE64.encode(nonce), ct: BASE64.encode(ct) },
  })
}

/// 用密码从密钥头解开 DEK
///
/// # 参数
/// - `password`: 用户密码
/// - `header`: 已读入的密钥头
///
/// # 返回值
/// 成功时返回会在丢弃时清零的 DEK
///
/// # 错误
/// - 头字段不合法、Base64 无法解码，或 nonce 不是 24 字节时返回 `Corrupt`
/// - 派生 KEK 失败时返回该 `CryptoError`
/// - 解密或明文长度不对时返回 `unwrap_dek` 给出的 `CryptoError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let header = KeyHeader {
///   v: 2,
///   kdf: KdfHeader {
///     algo: "argon2id".into(),
///     version: 0x13,
///     m: 8,
///     t: 1,
///     p: 1,
///     salt: String::new(),
///   },
///   wrap: WrapHeader {
///     algo: "xchacha20poly1305".into(),
///     nonce: String::new(),
///     ct: String::new(),
///   },
/// };
/// assert!(open_dek("password", &header).is_err());
/// ```
pub fn open_dek(password: &str, header: &KeyHeader) -> Result<Zeroizing<[u8; DEK_LEN]>, CryptoError> {
  header.validate()?;

  let salt = decode_b64(&header.kdf.salt)?;
  let nonce = decode_b64(&header.wrap.nonce)?;
  let ct = decode_b64(&header.wrap.ct)?;

  // 检查 nonce 长度是否正确
  if nonce.len() != NONCE_LEN {
    return Err(CryptoError::Corrupt);
  }

  let kek = derive_kek(password, &salt, header.kdf.m, header.kdf.t, header.kdf.p)?;

  unwrap_dek(&kek, &nonce, &ct)
}

/// 用 Argon2id 从密码和盐派生 32 字节 KEK
///
/// # 参数
/// - `password`: 用户密码
/// - `salt`: 盐
/// - `m_kib`: 内存 KiB
/// - `t`: 迭代次数
/// - `p`: 并行度
///
/// # 返回值
/// 成功时返回会在丢弃时清零的 KEK
///
/// # 错误
/// - `m_kib` 不在允许范围，或 `t`、`p` 为 0 时返回 `Corrupt`
/// - Argon2 参数无法构造时返回 `Corrupt`
/// - 哈希失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let kek = derive_kek("password", b"saltsalt", 8, 1, 1).unwrap();
/// assert_eq!(kek.len(), 32);
/// ```
fn derive_kek(
  password: &str,
  salt: &[u8],
  m_kib: u32,
  t: u32,
  p: u32,
) -> Result<Zeroizing<[u8; KEK_LEN]>, CryptoError> {
  // 先检查参数合不合法
  if !(MIN_M_KIB..=MAX_M_KIB).contains(&m_kib) || t == 0 || p == 0 {
    return Err(CryptoError::Corrupt);
  }

  // 配好 Argon2 这台「慢搅拌机」
  let params = Params::new(m_kib, t, p, Some(KEK_LEN)).map_err(|_| CryptoError::Corrupt)?;
  let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

  // 准备一个 32 字节全 0 的缓冲区，作为 KEK 的容器
  let mut kek = Zeroizing::new([0u8; KEK_LEN]);

  // 调用 Argon2 这台「慢搅拌机」，把用户密码和盐，搅拌成 KEK
  argon2.hash_password_into(password.as_bytes(), salt, kek.as_mut()).map_err(|_| CryptoError::Internal)?;

  // 返回搅拌好的 KEK
  Ok(kek)
}

/// 用 KEK 和随机 nonce 包装 DEK
///
/// # 参数
/// - `kek`: 32 字节 KEK
/// - `dek`: 32 字节 DEK
///
/// # 返回值
/// 成功时返回 24 字节 nonce 和带校验码的密文
///
/// # 错误
/// 生成 nonce 或加密失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let (nonce, ct) = wrap_dek(kek, dek)?;
/// ```
pub(crate) fn wrap_dek(kek: &[u8; KEK_LEN], dek: &[u8; DEK_LEN]) -> Result<([u8; NONCE_LEN], Vec<u8>), CryptoError> {
  // 用 KEK 装好锁
  // ChaCha20 负责把 DEK 打乱成密文
  // Poly1305 负责贴一张「防伪标签」，改一个字节都能发现
  // 前面的 X 表示 nonce 更长（24 字节），随机生成时撞车概率极低
  let cipher = XChaCha20Poly1305::new(&Key::from(*kek));

  // 随机生成一个 nonce
  // nonce 可以理解成「这次上锁用的批次号」
  // 同一把 KEK、同一段 DEK，如果每次加密都用相同 nonce，密文会暴露规律，这在这类流密码里是严重问题。所以每次包装都从操作系统随机源 SysRng 抽 24 字节新 nonce。
  let mut nonce_bytes = [0u8; NONCE_LEN];
  SysRng.try_fill_bytes(&mut nonce_bytes).map_err(|_| CryptoError::Internal)?;
  let nonce = XNonce::from(nonce_bytes);

  // 加密 DEK，并绑上身份标签 AAD
  // payload msg: DEK(32字节明文) 真正被加密的内容
  // payload aad: "precession.db.dek.v1" 不加密，但会算进防伪标签
  let ct = cipher.encrypt(&nonce, Payload { msg: dek, aad: AAD }).map_err(|_| CryptoError::Internal)?;

  // 返回 nonce 和密文。
  // 密文里面不只是打乱后的 DEK，还附带 Poly1305 校验码，所以会比 32 字节更长
  Ok((nonce_bytes, ct))
}

/// 用 KEK、nonce 和密文解开 DEK
///
/// 解开后把临时明文缓冲清零，只把 DEK 放进 `Zeroizing`。
///
/// # 参数
/// - `kek`: 32 字节 KEK
/// - `nonce`: 包装时的 nonce
/// - `ct`: 密文
///
/// # 返回值
/// 成功时返回会在丢弃时清零的 DEK
///
/// # 错误
/// - nonce 不能转成 24 字节时返回 `Corrupt`
/// - 解密失败时返回 `WrongPassword`
/// - 明文不是 32 字节时返回 `Corrupt`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let kek = [7u8; 32];
/// let err = unwrap_dek(&kek, b"short", b"ct").unwrap_err();
/// assert!(matches!(err, CryptoError::Corrupt));
/// ```
pub(crate) fn unwrap_dek(
  kek: &[u8; KEK_LEN],
  nonce: &[u8],
  ct: &[u8],
) -> Result<Zeroizing<[u8; DEK_LEN]>, CryptoError> {
  // 用 KEK 装好锁
  let cipher = XChaCha20Poly1305::new(&Key::from(*kek));

  // nonce 这次不是新生成的，而是读回当时那 24 字节
  let nonce = XNonce::try_from(nonce).map_err(|_| CryptoError::Corrupt)?;

  // 解密 DEK，并检查防伪标签
  // decrypt 得到的 plain 是一份临时缓冲区，内容和 DEK 相同
  let plain = cipher.decrypt(&nonce, Payload { msg: ct, aad: AAD }).map_err(|_| CryptoError::WrongPassword)?;

  // 确认解开的正好是 32 字节 DEK
  let dek: [u8; DEK_LEN] = plain.as_slice().try_into().map_err(|_| CryptoError::Corrupt)?;

  // 把临时明文擦掉，只留下 DEK
  let mut plain = plain;
  plain.zeroize();

  // 返回解开的 DEK(在变量丢弃时再清一次)
  Ok(Zeroizing::new(dek))
}

/// 从系统随机源生成 `N` 字节
///
/// # 返回值
/// 成功时返回会在丢弃时清零的字节数组
///
/// # 错误
/// 随机源失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let salt = random_bytes::<16>()?;
/// ```
fn random_bytes<const N: usize>() -> Result<Zeroizing<[u8; N]>, CryptoError> {
  let mut bytes = Zeroizing::new([0u8; N]);
  SysRng.try_fill_bytes(bytes.as_mut()).map_err(|_| CryptoError::Internal)?;
  Ok(bytes)
}

/// 去掉首尾空白后解码 Base64
///
/// # 参数
/// - `value`: Base64 文本
///
/// # 返回值
/// 解码后的字节
///
/// # 错误
/// 不是合法 Base64 时返回 `Corrupt`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(decode_b64("YQ==").unwrap(), b"a");
/// assert!(decode_b64("???").is_err());
/// ```
fn decode_b64(value: &str) -> Result<Vec<u8>, CryptoError> {
  BASE64.decode(value.trim()).map_err(|_| CryptoError::Corrupt)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_kdf() -> KdfParams {
    KdfParams { m_kib: MIN_M_KIB, t: 1, p: 1 }
  }

  #[test]
  fn seal_then_open_roundtrip() {
    let dek = random_dek().unwrap();
    let header = seal_dek("correct horse", &dek, test_kdf()).unwrap();
    let opened = open_dek("correct horse", &header).unwrap();
    assert_eq!(dek.as_ref(), opened.as_ref());
  }

  #[test]
  fn open_rejects_wrong_password() {
    let dek = random_dek().unwrap();
    let header = seal_dek("correct horse", &dek, test_kdf()).unwrap();
    let err = open_dek("wrong password", &header).unwrap_err();
    assert!(matches!(err, CryptoError::WrongPassword));
  }

  #[test]
  fn unwrap_rejects_wrong_nonce_len() {
    let kek = [7u8; KEK_LEN];
    let dek = [9u8; DEK_LEN];
    let (nonce, ct) = wrap_dek(&kek, &dek).unwrap();
    let err = unwrap_dek(&kek, &nonce[..NONCE_LEN - 1], &ct).unwrap_err();
    assert!(matches!(err, CryptoError::Corrupt));
  }
}
