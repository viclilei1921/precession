use tauri::State;

use super::{dto::DbStatus, error::DbError, state::DbState};

/// 获取数据库状态
#[tauri::command]
pub fn db_status(state: State<'_, DbState>) -> Result<DbStatus, DbError> {
  Ok(state.status())
}

/// 创建数据库
#[tauri::command]
pub async fn db_create(state: State<'_, DbState>, password: String, password_confirm: String) -> Result<(), DbError> {
  if password.is_empty() {
    return Err(DbError::PasswordEmpty);
  }

  // 密码不一致
  if password != password_confirm {
    return Err(DbError::PasswordMismatch);
  }

  state.create(password.as_str())
}

/// 解锁数据库
#[tauri::command]
pub async fn db_unlock(state: State<'_, DbState>, password: String) -> Result<(), DbError> {
  state.unlock(password.as_str())
}

/// 锁定数据库
#[tauri::command]
pub fn db_lock(state: State<'_, DbState>) -> Result<(), DbError> {
  state.lock();
  Ok(())
}
