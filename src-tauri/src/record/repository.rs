//! 统一记录、挂接和媒体元数据。只收已解锁的 `&Connection`。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{
  MEDIA_TABLE, RECORD_LINK_TABLE, RECORD_MEMBER_TABLE, RECORD_PLACE_TABLE, RECORD_TABLE, RECORD_TAG_TABLE,
  SCHEMA_VERSION, SCHEMA_VERSION_KEY,
};
use super::dto::{MediaItem, RecordCard, RecordDetail, RecordLink, RecordWrite};
use super::error::RecordError;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> RecordError {
  tauri_plugin_log::log::error!("{context}: {err}");
  RecordError::Internal
}

fn flag(value: i64) -> bool {
  value != 0
}

fn bit(value: bool) -> i64 {
  if value { 1 } else { 0 }
}

fn map_card(row: &rusqlite::Row<'_>) -> rusqlite::Result<RecordCard> {
  Ok(RecordCard {
    id: row.get(0)?,
    r#type: row.get(1)?,
    occurred_at: row.get(2)?,
    title: row.get(3)?,
    body: row.get(4)?,
    subject_member_id: row.get(5)?,
    locked: flag(row.get(6)?),
    highlight: flag(row.get(7)?),
    created_at: row.get(8)?,
    updated_at: row.get(9)?,
  })
}

fn map_media(row: &rusqlite::Row<'_>) -> rusqlite::Result<MediaItem> {
  Ok(MediaItem {
    id: row.get(0)?,
    record_id: row.get(1)?,
    media_kind: row.get(2)?,
    rel_path: row.get(3)?,
    mime: row.get(4)?,
    sort: row.get(5)?,
    locked: flag(row.get(6)?),
    created_at: row.get(7)?,
  })
}

fn map_link(row: &rusqlite::Row<'_>) -> rusqlite::Result<RecordLink> {
  Ok(RecordLink {
    id: row.get(0)?,
    from_id: row.get(1)?,
    to_id: row.get(2)?,
    kind: row.get(3)?,
    created_at: row.get(4)?,
  })
}

