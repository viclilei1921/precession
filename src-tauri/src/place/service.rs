//! 地点。不持有连接，一律走 `DbState::with_conn`。

use super::dto::Place;
use super::error::PlaceError;
use super::repository;
use crate::db::state::DbState;

/// 列出地点
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 返回地点列表
///
/// # 错误
/// 查询失败时返回地点错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list(db)?;
/// ```
pub fn list(db: &DbState) -> Result<Vec<Place>, PlaceError> {
  db.with_conn(repository::list)
}

/// 校验名称后创建地点
///
/// # 参数
/// - `db`: 数据库状态
/// - `name`: 地点名称
///
/// # 返回值
/// 返回新建的地点
///
/// # 错误
/// 名称为空或写入失败时返回地点错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = create(db, name)?;
/// ```
pub fn create(db: &DbState, name: &str) -> Result<Place, PlaceError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::create(conn, name))
}

/// 校验名称后更新地点
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 地点标识
/// - `name`: 地点名称
///
/// # 返回值
/// 返回更新后的地点
///
/// # 错误
/// 名称为空或更新失败时返回地点错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = update(db, id, name)?;
/// ```
pub fn update(db: &DbState, id: &str, name: &str) -> Result<Place, PlaceError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::update(conn, id, name))
}

/// 按标识删除地点
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 地点标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回地点错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete(db, id)?;
/// ```
pub fn delete(db: &DbState, id: &str) -> Result<(), PlaceError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

/// 去掉地点名称的首尾空白并校验非空
///
/// # 参数
/// - `name`: 地点名称
///
/// # 返回值
/// 返回去掉首尾空白后的名称
///
/// # 错误
/// 去掉空白后为空时返回名称不能为空
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let name = trim_name(" 公园 ");
/// ```
fn trim_name(name: &str) -> Result<&str, PlaceError> {
  let name = name.trim();
  if name.is_empty() {
    return Err(PlaceError::NameEmpty);
  }
  Ok(name)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rename_and_lock() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::place::migrate]);
    db.create("test-password-123", "test-password-123").expect("create db");

    let place = create(&db, " 公园 ").expect("place");
    assert_eq!(place.name, "公园");
    let renamed = update(&db, &place.id, "小区花园").expect("rename");
    assert_eq!(renamed.name, "小区花园");

    db.lock().expect("lock");
    assert!(matches!(list(&db), Err(PlaceError::Locked)));
  }
}
