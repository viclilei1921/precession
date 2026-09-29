//! 计划、清单、步骤、重复、提醒和评论。成员、标签、地点和媒体由各自模块写入。

use rusqlite::{Connection, OptionalExtension, params, types::ToSql};

use super::constants::{
  PLAN_COMMENT_TABLE, PLAN_GROUP_TABLE, PLAN_REMINDER_TABLE, PLAN_REPEAT_TABLE, PLAN_STEP_TABLE, PLAN_TABLE,
  SCHEMA_VERSION, SCHEMA_VERSION_KEY,
};
use super::dto::{
  Plan, PlanComment, PlanGroup, PlanGroupInput, PlanGroupPatch, PlanInput, PlanPatch, PlanReminder, PlanReminderInput,
  PlanRepeat, PlanRepeatInput, PlanStep, PlanStepInput,
};
use super::error::PlanError;
use crate::owner::Owner;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> PlanError {
  tauri_plugin_log::log::error!("{context}: {err}");
  PlanError::Internal
}

fn is_constraint(err: &rusqlite::Error) -> bool {
  matches!(err, rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation)
}

const COLUMNS: &str = "id, title, body, status, priority, scheduled_at, due_at, completed_at, result, locked, highlight, group_id, parent_id, all_day, archived, sort, time_zone, created_at, updated_at";

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
    group_id: row.get(11)?,
    parent_id: row.get(12)?,
    all_day: row.get(13)?,
    archived: row.get(14)?,
    sort: row.get(15)?,
    time_zone: row.get(16)?,
    steps: Vec::new(),
    member_ids: Vec::new(),
    tag_ids: Vec::new(),
    place_ids: Vec::new(),
    repeat: None,
    reminders: Vec::new(),
    media: Vec::new(),
    comments: Vec::new(),
    created_at: row.get(17)?,
    updated_at: row.get(18)?,
  })
}

