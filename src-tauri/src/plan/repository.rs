//! 计划、清单、步骤、重复、提醒和评论。成员、标签、地点和媒体由各自模块写入。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{
  PLAN_COMMENT_TABLE, PLAN_GROUP_TABLE, PLAN_REMINDER_TABLE, PLAN_REPEAT_TABLE, PLAN_STEP_TABLE, PLAN_TABLE,
  SCHEMA_VERSION, SCHEMA_VERSION_KEY,
};
use super::dto::{
  Plan, PlanComment, PlanGroup, PlanGroupInput, PlanGroupPatch, PlanInput, PlanPatch, PlanQuery, PlanReminder,
  PlanReminderInput, PlanRepeat, PlanRepeatInput, PlanStep, PlanStepInput,
};
use super::error::PlanError;
use crate::owner::Owner;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

/// 按架构版本创建计划相关表
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 读版本、建表或写版本失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// ensure_schema(conn)?;
/// ```
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

/// 记录数据库错误并返回内部错误
///
/// # 参数
/// - `context`: 日志中的操作说明
/// - `err`: 原始数据库错误
///
/// # 返回值
/// 返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let err = db_fail("plan", rusqlite_err);
/// ```
fn db_fail(context: &str, err: rusqlite::Error) -> PlanError {
  tauri_plugin_log::log::error!("{context}: {err}");
  PlanError::Internal
}

/// 判断数据库错误是否为约束冲突
///
/// # 参数
/// - `err`: 数据库错误
///
/// # 返回值
/// 错误码为 `ConstraintViolation` 时返回 true，否则返回 false
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let hit = is_constraint(&err);
/// ```
fn is_constraint(err: &rusqlite::Error) -> bool {
  matches!(err, rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::ConstraintViolation)
}

/// 查询未删除计划是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键
///
/// # 返回值
/// 存在时返回 true，不存在时返回 false
///
/// # 错误
/// 查询失败时返回 `rusqlite::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let found = exists(conn, id)?;
/// ```
pub(super) fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  let found: Option<i64> = conn
    .query_row("SELECT 1 FROM plan WHERE id = ?1 AND deleted_at IS NULL", [id], |row| {
      row.get(0)
    })
    .optional()?;
  Ok(found.is_some())
}

/// 查询未删除计划是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键
///
/// # 返回值
/// 存在时返回 true，不存在时返回 false
///
/// # 错误
/// 查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let found = plan_exists(conn, id)?;
/// ```
pub(super) fn plan_exists(conn: &Connection, id: &str) -> Result<bool, PlanError> {
  exists(conn, id).map_err(|e| db_fail("plan exists", e))
}

