//! 单件文件加解密。格式与 rigel 的 `encrypt` 一致，和档案 DEK 不是一把密钥。
//!
//! 这是 ChaCha20 密钥流，没有校验和。密码错了不会报错，只会解出另一串字节。

use std::fs::{self, File};
use std::io::{self, Read, Write};
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

fn check_request(input: &Path, output: &Path, password: &str) -> Result<(), FileError> {
  if password.is_empty() {
    return Err(FileError::PasswordEmpty);
  }
  if same_path(input, output) {
    return Err(FileError::SamePath);
  }
  Ok(())
}

fn same_path(input: &Path, output: &Path) -> bool {
  if input == output {
    return true;
  }
  match (fs::canonicalize(input), fs::canonicalize(output)) {
    (Ok(left), Ok(right)) => left == right,
    _ => false,
  }
}

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

fn write_new_file(output: &Path, body: impl FnOnce(&mut File) -> Result<(), FileError>) -> Result<(), FileError> {
  let mut file = File::create(output).map_err(io_fail)?;
  let result = body(&mut file);
  if result.is_err() {
    drop(file);
    let _ = fs::remove_file(output);
  }
  result
}

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

fn read_exact(reader: &mut impl Read, buf: &mut [u8]) -> Result<(), FileError> {
  reader.read_exact(buf).map_err(|err| {
    if err.kind() == io::ErrorKind::UnexpectedEof { FileError::Truncated } else { io_fail(err) }
  })
}

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

fn apply_at(data: &mut [u8], offset: u64, key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN]) {
  let mut cipher = ChaCha20::new(key.into(), nonce.into());
  cipher.seek(offset);
  cipher.apply_keystream(data);
}

fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; KEY_LEN]>, FileError> {
  let params = Params::new(KDF_M_KIB, KDF_T, KDF_P, Some(KEY_LEN)).map_err(|_| FileError::Internal)?;
  let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
  let mut key = Zeroizing::new([0u8; KEY_LEN]);
  argon2.hash_password_into(password.as_bytes(), salt, key.as_mut()).map_err(|_| FileError::Internal)?;
  Ok(key)
}

fn random_bytes<const N: usize>() -> Result<[u8; N], FileError> {
  let mut buf = [0u8; N];
  SysRng.try_fill_bytes(&mut buf).map_err(|_| FileError::Internal)?;
  Ok(buf)
}

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
  fn memory_text_roundtrip() {
    let sealed = encrypt_bytes("今天下雨".as_bytes(), "diary-password").expect("seal");
    let opened = decrypt_bytes(&sealed, "diary-password").expect("open");
    assert_eq!(opened, "今天下雨".as_bytes());
    assert!(matches!(decrypt_bytes(&sealed, ""), Err(FileError::PasswordEmpty)));
  }
}
