pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod repository;
mod service;

use rusqlite::Connection;

use crate::db::error::DbError;

/// 创建计划相关表
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

/// 查询未删除计划是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键
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

/// 按时间范围列出未删除计划
///
/// # 参数
/// - `conn`: 数据库连接
/// - `from`: 起始时间，空则不限制
/// - `to`: 结束时间，空则不限制
///
/// # 返回值
/// 返回该范围内的计划
///
/// # 错误
/// 查询失败时返回 `PlanError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plans = list_between(conn, from, to)?;
/// ```
pub(crate) fn list_between(
  conn: &Connection,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<dto::Plan>, error::PlanError> {
  repository::list_between(conn, from, to)
}