/// 查询未删除清单是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 清单主键
///
/// # 返回值
/// 存在时返回 true，不存在时返回 false
///
/// # 错误
/// 查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let found = group_exists(conn, id)?;
/// ```
pub(super) fn group_exists(conn: &Connection, id: &str) -> Result<bool, PlanError> {
  let sql = format!("SELECT 1 FROM {PLAN_GROUP_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  let found: Option<i64> =
    conn.query_row(&sql, [id], |row| row.get(0)).optional().map_err(|e| db_fail("plan group", e))?;
  Ok(found.is_some())
}

/// 读取未删除计划的父计划
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键
///
/// # 返回值
/// 计划不存在时返回 `None`；没有父计划时返回 `Some(None)`；有父计划时返回 `Some(Some(父主键))`
///
/// # 错误
/// 查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let parent = parent_of(conn, id)?;
/// ```
pub(super) fn parent_of(conn: &Connection, id: &str) -> Result<Option<Option<String>>, PlanError> {
  conn
    .query_row("SELECT parent_id FROM plan WHERE id = ?1 AND deleted_at IS NULL", [id], |row| {
      row.get(0)
    })
    .optional()
    .map_err(|e| db_fail("plan parent", e))
}

/// 按条件列出未删除计划
///
/// # 参数
/// - `conn`: 数据库连接
/// - `filter`: 时间范围和清单条件
///
/// # 返回值
/// 返回计划列表，并带上步骤、重复、提醒、成员、标签、地点和媒体
///
/// # 错误
/// 查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plans = list(conn, filter)?;
/// ```
pub(super) fn list(conn: &Connection, filter: &PlanQuery) -> Result<Vec<Plan>, PlanError> {
  query(conn, None, filter.from, filter.to, filter.group_id.as_deref())
}

/// 按时间范围列出未删除计划
///
/// # 参数
/// - `conn`: 数据库连接
/// - `from`: 起始时间，空则不限制
/// - `to`: 结束时间，空则不限制
///
/// # 返回值
/// 返回范围内的计划列表
///
/// # 错误
/// 查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plans = list_between(conn, from, to)?;
/// ```
pub(super) fn list_between(conn: &Connection, from: Option<i64>, to: Option<i64>) -> Result<Vec<Plan>, PlanError> {
  query(conn, None, from, to, None)
}

/// 按主键读取未删除计划并附带评论
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键
///
/// # 返回值
/// 返回计划
///
/// # 错误
/// 没有对应行时返回 `PlanError::NotFound`，查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plan = get(conn, id)?;
/// ```
pub(super) fn get(conn: &Connection, id: &str) -> Result<Plan, PlanError> {
  let mut items = query(conn, Some(id), None, None, None)?;
  let mut plan = items.pop().ok_or(PlanError::NotFound)?;
  plan.comments = comments_for(conn, id)?;
  Ok(plan)
}

/// 插入计划及其步骤、引用、重复和提醒
///
/// # 参数
/// - `conn`: 数据库连接
/// - `input`: 新建计划内容
///
/// # 返回值
/// 返回写入后的计划
///
/// # 错误
/// 写入或回读失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plan = create(conn, input)?;
/// ```
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
    replace_repeat(&tx, &id, repeat, now)?;
  }
  replace_reminders(&tx, &id, &input.reminders, now)?;
  tx.commit().map_err(|e| db_fail("plan create commit", e))?;
  get(conn, &id)
}

/// 用补丁更新未删除计划，并按需替换步骤、引用、重复和提醒
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键
/// - `patch`: 要改的字段，空字段保持原值
///
/// # 返回值
/// 返回更新后的计划
///
/// # 错误
/// 没有对应行时返回 `PlanError::NotFound`，写入失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plan = update(conn, id, patch)?;
/// ```
pub(super) fn update(conn: &Connection, id: &str, patch: &PlanPatch) -> Result<Plan, PlanError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("plan update tx", e))?;
  let sql = format!(
    "UPDATE {PLAN_TABLE}
     SET title = COALESCE(?1, title),
         body = COALESCE(?2, body),
         status = COALESCE(?3, status),
         priority = COALESCE(?4, priority),
         scheduled_at = CASE WHEN ?18 THEN NULL ELSE COALESCE(?5, scheduled_at) END,
         due_at = CASE WHEN ?19 THEN NULL ELSE COALESCE(?6, due_at) END,
         result = COALESCE(?7, result),
         locked = COALESCE(?8, locked),
         highlight = COALESCE(?9, highlight),
         group_id = COALESCE(?10, group_id),
         parent_id = COALESCE(?11, parent_id),
         all_day = COALESCE(?12, all_day),
         archived = COALESCE(?13, archived),
         sort = COALESCE(?14, sort),
         time_zone = COALESCE(?15, time_zone),
         updated_at = ?16
     WHERE id = ?17 AND deleted_at IS NULL"
  );
  let n = tx
    .execute(
      &sql,
      params![
        patch.title,
        patch.body,
        patch.status,
        patch.priority,
        patch.scheduled_at,
        patch.due_at,
        patch.result,
        patch.locked,
        patch.highlight,
        patch.group_id,
        patch.parent_id,
        patch.all_day,
        patch.archived,
        patch.sort,
        patch.time_zone,
        now,
        id,
        patch.clear_scheduled_at,
        patch.clear_due_at
      ],
    )
    .map_err(|e| db_fail("plan update", e))?;
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
    replace_repeat(&tx, id, repeat, now)?;
  }
  if let Some(reminders) = &patch.reminders {
    replace_reminders(&tx, id, reminders, now)?;
  }
  tx.commit().map_err(|e| db_fail("plan update commit", e))?;
  get(conn, id)
}

