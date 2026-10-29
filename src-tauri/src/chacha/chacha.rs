//! 单件文件加解密。格式与 rigel 的 `encrypt` 一致，和档案 DEK 不是一把密钥。
//!
//! 这是 ChaCha20 密钥流，没有校验和。密码错了不会报错，只会解出另一串字节。

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use rand::TryRng;
use rand::rngs::SysRng;
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"chacha20";
const VERSION: &[u8; 1] = &[0x01];
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const CHUNK: usize = 64 * 1024;
const HEADER_LEN: usize = MAGIC.len() + VERSION.len() + SALT_LEN + NONCE_LEN;

/// rigel 写死的 Argon2id：15 MiB、3 次迭代、并行度 1。不要改成档案库的平台参数。
const KDF_M_KIB: u32 = 15_000;
const KDF_T: u32 = 3;
const KDF_P: u32 = 1;

/// 单件文件加解密失败
///
/// 这是 ChaCha20 密钥流，没有校验和。密码错了不会落在这些变体里，只会解出另一串字节。
///
/// # 变体
/// - `PasswordEmpty`: 密码不能为空
/// - `SamePath`: 输入和输出不能是同一个文件
/// - `BadMagic`: 无效的加密格式
/// - `BadVersion`: 不支持的加密版本
/// - `Truncated`: 密文太短
/// - `Canceled`: 调用方取消
/// - `Internal`: 操作失败
#[derive(Debug, thiserror::Error)]
pub enum FileError {
  /// 密码不能为空
  #[error("密码不能为空")]
  PasswordEmpty,
  /// 输入和输出不能是同一个文件
  #[error("输入和输出不能是同一个文件")]
  SamePath,
  /// 无效的加密格式
  #[error("无效的加密格式")]
  BadMagic,
  /// 不支持的加密版本
  #[error("不支持的加密版本")]
  BadVersion,
  /// 密文太短
  #[error("密文太短")]
  Truncated,
  /// 调用方取消
  #[error("已取消")]
  Canceled,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

/// 看文件开头是不是本格式的密文
///
/// 只读魔数和版本。打不开、太短、魔数或版本不对时返回 `false`。
///
/// # 参数
/// - `path`: 要查看的文件
///
/// # 返回值
/// 开头是 `chacha20` 且版本为 1 时返回 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let sealed = is_sealed(path);
/// ```
pub(crate) fn is_sealed(path: &Path) -> bool {
  let mut file = match File::open(path) {
    Ok(file) => file,
    Err(_) => return false,
  };
  let mut magic = [0u8; MAGIC.len()];
  let mut version = [0u8; VERSION.len()];
  if file.read_exact(&mut magic).is_err() || file.read_exact(&mut version).is_err() {
    return false;
  }
  &magic == MAGIC && version == *VERSION
}

/// 把输入文件加密写到另一个路径
///
/// # 参数
/// - `input`: 明文文件
/// - `output`: 密文输出路径
/// - `password`: 文件密码
/// - `progress`: 按已处理字节和总长度回调；返回错误时中止
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// - 密码为空返回 `PasswordEmpty`
/// - 输入和输出是同一路径返回 `SamePath`
/// - 打开文件或读取长度失败返回 `Internal`
/// - 写出密文失败时返回 `encrypt_reader` 给出的 `FileError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// encrypt_path(input, output, password, &mut |done, total| Ok(()))?;
/// ```
pub(crate) fn encrypt_path(
  input: &Path,
  output: &Path,
  password: &str,
  progress: &mut dyn FnMut(u64, u64) -> Result<(), FileError>,
) -> Result<(), FileError> {
  check_request(input, output, password)?;
  let reader = File::open(input).map_err(io_fail)?;
  let total = reader.metadata().map_err(io_fail)?.len();
  encrypt_reader(reader, output, password, total, progress)
}

/// 把加密文件解密写到另一个路径
///
/// 总长度按文件长度减去文件头计算，不足头长度时按 0。
///
/// # 参数
/// - `input`: 密文文件
/// - `output`: 明文输出路径
/// - `password`: 文件密码
/// - `progress`: 按已处理字节和总长度回调；返回错误时中止
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// - 密码为空返回 `PasswordEmpty`
/// - 输入和输出是同一路径返回 `SamePath`
/// - 打开文件或读取长度失败返回 `Internal`
/// - 解密失败时返回 `write_new_file` 传出的 `FileError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// decrypt_path(input, output, password, &mut |done, total| Ok(()))?;
/// ```
pub(crate) fn decrypt_path(
  input: &Path,
  output: &Path,
  password: &str,
  progress: &mut dyn FnMut(u64, u64) -> Result<(), FileError>,
) -> Result<(), FileError> {
  check_request(input, output, password)?;
  let mut reader = File::open(input).map_err(io_fail)?;
  let file_len = reader.metadata().map_err(io_fail)?.len();
  let total = file_len.saturating_sub(HEADER_LEN as u64);
  write_new_file(output, |writer| decrypt_into(&mut reader, writer, password, total, progress))
}

/// 把字节加密成带 `chacha20` 头的缓冲区
///
/// # 参数
/// - `plaintext`: 明文
/// - `password`: 文件密码
///
/// # 返回值
/// 魔数、版本、盐、nonce 和密钥流处理后的正文
///
/// # 错误
/// 密码为空返回 `PasswordEmpty`；生成盐、nonce 或派生密钥失败返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let sealed = encrypt_bytes(b"hello", "password")?;
/// ```
pub(crate) fn encrypt_bytes(plaintext: &[u8], password: &str) -> Result<Vec<u8>, FileError> {
  if password.is_empty() {
    return Err(FileError::PasswordEmpty);
  }
  let salt = random_bytes::<SALT_LEN>()?;
  let nonce = random_bytes::<NONCE_LEN>()?;
  let key = derive_key(password, &salt)?;
  let mut body = plaintext.to_vec();
  apply_at(&mut body, 0, &key, &nonce);
  let mut out = Vec::with_capacity(HEADER_LEN + body.len());
  out.extend_from_slice(MAGIC);
  out.extend_from_slice(VERSION);
  out.extend_from_slice(&salt);
  out.extend_from_slice(&nonce);
  out.extend_from_slice(&body);
  Ok(out)
}

/// 按文件头解开内存中的密文
///
/// 密码不对时不会返回错误，只会得到另一串字节。
///
/// # 参数
/// - `encrypted`: 带文件头的密文
/// - `password`: 文件密码
///
/// # 返回值
/// 密钥流处理后的正文
///
/// # 错误
/// - 密码为空返回 `PasswordEmpty`
/// - 短于文件头返回 `Truncated`
/// - 魔数不符返回 `BadMagic`
/// - 版本不符返回 `BadVersion`
/// - 盐或 nonce 切片无法转成定长数组时返回 `Truncated`
/// - 派生密钥失败返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(matches!(decrypt_bytes(b"short", "password"), Err(FileError::Truncated)));
/// assert!(matches!(decrypt_bytes(b"short", ""), Err(FileError::PasswordEmpty)));
/// ```
pub(crate) fn decrypt_bytes(encrypted: &[u8], password: &str) -> Result<Vec<u8>, FileError> {
  if password.is_empty() {
    return Err(FileError::PasswordEmpty);
  }
  if encrypted.len() < HEADER_LEN {
    return Err(FileError::Truncated);
  }
  if &encrypted[..MAGIC.len()] != MAGIC {
    return Err(FileError::BadMagic);
  }
  if encrypted[MAGIC.len()] != VERSION[0] {
    return Err(FileError::BadVersion);
  }
  let salt_at = MAGIC.len() + VERSION.len();
  let nonce_at = salt_at + SALT_LEN;
  let salt: [u8; SALT_LEN] = encrypted[salt_at..nonce_at].try_into().map_err(|_| FileError::Truncated)?;
  let nonce: [u8; NONCE_LEN] = encrypted[nonce_at..HEADER_LEN].try_into().map_err(|_| FileError::Truncated)?;
  let key = derive_key(password, &salt)?;
  let mut plain = encrypted[HEADER_LEN..].to_vec();
  apply_at(&mut plain, 0, &key, &nonce);
  Ok(plain)
}

/// 拒绝空密码和同一路径上的输入输出
///
/// # 参数
/// - `input`: 输入路径
/// - `output`: 输出路径
/// - `password`: 文件密码
///
/// # 返回值
/// 通过时不返回数据
///
/// # 错误
/// 密码为空返回 `PasswordEmpty`；两条路径相同返回 `SamePath`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// check_request(input, output, password)?;
/// ```
fn check_request(input: &Path, output: &Path, password: &str) -> Result<(), FileError> {
  if password.is_empty() {
    return Err(FileError::PasswordEmpty);
  }
  if same_path(input, output) {
    return Err(FileError::SamePath);
  }
  Ok(())
}

/// 判断两个路径是不是同一个文件
///
/// 原始路径相等时直接返回 true。两边都能规范化时再比较规范化结果；任一侧失败则返回 false。
///
/// # 参数
/// - `input`: 输入路径
/// - `output`: 输出路径
///
/// # 返回值
/// 视为同一路径时为 true
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let same = same_path(input, output);
/// ```
fn same_path(input: &Path, output: &Path) -> bool {
  if input == output {
    return true;
  }
  match (fs::canonicalize(input), fs::canonicalize(output)) {
    (Ok(left), Ok(right)) => left == right,
    _ => false,
  }
}

/// 生成盐和 nonce，把读入的明文加密写到新文件
///
/// # 参数
/// - `reader`: 明文来源
/// - `output`: 密文输出路径
/// - `password`: 文件密码
/// - `total`: 进度回调使用的总长度
/// - `progress`: 按已处理字节和总长度回调
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 生成盐或 nonce 失败返回 `Internal`；写文件失败时返回 `write_new_file` 传出的 `FileError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// encrypt_reader(reader, output, password, total, progress)?;
/// ```
fn encrypt_reader(
  reader: impl Read,
  output: &Path,
  password: &str,
  total: u64,
  progress: &mut dyn FnMut(u64, u64) -> Result<(), FileError>,
) -> Result<(), FileError> {
  let salt = random_bytes::<SALT_LEN>()?;
  let nonce = random_bytes::<NONCE_LEN>()?;
  write_new_file(output, |writer| {
    writer.write_all(MAGIC).map_err(io_fail)?;
    writer.write_all(VERSION).map_err(io_fail)?;
    writer.write_all(&salt).map_err(io_fail)?;
    writer.write_all(&nonce).map_err(io_fail)?;
    let key = derive_key(password, &salt)?;
    apply_stream(reader, writer, &key, &nonce, total, progress)
  })
}

/// 读文件头后把正文解密写到已打开的文件
///
/// # 参数
/// - `reader`: 已打开的密文
/// - `writer`: 明文输出文件
/// - `password`: 文件密码
/// - `total`: 进度回调使用的总长度
/// - `progress`: 按已处理字节和总长度回调
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 文件头、派生密钥或密钥流处理失败时返回对应的 `FileError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// decrypt_into(&mut reader, writer, password, total, progress)?;
/// ```
fn decrypt_into(
  reader: &mut impl Read,
  writer: &mut File,
  password: &str,
  total: u64,
  progress: &mut dyn FnMut(u64, u64) -> Result<(), FileError>,
) -> Result<(), FileError> {
  let (salt, nonce) = read_header(reader)?;
  let key = derive_key(password, &salt)?;
  apply_stream(reader, writer, &key, &nonce, total, progress)
}

/// 创建输出文件并执行写入；失败时删掉该文件
///
/// # 参数
/// - `output`: 输出路径
/// - `body`: 拿到新文件后要执行的写入
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 创建文件失败返回 `Internal`；`body` 失败时删除输出文件并返回原来的 `FileError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// write_new_file(output, |file| Ok(()))?;
/// ```
fn write_new_file(output: &Path, body: impl FnOnce(&mut File) -> Result<(), FileError>) -> Result<(), FileError> {
  let mut file = File::create(output).map_err(io_fail)?;
  let result = body(&mut file);
  if result.is_err() {
    drop(file);
    let _ = fs::remove_file(output);
  }
  result
}

/// 读取并核对魔数、版本、盐和 nonce
///
/// # 参数
/// - `reader`: 密文读取器
///
/// # 返回值
/// 成功时返回盐和 nonce
///
/// # 错误
/// - 读不满返回 `Truncated` 或其他 `read_exact` 错误
/// - 魔数不符返回 `BadMagic`
/// - 版本不符返回 `BadVersion`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let (salt, nonce) = read_header(&mut reader)?;
/// ```
fn read_header(reader: &mut impl Read) -> Result<([u8; SALT_LEN], [u8; NONCE_LEN]), FileError> {
  let mut magic = [0u8; MAGIC.len()];
  let mut version = [0u8; VERSION.len()];
  let mut salt = [0u8; SALT_LEN];
  let mut nonce = [0u8; NONCE_LEN];
  read_exact(reader, &mut magic)?;
  read_exact(reader, &mut version)?;
  if &magic != MAGIC {
    return Err(FileError::BadMagic);
  }
  if &version != VERSION {
    return Err(FileError::BadVersion);
  }
  read_exact(reader, &mut salt)?;
  read_exact(reader, &mut nonce)?;
  Ok((salt, nonce))
}

/// 读满缓冲区，并把提前结束区分出来
///
/// # 参数
/// - `reader`: 密文读取器
/// - `buf`: 要填满的缓冲区
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// `UnexpectedEof` 返回 `Truncated`；其他读错误返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// read_exact(&mut reader, &mut buf)?;
/// ```
fn read_exact(reader: &mut impl Read, buf: &mut [u8]) -> Result<(), FileError> {
  reader.read_exact(buf).map_err(|err| {
    if err.kind() == io::ErrorKind::UnexpectedEof { FileError::Truncated } else { io_fail(err) }
  })
}

/// 按块把密钥流作用到读入的数据并写出
///
/// 总长度为 0 时仍先回调一次 `(0, 0)`。读到 0 字节后刷出写入端。
///
/// # 参数
/// - `reader`: 数据来源
/// - `writer`: 数据去向
/// - `key`: 32 字节密钥
/// - `nonce`: 12 字节 nonce
/// - `total`: 进度回调使用的总长度
/// - `progress`: 按已处理字节和总长度回调
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 读、写或刷出失败返回 `Internal`；回调返回的 `FileError` 原样传出
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// apply_stream(reader, writer, key, nonce, total, progress)?;
/// ```
fn apply_stream(
  mut reader: impl Read,
  mut writer: impl Write,
  key: &[u8; KEY_LEN],
  nonce: &[u8; NONCE_LEN],
  total: u64,
  progress: &mut dyn FnMut(u64, u64) -> Result<(), FileError>,
) -> Result<(), FileError> {
  let mut buffer = [0u8; CHUNK];
  let mut offset = 0u64;
  if total == 0 {
    progress(0, 0)?;
  }
  loop {
    let n = reader.read(&mut buffer).map_err(io_fail)?;
    if n == 0 {
      break;
    }
    apply_at(&mut buffer[..n], offset, key, nonce);
    writer.write_all(&buffer[..n]).map_err(io_fail)?;
    offset += n as u64;
    progress(offset, total)?;
  }
  writer.flush().map_err(io_fail)?;
  Ok(())
}

/// 从给定偏移把 ChaCha20 密钥流作用到缓冲区
///
/// # 参数
/// - `data`: 就地处理的字节
/// - `offset`: 密钥流起始偏移
/// - `key`: 32 字节密钥
/// - `nonce`: 12 字节 nonce
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut data = b"hello".to_vec();
/// let key = [1u8; 32];
/// let nonce = [2u8; 12];
/// apply_at(&mut data, 0, &key, &nonce);
/// apply_at(&mut data, 0, &key, &nonce);
/// assert_eq!(data, b"hello");
/// ```
fn apply_at(data: &mut [u8], offset: u64, key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN]) {
  let mut cipher = ChaCha20::new(key.into(), nonce.into());
  cipher.seek(offset);
  cipher.apply_keystream(data);
}

