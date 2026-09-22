//! 示例表 SQL。只收已解锁的 `&Connection`，不自己 `Connection::open`。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{SCHEMA_VERSION, SCHEMA_VERSION_KEY, TABLE_NAME};
use super::dto::DemoItem;
use super::error::DemoError;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

/// 把数据库错误转成业务错误。
fn db_fail(context: &str, err: rusqlite::Error) -> DemoError {
  tauri_plugin_log::log::error!("{context}: {err}");
  DemoError::Internal
}

/// 把数据库行转成业务对象。
fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DemoItem> {
  Ok(DemoItem {
    id: row.get(0)?,
    title: row.get(1)?,
    body: row.get(2)?,
    created_at: row.get(3)?,
    updated_at: row.get(4)?,
  })
}

/// 幂等建表示例表，并记下本模块 schema 版本。
/// 升 `SCHEMA_VERSION` 时在下面加 `if current < N { ALTER ... }`，不要只改数字。
pub(super) fn ensure_schema(conn: &Connection) -> Result<(), DemoError> {
  let current = schema_version(conn)?;

  if current < 1 {
    let sql = format!(
      "CREATE TABLE IF NOT EXISTS {TABLE_NAME} (
         id TEXT PRIMARY KEY NOT NULL,
         title TEXT NOT NULL,
         body TEXT NOT NULL DEFAULT '',
         created_at INTEGER NOT NULL,
         updated_at INTEGER NOT NULL
       )"
    );
    conn.execute(&sql, []).map_err(|e| db_fail("demo migrate v1", e))?;
  }

  if current < SCHEMA_VERSION {
    write_schema_version(conn)?;
  }

  Ok(())
}

fn schema_version(conn: &Connection) -> Result<i64, DemoError> {
  let current: Option<String> = conn
    .query_row("SELECT value FROM db_meta WHERE key = ?1", [SCHEMA_VERSION_KEY], |row| {
      row.get(0)
    })
    .optional()
    .map_err(|e| db_fail("demo schema_version", e))?;
  Ok(current.and_then(|s| s.parse().ok()).unwrap_or(0))
}

fn write_schema_version(conn: &Connection) -> Result<(), DemoError> {
  let version = SCHEMA_VERSION.to_string();
  conn
    .execute(
      "INSERT INTO db_meta (key, value) VALUES (?1, ?2)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      params![SCHEMA_VERSION_KEY, version],
    )
    .map_err(|e| db_fail("demo write schema_version", e))?;
  Ok(())
}

pub(super) fn list(conn: &Connection) -> Result<Vec<DemoItem>, DemoError> {
  let sql = format!("SELECT id, title, body, created_at, updated_at FROM {TABLE_NAME} ORDER BY updated_at DESC");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("demo list prepare", e))?;
  let rows = stmt.query_map([], map_row).map_err(|e| db_fail("demo list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("demo list collect", e))
}

pub(super) fn get(conn: &Connection, id: &str) -> Result<DemoItem, DemoError> {
  let sql = format!("SELECT id, title, body, created_at, updated_at FROM {TABLE_NAME} WHERE id = ?1");
  conn.query_row(&sql, [id], map_row).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => DemoError::RecordNotFound,
    other => db_fail("demo get", other),
  })
}

pub(super) fn create(conn: &Connection, title: &str, body: &str) -> Result<DemoItem, DemoError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!("INSERT INTO {TABLE_NAME} (id, title, body, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)");
  conn.execute(&sql, params![id, title, body, now, now]).map_err(|e| db_fail("demo create", e))?;
  get(conn, &id)
}

pub(super) fn update(conn: &Connection, id: &str, title: &str, body: &str) -> Result<DemoItem, DemoError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {TABLE_NAME} SET title = ?1, body = ?2, updated_at = ?3 WHERE id = ?4");
  let n = conn.execute(&sql, params![title, body, now, id]).map_err(|e| db_fail("demo update", e))?;
  if n == 0 {
    return Err(DemoError::RecordNotFound);
  }
  get(conn, id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), DemoError> {
  let sql = format!("DELETE FROM {TABLE_NAME} WHERE id = ?1");
  let n = conn.execute(&sql, [id]).map_err(|e| db_fail("demo delete", e))?;
  if n == 0 {
    return Err(DemoError::RecordNotFound);
  }
  Ok(())
}