/// 写入完成状态、结果和完成时间
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键
/// - `status`: 状态
/// - `result`: 完成结果
/// - `completed_at`: 完成时间
///
/// # 返回值
/// 返回更新后的计划
///
/// # 错误
/// 没有对应行时返回 `PlanError::NotFound`，写入失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plan = complete(conn, id, status, result, completed_at)?;
/// ```
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

/// 软删除计划，并清掉子计划父级、步骤、重复、提醒、评论和引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 没有对应行时返回 `PlanError::NotFound`，写入失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete(conn, id)?;
/// ```
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

/// 按排序和名称列出未删除清单
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 返回清单列表
///
/// # 错误
/// 查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let groups = group_list(conn)?;
/// ```
pub(super) fn group_list(conn: &Connection) -> Result<Vec<PlanGroup>, PlanError> {
  let sql = format!(
    "SELECT id, name, color, system, sort, created_at, updated_at
     FROM {PLAN_GROUP_TABLE} WHERE deleted_at IS NULL ORDER BY sort, name"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("plan group list", e))?;
  let rows = stmt
    .query_map([], |row| {
      Ok(PlanGroup {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        system: row.get(3)?,
        sort: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
      })
    })
    .map_err(|e| db_fail("plan group list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("plan group list", e))
}

/// 按主键读取未删除清单
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 清单主键
///
/// # 返回值
/// 返回清单
///
/// # 错误
/// 没有对应行时返回 `PlanError::NotFound`，查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let group = group_get(conn, id)?;
/// ```
pub(super) fn group_get(conn: &Connection, id: &str) -> Result<PlanGroup, PlanError> {
  let sql = format!(
    "SELECT id, name, color, system, sort, created_at, updated_at
     FROM {PLAN_GROUP_TABLE} WHERE id = ?1 AND deleted_at IS NULL"
  );
  conn
    .query_row(&sql, [id], |row| {
      Ok(PlanGroup {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        system: row.get(3)?,
        sort: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
      })
    })
    .map_err(|e| match e {
      rusqlite::Error::QueryReturnedNoRows => PlanError::NotFound,
      other => db_fail("plan group get", other),
    })
}

/// 插入清单并回读
///
/// # 参数
/// - `conn`: 数据库连接
/// - `input`: 新建清单内容
///
/// # 返回值
/// 返回新建的清单
///
/// # 错误
/// 名称冲突时返回 `PlanError::NameTaken`，其余写入失败返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let group = group_create(conn, input)?;
/// ```
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

/// 用补丁更新未删除清单
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 清单主键
/// - `patch`: 要改的名称、颜色和排序
///
/// # 返回值
/// 返回更新后的清单
///
/// # 错误
/// 名称冲突时返回 `PlanError::NameTaken`，没有对应行时返回 `PlanError::NotFound`，其余失败返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let group = group_update(conn, id, patch)?;
/// ```
pub(super) fn group_update(conn: &Connection, id: &str, patch: &PlanGroupPatch) -> Result<PlanGroup, PlanError> {
  let now = now_unix_ms();
  let sql = format!(
    "UPDATE {PLAN_GROUP_TABLE}
     SET name = COALESCE(?1, name),
         color = COALESCE(?2, color),
         sort = COALESCE(?3, sort),
         updated_at = ?4
     WHERE id = ?5 AND deleted_at IS NULL"
  );
  let n = conn.execute(&sql, params![patch.name, patch.color, patch.sort, now, id]).map_err(|e| {
    if is_constraint(&e) { PlanError::NameTaken } else { db_fail("plan group update", e) }
  })?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  group_get(conn, id)
}

/// 摘掉计划上的清单归属并软删除清单
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 清单主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 没有对应行时返回 `PlanError::NotFound`，写入失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// group_delete(conn, id)?;
/// ```
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

/// 插入评论并回读
///
/// # 参数
/// - `conn`: 数据库连接
/// - `plan_id`: 计划主键
/// - `body`: 评论正文
///
/// # 返回值
/// 返回新建的评论
///
/// # 错误
/// 写入或回读失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let comment = comment_create(conn, plan_id, body)?;
/// ```
pub(super) fn comment_create(conn: &Connection, plan_id: &str, body: &str) -> Result<PlanComment, PlanError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql =
    format!("INSERT INTO {PLAN_COMMENT_TABLE} (id, plan_id, body, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)");
  conn.execute(&sql, params![id, plan_id, body, now]).map_err(|e| db_fail("plan comment create", e))?;
  comment_get(conn, &id)
}

/// 更新评论正文并回读
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 评论主键
/// - `body`: 评论正文
///
/// # 返回值
/// 返回更新后的评论
///
/// # 错误
/// 没有对应行时返回 `PlanError::NotFound`，写入失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let comment = comment_update(conn, id, body)?;
/// ```
pub(super) fn comment_update(conn: &Connection, id: &str, body: &str) -> Result<PlanComment, PlanError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {PLAN_COMMENT_TABLE} SET body = ?1, updated_at = ?2 WHERE id = ?3");
  let n = conn.execute(&sql, params![body, now, id]).map_err(|e| db_fail("plan comment update", e))?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  comment_get(conn, id)
}

/// 按主键硬删除评论
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 评论主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 没有对应行时返回 `PlanError::NotFound`，删除失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// comment_delete(conn, id)?;
/// ```
pub(super) fn comment_delete(conn: &Connection, id: &str) -> Result<(), PlanError> {
  let sql = format!("DELETE FROM {PLAN_COMMENT_TABLE} WHERE id = ?1");
  let n = conn.execute(&sql, [id]).map_err(|e| db_fail("plan comment delete", e))?;
  if n == 0 {
    return Err(PlanError::NotFound);
  }
  Ok(())
}

/// 按主键、时间和清单查询未删除计划并填上关联数据
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键，空则不限
/// - `from`: 起始时间，含，空则不限
/// - `to`: 结束时间，不含，空则不限
/// - `group_id`: 清单主键，空则不限
///
/// # 返回值
/// 返回按安排时间或创建时间排序的计划，评论留空
///
/// # 错误
/// 查询关联数据失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plans = query(conn, id, from, to, group_id)?;
/// ```
fn query(
  conn: &Connection,
  id: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
  group_id: Option<&str>,
) -> Result<Vec<Plan>, PlanError> {
  let sql = format!(
    "SELECT id, title, body, status, priority, scheduled_at, due_at, completed_at, result, locked, highlight, group_id, parent_id, all_day, archived, sort, time_zone, created_at, updated_at
     FROM {PLAN_TABLE}
     WHERE deleted_at IS NULL
       AND (?1 IS NULL OR id = ?1)
       AND (?2 IS NULL OR COALESCE(scheduled_at, created_at) >= ?2)
       AND (?3 IS NULL OR COALESCE(scheduled_at, created_at) < ?3)
       AND (?4 IS NULL OR group_id = ?4)
     ORDER BY COALESCE(scheduled_at, created_at)"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("plan list", e))?;
  let rows = stmt
    .query_map(params![id, from, to, group_id], |row| {
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
        uploads: Vec::new(),
        comments: Vec::new(),
        created_at: row.get(17)?,
        updated_at: row.get(18)?,
      })
    })
    .map_err(|e| db_fail("plan list", e))?;
  let mut items = rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("plan list", e))?;
  let owner = Owner::Plan.as_str();
  for item in &mut items {
    item.steps = steps_for(conn, &item.id)?;
    item.repeat = repeat_for(conn, &item.id)?;
    item.reminders = reminders_for(conn, &item.id)?;
    item.member_ids = crate::member::list_ids(conn, owner, &item.id).map_err(|_| PlanError::Internal)?;
    item.tag_ids = crate::tag::list_ids(conn, owner, &item.id).map_err(|_| PlanError::Internal)?;
    item.place_ids = crate::place::list_ids(conn, owner, &item.id).map_err(|_| PlanError::Internal)?;
    item.uploads = crate::upload::list_for(conn, owner, &item.id).map_err(|_| PlanError::Internal)?;
  }
  Ok(items)
}

