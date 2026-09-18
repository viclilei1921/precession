use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand::RngCore;
use rand::rngs::OsRng;
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

/// 桌面多给内存和并行；移动端降一档，避免前台被系统杀掉。
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

/// 随机生成一个 DEK
pub fn random_dek() -> Result<Zeroizing<[u8; DEK_LEN]>, CryptoError> {
  random_bytes()
}

/// 用用户密码包装 DEK
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

/// 用用户密码打开 DEK
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

/// 把用户密码，变成一把真正能用来加密的密钥(KEK)
/// 1. 拿上门牌号（密码）
/// 2. 再加一把只有你这份数据才有的随机调料（盐）
/// 3. 用一台又耗内存又耗时间的搅拌机（Argon2id）搅很久
/// 4. 搅出一把真正能开保险箱的金属钥匙（KEK）
/// 5. 这把 KEK 不会拿去直接加密整库数据，只用来包住真正的数据钥匙 DEK。这样以后改密码时，往往只需重新包一层 DEK，而不必把所有数据重新加密
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

/// 用 KEK 把 DEK 包起来
fn wrap_dek(kek: &[u8; KEK_LEN], dek: &[u8; DEK_LEN]) -> Result<([u8; NONCE_LEN], Vec<u8>), CryptoError> {
  // 用 KEK 装好锁
  // ChaCha20 负责把 DEK 打乱成密文
  // Poly1305 负责贴一张「防伪标签」，改一个字节都能发现
  // 前面的 X 表示 nonce 更长（24 字节），随机生成时撞车概率极低
  let cipher = XChaCha20Poly1305::new(Key::from_slice(kek));

  // 随机生成一个 nonce
  // nonce 可以理解成「这次上锁用的批次号」
  // 同一把 KEK、同一段 DEK，如果每次加密都用相同 nonce，密文会暴露规律，这在这类流密码里是严重问题。所以每次包装都从操作系统随机源 OsRng 抽 24 字节新 nonce。
  let mut nonce_bytes = [0u8; NONCE_LEN];
  OsRng.fill_bytes(&mut nonce_bytes);
  let nonce = XNonce::from_slice(&nonce_bytes);

  // 加密 DEK，并绑上身份标签 AAD
  // payload msg: DEK(32字节明文) 真正被加密的内容
  // payload aad: "precession.db.dek.v1" 不加密，但会算进防伪标签
  let ct = cipher.encrypt(nonce, Payload { msg: dek, aad: AAD }).map_err(|_| CryptoError::Internal)?;

  // 返回 nonce 和密文。
  // 密文里面不只是打乱后的 DEK，还附带 Poly1305 校验码，所以会比 32 字节更长
  Ok((nonce_bytes, ct))
}

/// 解包 DEK
/// 用 KEK、当时的 nonce 和密文，把 DEK 从保险箱里拿出来。
fn unwrap_dek(kek: &[u8; KEK_LEN], nonce: &[u8], ct: &[u8]) -> Result<Zeroizing<[u8; DEK_LEN]>, CryptoError> {
  // 用 KEK 装好锁
  let cipher = XChaCha20Poly1305::new(Key::from_slice(kek));

  // nonce 这次不是新生成的，而是读回当时那 24 字节
  let nonce = XNonce::from_slice(nonce);

  // 解密 DEK，并检查防伪标签
  // decrypt 得到的 plain 是一份临时缓冲区，内容和 DEK 相同
  let plain = cipher.decrypt(nonce, Payload { msg: ct, aad: AAD }).map_err(|_| CryptoError::WrongPassword)?;

  // 确认解开的正好是 32 字节 DEK
  let dek: [u8; DEK_LEN] = plain.as_slice().try_into().map_err(|_| CryptoError::Corrupt)?;

  // 把临时明文擦掉，只留下 DEK
  let mut plain = plain;
  plain.zeroize();

  // 返回解开的 DEK(在变量丢弃时再清一次)
  Ok(Zeroizing::new(dek))
}

/// 随机生成字节数组
fn random_bytes<const N: usize>() -> Result<Zeroizing<[u8; N]>, CryptoError> {
  let mut bytes = Zeroizing::new([0u8; N]);
  OsRng.fill_bytes(bytes.as_mut());
  Ok(bytes)
}

/// 解码 Base64 字符串
fn decode_b64(value: &str) -> Result<Vec<u8>, CryptoError> {
  BASE64.decode(value.trim()).map_err(|_| CryptoError::Corrupt)
}
