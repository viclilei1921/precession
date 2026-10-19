pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod lunar;
mod repository;
mod service;

use rusqlite::Connection;

use crate::db::error::DbError;

/// 创建日历相关表
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 建表失败时返回 `DbError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// migrate(conn)?;
/// ```
pub(crate) fn migrate(conn: &Connection) -> Result<(), DbError> {
  repository::ensure_schema(conn).map_err(|_| DbError::Internal)
}
