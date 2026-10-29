//! 成长记录。主体成员写在本表，其他成员走 member_ref。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{ENTRY_TABLE, KIND_MILESTONE, KIND_MOMENT, MEASURE_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::{GrowthEntry, GrowthEntryInput, GrowthMeasure, GrowthMeasureInput};
use super::error::GrowthError;
use crate::owner::Owner;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

/// 记录数据库错误并返回内部错误
///
/// # 参数
/// - `context`: 出错位置说明
/// - `err`: 数据库错误
///
/// # 返回值
/// 始终返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let err = db_fail("growth list", rusqlite_err);
/// ```
fn db_fail(context: &str, err: rusqlite::Error) -> GrowthError {
  tauri_plugin_log::log::error!("{context}: {err}");
  GrowthError::Internal
}

const COLUMNS: &str = "id, kind, member_id, occurred_at, title, body, locked, highlight, created_at, updated_at";

/// 把查询行映射为成长记录
///
/// # 参数
/// - `row`: 查询结果行
///
/// # 返回值
/// 返回成员、标签和地点引用均为空的成长记录
///
/// # 错误
/// 读取列失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entry = map_entry(row)?;
/// ```
fn map_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<GrowthEntry> {
  Ok(GrowthEntry {
    id: row.get(0)?,
    kind: row.get(1)?,
    member_id: row.get(2)?,
    occurred_at: row.get(3)?,
    title: row.get(4)?,
    body: row.get(5)?,
    locked: row.get(6)?,
    highlight: row.get(7)?,
    member_ids: Vec::new(),
    tag_ids: Vec::new(),
    place_ids: Vec::new(),
    created_at: row.get(8)?,
    updated_at: row.get(9)?,
  })
}

/// 确保成长记录表和模式版本存在
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 表结构已就绪时返回空成功
///
/// # 错误
/// 读取版本、建表或写入版本失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// ensure_schema(conn)?;
/// ```
pub(super) fn ensure_schema(conn: &Connection) -> Result<(), GrowthError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("growth schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS growth_entry (
           id TEXT PRIMARY KEY NOT NULL,
           kind TEXT NOT NULL,
           member_id TEXT NOT NULL,
           occurred_at INTEGER NOT NULL,
           title TEXT NOT NULL,
           body TEXT NOT NULL DEFAULT '',
           locked INTEGER NOT NULL DEFAULT 0,
           highlight INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS growth_entry_member ON growth_entry(member_id, occurred_at);",
      )
      .map_err(|e| db_fail("growth migrate v1", e))?;
  }
  if current < 2 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS growth_measure (
           id TEXT PRIMARY KEY NOT NULL,
           member_id TEXT NOT NULL,
           occurred_at INTEGER NOT NULL,
           height_mm INTEGER NOT NULL,
           weight_g INTEGER NOT NULL,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS growth_measure_member ON growth_measure(member_id, occurred_at);",
      )
      .map_err(|e| db_fail("growth migrate v2", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("growth schema", e))?;
  }
  Ok(())
}

/// 判断成长记录类型是否受支持
///
/// # 参数
/// - `kind`: 记录类型
///
/// # 返回值
/// 类型为里程碑或精彩瞬间时返回 true
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let ok = allowed_kind("milestone");
/// ```
pub(super) fn allowed_kind(kind: &str) -> bool {
  kind == KIND_MILESTONE || kind == KIND_MOMENT
}

/// 检查未删除的成长记录是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 成长记录标识
///
/// # 返回值
/// 存在未删除记录时返回 true
///
/// # 错误
/// 查询失败时返回数据库错误
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
    .query_row("SELECT 1 FROM growth_entry WHERE id = ?1 AND deleted_at IS NULL", [id], |row| {
      row.get(0)
    })
    .optional()?;
  Ok(found.is_some())
}

