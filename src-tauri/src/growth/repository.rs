//! 成长记录。主体成员写在本表，其他成员走 member_ref。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{ENTRY_TABLE, KIND_MILESTONE, KIND_MOMENT, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::{GrowthEntry, GrowthEntryInput};
use super::error::GrowthError;
use crate::owner::Owner;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> GrowthError {
  tauri_plugin_log::log::error!("{context}: {err}");
  GrowthError::Internal
}

const COLUMNS: &str = "id, kind, member_id, occurred_at, title, body, locked, highlight, created_at, updated_at";

fn map_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<GrowthEntry> {
  Ok(GrowthEntry {
    id: row.get(0)?,
    kind: row.get(1)?,
    member_id: row.get(2)?,
    occurred_at: row.get(3)?,
    title: row.get(4)?,
    body: row.get(5)?,
    locked: row.get(6)?,
    highlight: row.get(7)?,
    member_ids: Vec::new(),
    tag_ids: Vec::new(),
    place_ids: Vec::new(),
    created_at: row.get(8)?,
    updated_at: row.get(9)?,
  })
}

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), GrowthError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("growth schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS growth_entry (
           id TEXT PRIMARY KEY NOT NULL,
           kind TEXT NOT NULL,
           member_id TEXT NOT NULL,
           occurred_at INTEGER NOT NULL,
           title TEXT NOT NULL,
           body TEXT NOT NULL DEFAULT '',
           locked INTEGER NOT NULL DEFAULT 0,
           highlight INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS growth_entry_member ON growth_entry(member_id, occurred_at);",
      )
      .map_err(|e| db_fail("growth migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("growth schema", e))?;
  }
  Ok(())
}

pub(super) fn allowed_kind(kind: &str) -> bool {
  kind == KIND_MILESTONE || kind == KIND_MOMENT
}

pub(super) fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  let found: Option<i64> = conn
    .query_row("SELECT 1 FROM growth_entry WHERE id = ?1 AND deleted_at IS NULL", [id], |row| {
      row.get(0)
    })
    .optional()?;
  Ok(found.is_some())
}

pub(super) fn list(
  conn: &Connection,
  member_id: Option<&str>,
  kind: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<GrowthEntry>, GrowthError> {
  query(conn, None, member_id, kind, from, to)
}

pub(super) fn list_between(
  conn: &Connection,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<GrowthEntry>, GrowthError> {
  query(conn, None, None, None, from, to)
}

pub(super) fn get(conn: &Connection, id: &str) -> Result<GrowthEntry, GrowthError> {
  let mut items = query(conn, Some(id), None, None, None, None)?;
  items.pop().ok_or(GrowthError::NotFound)
}

pub(super) fn create(conn: &Connection, input: &GrowthEntryInput) -> Result<GrowthEntry, GrowthError> {
  require_member(conn, &input.member_id)?;
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("growth create tx", e))?;
  tx.execute(
    "INSERT INTO growth_entry
       (id, kind, member_id, occurred_at, title, body, locked, highlight, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
    params![
      id,
      input.kind,
      input.member_id,
      input.occurred_at,
      input.title,
      input.body,
      input.locked,
      input.highlight,
      now
    ],
  )
  .map_err(|e| db_fail("growth create", e))?;
  replace_refs(&tx, &id, input)?;
  tx.commit().map_err(|e| db_fail("growth create commit", e))?;
  get(conn, &id)
}

pub(super) fn update(conn: &Connection, id: &str, input: &GrowthEntryInput) -> Result<GrowthEntry, GrowthError> {
  require_member(conn, &input.member_id)?;
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("growth update tx", e))?;
  let n = tx
    .execute(
      "UPDATE growth_entry
       SET kind = ?1, member_id = ?2, occurred_at = ?3, title = ?4, body = ?5, locked = ?6, highlight = ?7, updated_at = ?8
       WHERE id = ?9 AND deleted_at IS NULL",
      params![
        input.kind,
        input.member_id,
        input.occurred_at,
        input.title,
        input.body,
        input.locked,
        input.highlight,
        now,
        id
      ],
    )
    .map_err(|e| db_fail("growth update", e))?;
  if n == 0 {
    return Err(GrowthError::NotFound);
  }
  replace_refs(&tx, id, input)?;
  tx.commit().map_err(|e| db_fail("growth update commit", e))?;
  get(conn, id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), GrowthError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("growth delete tx", e))?;
  let n = tx
    .execute(
      "UPDATE growth_entry SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
      params![now, id],
    )
    .map_err(|e| db_fail("growth delete", e))?;
  if n == 0 {
    return Err(GrowthError::NotFound);
  }
  clear_refs(&tx, id)?;
  tx.commit().map_err(|e| db_fail("growth delete commit", e))?;
  Ok(())
}

fn query(
  conn: &Connection,
  id: Option<&str>,
  member_id: Option<&str>,
  kind: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<GrowthEntry>, GrowthError> {
  let sql = format!(
    "SELECT {COLUMNS} FROM {ENTRY_TABLE}
     WHERE deleted_at IS NULL
       AND (?1 IS NULL OR id = ?1)
       AND (?2 IS NULL OR member_id = ?2)
       AND (?3 IS NULL OR kind = ?3)
       AND (?4 IS NULL OR occurred_at >= ?4)
       AND (?5 IS NULL OR occurred_at < ?5)
     ORDER BY occurred_at DESC"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("growth list", e))?;
  let rows =
    stmt.query_map(params![id, member_id, kind, from, to], map_entry).map_err(|e| db_fail("growth list", e))?;
  let mut items = rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("growth list", e))?;
  for item in &mut items {
    let owner = Owner::GrowthEntry.as_str();
    item.member_ids = crate::member::list_ids(conn, owner, &item.id).map_err(|_| GrowthError::Internal)?;
    item.tag_ids = crate::tag::list_ids(conn, owner, &item.id).map_err(|_| GrowthError::Internal)?;
    item.place_ids = crate::place::list_ids(conn, owner, &item.id).map_err(|_| GrowthError::Internal)?;
  }
  Ok(items)
}

fn require_member(conn: &Connection, id: &str) -> Result<(), GrowthError> {
  let alive = crate::member::exists(conn, id).map_err(|e| db_fail("growth member", e))?;
  if alive { Ok(()) } else { Err(GrowthError::ReferencedMissing) }
}

fn replace_refs(conn: &Connection, id: &str, input: &GrowthEntryInput) -> Result<(), GrowthError> {
  let owner = Owner::GrowthEntry.as_str();
  crate::member::replace(conn, owner, id, &input.member_ids).map_err(ref_err)?;
  crate::tag::replace(conn, owner, id, &input.tag_ids).map_err(ref_err)?;
  crate::place::replace(conn, owner, id, &input.place_ids).map_err(ref_err)?;
  Ok(())
}

fn clear_refs(conn: &Connection, id: &str) -> Result<(), GrowthError> {
  let owner = Owner::GrowthEntry.as_str();
  crate::member::clear(conn, owner, id).map_err(|_| GrowthError::Internal)?;
  crate::tag::clear(conn, owner, id).map_err(|_| GrowthError::Internal)?;
  crate::place::clear(conn, owner, id).map_err(|_| GrowthError::Internal)?;
  crate::media::clear(conn, owner, id).map_err(|_| GrowthError::Internal)?;
  Ok(())
}

fn ref_err<E: std::fmt::Display>(err: E) -> GrowthError {
  if err.to_string() == "引用不存在" { GrowthError::ReferencedMissing } else { GrowthError::Internal }
}