/// 从一个加密文件的头和密码派生出的密钥
///
/// 盐和 nonce 每个文件不同，所以只能解开派生它的那个文件。派生要跑一次 Argon2id，查看期间留着它，后续随机读不用再派生。
///
/// # 字段
/// - `key`: 32 字节密钥，丢弃时清零
/// - `nonce`: 文件头里的 12 字节 nonce
///
/// # 示例
/// ```ignore
/// let key = FileKey::read(&mut file, password)?;
/// let reader = CipherReader::with_key(file, key)?;
/// ```
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone)]
pub(crate) struct FileKey {
  /// 32 字节密钥，丢弃时清零
  key: Zeroizing<[u8; KEY_LEN]>,
  /// 文件头里的 12 字节 nonce
  nonce: [u8; NONCE_LEN],
}

#[cfg_attr(not(test), allow(dead_code))]
impl FileKey {
  /// 从文件开头读文件头，再用密码派生密钥
  ///
  /// 密码不对时不会返回错误，之后只会解出另一串字节。读完后读取位置停在正文开头。
  ///
  /// # 参数
  /// - `reader`: 密文，读取位置在文件开头
  /// - `password`: 文件密码
  ///
  /// # 返回值
  /// 这个文件的密钥
  ///
  /// # 错误
  /// - 密码为空返回 `PasswordEmpty`
  /// - 文件头不完整返回 `Truncated`，魔数不符返回 `BadMagic`，版本不符返回 `BadVersion`
  /// - 派生密钥失败返回 `Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let key = FileKey::read(&mut file, password)?;
  /// ```
  pub(crate) fn read(reader: &mut impl Read, password: &str) -> Result<Self, FileError> {
    if password.is_empty() {
      return Err(FileError::PasswordEmpty);
    }
    let (salt, nonce) = read_header(reader)?;
    let key = derive_key(password, &salt)?;
    Ok(Self { key, nonce })
  }
}

