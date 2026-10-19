pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod repository;
mod service;

use rusqlite::Connection;

use crate::db::error::DbError;

/// 迁移标签表结构
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

/// 替换记录上的标签引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
/// - `ids`: 标签标识列表
///
/// # 返回值
/// 替换完成时返回空成功
///
/// # 错误
/// 替换失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// replace(conn, owner, owner_id, ids)?;
/// ```
pub(crate) fn replace(conn: &Connection, owner: &str, owner_id: &str, ids: &[String]) -> Result<(), error::TagError> {
  repository::replace(conn, owner, owner_id, ids)
}

/// 列出记录上的标签标识
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 返回标签标识
///
/// # 错误
/// 查询失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let ids = list_ids(conn, owner, owner_id)?;
/// ```
pub(crate) fn list_ids(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<String>, error::TagError> {
  repository::list_ids(conn, owner, owner_id)
}

/// 清除记录上的标签引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 清除完成时返回空成功
///
/// # 错误
/// 清除失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// clear(conn, owner, owner_id)?;
/// ```
pub(crate) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), error::TagError> {
  repository::clear(conn, owner, owner_id)
}
