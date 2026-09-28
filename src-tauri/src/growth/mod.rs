pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod repository;
mod service;

use rusqlite::Connection;

use crate::db::error::DbError;

/// 迁移成长记录表
/// 参数：
/// - conn: 数据库连接
/// 返回：
/// - 是否成功
pub(crate) fn migrate(conn: &Connection) -> Result<(), DbError> {
  repository::ensure_schema(conn).map_err(|_| DbError::Internal)
}

/// 检查成长记录是否存在
/// 参数：
/// - conn: 数据库连接
/// - id: 成长记录ID
/// 返回：
/// - 是否存在
pub(crate) fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  repository::exists(conn, id)
}

/// 获取成长记录列表
/// 参数：
/// - conn: 数据库连接
/// - from: 开始时间
/// - to: 结束时间
/// 返回：
/// - 成长记录列表
pub(crate) fn list_between(
  conn: &Connection,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<dto::GrowthEntry>, error::GrowthError> {
  repository::list_between(conn, from, to)
}
