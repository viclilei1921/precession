//! 图片和视频的元数据。文件本体不在这里。

use rusqlite::{Connection, params};

use super::constants::{KIND_IMAGE, KIND_VIDEO, MEDIA_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::{Media, MediaInput};
use super::error::MediaError;
use crate::owner::{self, Owner};
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
/// let err = db_fail("media list", rusqlite_err);
/// ```
fn db_fail(context: &str, err: rusqlite::Error) -> MediaError {
  tauri_plugin_log::log::error!("{context}: {err}");
  MediaError::Internal
}

/// 把查询行映射为媒体
///
/// # 参数
/// - `row`: 查询结果行
///
/// # 返回值
/// 返回媒体
///
/// # 错误
/// 读取列失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let media = map_media(row)?;
/// ```
fn map_media(row: &rusqlite::Row<'_>) -> rusqlite::Result<Media> {
  Ok(Media {
    id: row.get(0)?,
    owner: row.get(1)?,
    owner_id: row.get(2)?,
    kind: row.get(3)?,
    rel_path: row.get(4)?,
    mime: row.get(5)?,
    sort: row.get(6)?,
    locked: row.get(7)?,
    encrypted: row.get(8)?,
    created_at: row.get(9)?,
  })
}

/// 确保媒体表和模式版本存在
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 表结构已就绪时返回空成功
///
/// # 错误
/// 读取版本、建表、增加加密列或写入版本失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// ensure_schema(conn)?;
/// ```
pub(super) fn ensure_schema(conn: &Connection) -> Result<(), MediaError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("media schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS media (
           id TEXT PRIMARY KEY NOT NULL,
           owner TEXT NOT NULL,
           owner_id TEXT NOT NULL,
           kind TEXT NOT NULL,
           rel_path TEXT NOT NULL DEFAULT '',
           mime TEXT NOT NULL DEFAULT '',
           sort INTEGER NOT NULL DEFAULT 0,
           locked INTEGER NOT NULL DEFAULT 0,
           created_at INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS media_owner ON media(owner, owner_id);",
      )
      .map_err(|e| db_fail("media migrate v1", e))?;
  }
  if current < 2 {
    conn
      .execute("ALTER TABLE media ADD COLUMN encrypted INTEGER NOT NULL DEFAULT 0", [])
      .map_err(|e| db_fail("media migrate v2", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("media schema", e))?;
  }
  Ok(())
}

/// 整理并校验媒体输入
///
/// # 参数
/// - `input`: 待写入的媒体
///
/// # 返回值
/// 返回解析出的所属类型，以及去掉所属类型、所属标识、类型、相对路径和 MIME 首尾空白后的输入
///
/// # 错误
/// 类型不是图片或视频，或所属类型无法识别时返回类型不正确
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let parsed = prepare(raw);
/// ```
pub(super) fn prepare(mut input: MediaInput) -> Result<(Owner, MediaInput), MediaError> {
  input.owner = input.owner.trim().to_string();
  input.owner_id = input.owner_id.trim().to_string();
  input.kind = input.kind.trim().to_string();
  input.rel_path = input.rel_path.trim().to_string();
  input.mime = input.mime.trim().to_string();
  if input.kind != KIND_IMAGE && input.kind != KIND_VIDEO {
    return Err(MediaError::KindInvalid);
  }
  let Some(owner) = Owner::parse(&input.owner) else {
    return Err(MediaError::KindInvalid);
  };
  Ok((owner, input))
}

