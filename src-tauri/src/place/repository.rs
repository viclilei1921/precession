//! 地点，以及记录上的地点引用。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{PLACE_REF_TABLE, PLACE_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::Place;
use super::error::PlaceError;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> PlaceError {
  tauri_plugin_log::log::error!("{context}: {err}");
  PlaceError::Internal
}

fn map_place(row: &rusqlite::Row<'_>) -> rusqlite::Result<Place> {
  Ok(Place { id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)?, updated_at: row.get(3)? })
}

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), PlaceError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("place schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS place (
           id TEXT PRIMARY KEY NOT NULL,
           name TEXT NOT NULL,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS place_updated_at ON place(updated_at);
         CREATE TABLE IF NOT EXISTS place_ref (
           place_id TEXT NOT NULL,
           owner TEXT NOT NULL,
           owner_id TEXT NOT NULL,
           PRIMARY KEY (place_id, owner, owner_id)
         );
         CREATE INDEX IF NOT EXISTS place_ref_owner ON place_ref(owner, owner_id);",
      )
      .map_err(|e| db_fail("place migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("place schema", e))?;
  }
  Ok(())
}

fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  let found: Option<i64> = conn
    .query_row("SELECT 1 FROM place WHERE id = ?1 AND deleted_at IS NULL", [id], |row| {
      row.get(0)
    })
    .optional()?;
  Ok(found.is_some())
}

pub(super) fn list(conn: &Connection) -> Result<Vec<Place>, PlaceError> {
  let sql =
    format!("SELECT id, name, created_at, updated_at FROM {PLACE_TABLE} WHERE deleted_at IS NULL ORDER BY name");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("place list", e))?;
  let rows = stmt.query_map([], map_place).map_err(|e| db_fail("place list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("place list", e))
}

fn get(conn: &Connection, id: &str) -> Result<Place, PlaceError> {
  let sql = format!("SELECT id, name, created_at, updated_at FROM {PLACE_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  conn.query_row(&sql, [id], map_place).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => PlaceError::NotFound,
    other => db_fail("place get", other),
  })
}

pub(super) fn create(conn: &Connection, name: &str) -> Result<Place, PlaceError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!("INSERT INTO {PLACE_TABLE} (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)");
  conn.execute(&sql, params![id, name, now]).map_err(|e| db_fail("place create", e))?;
  get(conn, &id)
}

pub(super) fn update(conn: &Connection, id: &str, name: &str) -> Result<Place, PlaceError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {PLACE_TABLE} SET name = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![name, now, id]).map_err(|e| db_fail("place update", e))?;
  if n == 0 {
    return Err(PlaceError::NotFound);
  }
  get(conn, id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), PlaceError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {PLACE_TABLE} SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![now, id]).map_err(|e| db_fail("place delete", e))?;
  if n == 0 {
    return Err(PlaceError::NotFound);
  }
  Ok(())
}

pub(super) fn replace(conn: &Connection, owner: &str, owner_id: &str, ids: &[String]) -> Result<(), PlaceError> {
  for id in ids {
    if !exists(conn, id).map_err(|e| db_fail("place ref", e))? {
      return Err(PlaceError::ReferencedMissing);
    }
  }
  let sql = format!("DELETE FROM {PLACE_REF_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("place ref", e))?;
  let sql = format!("INSERT INTO {PLACE_REF_TABLE} (place_id, owner, owner_id) VALUES (?1, ?2, ?3)");
  for id in ids {
    conn.execute(&sql, params![id, owner, owner_id]).map_err(|e| db_fail("place ref", e))?;
  }
  Ok(())
}

pub(super) fn list_ids(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<String>, PlaceError> {
  let sql = format!(
    "SELECT r.place_id FROM {PLACE_REF_TABLE} r
     JOIN {PLACE_TABLE} p ON p.id = r.place_id AND p.deleted_at IS NULL
     WHERE r.owner = ?1 AND r.owner_id = ?2
     ORDER BY r.place_id"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("place ref list", e))?;
  let rows = stmt.query_map(params![owner, owner_id], |row| row.get(0)).map_err(|e| db_fail("place ref list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("place ref list", e))
}

pub(super) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), PlaceError> {
  let sql = format!("DELETE FROM {PLACE_REF_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("place ref clear", e))?;
  Ok(())
}
