//! 官方节假日与个人单日覆盖。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{DAY_TABLE, OFFICIAL_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::CalendarOverride;
use super::error::CalendarError;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> CalendarError {
  tauri_plugin_log::log::error!("{context}: {err}");
  CalendarError::Internal
}

pub(super) struct OfficialRow {
  pub cal_date: String,
  pub name: String,
  pub is_off_day: bool,
}

pub(super) struct OfficialInsert {
  pub cal_date: String,
  pub name: String,
  pub is_off_day: bool,
}

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), CalendarError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("calendar schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS calendar_official (
           id TEXT PRIMARY KEY NOT NULL,
           cal_date TEXT NOT NULL UNIQUE,
           year INTEGER NOT NULL,
           name TEXT NOT NULL,
           is_off_day INTEGER NOT NULL,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS calendar_official_year ON calendar_official(year);
         CREATE TABLE IF NOT EXISTS calendar_day (
           id TEXT PRIMARY KEY NOT NULL,
           cal_date TEXT NOT NULL UNIQUE,
           is_workday INTEGER NOT NULL,
           note TEXT,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL
         );",
      )
      .map_err(|e| db_fail("calendar migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("calendar schema", e))?;
  }
  Ok(())
}

pub(super) fn list_official(conn: &Connection, from: &str, to: &str) -> Result<Vec<OfficialRow>, CalendarError> {
  let sql = format!(
    "SELECT cal_date, name, is_off_day FROM {OFFICIAL_TABLE} WHERE cal_date >= ?1 AND cal_date <= ?2 ORDER BY cal_date"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("calendar official list", e))?;
  let rows = stmt
    .query_map(params![from, to], |row| {
      Ok(OfficialRow { cal_date: row.get(0)?, name: row.get(1)?, is_off_day: row.get(2)? })
    })
    .map_err(|e| db_fail("calendar official list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("calendar official list", e))
}

pub(super) fn list_overrides(conn: &Connection, from: &str, to: &str) -> Result<Vec<CalendarOverride>, CalendarError> {
  let sql = format!(
    "SELECT id, cal_date, is_workday, note, created_at, updated_at FROM {DAY_TABLE}
     WHERE cal_date >= ?1 AND cal_date <= ?2 ORDER BY cal_date"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("calendar day list", e))?;
  let rows = stmt.query_map(params![from, to], map_override).map_err(|e| db_fail("calendar day list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("calendar day list", e))
}

pub(super) fn replace_official_year(
  conn: &Connection,
  year: i32,
  days: &[OfficialInsert],
) -> Result<i64, CalendarError> {
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("calendar official tx", e))?;
  let sql = format!("DELETE FROM {OFFICIAL_TABLE} WHERE year = ?1");
  tx.execute(&sql, params![year]).map_err(|e| db_fail("calendar official delete", e))?;
  let sql = format!(
    "INSERT INTO {OFFICIAL_TABLE} (id, cal_date, year, name, is_off_day, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)"
  );
  for day in days {
    let id = new_uuid_v4();
    let now = now_unix_ms();
    tx.execute(&sql, params![id, day.cal_date, year, day.name, day.is_off_day, now])
      .map_err(|e| db_fail("calendar official insert", e))?;
  }
  tx.commit().map_err(|e| db_fail("calendar official commit", e))?;
  Ok(days.len() as i64)
}

pub(super) fn upsert_day(
  conn: &Connection,
  date: &str,
  is_workday: bool,
  note: Option<&str>,
) -> Result<CalendarOverride, CalendarError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!(
    "INSERT INTO {DAY_TABLE} (id, cal_date, is_workday, note, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?5)
     ON CONFLICT(cal_date) DO UPDATE SET
       is_workday = excluded.is_workday,
       note = excluded.note,
       updated_at = excluded.updated_at"
  );
  conn.execute(&sql, params![id, date, is_workday, note, now]).map_err(|e| db_fail("calendar day upsert", e))?;
  get_override(conn, date)
}

pub(super) fn delete_day(conn: &Connection, date: &str) -> Result<(), CalendarError> {
  let sql = format!("DELETE FROM {DAY_TABLE} WHERE cal_date = ?1");
  let n = conn.execute(&sql, params![date]).map_err(|e| db_fail("calendar day delete", e))?;
  if n == 0 {
    return Err(CalendarError::OverrideMissing);
  }
  Ok(())
}

fn get_override(conn: &Connection, date: &str) -> Result<CalendarOverride, CalendarError> {
  let sql =
    format!("SELECT id, cal_date, is_workday, note, created_at, updated_at FROM {DAY_TABLE} WHERE cal_date = ?1");
  conn
    .query_row(&sql, params![date], map_override)
    .optional()
    .map_err(|e| db_fail("calendar day get", e))?
    .ok_or(CalendarError::Internal)
}

fn map_override(row: &rusqlite::Row<'_>) -> rusqlite::Result<CalendarOverride> {
  Ok(CalendarOverride {
    id: row.get(0)?,
    date: row.get(1)?,
    is_workday: row.get(2)?,
    note: row.get(3)?,
    created_at: row.get(4)?,
    updated_at: row.get(5)?,
  })
}
