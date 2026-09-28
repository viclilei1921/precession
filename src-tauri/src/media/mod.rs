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

pub(crate) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), error::MediaError> {
  repository::clear(conn, owner, owner_id)
}
