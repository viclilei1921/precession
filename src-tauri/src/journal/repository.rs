//! 手记、流转和书摘引用。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{
  CITATION_TABLE, ENTRY_TABLE, KIND_DIARY, KIND_SPARK, KIND_WRITING, LINK_SPARK_TO_WRITING, LINK_TABLE,
  LINK_WRITING_TO_DIARY, SCHEMA_VERSION, SCHEMA_VERSION_KEY,
};
use super::dto::{JournalCitation, JournalEntry, JournalEntryInput, JournalLink};
use super::error::JournalError;
use crate::owner::Owner;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> JournalError {
  tauri_plugin_log::log::error!("{context}: {err}");
  JournalError::Internal
}

const COLUMNS: &str = "id, kind, occurred_at, title, body, locked, highlight, created_at, updated_at";

fn map_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<JournalEntry> {
  Ok(JournalEntry {
    id: row.get(0)?,
    kind: row.get(1)?,
    occurred_at: row.get(2)?,
    title: row.get(3)?,
    body: row.get(4)?,
    locked: row.get(5)?,
    highlight: row.get(6)?,
    member_ids: Vec::new(),
    tag_ids: Vec::new(),
    place_ids: Vec::new(),
    created_at: row.get(7)?,
    updated_at: row.get(8)?,
  })
}

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), JournalError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("journal schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS journal_entry (
           id TEXT PRIMARY KEY NOT NULL,
           kind TEXT NOT NULL,
           occurred_at INTEGER NOT NULL,
           title TEXT NOT NULL,
           body TEXT NOT NULL DEFAULT '',
           locked INTEGER NOT NULL DEFAULT 0,
           highlight INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS journal_entry_occurred ON journal_entry(kind, occurred_at);
         CREATE TABLE IF NOT EXISTS journal_link (
           id TEXT PRIMARY KEY NOT NULL,
           from_id TEXT NOT NULL,
           to_id TEXT NOT NULL,
           kind TEXT NOT NULL,
           created_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS journal_citation (
           id TEXT PRIMARY KEY NOT NULL,
           entry_id TEXT NOT NULL,
           book_note_id TEXT NOT NULL,
           created_at INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS journal_citation_entry ON journal_citation(entry_id);
         CREATE INDEX IF NOT EXISTS journal_citation_note ON journal_citation(book_note_id);",
      )
      .map_err(|e| db_fail("journal migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("journal schema", e))?;
  }
  Ok(())
}

pub(super) fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  let found: Option<i64> = conn
    .query_row(
      "SELECT 1 FROM journal_entry WHERE id = ?1 AND deleted_at IS NULL",
      [id],
      |row| row.get(0),
    )
    .optional()?;
  Ok(found.is_some())
}

pub(super) fn list(
  conn: &Connection,
  kind: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<JournalEntry>, JournalError> {
  query(conn, None, kind, from, to)
}

pub(super) fn list_between(
  conn: &Connection,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<JournalEntry>, JournalError> {
  query(conn, None, None, from, to)
}

pub(super) fn get(conn: &Connection, id: &str) -> Result<JournalEntry, JournalError> {
  let mut items = query(conn, Some(id), None, None, None)?;
  items.pop().ok_or(JournalError::NotFound)
}

pub(super) fn create(conn: &Connection, input: &JournalEntryInput) -> Result<JournalEntry, JournalError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("journal create tx", e))?;
  tx.execute(
    "INSERT INTO journal_entry
       (id, kind, occurred_at, title, body, locked, highlight, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
    params![id, input.kind, input.occurred_at, input.title, input.body, input.locked, input.highlight, now],
  )
  .map_err(|e| db_fail("journal create", e))?;
  replace_refs(&tx, &id, input)?;
  tx.commit().map_err(|e| db_fail("journal create commit", e))?;
  get(conn, &id)
}

pub(super) fn update(conn: &Connection, id: &str, input: &JournalEntryInput) -> Result<JournalEntry, JournalError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("journal update tx", e))?;
  let n = tx
    .execute(
      "UPDATE journal_entry
       SET kind = ?1, occurred_at = ?2, title = ?3, body = ?4, locked = ?5, highlight = ?6, updated_at = ?7
       WHERE id = ?8 AND deleted_at IS NULL",
      params![input.kind, input.occurred_at, input.title, input.body, input.locked, input.highlight, now, id],
    )
    .map_err(|e| db_fail("journal update", e))?;
  if n == 0 {
    return Err(JournalError::NotFound);
  }
  replace_refs(&tx, id, input)?;
  tx.commit().map_err(|e| db_fail("journal update commit", e))?;
  get(conn, id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), JournalError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("journal delete tx", e))?;
  let n = tx
    .execute(
      "UPDATE journal_entry SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
      params![now, id],
    )
    .map_err(|e| db_fail("journal delete", e))?;
  if n == 0 {
    return Err(JournalError::NotFound);
  }
  tx.execute("DELETE FROM journal_link WHERE from_id = ?1 OR to_id = ?1", [id])
    .map_err(|e| db_fail("journal link", e))?;
  tx.execute("DELETE FROM journal_citation WHERE entry_id = ?1", [id]).map_err(|e| db_fail("journal citation", e))?;
  clear_refs(&tx, id)?;
  tx.commit().map_err(|e| db_fail("journal delete commit", e))?;
  Ok(())
}

