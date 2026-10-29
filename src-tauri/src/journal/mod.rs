pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod repository;
mod service;

use rusqlite::Connection;

use crate::db::error::DbError;

/// 创建手记相关表
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

/// 查询未删除手记是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 手记主键
///
/// # 返回值
/// 存在时返回 true，不存在时返回 false
///
/// # 错误
/// 查询失败时返回 `rusqlite::Error`
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

/// 按发生时间范围列出未删除手记
///
/// # 参数
/// - `conn`: 数据库连接
/// - `from`: 起始时间，空则不限制
/// - `to`: 结束时间，空则不限制
///
/// # 返回值
/// 返回范围内的手记
///
/// # 错误
/// 查询失败时返回 `JournalError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entries = list_between(conn, from, to)?;
/// ```
pub(crate) fn list_between(
  conn: &Connection,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<dto::JournalEntry>, error::JournalError> {
  repository::list_between(conn, from, to)
}

/// 按书摘主键删除手记引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `book_note_id`: 书摘主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 删除失败时返回 `JournalError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// clear_note(conn, book_note_id)?;
/// ```
pub(crate) fn clear_note(conn: &Connection, book_note_id: &str) -> Result<(), error::JournalError> {
  repository::clear_note(conn, book_note_id)
}
