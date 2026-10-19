pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod repository;
mod service;

use rusqlite::Connection;

use crate::db::error::DbError;

/// 迁移成长记录表结构
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 迁移完成时返回空成功
///
/// # 错误
/// 模式迁移失败时返回内部错误
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

/// 检查成长记录是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 成长记录标识
///
/// # 返回值
/// 存在时返回 true，不存在时返回 false
///
/// # 错误
/// 查询失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let found = exists(conn, id)?;
/// ```
pub(crate) fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  repository::exists(conn, id)
}

/// 按发生时间范围列出成长记录
///
/// # 参数
/// - `conn`: 数据库连接
/// - `from`: 起始发生时间
/// - `to`: 结束发生时间
///
/// # 返回值
/// 返回该范围内的成长记录
///
/// # 错误
/// 查询失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list_between(conn, from, to)?;
/// ```
pub(crate) fn list_between(
  conn: &Connection,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<dto::GrowthEntry>, error::GrowthError> {
  repository::list_between(conn, from, to)
}
