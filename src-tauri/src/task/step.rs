//! 一条任务里按顺序执行的步骤。转码、裁剪、合并以后也实现这里，不另开队列。

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use super::error::TaskError;
use crate::chacha::chacha::{self, FileError};

const CHUNK: usize = 64 * 1024;

/// 一条任务里按顺序执行的步骤
///
/// # 变体
/// - `Copy`: 复制文件，载荷是源路径和目标路径
/// - `Encrypt`: 加密文件，载荷是源路径和目标路径
/// - `Decrypt`: 解密文件，载荷是源路径和目标路径
pub(super) enum Step {
  /// 复制文件，载荷是源路径和目标路径
  Copy { from: PathBuf, to: PathBuf },
  /// 加密文件，载荷是源路径和目标路径
  Encrypt { from: PathBuf, to: PathBuf },
  /// 解密文件，载荷是源路径和目标路径
  Decrypt { from: PathBuf, to: PathBuf },
}

impl Step {
  /// 返回这一步在总进度里的权重
  ///
  /// # 返回值
  /// 固定为 `1.0`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let weight = step.weight();
  /// assert_eq!(weight, 1.0);
  /// ```
  pub(super) fn weight(&self) -> f64 {
    1.0
  }

  /// 按步骤种类复制、加密或解密
  ///
  /// # 参数
  /// - `password`: 加解密口令。复制时不用
  /// - `cancel`: 取消标志
  /// - `on_bytes`: 已处理字节和总字节的回调
  ///
  /// # 返回值
  /// 成功时无额外值
  ///
  /// # 错误
  /// 复制、加密或解密失败时返回 `TaskError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// step.run(password, cancel, &mut on_bytes)?;
  /// ```
  pub(super) fn run(
    &self,
    password: &str,
    cancel: &AtomicBool,
    on_bytes: &mut dyn FnMut(u64, u64) -> Result<(), TaskError>,
  ) -> Result<(), TaskError> {
    match self {
      Self::Copy { from, to } => copy_file(from, to, cancel, on_bytes),
      Self::Encrypt { from, to } => cipher(true, from, to, password, cancel, on_bytes),
      Self::Decrypt { from, to } => cipher(false, from, to, password, cancel, on_bytes),
    }
  }
}

/// 加密或解密整个文件，并把进度回调转回任务错误
///
/// 回调里发现取消时先变成 `FileError::Canceled`，其他任务错误变成 `FileError::Internal`，最后再用 `map_file` 映回 `TaskError`。
///
/// # 参数
/// - `encrypt`: 为真时加密，否则解密
/// - `from`: 源文件
/// - `to`: 目标文件
/// - `password`: 口令
/// - `cancel`: 取消标志
/// - `on_bytes`: 已处理字节和总字节的回调
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 口令为空返回 `PasswordEmpty`，源和目标相同返回 `SamePath`，格式不对返回 `BadFormat`、`BadVersion` 或 `Truncated`，取消返回 `Canceled`，其余返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// cipher(true, from, to, password, cancel, &mut on_bytes)?;
/// ```
fn cipher(
  encrypt: bool,
  from: &Path,
  to: &Path,
  password: &str,
  cancel: &AtomicBool,
  on_bytes: &mut dyn FnMut(u64, u64) -> Result<(), TaskError>,
) -> Result<(), TaskError> {
  let mut bridge = |current: u64, total: u64| {
    if cancel.load(Ordering::Relaxed) {
      return Err(FileError::Canceled);
    }
    on_bytes(current, total).map_err(|err| match err {
      TaskError::Canceled => FileError::Canceled,
      _ => FileError::Internal,
    })
  };
  let result = if encrypt {
    chacha::encrypt_path(from, to, password, &mut bridge)
  } else {
    chacha::decrypt_path(from, to, password, &mut bridge)
  };
  result.map_err(map_file)
}

