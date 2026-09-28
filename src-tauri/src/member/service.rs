//! 成员。不持有连接，一律走 `DbState::with_conn`。

use super::dto::{Member, MemberInput};
use super::error::MemberError;
use super::repository;
use crate::db::state::DbState;

pub fn list(db: &DbState) -> Result<Vec<Member>, MemberError> {
  db.with_conn(repository::list)
}

pub fn get(db: &DbState, id: &str) -> Result<Member, MemberError> {
  db.with_conn(|conn| repository::get(conn, id))
}

pub fn create(db: &DbState, input: MemberInput) -> Result<Member, MemberError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::create(conn, &input))
}

pub fn update(db: &DbState, id: &str, input: MemberInput) -> Result<Member, MemberError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::update(conn, id, &input))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), MemberError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

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