/// 建计划相关的表。时间都是 Unix 毫秒。布尔用 `0` / `1`。
/// 查询只返回 `deleted_at` 为空的行。版本保持 1。
///
/// ## plan
///
/// 一行是一条计划。附件不在这张表里，经 `owner = "plan"` 挂在 `media` 上。
///
/// | 列 | 类型 | 约束 | 含义 |
/// | --- | --- | --- | --- |
/// | id | TEXT | 主键 | UUID |
/// | title | TEXT | 非空 | 标题 |
/// | body | TEXT | 非空，默认 `''` | 正文 |
/// | status | TEXT | 非空，默认 `draft` | `draft` 草稿、`scheduled` 已安排、`done` 已完成 |
/// | priority | INTEGER | 非空，默认 `0` | 优先级 |
/// | scheduled_at | INTEGER | 可空 | 安排时间；列表排序时缺它就用 `created_at` |
/// | due_at | INTEGER | 可空 | 截止时间 |
/// | completed_at | INTEGER | 可空 | 完成时间，只在完成时写入 |
/// | result | TEXT | 非空，默认 `''` | 完成结果 |
/// | locked | INTEGER | 非空，默认 `0` | 是否锁定 |
/// | highlight | INTEGER | 非空，默认 `0` | 是否高亮 |
/// | group_id | TEXT | 可空 | 所属清单 |
/// | parent_id | TEXT | 可空 | 父计划 |
/// | all_day | INTEGER | 非空，默认 `0` | 是否全天 |
/// | archived | INTEGER | 非空，默认 `0` | 是否归档 |
/// | sort | INTEGER | 非空，默认 `0` | 清单内的手动顺序 |
/// | time_zone | TEXT | 非空，默认 `''` | 时区；空表示用本机时区 |
/// | created_at | INTEGER | 非空 | 创建时间 |
/// | updated_at | INTEGER | 非空 | 更新时间 |
/// | deleted_at | INTEGER | 可空 | 软删除时间；空表示仍在 |
///
/// 索引：`plan_scheduled (scheduled_at) WHERE deleted_at IS NULL`，
/// `plan_group_sort (group_id, sort) WHERE deleted_at IS NULL`，
/// `plan_due (due_at) WHERE deleted_at IS NULL AND archived = 0 AND status != 'done'`，
/// `plan_parent (parent_id) WHERE parent_id IS NOT NULL`。
///
/// ## plan_step
///
/// 一行是计划下的一个步骤，`plan_id` 指向 `plan.id`。
/// 传来步骤时整份替换，`sort` 按写入顺序从 `0` 起。删除计划时一并清掉。
///
/// | 列 | 类型 | 约束 | 含义 |
/// | --- | --- | --- | --- |
/// | id | TEXT | 主键 | UUID |
/// | plan_id | TEXT | 非空 | 所属计划 |
/// | title | TEXT | 非空 | 步骤标题 |
/// | done | INTEGER | 非空，默认 `0` | 是否完成 |
/// | sort | INTEGER | 非空，默认 `0` | 排序，从小到大 |
///
/// 索引：`plan_step_plan (plan_id, sort)`。
///
/// ## plan_group
///
/// 一行是一份清单。删除清单只摘掉计划上的 `group_id`，不删计划。
///
/// | 列 | 类型 | 约束 | 含义 |
/// | --- | --- | --- | --- |
/// | id | TEXT | 主键 | UUID |
/// | name | TEXT | 非空 | 名称；未删除的名字唯一 |
/// | color | TEXT | 非空，默认 `''` | 颜色 |
/// | system | INTEGER | 非空，默认 `0` | 系统清单不可删除 |
/// | sort | INTEGER | 非空，默认 `0` | 排序 |
/// | created_at | INTEGER | 非空 | 创建时间 |
/// | updated_at | INTEGER | 非空 | 更新时间 |
/// | deleted_at | INTEGER | 可空 | 软删除时间 |
///
/// 索引：`plan_group_name (name) WHERE deleted_at IS NULL`，
/// `plan_group_order (sort) WHERE deleted_at IS NULL`。
///
/// ## plan_repeat
///
/// 一条计划最多一行。计划自己就是重复模板，不生成实例。
///
/// | 列 | 类型 | 约束 | 含义 |
/// | --- | --- | --- | --- |
/// | id | TEXT | 主键 | UUID |
/// | plan_id | TEXT | 非空，唯一 | 所属计划 |
/// | kind | TEXT | 非空 | `daily` / `weekly` / `monthly` / `yearly` |
/// | interval | INTEGER | 非空，默认 `1` | 间隔，至少为 `1` |
/// | weekdays | TEXT | 非空，默认 `''` | 每周重复的星期，例如 `1,3,5` |
/// | until_at | INTEGER | 可空 | 截止 |
/// | created_at | INTEGER | 非空 | 创建时间 |
/// | updated_at | INTEGER | 非空 | 更新时间 |
///
/// ## plan_reminder
///
/// 一行是一次提醒。这里只存记录，不到点弹出。
///
/// | 列 | 类型 | 约束 | 含义 |
/// | --- | --- | --- | --- |
/// | id | TEXT | 主键 | UUID |
/// | plan_id | TEXT | 非空 | 所属计划 |
/// | remind_at | INTEGER | 非空 | 提醒时间 |
/// | note | TEXT | 非空，默认 `''` | 说明 |
/// | triggered | INTEGER | 非空，默认 `0` | 是否已触发 |
/// | triggered_at | INTEGER | 可空 | 触发时间 |
/// | created_at | INTEGER | 非空 | 创建时间 |
///
/// 索引：`plan_reminder_plan (plan_id)`，`plan_reminder_due (remind_at) WHERE triggered = 0`。
///
/// ## plan_comment
///
/// 一行是一条评论。没有作者。删除是硬删除。
///
/// | 列 | 类型 | 约束 | 含义 |
/// | --- | --- | --- | --- |
/// | id | TEXT | 主键 | UUID |
/// | plan_id | TEXT | 非空 | 所属计划 |
/// | body | TEXT | 非空 | 正文 |
/// | created_at | INTEGER | 非空 | 创建时间 |
/// | updated_at | INTEGER | 非空 | 更新时间 |
///
/// 索引：`plan_comment_plan (plan_id)`。
///
/// 成员、标签、地点、媒体不在这些表里，经 `owner = "plan"` 挂在各自的表上。
pub(super) fn ensure_schema(conn: &Connection) -> Result<(), PlanError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("plan schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS plan (
           id TEXT PRIMARY KEY NOT NULL,
           title TEXT NOT NULL,
           body TEXT NOT NULL DEFAULT '',
           status TEXT NOT NULL DEFAULT 'draft',
           priority INTEGER NOT NULL DEFAULT 0,
           scheduled_at INTEGER,
           due_at INTEGER,
           completed_at INTEGER,
           result TEXT NOT NULL DEFAULT '',
           locked INTEGER NOT NULL DEFAULT 0,
           highlight INTEGER NOT NULL DEFAULT 0,
           group_id TEXT,
           parent_id TEXT,
           all_day INTEGER NOT NULL DEFAULT 0,
           archived INTEGER NOT NULL DEFAULT 0,
           sort INTEGER NOT NULL DEFAULT 0,
           time_zone TEXT NOT NULL DEFAULT '',
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS plan_scheduled ON plan(scheduled_at) WHERE deleted_at IS NULL;
         CREATE INDEX IF NOT EXISTS plan_group_sort ON plan(group_id, sort) WHERE deleted_at IS NULL;
         CREATE INDEX IF NOT EXISTS plan_due ON plan(due_at) WHERE deleted_at IS NULL AND archived = 0 AND status != 'done';
         CREATE INDEX IF NOT EXISTS plan_parent ON plan(parent_id) WHERE parent_id IS NOT NULL;
         CREATE TABLE IF NOT EXISTS plan_step (
           id TEXT PRIMARY KEY NOT NULL,
           plan_id TEXT NOT NULL,
           title TEXT NOT NULL,
           done INTEGER NOT NULL DEFAULT 0,
           sort INTEGER NOT NULL DEFAULT 0
         );
         CREATE INDEX IF NOT EXISTS plan_step_plan ON plan_step(plan_id, sort);
         CREATE TABLE IF NOT EXISTS plan_group (
           id TEXT PRIMARY KEY NOT NULL,
           name TEXT NOT NULL,
           color TEXT NOT NULL DEFAULT '',
           system INTEGER NOT NULL DEFAULT 0,
           sort INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE UNIQUE INDEX IF NOT EXISTS plan_group_name ON plan_group(name) WHERE deleted_at IS NULL;
         CREATE INDEX IF NOT EXISTS plan_group_order ON plan_group(sort) WHERE deleted_at IS NULL;
         CREATE TABLE IF NOT EXISTS plan_repeat (
           id TEXT PRIMARY KEY NOT NULL,
           plan_id TEXT NOT NULL,
           kind TEXT NOT NULL,
           interval INTEGER NOT NULL DEFAULT 1,
           weekdays TEXT NOT NULL DEFAULT '',
           until_at INTEGER,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL
         );
         CREATE UNIQUE INDEX IF NOT EXISTS plan_repeat_plan ON plan_repeat(plan_id);
         CREATE TABLE IF NOT EXISTS plan_reminder (
           id TEXT PRIMARY KEY NOT NULL,
           plan_id TEXT NOT NULL,
           remind_at INTEGER NOT NULL,
           note TEXT NOT NULL DEFAULT '',
           triggered INTEGER NOT NULL DEFAULT 0,
           triggered_at INTEGER,
           created_at INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS plan_reminder_plan ON plan_reminder(plan_id);
         CREATE INDEX IF NOT EXISTS plan_reminder_due ON plan_reminder(remind_at) WHERE triggered = 0;
         CREATE TABLE IF NOT EXISTS plan_comment (
           id TEXT PRIMARY KEY NOT NULL,
           plan_id TEXT NOT NULL,
           body TEXT NOT NULL,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS plan_comment_plan ON plan_comment(plan_id);",
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

pub(super) fn plan_exists(conn: &Connection, id: &str) -> Result<bool, PlanError> {
  exists(conn, id).map_err(|e| db_fail("plan exists", e))
}

pub(super) fn group_exists(conn: &Connection, id: &str) -> Result<bool, PlanError> {
  let sql = format!("SELECT 1 FROM {PLAN_GROUP_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  let found: Option<i64> =
    conn.query_row(&sql, [id], |row| row.get(0)).optional().map_err(|e| db_fail("plan group", e))?;
  Ok(found.is_some())
}

/// 计划不存在时返回 `None`。存在时返回它的父计划，没有父计划则是 `Some(None)`。
pub(super) fn parent_of(conn: &Connection, id: &str) -> Result<Option<Option<String>>, PlanError> {
  conn
    .query_row("SELECT parent_id FROM plan WHERE id = ?1 AND deleted_at IS NULL", [id], |row| {
      row.get(0)
    })
    .optional()
    .map_err(|e| db_fail("plan parent", e))
}

pub(super) fn list(conn: &Connection) -> Result<Vec<Plan>, PlanError> {
  query(conn, None, None, None)
}

pub(super) fn list_between(conn: &Connection, from: Option<i64>, to: Option<i64>) -> Result<Vec<Plan>, PlanError> {
  query(conn, None, from, to)
}

pub(super) fn get(conn: &Connection, id: &str) -> Result<Plan, PlanError> {
  let mut plan = one(conn, id)?;
  plan.comments = comments_for(conn, id)?;
  Ok(plan)
}

pub(super) fn create(conn: &Connection, input: &PlanInput) -> Result<Plan, PlanError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan create tx", e))?;
  tx.execute(
    "INSERT INTO plan
       (id, title, body, status, priority, scheduled_at, due_at, result, locked, highlight,
        group_id, parent_id, all_day, archived, sort, time_zone, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?17)",
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
      input.group_id,
      input.parent_id,
      input.all_day,
      input.archived,
      input.sort,
      input.time_zone,
      now
    ],
  )
  .map_err(|e| db_fail("plan create", e))?;
  replace_steps(&tx, &id, &input.steps)?;
  replace_refs(&tx, &id, Some(&input.member_ids), Some(&input.tag_ids), Some(&input.place_ids))?;
  if let Some(repeat) = &input.repeat {
    write_repeat(&tx, &id, repeat, now)?;
  }
  replace_reminders(&tx, &id, &input.reminders, now)?;
  tx.commit().map_err(|e| db_fail("plan create commit", e))?;
  get(conn, &id)
}

pub(super) fn update(conn: &Connection, id: &str, patch: &PlanPatch) -> Result<Plan, PlanError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan update tx", e))?;
  let n = apply_patch(&tx, id, patch, now)?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  if let Some(steps) = &patch.steps {
    replace_steps(&tx, id, steps)?;
  }
  replace_refs(
    &tx,
    id,
    patch.member_ids.as_deref(),
    patch.tag_ids.as_deref(),
    patch.place_ids.as_deref(),
  )?;
  if let Some(repeat) = &patch.repeat {
    write_repeat(&tx, id, repeat, now)?;
  }
  if let Some(reminders) = &patch.reminders {
    replace_reminders(&tx, id, reminders, now)?;
  }
  tx.commit().map_err(|e| db_fail("plan update commit", e))?;
  get(conn, id)
}

pub(super) fn complete(
  conn: &Connection,
  id: &str,
  status: &str,
  result: &str,
  completed_at: i64,
) -> Result<Plan, PlanError> {
  let sql = format!(
    "UPDATE {PLAN_TABLE}
     SET status = ?1, result = ?2, completed_at = ?3, updated_at = ?3
     WHERE id = ?4 AND deleted_at IS NULL"
  );
  let n = conn.execute(&sql, params![status, result, completed_at, id]).map_err(|e| db_fail("plan complete", e))?;
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
  tx.execute(
    "UPDATE plan SET parent_id = NULL, updated_at = ?1 WHERE parent_id = ?2 AND deleted_at IS NULL",
    params![now, id],
  )
  .map_err(|e| db_fail("plan children", e))?;
  for table in [PLAN_STEP_TABLE, PLAN_REPEAT_TABLE, PLAN_REMINDER_TABLE, PLAN_COMMENT_TABLE] {
    let sql = format!("DELETE FROM {table} WHERE plan_id = ?1");
    tx.execute(&sql, [id]).map_err(|e| db_fail("plan children clear", e))?;
  }
  clear_refs(&tx, id)?;
  tx.commit().map_err(|e| db_fail("plan delete commit", e))?;
  Ok(())
}

pub(super) fn group_list(conn: &Connection) -> Result<Vec<PlanGroup>, PlanError> {
  let sql = format!(
    "SELECT id, name, color, system, sort, created_at, updated_at
     FROM {PLAN_GROUP_TABLE} WHERE deleted_at IS NULL ORDER BY sort, name"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("plan group list", e))?;
  let rows = stmt.query_map([], map_group).map_err(|e| db_fail("plan group list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("plan group list", e))
}

pub(super) fn group_get(conn: &Connection, id: &str) -> Result<PlanGroup, PlanError> {
  let sql = format!(
    "SELECT id, name, color, system, sort, created_at, updated_at
     FROM {PLAN_GROUP_TABLE} WHERE id = ?1 AND deleted_at IS NULL"
  );
  conn.query_row(&sql, [id], map_group).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => PlanError::NotFound,
    other => db_fail("plan group get", other),
  })
}

pub(super) fn group_create(conn: &Connection, input: &PlanGroupInput) -> Result<PlanGroup, PlanError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!(
    "INSERT INTO {PLAN_GROUP_TABLE} (id, name, color, system, sort, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)"
  );
  conn.execute(&sql, params![id, input.name, input.color, input.system, input.sort, now]).map_err(|e| {
    if is_constraint(&e) { PlanError::NameTaken } else { db_fail("plan group create", e) }
  })?;
  group_get(conn, &id)
}

pub(super) fn group_update(conn: &Connection, id: &str, patch: &PlanGroupPatch) -> Result<PlanGroup, PlanError> {
  let now = now_unix_ms();
  let mut sets = Sets::default();
  if let Some(name) = &patch.name {
    sets.set("name", name.clone());
  }
  if let Some(color) = &patch.color {
    sets.set("color", color.clone());
  }
  if let Some(sort) = patch.sort {
    sets.set("sort", sort);
  }
  sets.set("updated_at", now);
  let sql = format!(
    "UPDATE {PLAN_GROUP_TABLE} SET {} WHERE id = ?{} AND deleted_at IS NULL",
    sets.sql.join(", "),
    sets.values.len() + 1
  );
  sets.values.push(Box::new(id.to_string()));
  let n = exec(conn, &sql, &sets.values).map_err(|e| {
    if is_constraint(&e) { PlanError::NameTaken } else { db_fail("plan group update", e) }
  })?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  group_get(conn, id)
}

pub(super) fn group_delete(conn: &Connection, id: &str) -> Result<(), PlanError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan group delete tx", e))?;
  tx.execute(
    "UPDATE plan SET group_id = NULL, updated_at = ?1 WHERE group_id = ?2",
    params![now, id],
  )
  .map_err(|e| db_fail("plan group detach", e))?;
  let sql =
    format!("UPDATE {PLAN_GROUP_TABLE} SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL");
  let n = tx.execute(&sql, params![now, id]).map_err(|e| db_fail("plan group delete", e))?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  tx.commit().map_err(|e| db_fail("plan group delete commit", e))?;
  Ok(())
}

pub(super) fn comment_create(conn: &Connection, plan_id: &str, body: &str) -> Result<PlanComment, PlanError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql =
    format!("INSERT INTO {PLAN_COMMENT_TABLE} (id, plan_id, body, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)");
  conn.execute(&sql, params![id, plan_id, body, now]).map_err(|e| db_fail("plan comment create", e))?;
  comment_get(conn, &id)
}

