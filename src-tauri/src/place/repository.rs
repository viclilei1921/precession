//! 地点，以及记录上的地点引用。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{PLACE_REF_TABLE, PLACE_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::Place;
use super::error::PlaceError;
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
/// let err = db_fail("place list", rusqlite_err);
/// ```
fn db_fail(context: &str, err: rusqlite::Error) -> PlaceError {
  tauri_plugin_log::log::error!("{context}: {err}");
  PlaceError::Internal
}

/// 把查询行映射为地点
///
/// # 参数
/// - `row`: 查询结果行
///
/// # 返回值
/// 返回地点
///
/// # 错误
/// 读取列失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let place = map_place(row)?;
/// ```
fn map_place(row: &rusqlite::Row<'_>) -> rusqlite::Result<Place> {
  Ok(Place { id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)?, updated_at: row.get(3)? })
}

/// 确保地点表、地点引用表和模式版本存在
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
pub(super) fn ensure_schema(conn: &Connection) -> Result<(), PlaceError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("place schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS place (
           id TEXT PRIMARY KEY NOT NULL,
           name TEXT NOT NULL,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS place_updated_at ON place(updated_at);
         CREATE TABLE IF NOT EXISTS place_ref (
           place_id TEXT NOT NULL,
           owner TEXT NOT NULL,
           owner_id TEXT NOT NULL,
           PRIMARY KEY (place_id, owner, owner_id)
         );
         CREATE INDEX IF NOT EXISTS place_ref_owner ON place_ref(owner, owner_id);",
      )
      .map_err(|e| db_fail("place migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("place schema", e))?;
  }
  Ok(())
}

/// 检查未删除的地点是否存在
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 地点标识
///
/// # 返回值
/// 存在未删除地点时返回 true
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
fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  let found: Option<i64> = conn
    .query_row("SELECT 1 FROM place WHERE id = ?1 AND deleted_at IS NULL", [id], |row| {
      row.get(0)
    })
    .optional()?;
  Ok(found.is_some())
}

/// 按名称列出未删除的地点
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 返回按名称排序的地点
///
/// # 错误
/// 查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list(conn)?;
/// ```
pub(super) fn list(conn: &Connection) -> Result<Vec<Place>, PlaceError> {
  let sql =
    format!("SELECT id, name, created_at, updated_at FROM {PLACE_TABLE} WHERE deleted_at IS NULL ORDER BY name");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("place list", e))?;
  let rows = stmt.query_map([], map_place).map_err(|e| db_fail("place list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("place list", e))
}

/// 按标识读取未删除的地点
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 地点标识
///
/// # 返回值
/// 返回对应的地点
///
/// # 错误
/// 没有匹配记录时返回不存在；其他查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = get(conn, id)?;
/// ```
fn get(conn: &Connection, id: &str) -> Result<Place, PlaceError> {
  let sql = format!("SELECT id, name, created_at, updated_at FROM {PLACE_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  conn.query_row(&sql, [id], map_place).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => PlaceError::NotFound,
    other => db_fail("place get", other),
  })
}

/// 插入地点并读回
///
/// # 参数
/// - `conn`: 数据库连接
/// - `name`: 地点名称
///
/// # 返回值
/// 返回新建后重新读出的地点
///
/// # 错误
/// 插入或读回失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = create(conn, name)?;
/// ```
pub(super) fn create(conn: &Connection, name: &str) -> Result<Place, PlaceError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!("INSERT INTO {PLACE_TABLE} (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)");
  conn.execute(&sql, params![id, name, now]).map_err(|e| db_fail("place create", e))?;
  get(conn, &id)
}

/// 更新未删除地点的名称并读回
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 地点标识
/// - `name`: 地点名称
///
/// # 返回值
/// 返回更新后重新读出的地点
///
/// # 错误
/// 没有匹配记录时返回不存在；更新或读回失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = update(conn, id, name)?;
/// ```
pub(super) fn update(conn: &Connection, id: &str, name: &str) -> Result<Place, PlaceError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {PLACE_TABLE} SET name = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![name, now, id]).map_err(|e| db_fail("place update", e))?;
  if n == 0 {
    return Err(PlaceError::NotFound);
  }
  get(conn, id)
}

/// 软删除地点
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 地点标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 没有匹配记录时返回不存在；更新失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete(conn, id)?;
/// ```
pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), PlaceError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {PLACE_TABLE} SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![now, id]).map_err(|e| db_fail("place delete", e))?;
  if n == 0 {
    return Err(PlaceError::NotFound);
  }
  Ok(())
}

/// 用给定地点标识替换记录上的地点引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
/// - `ids`: 地点标识列表
///
/// # 返回值
/// 替换完成时返回空成功
///
/// # 错误
/// 任一地点不存在时返回引用不存在；删除旧引用或写入新引用失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// replace(conn, owner, owner_id, ids)?;
/// ```
pub(super) fn replace(conn: &Connection, owner: &str, owner_id: &str, ids: &[String]) -> Result<(), PlaceError> {
  for id in ids {
    if !exists(conn, id).map_err(|e| db_fail("place ref", e))? {
      return Err(PlaceError::ReferencedMissing);
    }
  }
  let sql = format!("DELETE FROM {PLACE_REF_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("place ref", e))?;
  let sql = format!("INSERT INTO {PLACE_REF_TABLE} (place_id, owner, owner_id) VALUES (?1, ?2, ?3)");
  for id in ids {
    conn.execute(&sql, params![id, owner, owner_id]).map_err(|e| db_fail("place ref", e))?;
  }
  Ok(())
}

/// 列出记录上仍未删除的地点标识
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 返回按地点标识排序的标识
///
/// # 错误
/// 查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let ids = list_ids(conn, owner, owner_id)?;
/// ```
pub(super) fn list_ids(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<String>, PlaceError> {
  let sql = format!(
    "SELECT r.place_id FROM {PLACE_REF_TABLE} r
     JOIN {PLACE_TABLE} p ON p.id = r.place_id AND p.deleted_at IS NULL
     WHERE r.owner = ?1 AND r.owner_id = ?2
     ORDER BY r.place_id"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("place ref list", e))?;
  let rows = stmt.query_map(params![owner, owner_id], |row| row.get(0)).map_err(|e| db_fail("place ref list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("place ref list", e))
}

/// 删除记录上的全部地点引用
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 删除完成时返回空成功
///
/// # 错误
/// 删除失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// clear(conn, owner, owner_id)?;
/// ```
pub(super) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), PlaceError> {
  let sql = format!("DELETE FROM {PLACE_REF_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("place ref clear", e))?;
  Ok(())
}
