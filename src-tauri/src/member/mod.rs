pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod repository;
mod service;

use rusqlite::Connection;

use crate::db::error::DbError;

/// 创建成员相关表
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

/// 查询未删除成员是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 成员主键
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

/// 替换某条记录上的成员引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录主键
/// - `ids`: 成员主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 替换失败时返回 `MemberError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// replace(conn, owner, owner_id, ids)?;
/// ```
pub(crate) fn replace(
  conn: &Connection,
  owner: &str,
  owner_id: &str,
  ids: &[String],
) -> Result<(), error::MemberError> {
  repository::replace(conn, owner, owner_id, ids)
}

/// 列出某条记录上的成员主键
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录主键
///
/// # 返回值
/// 返回成员主键列表
///
/// # 错误
/// 查询失败时返回 `MemberError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let ids = list_ids(conn, owner, owner_id)?;
/// ```
pub(crate) fn list_ids(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<String>, error::MemberError> {
  repository::list_ids(conn, owner, owner_id)
}

/// 清除某条记录上的成员引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 清除失败时返回 `MemberError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// clear(conn, owner, owner_id)?;
/// ```
pub(crate) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), error::MemberError> {
  repository::clear(conn, owner, owner_id)
}
