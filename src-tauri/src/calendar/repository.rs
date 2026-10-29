//! 官方节假日与个人单日覆盖。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{DAY_TABLE, OFFICIAL_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::CalendarOverride;
use super::error::CalendarError;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

/// 记录数据库错误并返回内部错误
///
/// # 参数
/// - `context`: 日志中的操作说明
/// - `err`: 原始数据库错误
///
/// # 返回值
/// 返回 `CalendarError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let err = db_fail("calendar", rusqlite_err);
/// ```
fn db_fail(context: &str, err: rusqlite::Error) -> CalendarError {
  tauri_plugin_log::log::error!("{context}: {err}");
  CalendarError::Internal
}

/// 查询到的官方节假日
///
/// # 字段
/// - `cal_date`: 日期
/// - `name`: 节日名
/// - `is_off_day`: 是否放假，false 表示调休上班
///
/// # 示例
/// 见 [`list_official`]
pub(super) struct OfficialRow {
  /// 日期
  pub cal_date: String,
  /// 节日名
  pub name: String,
  /// 是否放假，false 表示调休上班
  pub is_off_day: bool,
}

/// 待写入的官方日期
///
/// 节日名已经去掉空白。`is_off_day` 为 false 表示调休上班。
///
/// # 字段
/// - `cal_date`: 日期
/// - `name`: 节日名
/// - `is_off_day`: 是否放假，false 表示调休上班
///
/// # 示例
/// ```ignore
/// let makeup = OfficialInsert {
///   cal_date: "2026-09-26".into(),
///   name: "国庆节".into(),
///   is_off_day: false,
/// };
/// assert!(!makeup.is_off_day);
/// ```
pub(super) struct OfficialInsert {
  /// 日期
  pub cal_date: String,
  /// 节日名
  pub name: String,
  /// 是否放假，false 表示调休上班
  pub is_off_day: bool,
}

/// 按架构版本创建官方节假日表和个人覆盖表
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 读版本、建表或写版本失败时返回 `CalendarError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// ensure_schema(conn)?;
/// ```
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

/// 按日期闭区间列出官方节假日
///
/// # 参数
/// - `conn`: 数据库连接
/// - `from`: 开始日期
/// - `to`: 结束日期
///
/// # 返回值
/// 返回按日期排序的官方记录
///
/// # 错误
/// 查询失败时返回 `CalendarError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list_official(conn, from, to)?;
/// ```
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

/// 按日期闭区间列出个人覆盖
///
/// # 参数
/// - `conn`: 数据库连接
/// - `from`: 开始日期
/// - `to`: 结束日期
///
/// # 返回值
/// 返回按日期排序的个人覆盖
///
/// # 错误
/// 查询失败时返回 `CalendarError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list_overrides(conn, from, to)?;
/// ```
pub(super) fn list_overrides(conn: &Connection, from: &str, to: &str) -> Result<Vec<CalendarOverride>, CalendarError> {
  let sql = format!(
    "SELECT id, cal_date, is_workday, note, created_at, updated_at FROM {DAY_TABLE}
     WHERE cal_date >= ?1 AND cal_date <= ?2 ORDER BY cal_date"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("calendar day list", e))?;
  let rows = stmt.query_map(params![from, to], map_override).map_err(|e| db_fail("calendar day list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("calendar day list", e))
}

/// 删掉该年官方记录后重新插入
///
/// # 参数
/// - `conn`: 数据库连接
/// - `year`: 年份
/// - `days`: 待写入的官方日期
///
/// # 返回值
/// 返回写入条数
///
/// # 错误
/// 写入失败时返回 `CalendarError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let count = replace_official_year(conn, year, days)?;
/// ```
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

/// 按日期插入或覆盖个人安排
///
/// # 参数
/// - `conn`: 数据库连接
/// - `date`: 日期
/// - `is_workday`: 是否上班
/// - `note`: 备注，可空
///
/// # 返回值
/// 返回保存后的个人覆盖
///
/// # 错误
/// 写入或回读失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let day = upsert_day(conn, date, is_workday, note)?;
/// ```
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

/// 按日期删除个人覆盖
///
/// # 参数
/// - `conn`: 数据库连接
/// - `date`: 日期
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 没有对应行时返回 `OverrideMissing`，删除失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete_day(conn, date)?;
/// ```
pub(super) fn delete_day(conn: &Connection, date: &str) -> Result<(), CalendarError> {
  let sql = format!("DELETE FROM {DAY_TABLE} WHERE cal_date = ?1");
  let n = conn.execute(&sql, params![date]).map_err(|e| db_fail("calendar day delete", e))?;
  if n == 0 {
    return Err(CalendarError::OverrideMissing);
  }
  Ok(())
}

/// 按日期读取个人覆盖
///
/// # 参数
/// - `conn`: 数据库连接
/// - `date`: 日期
///
/// # 返回值
/// 返回个人覆盖
///
/// # 错误
/// 没有对应行或查询失败时返回 `CalendarError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let day = get_override(conn, date)?;
/// ```
fn get_override(conn: &Connection, date: &str) -> Result<CalendarOverride, CalendarError> {
  let sql =
    format!("SELECT id, cal_date, is_workday, note, created_at, updated_at FROM {DAY_TABLE} WHERE cal_date = ?1");
  conn
    .query_row(&sql, params![date], map_override)
    .optional()
    .map_err(|e| db_fail("calendar day get", e))?
    .ok_or(CalendarError::Internal)
}

/// 从查询行组装个人覆盖
///
/// # 参数
/// - `row`: 查询行
///
/// # 返回值
/// 返回个人覆盖
///
/// # 错误
/// 列读取失败时返回 `rusqlite::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let day = map_override(row)?;
/// ```
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
