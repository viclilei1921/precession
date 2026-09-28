//! 图片和视频的元数据。文件本体不在这里。

use rusqlite::{Connection, params};

use super::constants::{KIND_IMAGE, KIND_VIDEO, MEDIA_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::{Media, MediaInput};
use super::error::MediaError;
use crate::owner::{self, Owner};
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> MediaError {
  tauri_plugin_log::log::error!("{context}: {err}");
  MediaError::Internal
}

fn map_media(row: &rusqlite::Row<'_>) -> rusqlite::Result<Media> {
  Ok(Media {
    id: row.get(0)?,
    owner: row.get(1)?,
    owner_id: row.get(2)?,
    kind: row.get(3)?,
    rel_path: row.get(4)?,
    mime: row.get(5)?,
    sort: row.get(6)?,
    locked: row.get(7)?,
    created_at: row.get(8)?,
  })
}

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), MediaError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("media schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS media (
           id TEXT PRIMARY KEY NOT NULL,
           owner TEXT NOT NULL,
           owner_id TEXT NOT NULL,
           kind TEXT NOT NULL,
           rel_path TEXT NOT NULL DEFAULT '',
           mime TEXT NOT NULL DEFAULT '',
           sort INTEGER NOT NULL DEFAULT 0,
           locked INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS media_owner ON media(owner, owner_id);",
      )
      .map_err(|e| db_fail("media migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("media schema", e))?;
  }
  Ok(())
}

pub(super) fn prepare(mut input: MediaInput) -> Result<(Owner, MediaInput), MediaError> {
  input.owner = input.owner.trim().to_string();
  input.owner_id = input.owner_id.trim().to_string();
  input.kind = input.kind.trim().to_string();
  input.rel_path = input.rel_path.trim().to_string();
  input.mime = input.mime.trim().to_string();
  if input.kind != KIND_IMAGE && input.kind != KIND_VIDEO {
    return Err(MediaError::KindInvalid);
  }
  let Some(owner) = Owner::parse(&input.owner) else {
    return Err(MediaError::KindInvalid);
  };
  Ok((owner, input))
}

pub(super) fn list(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<Media>, MediaError> {
  let sql = format!(
    "SELECT id, owner, owner_id, kind, rel_path, mime, sort, locked, created_at
     FROM {MEDIA_TABLE} WHERE owner = ?1 AND owner_id = ?2 ORDER BY sort, created_at"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("media list", e))?;
  let rows = stmt.query_map(params![owner, owner_id], map_media).map_err(|e| db_fail("media list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("media list", e))
}

pub(super) fn create(conn: &Connection, owner: Owner, input: &MediaInput) -> Result<Media, MediaError> {
  let alive = owner::exists(conn, owner, &input.owner_id).map_err(|e| db_fail("media owner", e))?;
  if !alive {
    return Err(MediaError::ReferencedMissing);
  }
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!(
    "INSERT INTO {MEDIA_TABLE} (id, owner, owner_id, kind, rel_path, mime, sort, locked, created_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
  );
  conn
    .execute(
      &sql,
      params![id, input.owner, input.owner_id, input.kind, input.rel_path, input.mime, input.sort, input.locked, now],
    )
    .map_err(|e| db_fail("media create", e))?;
  get(conn, &id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), MediaError> {
  let sql = format!("DELETE FROM {MEDIA_TABLE} WHERE id = ?1");
  let n = conn.execute(&sql, [id]).map_err(|e| db_fail("media delete", e))?;
  if n == 0 {
    return Err(MediaError::NotFound);
  }
  Ok(())
}

pub(super) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), MediaError> {
  let sql = format!("DELETE FROM {MEDIA_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("media clear", e))?;
  Ok(())
}

fn get(conn: &Connection, id: &str) -> Result<Media, MediaError> {
  let sql = format!(
    "SELECT id, owner, owner_id, kind, rel_path, mime, sort, locked, created_at FROM {MEDIA_TABLE} WHERE id = ?1"
  );
  conn.query_row(&sql, [id], map_media).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => MediaError::NotFound,
    other => db_fail("media get", other),
  })
}
