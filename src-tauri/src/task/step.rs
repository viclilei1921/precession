//! 一条任务里按顺序执行的步骤。转码、裁剪、合并以后也实现这里，不另开队列。

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use super::error::TaskError;
use crate::crypto::file::{self, FileError};

const CHUNK: usize = 64 * 1024;

pub(super) enum Step {
  Copy { from: PathBuf, to: PathBuf },
  Encrypt { from: PathBuf, to: PathBuf },
  Decrypt { from: PathBuf, to: PathBuf },
}

impl Step {
  pub(super) fn weight(&self) -> f64 {
    1.0
  }

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
    file::encrypt_path(from, to, password, &mut bridge)
  } else {
    file::decrypt_path(from, to, password, &mut bridge)
  };
  result.map_err(map_file)
}

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

pub(super) fn side_path(path: &Path, suffix: &str) -> PathBuf {
  let mut name = path.file_name().unwrap_or_default().to_os_string();
  name.push(".");
  name.push(suffix);
  path.with_file_name(name)
}

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

fn io_fail(err: std::io::Error) -> TaskError {
  tauri_plugin_log::log::error!("task step: {err}");
  TaskError::Internal
}
