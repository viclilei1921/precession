//! 标签。不持有连接，一律走 `DbState::with_conn`。

use super::dto::Tag;
use super::error::TagError;
use super::repository;
use crate::db::state::DbState;

/// 列出标签
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 返回标签列表
///
/// # 错误
/// 查询失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list(db)?;
/// ```
pub fn list(db: &DbState) -> Result<Vec<Tag>, TagError> {
  db.with_conn(repository::list)
}

/// 校验名称后创建标签
///
/// # 参数
/// - `db`: 数据库状态
/// - `name`: 标签名称
///
/// # 返回值
/// 返回新建的标签
///
/// # 错误
/// 名称为空或写入失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = create(db, name)?;
/// ```
pub fn create(db: &DbState, name: &str) -> Result<Tag, TagError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::create(conn, name))
}

/// 校验名称后更新标签
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 标签标识
/// - `name`: 标签名称
///
/// # 返回值
/// 返回更新后的标签
///
/// # 错误
/// 名称为空或更新失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = update(db, id, name)?;
/// ```
pub fn update(db: &DbState, id: &str, name: &str) -> Result<Tag, TagError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::update(conn, id, name))
}

/// 按标识删除标签
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 标签标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete(db, id)?;
/// ```
pub fn delete(db: &DbState, id: &str) -> Result<(), TagError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

/// 去掉标签名称的首尾空白并校验非空
///
/// # 参数
/// - `name`: 标签名称
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
/// let name = trim_name(" 成长 ");
/// ```
fn trim_name(name: &str) -> Result<&str, TagError> {
  let name = name.trim();
  if name.is_empty() {
    return Err(TagError::NameEmpty);
  }
  Ok(name)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::tag::migrate]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn unique_name_and_lock() {
    let (_dir, db) = setup();
    assert!(matches!(create(&db, "  "), Err(TagError::NameEmpty)));
    let tag = create(&db, "成长").expect("tag");
    assert!(matches!(create(&db, "成长"), Err(TagError::NameTaken)));
    delete(&db, &tag.id).expect("delete");
    let again = create(&db, "成长").expect("reuse");
    assert_ne!(again.id, tag.id);
    db.lock().expect("lock");
    assert!(matches!(list(&db), Err(TagError::Locked)));
  }
}
