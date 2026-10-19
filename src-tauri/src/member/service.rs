//! 成员。不持有连接，一律走 `DbState::with_conn`。

use super::dto::{Member, MemberInput};
use super::error::MemberError;
use super::repository;
use crate::db::state::DbState;

/// 列出成员
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 返回成员列表
///
/// # 错误
/// 连接或查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let members = list(db)?;
/// ```
pub fn list(db: &DbState) -> Result<Vec<Member>, MemberError> {
  db.with_conn(repository::list)
}

/// 按主键读取成员
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 成员主键
///
/// # 返回值
/// 返回成员
///
/// # 错误
/// 连接或查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let member = get(db, id)?;
/// ```
pub fn get(db: &DbState, id: &str) -> Result<Member, MemberError> {
  db.with_conn(|conn| repository::get(conn, id))
}

/// 整理输入后创建成员
///
/// # 参数
/// - `db`: 数据库状态
/// - `input`: 成员内容
///
/// # 返回值
/// 返回新建的成员
///
/// # 错误
/// 输入不合法或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let member = create(db, input)?;
/// ```
pub fn create(db: &DbState, input: MemberInput) -> Result<Member, MemberError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::create(conn, &input))
}

/// 整理输入后更新成员
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 成员主键
/// - `input`: 成员内容
///
/// # 返回值
/// 返回更新后的成员
///
/// # 错误
/// 输入不合法或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let member = update(db, id, input)?;
/// ```
pub fn update(db: &DbState, id: &str, input: MemberInput) -> Result<Member, MemberError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::update(conn, id, &input))
}

/// 删除成员
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 成员主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 连接或删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete(db, id)?;
/// ```
pub fn delete(db: &DbState, id: &str) -> Result<(), MemberError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

/// 去掉名称、关系和性别的空白，并拒绝空名称
///
/// # 参数
/// - `input`: 成员内容
///
/// # 返回值
/// 返回整理后的输入
///
/// # 错误
/// 名称去空白后为空时返回 `MemberError::NameEmpty`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let input = prepare(input)?;
/// ```
fn prepare(mut input: MemberInput) -> Result<MemberInput, MemberError> {
  input.name = input.name.trim().to_string();
  if input.name.is_empty() {
    return Err(MemberError::NameEmpty);
  }
  input.relation = input.relation.trim().to_string();
  input.gender = input.gender.trim().to_string();
  Ok(input)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::member::migrate]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn crud_and_lock() {
    let (_dir, db) = setup();
    assert!(matches!(
      create(
        &db,
        MemberInput { name: "  ".into(), relation: String::new(), gender: String::new(), birthday: None }
      ),
      Err(MemberError::NameEmpty)
    ));

    let member = create(
      &db,
      MemberInput { name: " 宝宝 ".into(), relation: "child".into(), gender: "男".into(), birthday: Some(1) },
    )
    .expect("create");
    assert_eq!(member.name, "宝宝");
    assert_eq!(list(&db).expect("list").len(), 1);

    delete(&db, &member.id).expect("delete");
    assert!(list(&db).expect("empty").is_empty());

    db.lock().expect("lock");
    assert!(matches!(list(&db), Err(MemberError::Locked)));
  }
}