const CARD_COLUMNS: &str =
  "id, type, occurred_at, title, body, subject_member_id, locked, highlight, created_at, updated_at";

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), RecordError> {
  let current = schema_version(conn)?;

  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS record (
           id TEXT PRIMARY KEY NOT NULL,
           type TEXT NOT NULL,
           occurred_at INTEGER NOT NULL,
           title TEXT NOT NULL,
           body TEXT NOT NULL DEFAULT '',
           subject_member_id TEXT,
           locked INTEGER NOT NULL DEFAULT 0,
           highlight INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS record_type_occurred ON record(type, occurred_at);
         CREATE INDEX IF NOT EXISTS record_occurred ON record(occurred_at);
         CREATE INDEX IF NOT EXISTS record_updated_at ON record(updated_at);

         CREATE TABLE IF NOT EXISTS record_member (
           record_id TEXT NOT NULL,
           member_id TEXT NOT NULL,
           PRIMARY KEY (record_id, member_id)
         );
         CREATE TABLE IF NOT EXISTS record_tag (
           record_id TEXT NOT NULL,
           tag_id TEXT NOT NULL,
           PRIMARY KEY (record_id, tag_id)
         );
         CREATE TABLE IF NOT EXISTS record_place (
           record_id TEXT NOT NULL,
           place_id TEXT NOT NULL,
           PRIMARY KEY (record_id, place_id)
         );

         CREATE TABLE IF NOT EXISTS record_link (
           id TEXT PRIMARY KEY NOT NULL,
           from_id TEXT NOT NULL,
           to_id TEXT NOT NULL,
           kind TEXT NOT NULL,
           created_at INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS media (
           id TEXT PRIMARY KEY NOT NULL,
           record_id TEXT NOT NULL,
           media_kind TEXT NOT NULL,
           rel_path TEXT NOT NULL DEFAULT '',
           mime TEXT NOT NULL DEFAULT '',
           sort INTEGER NOT NULL DEFAULT 0,
           locked INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS media_record ON media(record_id);",
      )
      .map_err(|e| db_fail("record migrate v1", e))?;
  }

  if current < SCHEMA_VERSION {
    write_schema_version(conn)?;
  }

  Ok(())
}

fn schema_version(conn: &Connection) -> Result<i64, RecordError> {
  let current: Option<String> = conn
    .query_row("SELECT value FROM db_meta WHERE key = ?1", [SCHEMA_VERSION_KEY], |row| {
      row.get(0)
    })
    .optional()
    .map_err(|e| db_fail("record schema_version", e))?;
  Ok(current.and_then(|s| s.parse().ok()).unwrap_or(0))
}

fn write_schema_version(conn: &Connection) -> Result<(), RecordError> {
  let version = SCHEMA_VERSION.to_string();
  conn
    .execute(
      "INSERT INTO db_meta (key, value) VALUES (?1, ?2)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      params![SCHEMA_VERSION_KEY, version],
    )
    .map_err(|e| db_fail("record write schema_version", e))?;
  Ok(())
}

pub(super) fn list(
  conn: &Connection,
  record_type: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<RecordCard>, RecordError> {
  let sql = format!(
    "SELECT {CARD_COLUMNS} FROM {RECORD_TABLE}
     WHERE deleted_at IS NULL
       AND (?1 IS NULL OR type = ?1)
       AND (?2 IS NULL OR occurred_at >= ?2)
       AND (?3 IS NULL OR occurred_at < ?3)
     ORDER BY occurred_at DESC"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("record list prepare", e))?;
  let rows = stmt.query_map(params![record_type, from, to], map_card).map_err(|e| db_fail("record list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("record list collect", e))
}

pub(super) fn get(conn: &Connection, id: &str) -> Result<RecordDetail, RecordError> {
  let record = card(conn, id)?;
  Ok(RecordDetail {
    member_ids: id_list(conn, RECORD_MEMBER_TABLE, "member_id", id, "record members")?,
    tag_ids: id_list(conn, RECORD_TAG_TABLE, "tag_id", id, "record tags")?,
    place_ids: id_list(conn, RECORD_PLACE_TABLE, "place_id", id, "record places")?,
    media: media_list(conn, id)?,
    record,
  })
}

pub(super) fn create(conn: &Connection, input: &RecordWrite) -> Result<RecordDetail, RecordError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("record create tx", e))?;
  let sql = format!(
    "INSERT INTO {RECORD_TABLE}
       (id, type, occurred_at, title, body, subject_member_id, locked, highlight, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)"
  );
  tx.execute(
    &sql,
    params![
      id,
      input.r#type,
      input.occurred_at,
      input.title,
      input.body,
      input.subject_member_id,
      bit(input.locked),
      bit(input.highlight),
      now
    ],
  )
  .map_err(|e| db_fail("record create", e))?;
  replace_refs(&tx, &id, input)?;
  tx.commit().map_err(|e| db_fail("record create commit", e))?;
  get(conn, &id)
}

pub(super) fn update(conn: &Connection, id: &str, input: &RecordWrite) -> Result<RecordDetail, RecordError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("record update tx", e))?;
  let sql = format!(
    "UPDATE {RECORD_TABLE}
     SET type = ?1, occurred_at = ?2, title = ?3, body = ?4, subject_member_id = ?5,
         locked = ?6, highlight = ?7, updated_at = ?8
     WHERE id = ?9 AND deleted_at IS NULL"
  );
  let n = tx
    .execute(
      &sql,
      params![
        input.r#type,
        input.occurred_at,
        input.title,
        input.body,
        input.subject_member_id,
        bit(input.locked),
        bit(input.highlight),
        now,
        id
      ],
    )
    .map_err(|e| db_fail("record update", e))?;
  if n == 0 {
    return Err(RecordError::RecordNotFound);
  }
  replace_refs(&tx, id, input)?;
  tx.commit().map_err(|e| db_fail("record update commit", e))?;
  get(conn, id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), RecordError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("record delete tx", e))?;
  clear_children(&tx, id)?;
  let sql = format!("UPDATE {RECORD_TABLE} SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL");
  let n = tx.execute(&sql, params![now, id]).map_err(|e| db_fail("record delete", e))?;
  if n == 0 {
    return Err(RecordError::RecordNotFound);
  }
  tx.commit().map_err(|e| db_fail("record delete commit", e))?;
  Ok(())
}

