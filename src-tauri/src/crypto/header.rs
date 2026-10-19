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
  /// 核对密钥头的版本和算法字段
  ///
  /// # 返回值
  /// 版本、KDF 算法、Argon2 版本和包装算法都匹配时不返回数据
  ///
  /// # 错误
  /// 任一字段不匹配时返回 `Corrupt`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let header = KeyHeader {
  ///   v: 1,
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
  /// assert!(header.validate().is_ok());
  /// ```
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

  /// 从 JSON 文本解析密钥头并校验字段
  ///
  /// # 参数
  /// - `raw`: 密钥头 JSON
  ///
  /// # 返回值
  /// 通过校验的 `KeyHeader`
  ///
  /// # 错误
  /// JSON 无法解析，或 `validate` 失败时返回 `Corrupt`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let raw = r#"{"v":1,"kdf":{"algo":"argon2id","version":19,"m":8,"t":1,"p":1,"salt":""},"wrap":{"algo":"xchacha20poly1305","nonce":"","ct":""}}"#;
  /// assert!(KeyHeader::from_json(raw).is_ok());
  /// assert!(KeyHeader::from_json("[]").is_err());
  /// ```
  pub fn from_json(raw: &str) -> Result<Self, CryptoError> {
    let header: KeyHeader = serde_json::from_str(raw).map_err(|_| CryptoError::Corrupt)?;
    header.validate()?;
    Ok(header)
  }

  /// 把密钥头格式化成 JSON 字节
  ///
  /// # 返回值
  /// 带缩进的 JSON 字节
  ///
  /// # 错误
  /// 序列化失败时返回 `Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let header = KeyHeader {
  ///   v: 1,
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
  /// let bytes = header.to_pretty_json().unwrap();
  /// assert!(bytes.starts_with(b"{"));
  /// ```
  pub fn to_pretty_json(&self) -> Result<Vec<u8>, CryptoError> {
    serde_json::to_vec_pretty(self).map_err(|_| CryptoError::Internal)
  }
}