/// 边读边解密的密文读取器
///
/// 实现 `Read + Seek`，位置以明文字节计。ChaCha20 可以从任意偏移开始生成密钥流，所以随机读不用解密前面的内容，视频拖动进度条、zip 读目录都靠这一点。没有校验和，密码错了只会读出另一串字节。
///
/// # 字段
/// - `inner`: 密文来源
/// - `key`: 这个文件的密钥
/// - `pos`: 当前明文位置
/// - `len`: 明文总长度，即密文长度减去文件头
///
/// # 示例
/// ```ignore
/// let mut reader = CipherReader::open(File::open(path)?, password)?;
/// reader.seek(SeekFrom::Start(1024))?;
/// ```
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CipherReader<R> {
  /// 密文来源
  inner: R,
  /// 这个文件的密钥
  key: FileKey,
  /// 当前明文位置
  pos: u64,
  /// 明文总长度，即密文长度减去文件头
  len: u64,
}

#[cfg_attr(not(test), allow(dead_code))]
impl<R: Read + Seek> CipherReader<R> {
  /// 读文件头、派生密钥，得到停在明文开头的读取器
  ///
  /// # 参数
  /// - `inner`: 密文来源
  /// - `password`: 文件密码
  ///
  /// # 返回值
  /// 位置为 0 的读取器
  ///
  /// # 错误
  /// 定位失败返回 `Internal`。文件头和派生密钥的错误同 [`FileKey::read`]
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let reader = CipherReader::open(file, password)?;
  /// ```
  pub(crate) fn open(mut inner: R, password: &str) -> Result<Self, FileError> {
    inner.seek(SeekFrom::Start(0)).map_err(io_fail)?;
    let key = FileKey::read(&mut inner, password)?;
    Self::with_key(inner, key)
  }

