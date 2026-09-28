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

pub(crate) fn replace(
  conn: &Connection,
  owner: &str,
  owner_id: &str,
  ids: &[String],
) -> Result<(), error::MemberError> {
  repository::replace(conn, owner, owner_id, ids)
}

pub(crate) fn list_ids(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<String>, error::MemberError> {
  repository::list_ids(conn, owner, owner_id)
}

pub(crate) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), error::MemberError> {
  repository::clear(conn, owner, owner_id)
}
