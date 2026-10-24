//! 上传文件的元数据。文件本体在上传目录，见 `storage`。

use std::collections::HashSet;

use rusqlite::{Connection, params};

use super::constants::{KINDS, SCHEMA_VERSION, SCHEMA_VERSION_KEY, UPLOAD_TABLE};
use super::dto::{Upload, UploadInput};
use super::error::UploadError;
use crate::owner::{self, Owner};
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

/// 查询文件记录时的列，顺序与 [`map_upload`] 一致
const COLUMNS: &str = "id, owner, owner_id, kind, rel_path, mime, name, size, sort, locked, encrypted, created_at";

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
/// let err = db_fail("upload list", rusqlite_err);
/// ```
fn db_fail(context: &str, err: rusqlite::Error) -> UploadError {
  tauri_plugin_log::log::error!("{context}: {err}");
  UploadError::Internal
}

/// 把查询行映射为文件记录
///
/// # 参数
/// - `row`: 查询结果行
///
/// # 返回值
/// 返回文件记录
///
/// # 错误
/// 读取列失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let upload = map_upload(row)?;
/// ```
fn map_upload(row: &rusqlite::Row<'_>) -> rusqlite::Result<Upload> {
  Ok(Upload {
    id: row.get(0)?,
    owner: row.get(1)?,
    owner_id: row.get(2)?,
    kind: row.get(3)?,
    rel_path: row.get(4)?,
    mime: row.get(5)?,
    name: row.get(6)?,
    size: row.get(7)?,
    sort: row.get(8)?,
    locked: row.get(9)?,
    encrypted: row.get(10)?,
    created_at: row.get(11)?,
  })
}

/// 确保上传文件表和模式版本存在
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
pub(super) fn ensure_schema(conn: &Connection) -> Result<(), UploadError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("upload schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS upload (
           id TEXT PRIMARY KEY NOT NULL,
           owner TEXT NOT NULL,
           owner_id TEXT NOT NULL,
           kind TEXT NOT NULL,
           rel_path TEXT NOT NULL DEFAULT '',
           mime TEXT NOT NULL DEFAULT '',
           name TEXT NOT NULL DEFAULT '',
           size INTEGER NOT NULL DEFAULT 0,
           sort INTEGER NOT NULL DEFAULT 0,
           locked INTEGER NOT NULL DEFAULT 0,
           encrypted INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS upload_owner ON upload(owner, owner_id);",
      )
      .map_err(|e| db_fail("upload migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("upload schema", e))?;
  }
  Ok(())
}

/// 整理并校验文件输入
///
/// # 参数
/// - `input`: 待写入的文件记录
///
/// # 返回值
/// 返回解析出的所属类型，以及去掉所属类型、所属标识、类型、相对路径、MIME 和文件名首尾空白后的输入
///
/// # 错误
/// 类型不是图片、视频、书或文件，或所属类型无法识别时返回类型不正确
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let (owner, input) = prepare(raw)?;
/// ```
pub(super) fn prepare(mut input: UploadInput) -> Result<(Owner, UploadInput), UploadError> {
  input.owner = input.owner.trim().to_string();
  input.owner_id = input.owner_id.trim().to_string();
  input.kind = input.kind.trim().to_string();
  input.rel_path = input.rel_path.trim().to_string();
  input.mime = input.mime.trim().to_string();
  input.name = input.name.trim().to_string();
  if !KINDS.contains(&input.kind.as_str()) {
    return Err(UploadError::KindInvalid);
  }
  let Some(owner) = Owner::parse(&input.owner) else {
    return Err(UploadError::KindInvalid);
  };
  Ok((owner, input))
}