  /// 用已经派生好的密钥打开读取器，不再跑 Argon2id
  ///
  /// 只核对长度够放下文件头，不重新读盐和 nonce。
  ///
  /// # 参数
  /// - `inner`: 密文来源
  /// - `key`: 从同一个文件派生的密钥
  ///
  /// # 返回值
  /// 位置为 0 的读取器
  ///
  /// # 错误
  /// 定位失败返回 `Internal`；短于文件头返回 `Truncated`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let reader = CipherReader::with_key(file, key.clone())?;
  /// ```
  pub(crate) fn with_key(mut inner: R, key: FileKey) -> Result<Self, FileError> {
    let total = inner.seek(SeekFrom::End(0)).map_err(io_fail)?;
    let len = total.checked_sub(HEADER_LEN as u64).ok_or(FileError::Truncated)?;
    inner.seek(SeekFrom::Start(HEADER_LEN as u64)).map_err(io_fail)?;
    Ok(Self { inner, key, pos: 0, len })
  }

  /// 返回明文总长度
  ///
  /// # 返回值
  /// 密文长度减去文件头
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let size = reader.len();
  /// ```
  pub(crate) fn len(&self) -> u64 {
    self.len
  }
}

impl<R: Read> Read for CipherReader<R> {
  /// 读出密文并按当前明文位置解密
  ///
  /// # 参数
  /// - `buf`: 接收明文的缓冲区
  ///
  /// # 返回值
  /// 读到的字节数，到末尾时为 0
  ///
  /// # 错误
  /// 底层读取失败时原样返回
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let n = reader.read(&mut buf)?;
  /// ```
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let n = self.inner.read(buf)?;
    apply_at(&mut buf[..n], self.pos, &self.key.key, &self.key.nonce);
    self.pos += n as u64;
    Ok(n)
  }
}

