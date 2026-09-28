//! `db_meta` 里的 schema 版本。表归 `db` 所有，业务模块只读写自己的版本键。

use rusqlite::{Connection, OptionalExtension, params};

pub(crate) fn read_version(conn: &Connection, key: &str) -> Result<i64, rusqlite::Error> {
  let current: Option<String> =
    conn.query_row("SELECT value FROM db_meta WHERE key = ?1", [key], |row| row.get(0)).optional()?;
  Ok(current.and_then(|value| value.parse().ok()).unwrap_or(0))
}

pub(crate) fn write_version(conn: &Connection, key: &str, version: i64) -> Result<(), rusqlite::Error> {
  let version = version.to_string();
  conn.execute(
    "INSERT INTO db_meta (key, value) VALUES (?1, ?2)
     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    params![key, version],
  )?;
  Ok(())
}
