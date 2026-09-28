//! 标签，以及记录上的标签引用。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{SCHEMA_VERSION, SCHEMA_VERSION_KEY, TAG_REF_TABLE, TAG_TABLE};
use super::dto::Tag;
use super::error::TagError;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> TagError {
  tauri_plugin_log::log::error!("{context}: {err}");
  TagError::Internal
}

fn is_constraint(err: &rusqlite::Error) -> bool {
  matches!(err, rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation)
}

fn map_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
  Ok(Tag { id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)?, updated_at: row.get(3)? })
}

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), TagError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("tag schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS tag (
           id TEXT PRIMARY KEY NOT NULL,
           name TEXT NOT NULL,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE UNIQUE INDEX IF NOT EXISTS tag_name_alive ON tag(name) WHERE deleted_at IS NULL;
         CREATE INDEX IF NOT EXISTS tag_updated_at ON tag(updated_at);
         CREATE TABLE IF NOT EXISTS tag_ref (
           tag_id TEXT NOT NULL,
           owner TEXT NOT NULL,
           owner_id TEXT NOT NULL,
           PRIMARY KEY (tag_id, owner, owner_id)
         );
         CREATE INDEX IF NOT EXISTS tag_ref_owner ON tag_ref(owner, owner_id);",
      )
      .map_err(|e| db_fail("tag migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION).map_err(|e| db_fail("tag schema", e))?;
  }
  Ok(())
}

fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  let found: Option<i64> =
    conn.query_row("SELECT 1 FROM tag WHERE id = ?1 AND deleted_at IS NULL", [id], |row| row.get(0)).optional()?;
  Ok(found.is_some())
}

pub(super) fn list(conn: &Connection) -> Result<Vec<Tag>, TagError> {
  let sql = format!("SELECT id, name, created_at, updated_at FROM {TAG_TABLE} WHERE deleted_at IS NULL ORDER BY name");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("tag list", e))?;
  let rows = stmt.query_map([], map_tag).map_err(|e| db_fail("tag list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("tag list", e))
}

fn get(conn: &Connection, id: &str) -> Result<Tag, TagError> {
  let sql = format!("SELECT id, name, created_at, updated_at FROM {TAG_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  conn.query_row(&sql, [id], map_tag).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => TagError::NotFound,
    other => db_fail("tag get", other),
  })
}

pub(super) fn create(conn: &Connection, name: &str) -> Result<Tag, TagError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!("INSERT INTO {TAG_TABLE} (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)");
  conn.execute(&sql, params![id, name, now]).map_err(|e| {
    if is_constraint(&e) { TagError::NameTaken } else { db_fail("tag create", e) }
  })?;
  get(conn, &id)
}

pub(super) fn update(conn: &Connection, id: &str, name: &str) -> Result<Tag, TagError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {TAG_TABLE} SET name = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![name, now, id]).map_err(|e| {
    if is_constraint(&e) { TagError::NameTaken } else { db_fail("tag update", e) }
  })?;
  if n == 0 {
    return Err(TagError::NotFound);
  }
  get(conn, id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), TagError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {TAG_TABLE} SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![now, id]).map_err(|e| db_fail("tag delete", e))?;
  if n == 0 {
    return Err(TagError::NotFound);
  }
  Ok(())
}

pub(super) fn replace(conn: &Connection, owner: &str, owner_id: &str, ids: &[String]) -> Result<(), TagError> {
  for id in ids {
    if !exists(conn, id).map_err(|e| db_fail("tag ref", e))? {
      return Err(TagError::ReferencedMissing);
    }
  }
  let sql = format!("DELETE FROM {TAG_REF_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("tag ref", e))?;
  let sql = format!("INSERT INTO {TAG_REF_TABLE} (tag_id, owner, owner_id) VALUES (?1, ?2, ?3)");
  for id in ids {
    conn.execute(&sql, params![id, owner, owner_id]).map_err(|e| db_fail("tag ref", e))?;
  }
  Ok(())
}

pub(super) fn list_ids(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<String>, TagError> {
  let sql = format!(
    "SELECT r.tag_id FROM {TAG_REF_TABLE} r
     JOIN {TAG_TABLE} t ON t.id = r.tag_id AND t.deleted_at IS NULL
     WHERE r.owner = ?1 AND r.owner_id = ?2
     ORDER BY r.tag_id"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("tag ref list", e))?;
  let rows = stmt.query_map(params![owner, owner_id], |row| row.get(0)).map_err(|e| db_fail("tag ref list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("tag ref list", e))
}

pub(super) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), TagError> {
  let sql = format!("DELETE FROM {TAG_REF_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("tag ref clear", e))?;
  Ok(())
}
