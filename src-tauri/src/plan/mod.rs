pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod repository;
mod service;

use rusqlite::Connection;

use crate::db::error::DbError;

pub(crate) fn migrate(conn: &Connection) -> Result<(), DbError> {
  repository::ensure_schema(conn).map_err(|_| DbError::Internal)
}

pub(crate) fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  repository::exists(conn, id)
}

pub(crate) fn list_between(
  conn: &Connection,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<dto::Plan>, error::PlanError> {
  repository::list_between(conn, from, to)
}