impl<R: Seek> Seek for CipherReader<R> {
  /// 按明文位置定位，底层跳过文件头
  ///
  /// 允许定位到末尾之后，之后的读取返回 0。
  ///
  /// # 参数
  /// - `to`: 目标位置，`End` 相对明文末尾
  ///
  /// # 返回值
  /// 新的明文位置
  ///
  /// # 错误
  /// 目标位置为负或溢出时返回 `InvalidInput`；底层定位失败时原样返回
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let pos = reader.seek(SeekFrom::Start(4096))?;
  /// ```
  fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
    let target = match to {
      SeekFrom::Start(at) => Some(at),
      SeekFrom::End(delta) => self.len.checked_add_signed(delta),
      SeekFrom::Current(delta) => self.pos.checked_add_signed(delta),
    };
    let target = target.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek out of range"))?;
    let raw = target
      .checked_add(HEADER_LEN as u64)
      .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek out of range"))?;
    self.inner.seek(SeekFrom::Start(raw))?;
    self.pos = target;
    Ok(target)
  }
}

/// 用固定的 Argon2id 参数从密码和盐派生 32 字节密钥
///
/// # 参数
/// - `password`: 文件密码
/// - `salt`: 盐
///
/// # 返回值
/// 成功时返回会在丢弃时清零的密钥
///
/// # 错误
/// 参数无法构造或哈希失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let key = derive_key("password", b"salt").unwrap();
/// assert_eq!(key.len(), 32);
/// ```
fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>, FileError> {
  let params = Params::new(KDF_M_KIB, KDF_T, KDF_P, Some(KEY_LEN)).map_err(|_| FileError::Internal)?;
  let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
  let mut key = Zeroizing::new([0u8; KEY_LEN]);
  argon2.hash_password_into(password.as_bytes(), salt, key.as_mut()).map_err(|_| FileError::Internal)?;
  Ok(key)
}

