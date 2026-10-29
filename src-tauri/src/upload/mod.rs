//! 上传文件。把窗口里选的文件导入应用，放进和数据库平级的 `upload` 目录，
//! 在业务记录上登记，再按需读取：图片查看、视频播放和拖动、书籍阅读。
//!
//! - `repository`、`service`：文件记录的增删查
//! - `storage`：落盘位置、导入提交锁、孤儿清理
//! - `media`：本机按区间读出明文，给图片和视频拖动进度

pub mod commands;
pub mod dto;
pub mod error;

mod constants;
pub(crate) mod media;
mod repository;
mod service;
mod storage;

pub(crate) use service::{attach, delete, list};

pub(crate) use constants::{KIND_IMAGE, KIND_VIDEO, KINDS, TMP_DIR, UPLOAD_DIR};
pub(crate) use storage::{clear_tmp, commit_guard, ext_of, on_open, rel_path, resolve, sweep_later, tmp_path};

use rusqlite::Connection;

use crate::db::error::DbError;

/// 迁移上传文件表结构
///
/// # 参数
/// - `conn`: 数据库连接
///
/// # 返回值
/// 迁移完成时返回空成功
///
/// # 错误
/// 模式迁移失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// migrate(conn)?;
/// ```
pub(crate) fn migrate(conn: &Connection) -> Result<(), DbError> {
  repository::ensure_schema(conn).map_err(|_| DbError::Internal)
}

/// 清除所属记录上的文件记录
///
/// 只删记录。文件由所属记录删除后的 [`sweep_later`] 清掉。
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 清除完成时返回空成功
///
/// # 错误
/// 删除失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// clear(conn, owner, owner_id)?;
/// ```
pub(crate) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), error::UploadError> {
  repository::clear(conn, owner, owner_id)
}

/// 列出所属记录上的文件
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 返回文件记录列表
///
/// # 错误
/// 查询失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list_for(conn, owner, owner_id)?;
/// ```
pub(crate) fn list_for(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<dto::Upload>, error::UploadError> {
  repository::list(conn, owner, owner_id)
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
/// 读取失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = get(conn, id)?;
/// ```
pub(crate) fn get(conn: &Connection, id: &str) -> Result<dto::Upload, error::UploadError> {
  repository::get(conn, id)
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
pub(crate) fn set_encrypted_name(
  conn: &Connection,
  id: &str,
  encrypted: bool,
  name: &str,
) -> Result<(), error::UploadError> {
  repository::set_encrypted_name(conn, id, encrypted, name)
}

/// 校验输入后按指定标识写入导入的文件记录
///
/// # 参数
/// - `conn`: 数据库连接
/// - `id`: 文件标识
/// - `input`: 文件输入
/// - `encrypted`: 是否已加密
///
/// # 返回值
/// 返回写入后的文件记录
///
/// # 错误
/// 输入不合法、所属记录不存在或写入失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = insert_imported(conn, id, input, encrypted)?;
/// ```
pub(crate) fn insert_imported(
  conn: &Connection,
  id: &str,
  input: dto::UploadInput,
  encrypted: bool,
) -> Result<dto::Upload, error::UploadError> {
  let (owner, input) = repository::prepare(input)?;
  repository::insert(conn, id, owner, &input, encrypted)
}