/// 按成员、类型和时间范围列出成长记录
///
/// # 参数
/// - `conn`: 数据库连接
/// - `member_id`: 成员标识
/// - `kind`: 记录类型
/// - `from`: 起始发生时间
/// - `to`: 结束发生时间
///
/// # 返回值
/// 返回符合条件的成长记录
///
/// # 错误
/// 查询失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list(conn, member_id, kind, from, to)?;
/// ```
pub(super) fn list(
  conn: &Connection,
  member_id: Option<&str>,
  kind: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<GrowthEntry>, GrowthError> {
  query(conn, None, member_id, kind, from, to)
}

/// 按发生时间范围列出成长记录
///
/// # 参数
/// - `conn`: 数据库连接
/// - `from`: 起始发生时间
/// - `to`: 结束发生时间
///
/// # 返回值
/// 返回该范围内的成长记录
///
/// # 错误
/// 查询失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list_between(conn, from, to)?;
/// ```
pub(super) fn list_between(
  conn: &Connection,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<GrowthEntry>, GrowthError> {
  query(conn, None, None, None, from, to)
}

/// 按标识读取未删除的成长记录
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 成长记录标识
///
/// # 返回值
/// 返回匹配的成长记录
///
/// # 错误
/// 没有匹配记录时返回不存在；查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = get(conn, id)?;
/// ```
pub(super) fn get(conn: &Connection, id: &str) -> Result<GrowthEntry, GrowthError> {
  let mut items = query(conn, Some(id), None, None, None, None)?;
  items.pop().ok_or(GrowthError::NotFound)
}

/// 写入成长记录并保存引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `input`: 成长记录输入
///
/// # 返回值
/// 返回新建后重新读出的成长记录
///
/// # 错误
/// 成员不存在、插入失败、引用写入失败或提交失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = create(conn, input)?;
/// ```
pub(super) fn create(conn: &Connection, input: &GrowthEntryInput) -> Result<GrowthEntry, GrowthError> {
  require_member(conn, &input.member_id)?;
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("growth create tx", e))?;
  tx.execute(
    "INSERT INTO growth_entry
       (id, kind, member_id, occurred_at, title, body, locked, highlight, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
    params![
      id,
      input.kind,
      input.member_id,
      input.occurred_at,
      input.title,
      input.body,
      input.locked,
      input.highlight,
      now
    ],
  )
  .map_err(|e| db_fail("growth create", e))?;
  replace_refs(&tx, &id, input)?;
  tx.commit().map_err(|e| db_fail("growth create commit", e))?;
  get(conn, &id)
}

/// 更新未删除的成长记录并替换引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 成长记录标识
/// - `input`: 成长记录输入
///
/// # 返回值
/// 返回更新后重新读出的成长记录
///
/// # 错误
/// 成员不存在、记录不存在、更新失败或提交失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = update(conn, id, input)?;
/// ```
pub(super) fn update(conn: &Connection, id: &str, input: &GrowthEntryInput) -> Result<GrowthEntry, GrowthError> {
  require_member(conn, &input.member_id)?;
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("growth update tx", e))?;
  let n = tx
    .execute(
      "UPDATE growth_entry
       SET kind = ?1, member_id = ?2, occurred_at = ?3, title = ?4, body = ?5, locked = ?6, highlight = ?7, updated_at = ?8
       WHERE id = ?9 AND deleted_at IS NULL",
      params![
        input.kind,
        input.member_id,
        input.occurred_at,
        input.title,
        input.body,
        input.locked,
        input.highlight,
        now,
        id
      ],
    )
    .map_err(|e| db_fail("growth update", e))?;
  if n == 0 {
    return Err(GrowthError::NotFound);
  }
  replace_refs(&tx, id, input)?;
  tx.commit().map_err(|e| db_fail("growth update commit", e))?;
  get(conn, id)
}

/// 软删除成长记录并清除引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 成长记录标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 记录不存在、清除引用失败或提交失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete(conn, id)?;
/// ```
pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), GrowthError> {
  let now = now_unix_ms();
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("growth delete tx", e))?;
  let n = tx
    .execute(
      "UPDATE growth_entry SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
      params![now, id],
    )
    .map_err(|e| db_fail("growth delete", e))?;
  if n == 0 {
    return Err(GrowthError::NotFound);
  }
  clear_refs(&tx, id)?;
  tx.commit().map_err(|e| db_fail("growth delete commit", e))?;
  Ok(())
}