pub(super) fn comment_update(conn: &Connection, id: &str, body: &str) -> Result<PlanComment, PlanError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {PLAN_COMMENT_TABLE} SET body = ?1, updated_at = ?2 WHERE id = ?3");
  let n = conn.execute(&sql, params![body, now, id]).map_err(|e| db_fail("plan comment update", e))?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  comment_get(conn, id)
}

pub(super) fn comment_delete(conn: &Connection, id: &str) -> Result<(), PlanError> {
  let sql = format!("DELETE FROM {PLAN_COMMENT_TABLE} WHERE id = ?1");
  let n = conn.execute(&sql, [id]).map_err(|e| db_fail("plan comment delete", e))?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  Ok(())
}

fn one(conn: &Connection, id: &str) -> Result<Plan, PlanError> {
  let mut items = query(conn, Some(id), None, None)?;
  items.pop().ok_or(PlanError::NotFound)
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
    fill(conn, item)?;
  }
  Ok(items)
}

fn fill(conn: &Connection, plan: &mut Plan) -> Result<(), PlanError> {
  plan.steps = steps_for(conn, &plan.id)?;
  plan.repeat = repeat_for(conn, &plan.id)?;
  plan.reminders = reminders_for(conn, &plan.id)?;
  let owner = Owner::Plan.as_str();
  plan.member_ids = crate::member::list_ids(conn, owner, &plan.id).map_err(|_| PlanError::Internal)?;
  plan.tag_ids = crate::tag::list_ids(conn, owner, &plan.id).map_err(|_| PlanError::Internal)?;
  plan.place_ids = crate::place::list_ids(conn, owner, &plan.id).map_err(|_| PlanError::Internal)?;
  plan.media = crate::media::list_for(conn, owner, &plan.id).map_err(|_| PlanError::Internal)?;
  Ok(())
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

fn repeat_for(conn: &Connection, plan_id: &str) -> Result<Option<PlanRepeat>, PlanError> {
  let sql = format!("SELECT id, kind, interval, weekdays, until_at FROM {PLAN_REPEAT_TABLE} WHERE plan_id = ?1");
  conn
    .query_row(&sql, [plan_id], |row| {
      Ok(PlanRepeat {
        id: row.get(0)?,
        kind: row.get(1)?,
        interval: row.get(2)?,
        weekdays: row.get(3)?,
        until_at: row.get(4)?,
      })
    })
    .optional()
    .map_err(|e| db_fail("plan repeat", e))
}

fn reminders_for(conn: &Connection, plan_id: &str) -> Result<Vec<PlanReminder>, PlanError> {
  let sql = format!(
    "SELECT id, remind_at, note, triggered, triggered_at, created_at
     FROM {PLAN_REMINDER_TABLE} WHERE plan_id = ?1 ORDER BY remind_at, created_at"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("plan reminders", e))?;
  let rows = stmt
    .query_map([plan_id], |row| {
      Ok(PlanReminder {
        id: row.get(0)?,
        remind_at: row.get(1)?,
        note: row.get(2)?,
        triggered: row.get(3)?,
        triggered_at: row.get(4)?,
        created_at: row.get(5)?,
      })
    })
    .map_err(|e| db_fail("plan reminders", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("plan reminders", e))
}

fn comments_for(conn: &Connection, plan_id: &str) -> Result<Vec<PlanComment>, PlanError> {
  let sql =
    format!("SELECT id, body, created_at, updated_at FROM {PLAN_COMMENT_TABLE} WHERE plan_id = ?1 ORDER BY created_at");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("plan comments", e))?;
  let rows = stmt
    .query_map([plan_id], |row| {
      Ok(PlanComment { id: row.get(0)?, body: row.get(1)?, created_at: row.get(2)?, updated_at: row.get(3)? })
    })
    .map_err(|e| db_fail("plan comments", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("plan comments", e))
}

fn comment_get(conn: &Connection, id: &str) -> Result<PlanComment, PlanError> {
  let sql = format!("SELECT id, body, created_at, updated_at FROM {PLAN_COMMENT_TABLE} WHERE id = ?1");
  conn
    .query_row(&sql, [id], |row| {
      Ok(PlanComment { id: row.get(0)?, body: row.get(1)?, created_at: row.get(2)?, updated_at: row.get(3)? })
    })
    .map_err(|e| match e {
      rusqlite::Error::QueryReturnedNoRows => PlanError::NotFound,
      other => db_fail("plan comment get", other),
    })
}

fn map_group(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlanGroup> {
  Ok(PlanGroup {
    id: row.get(0)?,
    name: row.get(1)?,
    color: row.get(2)?,
    system: row.get(3)?,
    sort: row.get(4)?,
    created_at: row.get(5)?,
    updated_at: row.get(6)?,
  })
}

fn replace_steps(conn: &Connection, plan_id: &str, steps: &[PlanStepInput]) -> Result<(), PlanError> {
  let sql = format!("DELETE FROM {PLAN_STEP_TABLE} WHERE plan_id = ?1");
  conn.execute(&sql, [plan_id]).map_err(|e| db_fail("plan steps", e))?;
  let sql = format!("INSERT INTO {PLAN_STEP_TABLE} (id, plan_id, title, done, sort) VALUES (?1, ?2, ?3, ?4, ?5)");
  for (index, step) in steps.iter().enumerate() {
    conn
      .execute(&sql, params![new_uuid_v4(), plan_id, step.title, step.done, index as i64])
      .map_err(|e| db_fail("plan step", e))?;
  }
  Ok(())
}

fn replace_reminders(
  conn: &Connection,
  plan_id: &str,
  reminders: &[PlanReminderInput],
  now: i64,
) -> Result<(), PlanError> {
  let sql = format!("DELETE FROM {PLAN_REMINDER_TABLE} WHERE plan_id = ?1");
  conn.execute(&sql, [plan_id]).map_err(|e| db_fail("plan reminders", e))?;
  let sql = format!(
    "INSERT INTO {PLAN_REMINDER_TABLE} (id, plan_id, remind_at, note, triggered, created_at)
     VALUES (?1, ?2, ?3, ?4, 0, ?5)"
  );
  for reminder in reminders {
    conn
      .execute(&sql, params![new_uuid_v4(), plan_id, reminder.remind_at, reminder.note, now])
      .map_err(|e| db_fail("plan reminder", e))?;
  }
  Ok(())
}

fn write_repeat(conn: &Connection, plan_id: &str, repeat: &PlanRepeatInput, now: i64) -> Result<(), PlanError> {
  let sql = format!("DELETE FROM {PLAN_REPEAT_TABLE} WHERE plan_id = ?1");
  conn.execute(&sql, [plan_id]).map_err(|e| db_fail("plan repeat", e))?;
  let sql = format!(
    "INSERT INTO {PLAN_REPEAT_TABLE} (id, plan_id, kind, interval, weekdays, until_at, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)"
  );
  conn
    .execute(
      &sql,
      params![new_uuid_v4(), plan_id, repeat.kind, repeat.interval, repeat.weekdays, repeat.until_at, now],
    )
    .map_err(|e| db_fail("plan repeat", e))?;
  Ok(())
}

fn replace_refs(
  conn: &Connection,
  id: &str,
  member_ids: Option<&[String]>,
  tag_ids: Option<&[String]>,
  place_ids: Option<&[String]>,
) -> Result<(), PlanError> {
  let owner = Owner::Plan.as_str();
  if let Some(ids) = member_ids {
    crate::member::replace(conn, owner, id, ids).map_err(ref_err)?;
  }
  if let Some(ids) = tag_ids {
    crate::tag::replace(conn, owner, id, ids).map_err(ref_err)?;
  }
  if let Some(ids) = place_ids {
    crate::place::replace(conn, owner, id, ids).map_err(ref_err)?;
  }
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

fn apply_patch(conn: &Connection, id: &str, patch: &PlanPatch, now: i64) -> Result<usize, PlanError> {
  let mut sets = Sets::default();
  if let Some(title) = &patch.title {
    sets.set("title", title.clone());
  }
  if let Some(body) = &patch.body {
    sets.set("body", body.clone());
  }
  if let Some(status) = &patch.status {
    sets.set("status", status.clone());
  }
  if let Some(priority) = patch.priority {
    sets.set("priority", priority);
  }
  if let Some(scheduled_at) = patch.scheduled_at {
    sets.set("scheduled_at", scheduled_at);
  }
  if let Some(due_at) = patch.due_at {
    sets.set("due_at", due_at);
  }
  if let Some(result) = &patch.result {
    sets.set("result", result.clone());
  }
  if let Some(locked) = patch.locked {
    sets.set("locked", locked);
  }
  if let Some(highlight) = patch.highlight {
    sets.set("highlight", highlight);
  }
  if let Some(group_id) = &patch.group_id {
    sets.set("group_id", group_id.clone());
  }
  if let Some(parent_id) = &patch.parent_id {
    sets.set("parent_id", parent_id.clone());
  }
  if let Some(all_day) = patch.all_day {
    sets.set("all_day", all_day);
  }
  if let Some(archived) = patch.archived {
    sets.set("archived", archived);
  }
  if let Some(sort) = patch.sort {
    sets.set("sort", sort);
  }
  if let Some(time_zone) = &patch.time_zone {
    sets.set("time_zone", time_zone.clone());
  }
  sets.set("updated_at", now);
  let sql = format!(
    "UPDATE {PLAN_TABLE} SET {} WHERE id = ?{} AND deleted_at IS NULL",
    sets.sql.join(", "),
    sets.values.len() + 1
  );
  sets.values.push(Box::new(id.to_string()));
  exec(conn, &sql, &sets.values).map_err(|e| db_fail("plan update", e))
}

#[derive(Default)]
struct Sets {
  sql: Vec<String>,
  values: Vec<Box<dyn ToSql>>,
}

impl Sets {
  fn set<T: ToSql + 'static>(&mut self, column: &str, value: T) {
    self.values.push(Box::new(value));
    self.sql.push(format!("{column} = ?{}", self.values.len()));
  }
}

fn exec(conn: &Connection, sql: &str, values: &[Box<dyn ToSql>]) -> Result<usize, rusqlite::Error> {
  let refs: Vec<&dyn ToSql> = values.iter().map(|value| value.as_ref()).collect();
  conn.execute(sql, refs.as_slice())
}

fn ref_err<E: std::fmt::Display>(err: E) -> PlanError {
  let text = err.to_string();
  if text == "引用不存在" { PlanError::ReferencedMissing } else { PlanError::Internal }
}
