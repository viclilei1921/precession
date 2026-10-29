//! 上传文件的元数据。文件本体在上传目录，见 `storage`。

use std::collections::HashSet;

use rusqlite::{Connection, params};

use super::constants::{KINDS, SCHEMA_VERSION, SCHEMA_VERSION_KEY, UPLOAD_TABLE};
use super::dto::{Upload, UploadCatalog, UploadInput};
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

/// 把目录查询行映射为带所属信息的文件记录
///
/// 前 12 列与 [`map_upload`] 相同，后面依次是标题、种类、封存、时间、父记录标识和父记录标题。
///
/// # 参数
/// - `row`: 查询结果行
///
/// # 返回值
/// 返回目录记录
///
/// # 错误
/// 读取列失败时返回数据库错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let item = map_catalog(row)?;
/// ```
fn map_catalog(row: &rusqlite::Row<'_>) -> rusqlite::Result<UploadCatalog> {
  Ok(UploadCatalog {
    upload: map_upload(row)?,
    owner_title: row.get(12)?,
    owner_kind: row.get(13)?,
    owner_sealed: row.get::<_, i64>(14)? != 0,
    owner_at: row.get(15)?,
    parent_id: row.get(16)?,
    parent_title: row.get(17)?,
  })
}

/// 列出全部文件，并带上所属记录的标题、种类、封存和时间
///
/// 表名与 [`Owner`] 的字符串一致。已删除的所属记录不参与连接，标题留空。
/// 手记、成长和书摘取种类与发生时间，书取书架状态与创建时间，计划取创建时间、种类为空。
/// 书摘再带上所属书的标识和标题。家人头像取家人的名字，种类为 `avatar`，时间用创建时间。
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 按创建时间从新到旧排列的目录
///
/// # 错误
/// 查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = catalog(conn)?;
/// ```
pub(super) fn catalog(conn: &Connection) -> Result<Vec<UploadCatalog>, UploadError> {
  let sql = format!(
    "SELECT u.id, u.owner, u.owner_id, u.kind, u.rel_path, u.mime, u.name, u.size, u.sort,
            u.locked, u.encrypted, u.created_at,
            COALESCE(
              (SELECT title FROM journal_entry WHERE id = u.owner_id AND u.owner = 'journal_entry' AND deleted_at IS NULL),
              (SELECT title FROM growth_entry WHERE id = u.owner_id AND u.owner = 'growth_entry' AND deleted_at IS NULL),
              (SELECT title FROM book WHERE id = u.owner_id AND u.owner = 'book' AND deleted_at IS NULL),
              (SELECT title FROM book_note WHERE id = u.owner_id AND u.owner = 'book_note' AND deleted_at IS NULL),
              (SELECT title FROM plan WHERE id = u.owner_id AND u.owner = 'plan' AND deleted_at IS NULL),
              (SELECT name FROM member WHERE id = u.owner_id AND u.owner = 'member' AND deleted_at IS NULL),
              ''
            ),
            COALESCE(
              (SELECT kind FROM journal_entry WHERE id = u.owner_id AND u.owner = 'journal_entry' AND deleted_at IS NULL),
              (SELECT kind FROM growth_entry WHERE id = u.owner_id AND u.owner = 'growth_entry' AND deleted_at IS NULL),
              (SELECT status FROM book WHERE id = u.owner_id AND u.owner = 'book' AND deleted_at IS NULL),
              (SELECT kind FROM book_note WHERE id = u.owner_id AND u.owner = 'book_note' AND deleted_at IS NULL),
              (SELECT 'avatar' FROM member WHERE id = u.owner_id AND u.owner = 'member' AND deleted_at IS NULL),
              ''
            ),
            COALESCE(
              (SELECT locked FROM journal_entry WHERE id = u.owner_id AND u.owner = 'journal_entry' AND deleted_at IS NULL),
              (SELECT locked FROM growth_entry WHERE id = u.owner_id AND u.owner = 'growth_entry' AND deleted_at IS NULL),
              (SELECT locked FROM book_note WHERE id = u.owner_id AND u.owner = 'book_note' AND deleted_at IS NULL),
              (SELECT locked FROM plan WHERE id = u.owner_id AND u.owner = 'plan' AND deleted_at IS NULL),
              0
            ),
            COALESCE(
              (SELECT occurred_at FROM journal_entry WHERE id = u.owner_id AND u.owner = 'journal_entry' AND deleted_at IS NULL),
              (SELECT occurred_at FROM growth_entry WHERE id = u.owner_id AND u.owner = 'growth_entry' AND deleted_at IS NULL),
              (SELECT created_at FROM book WHERE id = u.owner_id AND u.owner = 'book' AND deleted_at IS NULL),
              (SELECT occurred_at FROM book_note WHERE id = u.owner_id AND u.owner = 'book_note' AND deleted_at IS NULL),
              (SELECT created_at FROM plan WHERE id = u.owner_id AND u.owner = 'plan' AND deleted_at IS NULL),
              (SELECT created_at FROM member WHERE id = u.owner_id AND u.owner = 'member' AND deleted_at IS NULL),
              0
            ),
            COALESCE(
              (SELECT book_id FROM book_note WHERE id = u.owner_id AND u.owner = 'book_note' AND deleted_at IS NULL),
              ''
            ),
            COALESCE(
              (SELECT b.title FROM book_note n
                 JOIN book b ON b.id = n.book_id AND b.deleted_at IS NULL
                WHERE n.id = u.owner_id AND u.owner = 'book_note' AND n.deleted_at IS NULL),
              ''
            )
       FROM {UPLOAD_TABLE} u
      ORDER BY u.created_at DESC, u.id"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("upload catalog", e))?;
  let rows = stmt.query_map([], map_catalog).map_err(|e| db_fail("upload catalog", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("upload catalog", e))
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
/// 所属记录已封存时不删除。这张图是家人头像时，同时清空成员上的头像标识。
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 文件标识
///
/// # 返回值
/// 返回被删记录的相对路径，调用方据此删除文件
///
/// # 错误
/// 没有匹配记录时返回不存在。所属记录已封存时返回已封存。删除失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rel = delete(conn, id)?;
/// ```
pub(super) fn delete(conn: &Connection, id: &str) -> Result<String, UploadError> {
  let row = get(conn, id)?;
  let sealed = owner::sealed(conn, &row.owner, &row.owner_id).map_err(|e| db_fail("upload sealed", e))?;
  if sealed {
    return Err(UploadError::OwnerSealed);
  }
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("upload delete tx", e))?;
  if row.owner == Owner::Member.as_str() {
    let now = now_unix_ms();
    tx.execute(
      "UPDATE member SET avatar = '', updated_at = ?1 WHERE id = ?2 AND avatar = ?3 AND deleted_at IS NULL",
      params![now, row.owner_id, row.id],
    )
    .map_err(|e| db_fail("upload delete avatar", e))?;
  }
  let sql = format!("DELETE FROM {UPLOAD_TABLE} WHERE id = ?1");
  let n = tx.execute(&sql, [id]).map_err(|e| db_fail("upload delete", e))?;
  if n == 0 {
    return Err(UploadError::NotFound);
  }
  tx.commit().map_err(|e| db_fail("upload delete commit", e))?;
  Ok(row.rel_path)
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

/// 同时改加密标记和文件名
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 文件标识
/// - `encrypted`: 是否已加密
/// - `name`: 新的文件名
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
/// set_encrypted_name(conn, id, true, "a.jpg.env")?;
/// ```
pub(super) fn set_encrypted_name(conn: &Connection, id: &str, encrypted: bool, name: &str) -> Result<(), UploadError> {
  let sql = format!("UPDATE {UPLOAD_TABLE} SET encrypted = ?1, name = ?2 WHERE id = ?3");
  let n = conn.execute(&sql, params![encrypted, name, id]).map_err(|e| db_fail("upload encrypted name", e))?;
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

/// 把未挂载的文件改挂到一条已存在的记录上
///
/// 只改当前属于 `loose` 的记录。已经挂在别处、或标识对不上的行保持不动。
///
/// # 参数
/// - `conn`: 数据库连接
/// - `ids`: 要改挂的文件标识，空白项由调用方去掉
/// - `owner`: 目标所属类型，不能是未挂载
/// - `owner_id`: 目标记录标识
///
/// # 返回值
/// 改挂完成时返回空成功
///
/// # 错误
/// 目标记录不存在时返回 `ReferencedMissing`。目标已封存时返回 `OwnerSealed`。写入失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// attach(conn, &["file-id"], Owner::Plan, plan_id)?;
/// ```
pub(super) fn attach(conn: &Connection, ids: &[&str], owner: Owner, owner_id: &str) -> Result<(), UploadError> {
  let alive = owner::exists(conn, owner, owner_id).map_err(|e| db_fail("upload attach owner", e))?;
  if !alive {
    return Err(UploadError::ReferencedMissing);
  }
  let sealed = owner::sealed(conn, owner.as_str(), owner_id).map_err(|e| db_fail("upload attach sealed", e))?;
  if sealed {
    return Err(UploadError::OwnerSealed);
  }
  let tx = conn.unchecked_transaction().map_err(|e| db_fail("upload attach tx", e))?;
  let sql = format!("UPDATE {UPLOAD_TABLE} SET owner = ?1, owner_id = ?2 WHERE id = ?3 AND owner = 'loose'");
  for id in ids {
    tx.execute(&sql, params![owner.as_str(), owner_id, id]).map_err(|e| db_fail("upload attach", e))?;
  }
  tx.commit().map_err(|e| db_fail("upload attach commit", e))?;
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
