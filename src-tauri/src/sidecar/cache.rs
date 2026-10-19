use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use super::{SidecarError, SidecarResult};

/// 取应用缓存目录下的 `precession`，不存在就创建
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 成功时返回缓存目录
///
/// # 错误
/// 读不到系统缓存目录，或创建目录失败时，返回 `SidecarError::Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let dir = get_cache_dir(&app)?;
/// ```
pub fn get_cache_dir(app: &AppHandle) -> SidecarResult<PathBuf> {
  let cache_dir = app.path().cache_dir().map_err(|err| SidecarError::io(err.to_string()))?;
  let dir = cache_dir.join("precession");
  if !dir.exists() {
    fs::create_dir_all(&dir).map_err(|err| SidecarError::io(format!("创建缓存目录失败: {err}")))?;
  }
  Ok(dir)
}

/// 取缓存目录下的 `temp`，不存在就创建
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 成功时返回临时目录
///
/// # 错误
/// 缓存目录或临时目录创建失败时返回 `SidecarError::Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let dir = get_cache_temp_dir(&app)?;
/// ```
pub fn get_cache_temp_dir(app: &AppHandle) -> SidecarResult<PathBuf> {
  let dir = get_cache_dir(app)?.join("temp");
  if !dir.exists() {
    fs::create_dir_all(&dir).map_err(|err| SidecarError::io(format!("创建临时目录失败: {err}")))?;
  }
  Ok(dir)
}

/// 删掉缓存目录下的整个 `temp`
///
/// 目录不存在时直接成功。
///
/// # 参数
/// - `app`: Tauri 应用句柄
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 读缓存目录失败或删除失败时返回 `SidecarError::Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// clear_cache_temp_dir(&app)?;
/// ```
pub fn clear_cache_temp_dir(app: &AppHandle) -> SidecarResult<()> {
  let temp_dir = get_cache_dir(app)?.join("temp");
  if temp_dir.exists() {
    fs::remove_dir_all(&temp_dir).map_err(|err| SidecarError::io(format!("清理临时目录失败: {err}")))?;
  }
  Ok(())
}