pub(super) fn link_list(conn: &Connection, entry_id: &str) -> Result<Vec<JournalLink>, JournalError> {
  let sql = format!(
    "SELECT id, from_id, to_id, kind, created_at FROM {LINK_TABLE}
     WHERE from_id = ?1 OR to_id = ?1 ORDER BY created_at"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("journal link list", e))?;
  let rows = stmt
    .query_map([entry_id], |row| {
      Ok(JournalLink {
        id: row.get(0)?,
        from_id: row.get(1)?,
        to_id: row.get(2)?,
        kind: row.get(3)?,
        created_at: row.get(4)?,
      })
    })
    .map_err(|e| db_fail("journal link list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("journal link list", e))
}

pub(super) fn link_create(
  conn: &Connection,
  from_id: &str,
  to_id: &str,
  kind: &str,
) -> Result<JournalLink, JournalError> {
  if kind != LINK_SPARK_TO_WRITING && kind != LINK_WRITING_TO_DIARY {
    return Err(JournalError::KindInvalid);
  }
  if !exists(conn, from_id).map_err(|e| db_fail("journal link", e))?
    || !exists(conn, to_id).map_err(|e| db_fail("journal link", e))?
  {
    return Err(JournalError::NotFound);
  }
  let id = new_uuid_v4();
  let now = now_unix_ms();
  conn
    .execute(
      "INSERT INTO journal_link (id, from_id, to_id, kind, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
      params![id, from_id, to_id, kind, now],
    )
    .map_err(|e| db_fail("journal link create", e))?;
  Ok(JournalLink { id, from_id: from_id.into(), to_id: to_id.into(), kind: kind.into(), created_at: now })
}

pub(super) fn link_delete(conn: &Connection, id: &str) -> Result<(), JournalError> {
  let n =
    conn.execute("DELETE FROM journal_link WHERE id = ?1", [id]).map_err(|e| db_fail("journal link delete", e))?;
  if n == 0 {
    return Err(JournalError::NotFound);
  }
  Ok(())
}

pub(super) fn citation_list(conn: &Connection, entry_id: &str) -> Result<Vec<JournalCitation>, JournalError> {
  let sql = format!(
    "SELECT id, entry_id, book_note_id, created_at FROM {CITATION_TABLE} WHERE entry_id = ?1 ORDER BY created_at"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("journal citation list", e))?;
  let rows = stmt
    .query_map([entry_id], |row| {
      Ok(JournalCitation { id: row.get(0)?, entry_id: row.get(1)?, book_note_id: row.get(2)?, created_at: row.get(3)? })
    })
    .map_err(|e| db_fail("journal citation list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("journal citation list", e))
}

pub(super) fn citation_create(
  conn: &Connection,
  entry_id: &str,
  book_note_id: &str,
) -> Result<JournalCitation, JournalError> {
  if !exists(conn, entry_id).map_err(|e| db_fail("journal citation", e))? {
    return Err(JournalError::NotFound);
  }
  let note = crate::library::note_exists(conn, book_note_id).map_err(|e| db_fail("journal citation", e))?;
  if !note {
    return Err(JournalError::ReferencedMissing);
  }
  let id = new_uuid_v4();
  let now = now_unix_ms();
  conn
    .execute(
      "INSERT INTO journal_citation (id, entry_id, book_note_id, created_at) VALUES (?1, ?2, ?3, ?4)",
      params![id, entry_id, book_note_id, now],
    )
    .map_err(|e| db_fail("journal citation create", e))?;
  Ok(JournalCitation { id, entry_id: entry_id.into(), book_note_id: book_note_id.into(), created_at: now })
}

pub(super) fn citation_delete(conn: &Connection, id: &str) -> Result<(), JournalError> {
  let n = conn
    .execute("DELETE FROM journal_citation WHERE id = ?1", [id])
    .map_err(|e| db_fail("journal citation delete", e))?;
  if n == 0 {
    return Err(JournalError::NotFound);
  }
  Ok(())
}

pub(super) fn clear_note(conn: &Connection, book_note_id: &str) -> Result<(), JournalError> {
  conn
    .execute("DELETE FROM journal_citation WHERE book_note_id = ?1", [book_note_id])
    .map_err(|e| db_fail("journal citation clear", e))?;
  Ok(())
}

pub(super) fn allowed_kind(kind: &str) -> bool {
  kind == KIND_DIARY || kind == KIND_SPARK || kind == KIND_WRITING
}

fn query(
  conn: &Connection,
  id: Option<&str>,
  kind: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<JournalEntry>, JournalError> {
  let sql = format!(
    "SELECT {COLUMNS} FROM {ENTRY_TABLE}
     WHERE deleted_at IS NULL
       AND (?1 IS NULL OR id = ?1)
       AND (?2 IS NULL OR kind = ?2)
       AND (?3 IS NULL OR occurred_at >= ?3)
       AND (?4 IS NULL OR occurred_at < ?4)
     ORDER BY occurred_at DESC"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("journal list", e))?;
  let rows = stmt.query_map(params![id, kind, from, to], map_entry).map_err(|e| db_fail("journal list", e))?;
  let mut items = rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("journal list", e))?;
  for item in &mut items {
    fill_refs(conn, item)?;
  }
  Ok(items)
}

fn fill_refs(conn: &Connection, entry: &mut JournalEntry) -> Result<(), JournalError> {
  let owner = Owner::JournalEntry.as_str();
  entry.member_ids = crate::member::list_ids(conn, owner, &entry.id).map_err(|_| JournalError::Internal)?;
  entry.tag_ids = crate::tag::list_ids(conn, owner, &entry.id).map_err(|_| JournalError::Internal)?;
  entry.place_ids = crate::place::list_ids(conn, owner, &entry.id).map_err(|_| JournalError::Internal)?;
  Ok(())
}

fn replace_refs(conn: &Connection, id: &str, input: &JournalEntryInput) -> Result<(), JournalError> {
  let owner = Owner::JournalEntry.as_str();
  crate::member::replace(conn, owner, id, &input.member_ids).map_err(ref_err)?;
  crate::tag::replace(conn, owner, id, &input.tag_ids).map_err(ref_err)?;
  crate::place::replace(conn, owner, id, &input.place_ids).map_err(ref_err)?;
  Ok(())
}

fn clear_refs(conn: &Connection, id: &str) -> Result<(), JournalError> {
  let owner = Owner::JournalEntry.as_str();
  crate::member::clear(conn, owner, id).map_err(|_| JournalError::Internal)?;
  crate::tag::clear(conn, owner, id).map_err(|_| JournalError::Internal)?;
  crate::place::clear(conn, owner, id).map_err(|_| JournalError::Internal)?;
  crate::media::clear(conn, owner, id).map_err(|_| JournalError::Internal)?;
  Ok(())
}

fn ref_err<E: std::fmt::Display>(err: E) -> JournalError {
  if err.to_string() == "引用不存在" { JournalError::ReferencedMissing } else { JournalError::Internal }
}
