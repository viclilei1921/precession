use crate::{
  crypto::constants::{ARGON2_VERSION, HEADER_VERSION, KDF_ALGO, WRAP_ALGO},
  crypto::error::CryptoError,
};
use serde::{Deserialize, Serialize};

/// 与 `data.sqlite` 成对出现的密钥头。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyHeader {
  /// 版本号
  pub v: u32,
  /// 密钥派生函数头
  pub kdf: KdfHeader,
  /// 包装头
  pub wrap: WrapHeader,
}

/// 密钥派生函数头
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KdfHeader {
  /// 算法
  pub algo: String,
  /// 版本号
  pub version: u32,
  /// 内存 KiB
  pub m: u32,
  /// 迭代次数
  pub t: u32,
  /// 并行度
  pub p: u32,
  /// 盐
  pub salt: String,
}

/// 包装头
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WrapHeader {
  /// 算法
  pub algo: String,
  /// nonce
  pub nonce: String,
  /// 密文
  pub ct: String,
}

impl KeyHeader {
  /// 验证密钥头是否合法
  pub fn validate(&self) -> Result<(), CryptoError> {
    if self.v != HEADER_VERSION {
      return Err(CryptoError::Corrupt);
    }
    if self.kdf.algo != KDF_ALGO || self.kdf.version != ARGON2_VERSION {
      return Err(CryptoError::Corrupt);
    }
    if self.wrap.algo != WRAP_ALGO {
      return Err(CryptoError::Corrupt);
    }
    Ok(())
  }

  /// 从 JSON 字符串解析密钥头
  pub fn from_json(raw: &str) -> Result<Self, CryptoError> {
    let header: KeyHeader = serde_json::from_str(raw).map_err(|_| CryptoError::Corrupt)?;
    header.validate()?;
    Ok(header)
  }

  /// 将密钥头转换为 JSON 字符串
  /// 格式化输出，便于阅读
  pub fn to_pretty_json(&self) -> Result<Vec<u8>, CryptoError> {
    serde_json::to_vec_pretty(self).map_err(|_| CryptoError::Internal)
  }
}
