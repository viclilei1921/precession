pub mod commands;
pub mod dto;
pub mod error;

mod constants;
mod repository;
mod service;

pub(crate) use constants::{KIND_IMAGE, KIND_VIDEO};

use rusqlite::Connection;

use crate::db::error::DbError;

/// 迁移媒体表结构
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

/// 清除所属记录上的媒体
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
/// 删除失败时返回媒体错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// clear(conn, owner, owner_id)?;
/// ```
pub(crate) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), error::MediaError> {
  repository::clear(conn, owner, owner_id)
}

/// 列出所属记录上的媒体
///
/// # 参数
/// - `conn`: 数据库连接
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 返回媒体列表
///
/// # 错误
/// 查询失败时返回媒体错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list_for(conn, owner, owner_id)?;
/// ```
pub(crate) fn list_for(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<dto::Media>, error::MediaError> {
  repository::list(conn, owner, owner_id)
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
/// 读取失败时返回媒体错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = get(conn, id)?;
/// ```
pub(crate) fn get(conn: &Connection, id: &str) -> Result<dto::Media, error::MediaError> {
  repository::get(conn, id)
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
/// 更新失败时返回媒体错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// set_encrypted(conn, id, encrypted)?;
/// ```
pub(crate) fn set_encrypted(conn: &Connection, id: &str, encrypted: bool) -> Result<(), error::MediaError> {
  repository::set_encrypted(conn, id, encrypted)
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
/// 返回写入后的媒体
///
/// # 错误
/// 写入失败时返回媒体错误
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
  input: dto::MediaInput,
  encrypted: bool,
) -> Result<dto::Media, error::MediaError> {
  repository::insert_imported(conn, id, input, encrypted)
}
