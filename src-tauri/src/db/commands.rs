use tauri::State;

use super::{dto::DbStatus, error::DbError, state::DbState};

/// 返回数据库是否存在、是否解锁、是否登记设备槽，以及用户是否锁定过
///
/// # 参数
/// - `state`: 进程内数据库句柄
///
/// # 返回值
/// 成功时返回 `DbStatus`
///
/// # 错误
/// 函数体始终构造 `Ok`，不会返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let status = db_status(state)?;
/// ```
#[tauri::command]
pub fn db_status(state: State<'_, DbState>) -> Result<DbStatus, DbError> {
  Ok(state.status())
}

/// 用密码和确认密码创建数据库
///
/// # 参数
/// - `state`: 进程内数据库句柄
/// - `password`: 档案密码
/// - `password_confirm`: 再输入一次的密码
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// `DbState::create` 返回错误时原样传出该 `DbError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// db_create(state, password, password_confirm).await?;
/// ```
#[tauri::command]
pub async fn db_create(state: State<'_, DbState>, password: String, password_confirm: String) -> Result<(), DbError> {
  state.create(password.as_str(), password_confirm.as_str())
}

/// 用密码解锁数据库
///
/// # 参数
/// - `state`: 进程内数据库句柄
/// - `password`: 档案密码
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// `DbState::unlock` 返回错误时原样传出该 `DbError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// db_unlock(state, password).await?;
/// ```
#[tauri::command]
pub async fn db_unlock(state: State<'_, DbState>, password: String) -> Result<(), DbError> {
  state.unlock(password.as_str())
}

/// 锁定数据库
///
/// 同时清掉预览时留下的文件密钥。
///
/// # 参数
/// - `state`: 进程内数据库句柄
/// - `media`: 播放地址和文件密钥
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// `DbState::lock` 返回错误时原样传出该 `DbError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// db_lock(state, media)?;
/// ```
#[tauri::command]
pub fn db_lock(state: State<'_, DbState>, media: State<'_, crate::upload::media::MediaState>) -> Result<(), DbError> {
  media.clear();
  state.lock()
}

/// 用当前密码把已解开的库登记到这台设备的系统密钥库
///
/// # 参数
/// - `state`: 进程内数据库句柄
/// - `password`: 档案密码
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// `DbState::enable_device` 返回错误时原样传出该 `DbError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// db_enable_device_unlock(state, password).await?;
/// ```
#[tauri::command]
pub async fn db_enable_device_unlock(state: State<'_, DbState>, password: String) -> Result<(), DbError> {
  state.enable_device(password.as_str())
}

/// 用设备槽解锁数据库
///
/// # 参数
/// - `state`: 进程内数据库句柄
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// `DbState::unlock_device` 返回错误时原样传出该 `DbError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// db_unlock_device(state).await?;
/// ```
#[tauri::command]
pub async fn db_unlock_device(state: State<'_, DbState>) -> Result<(), DbError> {
  state.unlock_device()
}

/// 关闭这台设备上的免密解锁
///
/// # 参数
/// - `state`: 进程内数据库句柄
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// `DbState::disable_device` 返回错误时原样传出该 `DbError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// db_disable_device_unlock(state)?;
/// ```
#[tauri::command]
pub fn db_disable_device_unlock(state: State<'_, DbState>) -> Result<(), DbError> {
  state.disable_device()
}