/// 从系统随机源生成 `N` 字节
///
/// # 返回值
/// 成功时返回字节数组
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
fn random_bytes<const N: usize>() -> Result<[u8; N], FileError> {
  let mut buf = [0u8; N];
  SysRng.try_fill_bytes(&mut buf).map_err(|_| FileError::Internal)?;
  Ok(buf)
}

/// 记下 I/O 错误并返回 `Internal`
///
/// # 参数
/// - `err`: 标准库 I/O 错误
///
/// # 返回值
/// `FileError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let err = io_fail(io::Error::other("boom"));
/// ```
fn io_fail(err: io::Error) -> FileError {
  tauri_plugin_log::log::error!("file cipher: {err}");
  FileError::Internal
}

#[cfg(test)]
mod tests {
  use super::*;

  struct FailRead;

  impl Read for FailRead {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
      Err(io::Error::other("boom"))
    }
  }

  fn sample(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
  }

  #[test]
  fn roundtrip_across_chunk_boundary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let plain = dir.path().join("plain.bin");
    let enc = dir.path().join("plain.enc");
    let back = dir.path().join("back.bin");
    let body = sample(CHUNK + 100);
    fs::write(&plain, &body).expect("write");

    encrypt_path(&plain, &enc, "file-password", &mut (|_, _| Ok(()))).expect("encrypt");
    decrypt_path(&enc, &back, "file-password", &mut (|_, _| Ok(()))).expect("decrypt");
    assert_eq!(fs::read(&back).expect("read"), body);

    let header = fs::read(&enc).expect("header");
    assert_eq!(&header[..MAGIC.len()], MAGIC);
    assert_eq!(header[MAGIC.len()], VERSION[0]);
  }

  #[test]
  fn rejects_bad_header_and_empty_password() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = dir.path().join("in.bin");
    let output = dir.path().join("out.bin");
    fs::write(&input, b"hello").expect("write");

    assert!(matches!(
      encrypt_path(&input, &output, "", &mut (|_, _| Ok(()))),
      Err(FileError::PasswordEmpty)
    ));
    assert!(!output.exists());
    assert!(matches!(
      decrypt_path(&input, &output, "", &mut (|_, _| Ok(()))),
      Err(FileError::PasswordEmpty)
    ));

    fs::write(&input, b"nope").expect("short");
    assert!(matches!(
      decrypt_path(&input, &output, "file-password", &mut (|_, _| Ok(()))),
      Err(FileError::Truncated)
    ));
    assert!(!output.exists());

    let mut bad_magic = vec![0u8; MAGIC.len() + VERSION.len() + SALT_LEN + NONCE_LEN + 4];
    bad_magic[..8].copy_from_slice(b"notmagic");
    fs::write(&input, &bad_magic).expect("magic");
    assert!(matches!(
      decrypt_path(&input, &output, "file-password", &mut (|_, _| Ok(()))),
      Err(FileError::BadMagic)
    ));

    let mut bad_version = vec![0u8; MAGIC.len() + VERSION.len() + SALT_LEN + NONCE_LEN + 4];
    bad_version[..MAGIC.len()].copy_from_slice(MAGIC);
    bad_version[MAGIC.len()] = 0x02;
    fs::write(&input, &bad_version).expect("version");
    assert!(matches!(
      decrypt_path(&input, &output, "file-password", &mut (|_, _| Ok(()))),
      Err(FileError::BadVersion)
    ));
    assert!(!output.exists());
  }

  #[test]
  fn wrong_password_yields_different_bytes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let plain = dir.path().join("plain.bin");
    let enc = dir.path().join("plain.enc");
    let back = dir.path().join("back.bin");
    fs::write(&plain, b"archive-is-not-this-key").expect("write");

    encrypt_path(&plain, &enc, "right-password", &mut (|_, _| Ok(()))).expect("encrypt");
    decrypt_path(&enc, &back, "wrong-password", &mut (|_, _| Ok(()))).expect("decrypt still runs");
    assert_ne!(fs::read(&back).expect("read"), b"archive-is-not-this-key");
  }

  #[test]
  fn same_path_is_rejected_and_failed_write_leaves_nothing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let plain = dir.path().join("plain.bin");
    fs::write(&plain, b"keep-me").expect("write");
    assert!(matches!(
      encrypt_path(&plain, &plain, "file-password", &mut (|_, _| Ok(()))),
      Err(FileError::SamePath)
    ));
    assert_eq!(fs::read(&plain).expect("unchanged"), b"keep-me");

    let output = dir.path().join("partial.enc");
    assert!(encrypt_reader(FailRead, &output, "file-password", 4, &mut (|_, _| Ok(()))).is_err());
    assert!(!output.exists());
  }

  #[test]
  fn cancel_removes_partial_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    let plain = dir.path().join("plain.bin");
    let enc = dir.path().join("plain.enc");
    fs::write(&plain, sample(CHUNK + 10)).expect("write");
    let mut calls = 0u32;
    let err = encrypt_path(&plain, &enc, "file-password", &mut |_, _| {
      calls += 1;
      Err(FileError::Canceled)
    });
    assert!(matches!(err, Err(FileError::Canceled)));
    assert!(calls >= 1);
    assert!(!enc.exists());
  }

  #[test]
  fn cipher_reader_random_access_matches_plaintext() {
    let dir = tempfile::tempdir().expect("tempdir");
    let plain = dir.path().join("clip.bin");
    let enc = dir.path().join("clip.enc");
    let body = sample(CHUNK * 3 + 777);
    fs::write(&plain, &body).expect("write");
    encrypt_path(&plain, &enc, "video-password", &mut (|_, _| Ok(()))).expect("encrypt");

    let mut reader = CipherReader::open(File::open(&enc).expect("open"), "video-password").expect("reader");
    assert_eq!(reader.len(), body.len() as u64);

    let spans = [(0usize, 16usize), (CHUNK - 5, 10), (CHUNK * 2 + 1, CHUNK + 3), (body.len() - 9, 9), (12_345, 1)];
    for (start, len) in spans {
      reader.seek(SeekFrom::Start(start as u64)).expect("seek");
      let mut buf = vec![0u8; len];
      reader.read_exact(&mut buf).expect("read");
      assert_eq!(buf, &body[start..start + len], "span at {start}");
    }

    let tail = reader.seek(SeekFrom::End(-4)).expect("end");
    assert_eq!(tail, body.len() as u64 - 4);
    let mut rest = Vec::new();
    reader.read_to_end(&mut rest).expect("rest");
    assert_eq!(rest, &body[body.len() - 4..]);
    assert_eq!(reader.read(&mut [0u8; 8]).expect("eof"), 0);

    reader.seek(SeekFrom::Start(100)).expect("back");
    reader.seek(SeekFrom::Current(-40)).expect("current");
    let mut one = [0u8; 1];
    reader.read_exact(&mut one).expect("one");
    assert_eq!(one[0], body[60]);
    assert!(reader.seek(SeekFrom::Current(-1000)).is_err());

    let mut file = File::open(&enc).expect("reopen");
    let key = FileKey::read(&mut file, "video-password").expect("key");
    let mut keyed = CipherReader::with_key(file, key).expect("keyed");
    let mut all = Vec::new();
    keyed.read_to_end(&mut all).expect("all");
    assert_eq!(all, body);
  }

  #[test]
  fn cipher_reader_rejects_bad_input() {
    let dir = tempfile::tempdir().expect("tempdir");
    let short = dir.path().join("short.bin");
    fs::write(&short, b"chacha20").expect("short");
    assert!(matches!(
      CipherReader::open(File::open(&short).expect("open"), "pw"),
      Err(FileError::Truncated)
    ));
    assert!(matches!(
      CipherReader::open(File::open(&short).expect("open"), ""),
      Err(FileError::PasswordEmpty)
    ));

    let enc = dir.path().join("photo.enc");
    fs::write(&enc, encrypt_bytes(b"jpeg-bytes", "right").expect("seal")).expect("enc");
    let mut wrong = CipherReader::open(File::open(&enc).expect("open"), "wrong").expect("still opens");
    let mut out = Vec::new();
    wrong.read_to_end(&mut out).expect("read");
    assert_ne!(out, b"jpeg-bytes");
  }

  #[test]
  fn memory_text_roundtrip() {
    let sealed = encrypt_bytes("今天下雨".as_bytes(), "diary-password").expect("seal");
    let opened = decrypt_bytes(&sealed, "diary-password").expect("open");
    assert_eq!(opened, "今天下雨".as_bytes());
    assert!(matches!(decrypt_bytes(&sealed, ""), Err(FileError::PasswordEmpty)));
  }
}