/// 按块复制文件，并在失败时删掉未写完的目标
///
/// 总长度为 0 时也会回调一次 `(0, 0)`。每次读之前发现取消就返回 `Canceled`。
///
/// # 参数
/// - `from`: 源文件
/// - `to`: 目标文件
/// - `cancel`: 取消标志
/// - `on_bytes`: 已复制字节和总字节的回调
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 取消返回 `TaskError::Canceled`。打开、创建、读写失败，或回调返回其他错误时，返回对应的 `TaskError`；输入输出失败经 `io_fail` 变成 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// copy_file(from, to, cancel, &mut on_bytes)?;
/// ```
fn copy_file(
  from: &Path,
  to: &Path,
  cancel: &AtomicBool,
  on_bytes: &mut dyn FnMut(u64, u64) -> Result<(), TaskError>,
) -> Result<(), TaskError> {
  let mut reader = File::open(from).map_err(io_fail)?;
  let total = reader.metadata().map_err(io_fail)?.len();
  if let Some(parent) = to.parent() {
    fs::create_dir_all(parent).map_err(io_fail)?;
  }
  let mut writer = File::create(to).map_err(io_fail)?;
  let mut buffer = [0u8; CHUNK];
  let mut offset = 0u64;
  let result = (|| {
    if total == 0 {
      on_bytes(0, 0)?;
    }
    loop {
      if cancel.load(Ordering::Relaxed) {
        return Err(TaskError::Canceled);
      }
      let n = reader.read(&mut buffer).map_err(io_fail)?;
      if n == 0 {
        break;
      }
      writer.write_all(&buffer[..n]).map_err(io_fail)?;
      offset += n as u64;
      on_bytes(offset, total)?;
    }
    writer.flush().map_err(io_fail)?;
    Ok(())
  })();
  if result.is_err() {
    drop(writer);
    let _ = fs::remove_file(to);
  }
  result
}

/// 把目标改名为备份，再把临时文件换上去
///
/// 已有同名备份会先删掉。临时文件改名失败时，尝试把备份改回目标。
///
/// # 参数
/// - `temp`: 临时文件
/// - `target`: 要被替换的目标
///
/// # 返回值
/// 成功时返回备份路径
///
/// # 错误
/// 删除旧备份、改名失败时返回 `TaskError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let backup = replace_with_backup(temp, target)?;
/// ```
pub(super) fn replace_with_backup(temp: &Path, target: &Path) -> Result<PathBuf, TaskError> {
  let backup = side_path(target, "bak");
  if backup.exists() {
    fs::remove_file(&backup).map_err(io_fail)?;
  }
  fs::rename(target, &backup).map_err(io_fail)?;
  if let Err(err) = fs::rename(temp, target) {
    let _ = fs::rename(&backup, target);
    return Err(io_fail(err));
  }
  Ok(backup)
}

/// 尽量用备份盖回目标
///
/// 先把目标挪到旁路文件，再把备份改回目标。备份改名失败时把旁路文件改回去。出错只忽略，不返回。
///
/// # 参数
/// - `backup`: 备份文件
/// - `target`: 要恢复的目标
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// restore(backup, target);
/// ```
pub(super) fn restore(backup: &Path, target: &Path) {
  let aside = side_path(target, "aside");
  let _ = fs::remove_file(&aside);
  if fs::rename(target, &aside).is_ok() {
    if fs::rename(backup, target).is_ok() {
      let _ = fs::remove_file(&aside);
    } else {
      let _ = fs::rename(&aside, target);
    }
  }
}

/// 在原文件名后接上 `.` 和后缀，得到同目录旁路路径
///
/// # 参数
/// - `path`: 原路径
/// - `suffix`: 文件名后缀，不含点
///
/// # 返回值
/// 旁路路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let path = side_path(Path::new("media/a"), "part");
/// assert_eq!(path, PathBuf::from("media/a.part"));
/// ```
pub(super) fn side_path(path: &Path, suffix: &str) -> PathBuf {
  let mut name = path.file_name().unwrap_or_default().to_os_string();
  name.push(".");
  name.push(suffix);
  path.with_file_name(name)
}

/// 把文件加解密错误映射成任务错误
///
/// # 参数
/// - `err`: 文件错误
///
/// # 返回值
/// `PasswordEmpty`、`SamePath`、`BadFormat`、`BadVersion`、`Truncated`、`Canceled` 或 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let err = map_file(FileError::Canceled);
/// assert!(matches!(err, TaskError::Canceled));
/// ```
fn map_file(err: FileError) -> TaskError {
  match err {
    FileError::PasswordEmpty => TaskError::PasswordEmpty,
    FileError::SamePath => TaskError::SamePath,
    FileError::BadMagic => TaskError::BadFormat,
    FileError::BadVersion => TaskError::BadVersion,
    FileError::Truncated => TaskError::Truncated,
    FileError::Canceled => TaskError::Canceled,
    FileError::Internal => TaskError::Internal,
  }
}

/// 记下输入输出错误并返回 `Internal`
///
/// # 参数
/// - `err`: 输入输出错误
///
/// # 返回值
/// `TaskError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let err = io_fail(std::io::Error::other("fail"));
/// assert!(matches!(err, TaskError::Internal));
/// ```
fn io_fail(err: std::io::Error) -> TaskError {
  tauri_plugin_log::log::error!("task step: {err}");
  TaskError::Internal
}
