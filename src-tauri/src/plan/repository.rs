//! 计划及其步骤。成员、标签、地点和媒体由各自模块写入。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{PLAN_STEP_TABLE, PLAN_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::{Plan, PlanInput, PlanStep};
use super::error::PlanError;
use crate::owner::Owner;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> PlanError {
  tauri_plugin_log::log::error!("{context}: {err}");
  PlanError::Internal
}

const COLUMNS: &str = "id, title, body, status, priority, scheduled_at, due_at, completed_at, result, locked, highlight, created_at, updated_at";

fn map_plan(row: &rusqlite::Row<'_>) -> rusqlite::Result<Plan> {
  Ok(Plan {
    id: row.get(0)?,
    title: row.get(1)?,
    body: row.get(2)?,
    status: row.get(3)?,
    priority: row.get(4)?,
    scheduled_at: row.get(5)?,
    due_at: row.get(6)?,
    completed_at: row.get(7)?,
    result: row.get(8)?,
    locked: row.get(9)?,
    highlight: row.get(10)?,
    steps: Vec::new(),
    member_ids: Vec::new(),
    tag_ids: Vec::new(),
    place_ids: Vec::new(),
    created_at: row.get(11)?,
    updated_at: row.get(12)?,
  })
}

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), PlanError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("plan schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS plan (
           id TEXT PRIMARY KEY NOT NULL,
           title TEXT NOT NULL,
           body TEXT NOT NULL DEFAULT '',
           status TEXT NOT NULL,
           priority INTEGER NOT NULL DEFAULT 0,
           scheduled_at INTEGER,
           due_at INTEGER,
           completed_at INTEGER,
           result TEXT NOT NULL DEFAULT '',
           locked INTEGER NOT NULL DEFAULT 0,
           highlight INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS plan_scheduled ON plan(scheduled_at);
         CREATE TABLE IF NOT EXISTS plan_step (
           id TEXT PRIMARY KEY NOT NULL,
           plan_id TEXT NOT NULL,
           title TEXT NOT NULL,
           done INTEGER NOT NULL DEFAULT 0,
           sort INTEGER NOT NULL DEFAULT 0
         );
         CREATE INDEX IF NOT EXISTS plan_step_plan ON plan_step(plan_id, sort);",
      )
      .map_err(|e| db_fail("plan migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("plan schema", e))?;
  }
  Ok(())
}

pub(super) fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  let found: Option<i64> = conn
    .query_row("SELECT 1 FROM plan WHERE id = ?1 AND deleted_at IS NULL", [id], |row| {
      row.get(0)
    })
    .optional()?;
  Ok(found.is_some())
}

pub(super) fn list(conn: &Connection) -> Result<Vec<Plan>, PlanError> {
  query(conn, None, None, None)
}

pub(super) fn list_between(conn: &Connection, from: Option<i64>, to: Option<i64>) -> Result<Vec<Plan>, PlanError> {
  query(conn, None, from, to)
}

pub(super) fn get(conn: &Connection, id: &str) -> Result<Plan, PlanError> {
  let mut items = query(conn, Some(id), None, None)?;
  items.pop().ok_or(PlanError::NotFound)
}

pub(super) fn create(conn: &Connection, input: &PlanInput) -> Result<Plan, PlanError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan create tx", e))?;
  tx.execute(
    "INSERT INTO plan
       (id, title, body, status, priority, scheduled_at, due_at, result, locked, highlight, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
    params![
      id,
      input.title,
      input.body,
      input.status,
      input.priority,
      input.scheduled_at,
      input.due_at,
      input.result,
      input.locked,
      input.highlight,
      now
    ],
  )
  .map_err(|e| db_fail("plan create", e))?;
  replace_steps(&tx, &id, input)?;
  replace_refs(&tx, &id, input)?;
  tx.commit().map_err(|e| db_fail("plan create commit", e))?;
  get(conn, &id)
}

pub(super) fn update(conn: &Connection, id: &str, input: &PlanInput) -> Result<Plan, PlanError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan update tx", e))?;
  let n = tx
    .execute(
      "UPDATE plan
       SET title = ?1, body = ?2, status = ?3, priority = ?4, scheduled_at = ?5, due_at = ?6,
           result = ?7, locked = ?8, highlight = ?9, updated_at = ?10
       WHERE id = ?11 AND deleted_at IS NULL",
      params![
        input.title,
        input.body,
        input.status,
        input.priority,
        input.scheduled_at,
        input.due_at,
        input.result,
        input.locked,
        input.highlight,
        now,
        id
      ],
    )
    .map_err(|e| db_fail("plan update", e))?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  replace_steps(&tx, id, input)?;
  replace_refs(&tx, id, input)?;
  tx.commit().map_err(|e| db_fail("plan update commit", e))?;
  get(conn, id)
}

