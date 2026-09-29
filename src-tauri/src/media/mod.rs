pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod repository;
mod service;

pub(crate) use constants::{KIND_IMAGE, KIND_VIDEO};

use rusqlite::Connection;

use crate::db::error::DbError;

pub(crate) fn migrate(conn: &Connection) -> Result<(), DbError> {
  repository::ensure_schema(conn).map_err(|_| DbError::Internal)
}

pub(crate) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), error::MediaError> {
  repository::clear(conn, owner, owner_id)
}

pub(crate) fn list_for(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<dto::Media>, error::MediaError> {
  repository::list(conn, owner, owner_id)
}

pub(crate) fn get(conn: &Connection, id: &str) -> Result<dto::Media, error::MediaError> {
  repository::get(conn, id)
}

pub(crate) fn set_encrypted(conn: &Connection, id: &str, encrypted: bool) -> Result<(), error::MediaError> {
  repository::set_encrypted(conn, id, encrypted)
}

pub(crate) fn insert_imported(
  conn: &Connection,
  id: &str,
  input: dto::MediaInput,
  encrypted: bool,
) -> Result<dto::Media, error::MediaError> {
  repository::insert_imported(conn, id, input, encrypted)
}
