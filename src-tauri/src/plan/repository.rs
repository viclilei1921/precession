//! 计划明细。创建时在同一次事务里写入 `record` 和本模块的表。

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{PLAN_STEP_TABLE, PLAN_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::{PlanItem, PlanStep, PlanWrite};
use super::error::PlanError;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> PlanError {
  tauri_plugin_log::log::error!("{context}: {err}");
  PlanError::Internal
}

fn flag(value: i64) -> bool {
  value != 0
}

fn bit(value: bool) -> i64 {
  if value { 1 } else { 0 }
}

fn map_step(row: &rusqlite::Row<'_>) -> rusqlite::Result<(String, PlanStep)> {
  Ok((
    row.get(0)?,
    PlanStep { id: row.get(1)?, title: row.get(2)?, done: flag(row.get(3)?), sort: row.get(4)? },
  ))
}

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), PlanError> {
  let current = schema_version(conn)?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS plan (
           record_id TEXT PRIMARY KEY NOT NULL,
           status TEXT NOT NULL,
           priority INTEGER NOT NULL DEFAULT 0,
           due_at INTEGER,
           result TEXT NOT NULL DEFAULT ''
         );
         CREATE TABLE IF NOT EXISTS plan_step (
           id TEXT PRIMARY KEY NOT NULL,
           record_id TEXT NOT NULL,
           title TEXT NOT NULL,
           done INTEGER NOT NULL DEFAULT 0,
           sort INTEGER NOT NULL DEFAULT 0
         );
         CREATE INDEX IF NOT EXISTS plan_step_record ON plan_step(record_id, sort);",
      )
      .map_err(|e| db_fail("plan migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    write_schema_version(conn)?;
  }
  Ok(())
}

fn schema_version(conn: &Connection) -> Result<i64, PlanError> {
  let current: Option<String> = conn
    .query_row("SELECT value FROM db_meta WHERE key = ?1", [SCHEMA_VERSION_KEY], |row| {
      row.get(0)
    })
    .optional()
    .map_err(|e| db_fail("plan schema_version", e))?;
  Ok(current.and_then(|s| s.parse().ok()).unwrap_or(0))
}

fn write_schema_version(conn: &Connection) -> Result<(), PlanError> {
  let version = SCHEMA_VERSION.to_string();
  conn
    .execute(
      "INSERT INTO db_meta (key, value) VALUES (?1, ?2)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      params![SCHEMA_VERSION_KEY, version],
    )
    .map_err(|e| db_fail("plan write schema_version", e))?;
  Ok(())
}

pub(super) fn list(conn: &Connection) -> Result<Vec<PlanItem>, PlanError> {
  let mut items = load_plans(conn, None)?;
  attach_steps(conn, &mut items)?;
  Ok(items)
}

pub(super) fn get(conn: &Connection, id: &str) -> Result<PlanItem, PlanError> {
  let mut items = load_plans(conn, Some(id))?;
  let Some(mut item) = items.pop() else {
    return Err(PlanError::RecordNotFound);
  };
  item.steps = steps_for(conn, id)?;
  Ok(item)
}

pub(super) fn create(conn: &Connection, input: &PlanWrite) -> Result<PlanItem, PlanError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan create tx", e))?;
  tx.execute(
    "INSERT INTO record
       (id, type, occurred_at, title, body, locked, highlight, created_at, updated_at)
     VALUES (?1, 'plan', ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
    params![id, input.occurred_at, input.title, input.body, bit(input.locked), bit(input.highlight), now],
  )
  .map_err(|e| db_fail("plan create record", e))?;
  insert_plan(&tx, &id, input)?;
  replace_steps(&tx, &id, input)?;
  tx.commit().map_err(|e| db_fail("plan create commit", e))?;
  get(conn, &id)
}

pub(super) fn update(conn: &Connection, id: &str, input: &PlanWrite) -> Result<PlanItem, PlanError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan update tx", e))?;
  let n = tx
    .execute(
      "UPDATE record
       SET occurred_at = ?1, title = ?2, body = ?3, locked = ?4, highlight = ?5, updated_at = ?6
       WHERE id = ?7 AND type = 'plan' AND deleted_at IS NULL",
      params![input.occurred_at, input.title, input.body, bit(input.locked), bit(input.highlight), now, id],
    )
    .map_err(|e| db_fail("plan update record", e))?;
  if n == 0 {
    return Err(PlanError::RecordNotFound);
  }
  tx.execute(
    "UPDATE plan SET status = ?1, priority = ?2, due_at = ?3, result = ?4 WHERE record_id = ?5",
    params![input.status, input.priority, input.due_at, input.result, id],
  )
  .map_err(|e| db_fail("plan update", e))?;
  replace_steps(&tx, id, input)?;
  tx.commit().map_err(|e| db_fail("plan update commit", e))?;
  get(conn, id)
}

