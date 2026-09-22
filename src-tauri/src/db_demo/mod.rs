pub mod commands;
mod constants;
pub mod dto;
pub mod error;
mod repository;
mod service;

use rusqlite::Connection;

use crate::db::error::DbError;

/// 建库/解锁后由 `DbState` 调用。失败则整次建库或解锁失败。
pub(crate) fn migrate(conn: &Connection) -> Result<(), DbError> {
  repository::ensure_schema(conn).map_err(|_| DbError::Internal)
}
