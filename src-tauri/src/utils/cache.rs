use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::sidecar::{SidecarError, SidecarResult};

pub fn get_cache_dir(app: &AppHandle) -> SidecarResult<PathBuf> {
  let cache_dir = app.path().cache_dir().map_err(|err| SidecarError::io(err.to_string()))?;
  let dir = cache_dir.join("precession");
  if !dir.exists() {
    fs::create_dir_all(&dir).map_err(|err| SidecarError::io(format!("创建缓存目录失败: {err}")))?;
  }
  Ok(dir)
}

pub fn get_cache_temp_dir(app: &AppHandle) -> SidecarResult<PathBuf> {
  let dir = get_cache_dir(app)?.join("temp");
  if !dir.exists() {
    fs::create_dir_all(&dir).map_err(|err| SidecarError::io(format!("创建临时目录失败: {err}")))?;
  }
  Ok(dir)
}

pub fn clear_cache_temp_dir(app: &AppHandle) -> SidecarResult<()> {
  let temp_dir = get_cache_dir(app)?.join("temp");
  if temp_dir.exists() {
    fs::remove_dir_all(&temp_dir).map_err(|err| SidecarError::io(format!("清理临时目录失败: {err}")))?;
  }
  Ok(())
}