pub(super) fn complete(conn: &Connection, id: &str, result: &str) -> Result<Plan, PlanError> {
  let now = now_unix_ms();
  let sql = format!(
    "UPDATE {PLAN_TABLE}
     SET status = 'done', result = ?1, completed_at = ?2, updated_at = ?2
     WHERE id = ?3 AND deleted_at IS NULL"
  );
  let n = conn.execute(&sql, params![result, now, id]).map_err(|e| db_fail("plan complete", e))?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  get(conn, id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), PlanError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan delete tx", e))?;
  let n = tx
    .execute(
      "UPDATE plan SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
      params![now, id],
    )
    .map_err(|e| db_fail("plan delete", e))?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  let sql = format!("DELETE FROM {PLAN_STEP_TABLE} WHERE plan_id = ?1");
  tx.execute(&sql, [id]).map_err(|e| db_fail("plan steps clear", e))?;
  clear_refs(&tx, id)?;
  tx.commit().map_err(|e| db_fail("plan delete commit", e))?;
  Ok(())
}

fn query(conn: &Connection, id: Option<&str>, from: Option<i64>, to: Option<i64>) -> Result<Vec<Plan>, PlanError> {
  let sql = format!(
    "SELECT {COLUMNS} FROM {PLAN_TABLE}
     WHERE deleted_at IS NULL
       AND (?1 IS NULL OR id = ?1)
       AND (?2 IS NULL OR COALESCE(scheduled_at, created_at) >= ?2)
       AND (?3 IS NULL OR COALESCE(scheduled_at, created_at) < ?3)
     ORDER BY COALESCE(scheduled_at, created_at)"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("plan list", e))?;
  let rows = stmt.query_map(params![id, from, to], map_plan).map_err(|e| db_fail("plan list", e))?;
  let mut items = rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("plan list", e))?;
  for item in &mut items {
    item.steps = steps_for(conn, &item.id)?;
    fill_refs(conn, item)?;
  }
  Ok(items)
}

fn steps_for(conn: &Connection, plan_id: &str) -> Result<Vec<PlanStep>, PlanError> {
  let sql = format!("SELECT id, title, done, sort FROM {PLAN_STEP_TABLE} WHERE plan_id = ?1 ORDER BY sort");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("plan steps", e))?;
  let rows = stmt
    .query_map([plan_id], |row| {
      Ok(PlanStep { id: row.get(0)?, title: row.get(1)?, done: row.get(2)?, sort: row.get(3)? })
    })
    .map_err(|e| db_fail("plan steps", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("plan steps", e))
}

fn fill_refs(conn: &Connection, plan: &mut Plan) -> Result<(), PlanError> {
  let owner = Owner::Plan.as_str();
  plan.member_ids = crate::member::list_ids(conn, owner, &plan.id).map_err(|_| PlanError::Internal)?;
  plan.tag_ids = crate::tag::list_ids(conn, owner, &plan.id).map_err(|_| PlanError::Internal)?;
  plan.place_ids = crate::place::list_ids(conn, owner, &plan.id).map_err(|_| PlanError::Internal)?;
  Ok(())
}

fn replace_steps(conn: &Connection, plan_id: &str, input: &PlanInput) -> Result<(), PlanError> {
  let sql = format!("DELETE FROM {PLAN_STEP_TABLE} WHERE plan_id = ?1");
  conn.execute(&sql, [plan_id]).map_err(|e| db_fail("plan steps", e))?;
  let sql = format!("INSERT INTO {PLAN_STEP_TABLE} (id, plan_id, title, done, sort) VALUES (?1, ?2, ?3, ?4, ?5)");
  for (index, step) in input.steps.iter().enumerate() {
    conn
      .execute(&sql, params![new_uuid_v4(), plan_id, step.title, step.done, index as i64])
      .map_err(|e| db_fail("plan step", e))?;
  }
  Ok(())
}

fn replace_refs(conn: &Connection, id: &str, input: &PlanInput) -> Result<(), PlanError> {
  let owner = Owner::Plan.as_str();
  crate::member::replace(conn, owner, id, &input.member_ids).map_err(ref_err)?;
  crate::tag::replace(conn, owner, id, &input.tag_ids).map_err(ref_err)?;
  crate::place::replace(conn, owner, id, &input.place_ids).map_err(ref_err)?;
  Ok(())
}

fn clear_refs(conn: &Connection, id: &str) -> Result<(), PlanError> {
  let owner = Owner::Plan.as_str();
  crate::member::clear(conn, owner, id).map_err(|_| PlanError::Internal)?;
  crate::tag::clear(conn, owner, id).map_err(|_| PlanError::Internal)?;
  crate::place::clear(conn, owner, id).map_err(|_| PlanError::Internal)?;
  crate::media::clear(conn, owner, id).map_err(|_| PlanError::Internal)?;
  Ok(())
}

fn ref_err<E: std::fmt::Display>(err: E) -> PlanError {
  let text = err.to_string();
  if text == "引用不存在" { PlanError::ReferencedMissing } else { PlanError::Internal }
}