/// 按排序读取计划步骤
///
/// # 参数
/// - `conn`: 数据库连接
/// - `plan_id`: 计划主键
///
/// # 返回值
/// 返回步骤列表
///
/// # 错误
/// 查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let steps = steps_for(conn, plan_id)?;
/// ```
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

/// 读取计划的重复规则
///
/// # 参数
/// - `conn`: 数据库连接
/// - `plan_id`: 计划主键
///
/// # 返回值
/// 有规则时返回 `Some`，没有时返回 `None`
///
/// # 错误
/// 查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let repeat = repeat_for(conn, plan_id)?;
/// ```
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

/// 按提醒时间读取计划提醒
///
/// # 参数
/// - `conn`: 数据库连接
/// - `plan_id`: 计划主键
///
/// # 返回值
/// 返回提醒列表
///
/// # 错误
/// 查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let reminders = reminders_for(conn, plan_id)?;
/// ```
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

/// 按创建时间读取计划评论
///
/// # 参数
/// - `conn`: 数据库连接
/// - `plan_id`: 计划主键
///
/// # 返回值
/// 返回评论列表
///
/// # 错误
/// 查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let comments = comments_for(conn, plan_id)?;
/// ```
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

/// 按主键读取评论
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 评论主键
///
/// # 返回值
/// 返回评论
///
/// # 错误
/// 没有对应行时返回 `PlanError::NotFound`，查询失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let comment = comment_get(conn, id)?;
/// ```
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

