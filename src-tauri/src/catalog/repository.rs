//! 成员、标签、地点。只收已解锁的 `&Connection`。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{MEMBER_TABLE, PLACE_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY, TAG_TABLE};
use super::dto::{Member, MemberWrite, Place, Tag};
use super::error::CatalogError;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> CatalogError {
  tauri_plugin_log::log::error!("{context}: {err}");
  CatalogError::Internal
}

fn is_constraint(err: &rusqlite::Error) -> bool {
  matches!(err, rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation)
}

fn map_member(row: &rusqlite::Row<'_>) -> rusqlite::Result<Member> {
  Ok(Member {
    id: row.get(0)?,
    name: row.get(1)?,
    relation: row.get(2)?,
    gender: row.get(3)?,
    birthday: row.get(4)?,
    created_at: row.get(5)?,
    updated_at: row.get(6)?,
  })
}

fn map_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
  Ok(Tag { id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)?, updated_at: row.get(3)? })
}

fn map_place(row: &rusqlite::Row<'_>) -> rusqlite::Result<Place> {
  Ok(Place { id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)?, updated_at: row.get(3)? })
}

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), CatalogError> {
  let current = schema_version(conn)?;

  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS member (
           id TEXT PRIMARY KEY NOT NULL,
           name TEXT NOT NULL,
           relation TEXT NOT NULL DEFAULT '',
           gender TEXT NOT NULL DEFAULT '',
           birthday INTEGER,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS member_updated_at ON member(updated_at);

         CREATE TABLE IF NOT EXISTS tag (
           id TEXT PRIMARY KEY NOT NULL,
           name TEXT NOT NULL,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE UNIQUE INDEX IF NOT EXISTS tag_name_alive ON tag(name) WHERE deleted_at IS NULL;
         CREATE INDEX IF NOT EXISTS tag_updated_at ON tag(updated_at);

         CREATE TABLE IF NOT EXISTS place (
           id TEXT PRIMARY KEY NOT NULL,
           name TEXT NOT NULL,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS place_updated_at ON place(updated_at);",
      )
      .map_err(|e| db_fail("catalog migrate v1", e))?;
  }

  if current < SCHEMA_VERSION {
    write_schema_version(conn)?;
  }

  Ok(())
}

fn schema_version(conn: &Connection) -> Result<i64, CatalogError> {
  let current: Option<String> = conn
    .query_row("SELECT value FROM db_meta WHERE key = ?1", [SCHEMA_VERSION_KEY], |row| {
      row.get(0)
    })
    .optional()
    .map_err(|e| db_fail("catalog schema_version", e))?;
  Ok(current.and_then(|s| s.parse().ok()).unwrap_or(0))
}

fn write_schema_version(conn: &Connection) -> Result<(), CatalogError> {
  let version = SCHEMA_VERSION.to_string();
  conn
    .execute(
      "INSERT INTO db_meta (key, value) VALUES (?1, ?2)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      params![SCHEMA_VERSION_KEY, version],
    )
    .map_err(|e| db_fail("catalog write schema_version", e))?;
  Ok(())
}

pub(super) fn member_list(conn: &Connection) -> Result<Vec<Member>, CatalogError> {
  let sql = format!(
    "SELECT id, name, relation, gender, birthday, created_at, updated_at
     FROM {MEMBER_TABLE} WHERE deleted_at IS NULL ORDER BY updated_at DESC"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("member list prepare", e))?;
  let rows = stmt.query_map([], map_member).map_err(|e| db_fail("member list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("member list collect", e))
}

pub(super) fn member_get(conn: &Connection, id: &str) -> Result<Member, CatalogError> {
  let sql = format!(
    "SELECT id, name, relation, gender, birthday, created_at, updated_at
     FROM {MEMBER_TABLE} WHERE id = ?1 AND deleted_at IS NULL"
  );
  conn.query_row(&sql, [id], map_member).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => CatalogError::RecordNotFound,
    other => db_fail("member get", other),
  })
}

pub(super) fn member_create(conn: &Connection, input: &MemberWrite) -> Result<Member, CatalogError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!(
    "INSERT INTO {MEMBER_TABLE} (id, name, relation, gender, birthday, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)"
  );
  conn
    .execute(&sql, params![id, input.name, input.relation, input.gender, input.birthday, now])
    .map_err(|e| db_fail("member create", e))?;
  member_get(conn, &id)
}

pub(super) fn member_update(conn: &Connection, id: &str, input: &MemberWrite) -> Result<Member, CatalogError> {
  let now = now_unix_ms();
  let sql = format!(
    "UPDATE {MEMBER_TABLE}
     SET name = ?1, relation = ?2, gender = ?3, birthday = ?4, updated_at = ?5
     WHERE id = ?6 AND deleted_at IS NULL"
  );
  let n = conn
    .execute(&sql, params![input.name, input.relation, input.gender, input.birthday, now, id])
    .map_err(|e| db_fail("member update", e))?;
  if n == 0 {
    return Err(CatalogError::RecordNotFound);
  }
  member_get(conn, id)
}

pub(super) fn member_delete(conn: &Connection, id: &str) -> Result<(), CatalogError> {
  soft_delete(conn, MEMBER_TABLE, id, "member delete")
}

pub(super) fn tag_list(conn: &Connection) -> Result<Vec<Tag>, CatalogError> {
  let sql = format!("SELECT id, name, created_at, updated_at FROM {TAG_TABLE} WHERE deleted_at IS NULL ORDER BY name");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("tag list prepare", e))?;
  let rows = stmt.query_map([], map_tag).map_err(|e| db_fail("tag list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("tag list collect", e))
}

pub(super) fn tag_create(conn: &Connection, name: &str) -> Result<Tag, CatalogError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!("INSERT INTO {TAG_TABLE} (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)");
  conn.execute(&sql, params![id, name, now]).map_err(|e| {
    if is_constraint(&e) { CatalogError::NameTaken } else { db_fail("tag create", e) }
  })?;
  tag_get(conn, &id)
}

pub(super) fn tag_update(conn: &Connection, id: &str, name: &str) -> Result<Tag, CatalogError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {TAG_TABLE} SET name = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![name, now, id]).map_err(|e| {
    if is_constraint(&e) { CatalogError::NameTaken } else { db_fail("tag update", e) }
  })?;
  if n == 0 {
    return Err(CatalogError::RecordNotFound);
  }
  tag_get(conn, id)
}