pub(super) fn media_list(conn: &Connection, record_id: &str) -> Result<Vec<MediaItem>, RecordError> {
  let sql = format!(
    "SELECT id, record_id, media_kind, rel_path, mime, sort, locked, created_at
     FROM {MEDIA_TABLE} WHERE record_id = ?1 ORDER BY sort, created_at"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("media list prepare", e))?;
  let rows = stmt.query_map([record_id], map_media).map_err(|e| db_fail("media list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("media list collect", e))
}

pub(super) fn media_create(
  conn: &Connection,
  record_id: &str,
  media_kind: &str,
  rel_path: &str,
  mime: &str,
  sort: i64,
  locked: bool,
) -> Result<MediaItem, RecordError> {
  require_alive(conn, record_id)?;
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("media create tx", e))?;
  let sql = format!(
    "INSERT INTO {MEDIA_TABLE} (id, record_id, media_kind, rel_path, mime, sort, locked, created_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
  );
  tx.execute(&sql, params![id, record_id, media_kind, rel_path, mime, sort, bit(locked), now])
    .map_err(|e| db_fail("media create", e))?;
  touch_record(&tx, record_id, now)?;
  tx.commit().map_err(|e| db_fail("media create commit", e))?;
  media_get(conn, &id)
}

pub(super) fn media_delete(conn: &Connection, id: &str) -> Result<(), RecordError> {
  let item = media_get(conn, id)?;
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("media delete tx", e))?;
  let sql = format!("DELETE FROM {MEDIA_TABLE} WHERE id = ?1");
  tx.execute(&sql, [id]).map_err(|e| db_fail("media delete", e))?;
  touch_record(&tx, &item.record_id, now)?;
  tx.commit().map_err(|e| db_fail("media delete commit", e))?;
  Ok(())
}

pub(super) fn link_list(conn: &Connection, record_id: &str) -> Result<Vec<RecordLink>, RecordError> {
  let sql = format!(
    "SELECT id, from_id, to_id, kind, created_at FROM {RECORD_LINK_TABLE}
     WHERE from_id = ?1 OR to_id = ?1 ORDER BY created_at"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("link list prepare", e))?;
  let rows = stmt.query_map([record_id], map_link).map_err(|e| db_fail("link list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("link list collect", e))
}

pub(super) fn link_create(
  conn: &Connection,
  from_id: &str,
  to_id: &str,
  kind: &str,
) -> Result<RecordLink, RecordError> {
  require_alive(conn, from_id)?;
  require_alive(conn, to_id)?;
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("link create tx", e))?;
  let sql =
    format!("INSERT INTO {RECORD_LINK_TABLE} (id, from_id, to_id, kind, created_at) VALUES (?1, ?2, ?3, ?4, ?5)");
  tx.execute(&sql, params![id, from_id, to_id, kind, now]).map_err(|e| db_fail("link create", e))?;
  touch_record(&tx, from_id, now)?;
  touch_record(&tx, to_id, now)?;
  tx.commit().map_err(|e| db_fail("link create commit", e))?;
  link_get(conn, &id)
}

pub(super) fn link_delete(conn: &Connection, id: &str) -> Result<(), RecordError> {
  let link = link_get(conn, id)?;
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("link delete tx", e))?;
  let sql = format!("DELETE FROM {RECORD_LINK_TABLE} WHERE id = ?1");
  tx.execute(&sql, [id]).map_err(|e| db_fail("link delete", e))?;
  touch_record(&tx, &link.from_id, now)?;
  touch_record(&tx, &link.to_id, now)?;
  tx.commit().map_err(|e| db_fail("link delete commit", e))?;
  Ok(())
}

pub(super) fn require_alive(conn: &Connection, id: &str) -> Result<(), RecordError> {
  card(conn, id).map(|_| ())
}

fn card(conn: &Connection, id: &str) -> Result<RecordCard, RecordError> {
  let sql = format!("SELECT {CARD_COLUMNS} FROM {RECORD_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  conn.query_row(&sql, [id], map_card).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => RecordError::RecordNotFound,
    other => db_fail("record get", other),
  })
}

fn id_list(
  conn: &Connection,
  table: &str,
  column: &str,
  record_id: &str,
  context: &str,
) -> Result<Vec<String>, RecordError> {
  let sql = format!("SELECT {column} FROM {table} WHERE record_id = ?1 ORDER BY {column}");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail(context, e))?;
  let rows = stmt.query_map([record_id], |row| row.get(0)).map_err(|e| db_fail(context, e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail(context, e))
}

fn replace_refs(conn: &Connection, record_id: &str, input: &RecordWrite) -> Result<(), RecordError> {
  if let Some(member_id) = input.subject_member_id.as_deref() {
    require_catalog(conn, "member", member_id)?;
  }
  for member_id in &input.member_ids {
    require_catalog(conn, "member", member_id)?;
  }
  for tag_id in &input.tag_ids {
    require_catalog(conn, "tag", tag_id)?;
  }
  for place_id in &input.place_ids {
    require_catalog(conn, "place", place_id)?;
  }
  replace_ids(conn, RECORD_MEMBER_TABLE, "member_id", record_id, &input.member_ids)?;
  replace_ids(conn, RECORD_TAG_TABLE, "tag_id", record_id, &input.tag_ids)?;
  replace_ids(conn, RECORD_PLACE_TABLE, "place_id", record_id, &input.place_ids)?;
  Ok(())
}

fn require_catalog(conn: &Connection, table: &str, id: &str) -> Result<(), RecordError> {
  let sql = format!("SELECT 1 FROM {table} WHERE id = ?1 AND deleted_at IS NULL");
  let found: Option<i64> =
    conn.query_row(&sql, [id], |row| row.get(0)).optional().map_err(|e| db_fail("catalog ref", e))?;
  if found.is_none() {
    return Err(RecordError::ReferencedMissing);
  }
  Ok(())
}

fn replace_ids(
  conn: &Connection,
  table: &str,
  column: &str,
  record_id: &str,
  ids: &[String],
) -> Result<(), RecordError> {
  let delete_sql = format!("DELETE FROM {table} WHERE record_id = ?1");
  conn.execute(&delete_sql, [record_id]).map_err(|e| db_fail("replace refs", e))?;
  let insert_sql = format!("INSERT INTO {table} (record_id, {column}) VALUES (?1, ?2)");
  for id in ids {
    conn.execute(&insert_sql, params![record_id, id]).map_err(|e| db_fail("replace refs insert", e))?;
  }
  Ok(())
}

fn clear_children(conn: &Connection, record_id: &str) -> Result<(), RecordError> {
  for table in [RECORD_MEMBER_TABLE, RECORD_TAG_TABLE, RECORD_PLACE_TABLE, MEDIA_TABLE] {
    let sql = format!("DELETE FROM {table} WHERE record_id = ?1");
    conn.execute(&sql, [record_id]).map_err(|e| db_fail("record clear children", e))?;
  }
  let sql = format!("DELETE FROM {RECORD_LINK_TABLE} WHERE from_id = ?1 OR to_id = ?1");
  conn.execute(&sql, [record_id]).map_err(|e| db_fail("record clear links", e))?;
  Ok(())
}

fn touch_record(conn: &Connection, id: &str, now: i64) -> Result<(), RecordError> {
  let sql = format!("UPDATE {RECORD_TABLE} SET updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![now, id]).map_err(|e| db_fail("touch record", e))?;
  if n == 0 {
    return Err(RecordError::RecordNotFound);
  }
  Ok(())
}

fn media_get(conn: &Connection, id: &str) -> Result<MediaItem, RecordError> {
  let sql = format!(
    "SELECT id, record_id, media_kind, rel_path, mime, sort, locked, created_at FROM {MEDIA_TABLE} WHERE id = ?1"
  );
  conn.query_row(&sql, [id], map_media).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => RecordError::RecordNotFound,
    other => db_fail("media get", other),
  })
}

fn link_get(conn: &Connection, id: &str) -> Result<RecordLink, RecordError> {
  let sql = format!("SELECT id, from_id, to_id, kind, created_at FROM {RECORD_LINK_TABLE} WHERE id = ?1");
  conn.query_row(&sql, [id], map_link).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => RecordError::RecordNotFound,
    other => db_fail("link get", other),
  })
}