pub(super) fn complete(conn: &Connection, id: &str, result: &str) -> Result<PlanItem, PlanError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan complete tx", e))?;
  let n = tx
    .execute(
      "UPDATE record SET updated_at = ?1 WHERE id = ?2 AND type = 'plan' AND deleted_at IS NULL",
      params![now, id],
    )
    .map_err(|e| db_fail("plan complete record", e))?;
  if n == 0 {
    return Err(PlanError::RecordNotFound);
  }
  tx.execute(
    "UPDATE plan SET status = 'done', result = ?1 WHERE record_id = ?2",
    params![result, id],
  )
  .map_err(|e| db_fail("plan complete", e))?;
  tx.commit().map_err(|e| db_fail("plan complete commit", e))?;
  get(conn, id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), PlanError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan delete tx", e))?;
  let n = tx
    .execute(
      "UPDATE record SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND type = 'plan' AND deleted_at IS NULL",
      params![now, id],
    )
    .map_err(|e| db_fail("plan delete record", e))?;
  if n == 0 {
    return Err(PlanError::RecordNotFound);
  }
  clear_children(&tx, id)?;
  tx.commit().map_err(|e| db_fail("plan delete commit", e))?;
  Ok(())
}

fn load_plans(conn: &Connection, id: Option<&str>) -> Result<Vec<PlanItem>, PlanError> {
  let sql = "SELECT r.id, r.occurred_at, r.title, r.body, r.locked, r.highlight,
                    p.status, p.priority, p.due_at, p.result, r.created_at, r.updated_at
             FROM record r
             JOIN plan p ON p.record_id = r.id
             WHERE r.deleted_at IS NULL AND r.type = 'plan' AND (?1 IS NULL OR r.id = ?1)
             ORDER BY r.occurred_at";
  let mut stmt = conn.prepare(sql).map_err(|e| db_fail("plan list prepare", e))?;
  let rows = stmt
    .query_map(params![id], |row| {
      Ok(PlanItem {
        id: row.get(0)?,
        occurred_at: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        locked: flag(row.get(4)?),
        highlight: flag(row.get(5)?),
        status: row.get(6)?,
        priority: row.get(7)?,
        due_at: row.get(8)?,
        result: row.get(9)?,
        steps: Vec::new(),
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
      })
    })
    .map_err(|e| db_fail("plan list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("plan list collect", e))
}

fn attach_steps(conn: &Connection, items: &mut [PlanItem]) -> Result<(), PlanError> {
  let sql = format!(
    "SELECT s.record_id, s.id, s.title, s.done, s.sort
     FROM {PLAN_STEP_TABLE} s
     JOIN record r ON r.id = s.record_id
     WHERE r.deleted_at IS NULL
     ORDER BY s.sort"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("plan steps prepare", e))?;
  let rows = stmt.query_map([], map_step).map_err(|e| db_fail("plan steps query", e))?;
  let mut grouped: HashMap<String, Vec<PlanStep>> = HashMap::new();
  for row in rows {
    let (record_id, step) = row.map_err(|e| db_fail("plan steps collect", e))?;
    grouped.entry(record_id).or_default().push(step);
  }
  for item in items {
    if let Some(steps) = grouped.remove(&item.id) {
      item.steps = steps;
    }
  }
  Ok(())
}

fn steps_for(conn: &Connection, record_id: &str) -> Result<Vec<PlanStep>, PlanError> {
  let sql =
    format!("SELECT record_id, id, title, done, sort FROM {PLAN_STEP_TABLE} WHERE record_id = ?1 ORDER BY sort");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("plan step get prepare", e))?;
  let rows = stmt.query_map([record_id], map_step).map_err(|e| db_fail("plan step get query", e))?;
  rows
    .map(|row| row.map(|(_, step)| step))
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| db_fail("plan step get collect", e))
}

fn insert_plan(conn: &Connection, id: &str, input: &PlanWrite) -> Result<(), PlanError> {
  let sql =
    format!("INSERT INTO {PLAN_TABLE} (record_id, status, priority, due_at, result) VALUES (?1, ?2, ?3, ?4, ?5)");
  conn
    .execute(&sql, params![id, input.status, input.priority, input.due_at, input.result])
    .map_err(|e| db_fail("plan insert", e))?;
  Ok(())
}

fn replace_steps(conn: &Connection, record_id: &str, input: &PlanWrite) -> Result<(), PlanError> {
  let sql = format!("DELETE FROM {PLAN_STEP_TABLE} WHERE record_id = ?1");
  conn.execute(&sql, [record_id]).map_err(|e| db_fail("plan steps clear", e))?;
  let sql = format!("INSERT INTO {PLAN_STEP_TABLE} (id, record_id, title, done, sort) VALUES (?1, ?2, ?3, ?4, ?5)");
  for (index, step) in input.steps.iter().enumerate() {
    conn
      .execute(
        &sql,
        params![new_uuid_v4(), record_id, step.title, bit(step.done), index as i64],
      )
      .map_err(|e| db_fail("plan step insert", e))?;
  }
  Ok(())
}

fn clear_children(conn: &Connection, record_id: &str) -> Result<(), PlanError> {
  for sql in [
    format!("DELETE FROM {PLAN_STEP_TABLE} WHERE record_id = ?1"),
    format!("DELETE FROM {PLAN_TABLE} WHERE record_id = ?1"),
    "DELETE FROM media WHERE record_id = ?1".to_string(),
    "DELETE FROM record_member WHERE record_id = ?1".to_string(),
    "DELETE FROM record_tag WHERE record_id = ?1".to_string(),
    "DELETE FROM record_place WHERE record_id = ?1".to_string(),
  ] {
    conn.execute(&sql, [record_id]).map_err(|e| db_fail("plan clear", e))?;
  }
  conn
    .execute("DELETE FROM record_link WHERE from_id = ?1 OR to_id = ?1", [record_id])
    .map_err(|e| db_fail("plan clear links", e))?;
  Ok(())
}