pub(super) fn tag_delete(conn: &Connection, id: &str) -> Result<(), CatalogError> {
  soft_delete(conn, TAG_TABLE, id, "tag delete")
}

pub(super) fn place_list(conn: &Connection) -> Result<Vec<Place>, CatalogError> {
  let sql =
    format!("SELECT id, name, created_at, updated_at FROM {PLACE_TABLE} WHERE deleted_at IS NULL ORDER BY name");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("place list prepare", e))?;
  let rows = stmt.query_map([], map_place).map_err(|e| db_fail("place list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("place list collect", e))
}

pub(super) fn place_create(conn: &Connection, name: &str) -> Result<Place, CatalogError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!("INSERT INTO {PLACE_TABLE} (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)");
  conn.execute(&sql, params![id, name, now]).map_err(|e| db_fail("place create", e))?;
  place_get(conn, &id)
}

pub(super) fn place_update(conn: &Connection, id: &str, name: &str) -> Result<Place, CatalogError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {PLACE_TABLE} SET name = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![name, now, id]).map_err(|e| db_fail("place update", e))?;
  if n == 0 {
    return Err(CatalogError::RecordNotFound);
  }
  place_get(conn, id)
}

pub(super) fn place_delete(conn: &Connection, id: &str) -> Result<(), CatalogError> {
  soft_delete(conn, PLACE_TABLE, id, "place delete")
}

fn tag_get(conn: &Connection, id: &str) -> Result<Tag, CatalogError> {
  let sql = format!("SELECT id, name, created_at, updated_at FROM {TAG_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  conn.query_row(&sql, [id], map_tag).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => CatalogError::RecordNotFound,
    other => db_fail("tag get", other),
  })
}

fn place_get(conn: &Connection, id: &str) -> Result<Place, CatalogError> {
  let sql = format!("SELECT id, name, created_at, updated_at FROM {PLACE_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  conn.query_row(&sql, [id], map_place).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => CatalogError::RecordNotFound,
    other => db_fail("place get", other),
  })
}

fn soft_delete(conn: &Connection, table: &str, id: &str, context: &str) -> Result<(), CatalogError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {table} SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![now, id]).map_err(|e| db_fail(context, e))?;
  if n == 0 {
    return Err(CatalogError::RecordNotFound);
  }
  Ok(())
}