/// 删掉原步骤并按传入顺序重新插入
///
/// # 参数
/// - `conn`: 数据库连接
/// - `plan_id`: 计划主键
/// - `steps`: 新的步骤
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 写入失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// replace_steps(conn, plan_id, steps)?;
/// ```
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

/// 删掉原提醒并重新插入为未触发
///
/// # 参数
/// - `conn`: 数据库连接
/// - `plan_id`: 计划主键
/// - `reminders`: 新的提醒
/// - `now`: 创建时间
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 写入失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// replace_reminders(conn, plan_id, reminders, now)?;
/// ```
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

/// 删掉原重复规则并插入新规则
///
/// # 参数
/// - `conn`: 数据库连接
/// - `plan_id`: 计划主键
/// - `repeat`: 新的重复规则
/// - `now`: 写入时间
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 写入失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// replace_repeat(conn, plan_id, repeat, now)?;
/// ```
fn replace_repeat(conn: &Connection, plan_id: &str, repeat: &PlanRepeatInput, now: i64) -> Result<(), PlanError> {
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

/// 按传入列表替换计划的成员、标签和地点
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键
/// - `member_ids`: 成员主键，空则不改
/// - `tag_ids`: 标签主键，空则不改
/// - `place_ids`: 地点主键，空则不改
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 引用不存在时返回 `PlanError::ReferencedMissing`，其余失败返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// replace_refs(conn, id, member_ids, tag_ids, place_ids)?;
/// ```
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

/// 清除计划的成员、标签、地点和媒体引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 计划主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 清除失败时返回 `PlanError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// clear_refs(conn, id)?;
/// ```
fn clear_refs(conn: &Connection, id: &str) -> Result<(), PlanError> {
  let owner = Owner::Plan.as_str();
  crate::member::clear(conn, owner, id).map_err(|_| PlanError::Internal)?;
  crate::tag::clear(conn, owner, id).map_err(|_| PlanError::Internal)?;
  crate::place::clear(conn, owner, id).map_err(|_| PlanError::Internal)?;
  crate::upload::clear(conn, owner, id).map_err(|_| PlanError::Internal)?;
  Ok(())
}

/// 把引用错误映射成计划错误
///
/// # 参数
/// - `err`: 引用模块返回的错误
///
/// # 返回值
/// 文本为「引用不存在」时返回 `ReferencedMissing`，否则返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let err = ref_err(source);
/// ```
fn ref_err<E: std::fmt::Display>(err: E) -> PlanError {
  let text = err.to_string();
  if text == "引用不存在" { PlanError::ReferencedMissing } else { PlanError::Internal }
}