/// 按所属记录列出媒体
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 返回按排序值和创建时间排列的媒体
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
pub(super) fn list(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<Media>, MediaError> {
  let sql = format!(
    "SELECT id, owner, owner_id, kind, rel_path, mime, sort, locked, encrypted, created_at
     FROM {MEDIA_TABLE} WHERE owner = ?1 AND owner_id = ?2 ORDER BY sort, created_at"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("media list", e))?;
  let rows = stmt.query_map(params![owner, owner_id], map_media).map_err(|e| db_fail("media list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("media list", e))
}

/// 确认所属记录存在后写入媒体
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `input`: 媒体输入
///
/// # 返回值
/// 返回新建后重新读出的媒体
///
/// # 错误
/// 所属记录不存在、插入失败或读回失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = create(conn, owner, input)?;
/// ```
pub(super) fn create(conn: &Connection, owner: Owner, input: &MediaInput) -> Result<Media, MediaError> {
  let alive = owner::exists(conn, owner, &input.owner_id).map_err(|e| db_fail("media owner", e))?;
  if !alive {
    return Err(MediaError::ReferencedMissing);
  }
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!(
    "INSERT INTO {MEDIA_TABLE} (id, owner, owner_id, kind, rel_path, mime, sort, locked, created_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
  );
  conn
    .execute(
      &sql,
      params![id, input.owner, input.owner_id, input.kind, input.rel_path, input.mime, input.sort, input.locked, now],
    )
    .map_err(|e| db_fail("media create", e))?;
  get(conn, &id)
}

/// 按标识删除媒体
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 媒体标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 没有匹配记录时返回不存在；删除失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete(conn, id)?;
/// ```
pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), MediaError> {
  let sql = format!("DELETE FROM {MEDIA_TABLE} WHERE id = ?1");
  let n = conn.execute(&sql, [id]).map_err(|e| db_fail("media delete", e))?;
  if n == 0 {
    return Err(MediaError::NotFound);
  }
  Ok(())
}

/// 按指定标识写入导入的媒体
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 媒体标识
/// - `input`: 媒体输入
/// - `encrypted`: 是否已加密
///
/// # 返回值
/// 返回写入后重新读出的媒体
///
/// # 错误
/// 输入不合法、所属记录不存在、插入失败或读回失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = insert_imported(conn, id, input, encrypted)?;
/// ```
pub(super) fn insert_imported(
  conn: &Connection,
  id: &str,
  input: MediaInput,
  encrypted: bool,
) -> Result<Media, MediaError> {
  let (owner, input) = prepare(input)?;
  let alive = owner::exists(conn, owner, &input.owner_id).map_err(|e| db_fail("media owner", e))?;
  if !alive {
    return Err(MediaError::ReferencedMissing);
  }
  let now = now_unix_ms();
  let sql = format!(
    "INSERT INTO {MEDIA_TABLE}
       (id, owner, owner_id, kind, rel_path, mime, sort, locked, encrypted, created_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
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
        input.sort,
        input.locked,
        encrypted,
        now
      ],
    )
    .map_err(|e| db_fail("media import", e))?;
  get(conn, id)
}

/// 设置媒体的加密标记
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 媒体标识
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
pub(super) fn set_encrypted(conn: &Connection, id: &str, encrypted: bool) -> Result<(), MediaError> {
  let sql = format!("UPDATE {MEDIA_TABLE} SET encrypted = ?1 WHERE id = ?2");
  let n = conn.execute(&sql, params![encrypted, id]).map_err(|e| db_fail("media encrypted", e))?;
  if n == 0 {
    return Err(MediaError::NotFound);
  }
  Ok(())
}

/// 删除所属记录上的全部媒体
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
pub(super) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), MediaError> {
  let sql = format!("DELETE FROM {MEDIA_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("media clear", e))?;
  Ok(())
}

/// 按标识读取媒体
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 媒体标识
///
/// # 返回值
/// 返回对应的媒体
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
pub(super) fn get(conn: &Connection, id: &str) -> Result<Media, MediaError> {
  let sql = format!(
    "SELECT id, owner, owner_id, kind, rel_path, mime, sort, locked, encrypted, created_at
     FROM {MEDIA_TABLE} WHERE id = ?1"
  );
  conn.query_row(&sql, [id], map_media).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => MediaError::NotFound,
    other => db_fail("media get", other),
  })
}