/// 按所属记录列出文件
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 返回按排序值和创建时间排列的文件记录
///
/// # 错误
/// 查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list(conn, owner, owner_id)?;
/// ```
pub(super) fn list(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<Upload>, UploadError> {
  let sql =
    format!("SELECT {COLUMNS} FROM {UPLOAD_TABLE} WHERE owner = ?1 AND owner_id = ?2 ORDER BY sort, created_at");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("upload list", e))?;
  let rows = stmt.query_map(params![owner, owner_id], map_upload).map_err(|e| db_fail("upload list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("upload list", e))
}

/// 确认所属记录存在后，按指定标识写入文件记录
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 文件标识
/// - `owner`: 已解析的所属类型
/// - `input`: 已整理的文件输入
/// - `encrypted`: 是否已加密
///
/// # 返回值
/// 返回写入后重新读出的文件记录
///
/// # 错误
/// 所属记录不存在返回 `ReferencedMissing`；插入或读回失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = insert(conn, id, owner, &input, false)?;
/// ```
pub(super) fn insert(
  conn: &Connection,
  id: &str,
  owner: Owner,
  input: &UploadInput,
  encrypted: bool,
) -> Result<Upload, UploadError> {
  let alive = owner::exists(conn, owner, &input.owner_id).map_err(|e| db_fail("upload owner", e))?;
  if !alive {
    return Err(UploadError::ReferencedMissing);
  }
  let sql = format!(
    "INSERT INTO {UPLOAD_TABLE}
       (id, owner, owner_id, kind, rel_path, mime, name, size, sort, locked, encrypted, created_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
  );
  conn
    .execute(
      &sql,
      params![
        id,
        input.owner,
        input.owner_id,
        input.kind,
        input.rel_path,
        input.mime,
        input.name,
        input.size,
        input.sort,
        input.locked,
        encrypted,
        now_unix_ms()
      ],
    )
    .map_err(|e| db_fail("upload insert", e))?;
  get(conn, id)
}

/// 生成新标识并写入未加密的文件记录
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 已解析的所属类型
/// - `input`: 已整理的文件输入
///
/// # 返回值
/// 返回新建后重新读出的文件记录
///
/// # 错误
/// 同 [`insert`]
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = create(conn, owner, &input)?;
/// ```
pub(super) fn create(conn: &Connection, owner: Owner, input: &UploadInput) -> Result<Upload, UploadError> {
  insert(conn, &new_uuid_v4(), owner, input, false)
}

/// 按标识删除文件记录
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 文件标识
///
/// # 返回值
/// 返回被删记录的相对路径，调用方据此删除文件
///
/// # 错误
/// 没有匹配记录时返回不存在；删除失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rel = delete(conn, id)?;
/// ```
pub(super) fn delete(conn: &Connection, id: &str) -> Result<String, UploadError> {
  let rel = get(conn, id)?.rel_path;
  let sql = format!("DELETE FROM {UPLOAD_TABLE} WHERE id = ?1");
  let n = conn.execute(&sql, [id]).map_err(|e| db_fail("upload delete", e))?;
  if n == 0 {
    return Err(UploadError::NotFound);
  }
  Ok(rel)
}

/// 取出全部文件记录的相对路径
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 相对路径集合，反斜杠统一换成 `/`
///
/// # 错误
/// 查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let keep = all_paths(conn)?;
/// ```
pub(super) fn all_paths(conn: &Connection) -> Result<HashSet<String>, UploadError> {
  let sql = format!("SELECT rel_path FROM {UPLOAD_TABLE}");
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("upload paths", e))?;
  let rows = stmt.query_map([], |row| row.get::<_, String>(0)).map_err(|e| db_fail("upload paths", e))?;
  rows
    .map(|row| row.map(|rel| rel.replace('\\', "/")))
    .collect::<Result<HashSet<_>, _>>()
    .map_err(|e| db_fail("upload paths", e))
}

/// 设置文件的加密标记
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 文件标识
/// - `encrypted`: 是否已加密
///
/// # 返回值
/// 更新成功时返回空成功
///
/// # 错误
/// 没有匹配记录时返回不存在；更新失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// set_encrypted(conn, id, encrypted)?;
/// ```
pub(super) fn set_encrypted(conn: &Connection, id: &str, encrypted: bool) -> Result<(), UploadError> {
  let sql = format!("UPDATE {UPLOAD_TABLE} SET encrypted = ?1 WHERE id = ?2");
  let n = conn.execute(&sql, params![encrypted, id]).map_err(|e| db_fail("upload encrypted", e))?;
  if n == 0 {
    return Err(UploadError::NotFound);
  }
  Ok(())
}

/// 删除所属记录上的全部文件记录
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
pub(super) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), UploadError> {
  let sql = format!("DELETE FROM {UPLOAD_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("upload clear", e))?;
  Ok(())
}

/// 按标识读取文件记录
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 文件标识
///
/// # 返回值
/// 返回对应的文件记录
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
pub(super) fn get(conn: &Connection, id: &str) -> Result<Upload, UploadError> {
  let sql = format!("SELECT {COLUMNS} FROM {UPLOAD_TABLE} WHERE id = ?1");
  conn.query_row(&sql, [id], map_upload).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => UploadError::NotFound,
    other => db_fail("upload get", other),
  })
}
