pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod repository;
mod service;

use rusqlite::Connection;

use crate::db::error::DbError;

/// 迁移书库表结构
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

/// 检查书籍是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书籍标识
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
/// let found = book_exists(conn, id)?;
/// ```
pub(crate) fn book_exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  repository::book_exists(conn, id)
}

/// 检查书摘或笔记是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 书摘或笔记标识
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
/// let found = note_exists(conn, id)?;
/// ```
pub(crate) fn note_exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  repository::note_exists(conn, id)
}

/// 按发生时间范围列出书摘和笔记
///
/// # 参数
/// - `conn`: 数据库连接
/// - `from`: 起始发生时间
/// - `to`: 结束发生时间
///
/// # 返回值
/// 返回该范围内的书摘和笔记
///
/// # 错误
/// 查询失败时返回书库错误
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
) -> Result<Vec<dto::BookNote>, error::LibraryError> {
  repository::list_between(conn, from, to)
}
