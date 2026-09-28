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
) -> Result<Vec<dto::JournalEntry>, error::JournalError> {
  repository::list_between(conn, from, to)
}

pub(crate) fn clear_note(conn: &Connection, book_note_id: &str) -> Result<(), error::JournalError> {
  repository::clear_note(conn, book_note_id)
}