/// 按条件查询未删除的成长记录并补齐引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 成长记录标识，为空则不限制
/// - `member_id`: 成员标识，为空则不限制
/// - `kind`: 记录类型，为空则不限制
/// - `from`: 起始发生时间，包含该时刻；为空则不限制
/// - `to`: 结束发生时间，不包含该时刻；为空则不限制
///
/// # 返回值
/// 返回按发生时间倒序排列，并带上成员、标签和地点标识的记录
///
/// # 错误
/// 查询或读取引用失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = query(conn, id, member_id, kind, from, to)?;
/// ```
fn query(
  conn: &Connection,
  id: Option<&str>,
  member_id: Option<&str>,
  kind: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<GrowthEntry>, GrowthError> {
  let sql = format!(
    "SELECT {COLUMNS} FROM {ENTRY_TABLE}
     WHERE deleted_at IS NULL
       AND (?1 IS NULL OR id = ?1)
       AND (?2 IS NULL OR member_id = ?2)
       AND (?3 IS NULL OR kind = ?3)
       AND (?4 IS NULL OR occurred_at >= ?4)
       AND (?5 IS NULL OR occurred_at < ?5)
     ORDER BY occurred_at DESC"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("growth list", e))?;
  let rows =
    stmt.query_map(params![id, member_id, kind, from, to], map_entry).map_err(|e| db_fail("growth list", e))?;
  let mut items = rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("growth list", e))?;
  for item in &mut items {
    let owner = Owner::GrowthEntry.as_str();
    item.member_ids = crate::member::list_ids(conn, owner, &item.id).map_err(|_| GrowthError::Internal)?;
    item.tag_ids = crate::tag::list_ids(conn, owner, &item.id).map_err(|_| GrowthError::Internal)?;
    item.place_ids = crate::place::list_ids(conn, owner, &item.id).map_err(|_| GrowthError::Internal)?;
  }
  Ok(items)
}

/// 确认成员记录仍然存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 成员标识
///
/// # 返回值
/// 成员存在时返回空成功
///
/// # 错误
/// 查询失败时返回内部错误；成员不存在时返回引用不存在
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// require_member(conn, id)?;
/// ```
fn require_member(conn: &Connection, id: &str) -> Result<(), GrowthError> {
  let alive = crate::member::exists(conn, id).map_err(|e| db_fail("growth member", e))?;
  if alive { Ok(()) } else { Err(GrowthError::ReferencedMissing) }
}

/// 替换成长记录上的成员、标签和地点引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 成长记录标识
/// - `input`: 含引用标识的输入
///
/// # 返回值
/// 替换完成时返回空成功
///
/// # 错误
/// 引用不存在时返回引用不存在；其他失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// replace_refs(conn, id, input)?;
/// ```
fn replace_refs(conn: &Connection, id: &str, input: &GrowthEntryInput) -> Result<(), GrowthError> {
  let owner = Owner::GrowthEntry.as_str();
  crate::member::replace(conn, owner, id, &input.member_ids).map_err(ref_err)?;
  crate::tag::replace(conn, owner, id, &input.tag_ids).map_err(ref_err)?;
  crate::place::replace(conn, owner, id, &input.place_ids).map_err(ref_err)?;
  Ok(())
}

/// 清除成长记录上的成员、标签、地点和媒体
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 成长记录标识
///
/// # 返回值
/// 清除完成时返回空成功
///
/// # 错误
/// 任一清除失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// clear_refs(conn, id)?;
/// ```
fn clear_refs(conn: &Connection, id: &str) -> Result<(), GrowthError> {
  let owner = Owner::GrowthEntry.as_str();
  crate::member::clear(conn, owner, id).map_err(|_| GrowthError::Internal)?;
  crate::tag::clear(conn, owner, id).map_err(|_| GrowthError::Internal)?;
  crate::place::clear(conn, owner, id).map_err(|_| GrowthError::Internal)?;
  crate::upload::clear(conn, owner, id).map_err(|_| GrowthError::Internal)?;
  Ok(())
}

/// 把引用失败转成成长记录错误
///
/// # 参数
/// - `err`: 待转换的错误
///
/// # 返回值
/// 错误文本为「引用不存在」时返回引用不存在，否则返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mapped = ref_err(err);
/// ```
fn ref_err<E: std::fmt::Display>(err: E) -> GrowthError {
  if err.to_string() == "引用不存在" { GrowthError::ReferencedMissing } else { GrowthError::Internal }
}

/// 按成员列出未删除的身高体重，发生时间晚的在前
///
/// # 参数
/// - `conn`: 数据库连接
/// - `member_id`: 成员标识
///
/// # 返回值
/// 返回该成员的测量
///
/// # 错误
/// 查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list_measures(conn, member_id)?;
/// ```
pub(super) fn list_measures(conn: &Connection, member_id: &str) -> Result<Vec<GrowthMeasure>, GrowthError> {
  let sql = format!(
    "SELECT id, member_id, occurred_at, height_mm, weight_g, created_at, updated_at
     FROM {MEASURE_TABLE}
     WHERE deleted_at IS NULL AND member_id = ?1
     ORDER BY occurred_at DESC"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("growth measure list", e))?;
  let rows = stmt.query_map(params![member_id], map_measure).map_err(|e| db_fail("growth measure list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("growth measure list", e))
}

/// 按标识读取未删除的身高体重
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 测量标识
///
/// # 返回值
/// 返回匹配的测量
///
/// # 错误
/// 没有匹配记录时返回不存在；查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = get_measure(conn, id)?;
/// ```
pub(super) fn get_measure(conn: &Connection, id: &str) -> Result<GrowthMeasure, GrowthError> {
  let sql = format!(
    "SELECT id, member_id, occurred_at, height_mm, weight_g, created_at, updated_at
     FROM {MEASURE_TABLE}
     WHERE id = ?1 AND deleted_at IS NULL"
  );
  conn
    .query_row(&sql, params![id], map_measure)
    .optional()
    .map_err(|e| db_fail("growth measure get", e))?
    .ok_or(GrowthError::NotFound)
}

/// 写入一次身高体重
///
/// # 参数
/// - `conn`: 数据库连接
/// - `input`: 测量输入
///
/// # 返回值
/// 返回新建后重新读出的测量
///
/// # 错误
/// 成员不存在或插入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = create_measure(conn, input)?;
/// ```
pub(super) fn create_measure(conn: &Connection, input: &GrowthMeasureInput) -> Result<GrowthMeasure, GrowthError> {
  require_member(conn, &input.member_id)?;
  let id = new_uuid_v4();
  let now = now_unix_ms();
  conn
    .execute(
      "INSERT INTO growth_measure
         (id, member_id, occurred_at, height_mm, weight_g, created_at, updated_at)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
      params![id, input.member_id, input.occurred_at, input.height_mm, input.weight_g, now],
    )
    .map_err(|e| db_fail("growth measure create", e))?;
  get_measure(conn, &id)
}

/// 更新未删除的身高体重
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 测量标识
/// - `input`: 测量输入
///
/// # 返回值
/// 返回更新后重新读出的测量
///
/// # 错误
/// 成员不存在、记录不存在或更新失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = update_measure(conn, id, input)?;
/// ```
pub(super) fn update_measure(
  conn: &Connection,
  id: &str,
  input: &GrowthMeasureInput,
) -> Result<GrowthMeasure, GrowthError> {
  require_member(conn, &input.member_id)?;
  let now = now_unix_ms();
  let n = conn
    .execute(
      "UPDATE growth_measure
       SET member_id = ?1, occurred_at = ?2, height_mm = ?3, weight_g = ?4, updated_at = ?5
       WHERE id = ?6 AND deleted_at IS NULL",
      params![input.member_id, input.occurred_at, input.height_mm, input.weight_g, now, id],
    )
    .map_err(|e| db_fail("growth measure update", e))?;
  if n == 0 {
    return Err(GrowthError::NotFound);
  }
  get_measure(conn, id)
}

/// 软删除一次身高体重
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 测量标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 记录不存在或更新失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete_measure(conn, id)?;
/// ```
pub(super) fn delete_measure(conn: &Connection, id: &str) -> Result<(), GrowthError> {
  let now = now_unix_ms();
  let n = conn
    .execute(
      "UPDATE growth_measure SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
      params![now, id],
    )
    .map_err(|e| db_fail("growth measure delete", e))?;
  if n == 0 { Err(GrowthError::NotFound) } else { Ok(()) }
}

/// 把查询行映射为身高体重
///
/// # 参数
/// - `row`: 查询结果行
///
/// # 返回值
/// 返回一次测量
///
/// # 错误
/// 读取列失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = map_measure(row)?;
/// ```
fn map_measure(row: &rusqlite::Row<'_>) -> rusqlite::Result<GrowthMeasure> {
  Ok(GrowthMeasure {
    id: row.get(0)?,
    member_id: row.get(1)?,
    occurred_at: row.get(2)?,
    height_mm: row.get(3)?,
    weight_g: row.get(4)?,
    created_at: row.get(5)?,
    updated_at: row.get(6)?,
  })
}
