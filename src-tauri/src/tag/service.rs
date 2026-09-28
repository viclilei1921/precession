//! 标签。不持有连接，一律走 `DbState::with_conn`。

use super::dto::Tag;
use super::error::TagError;
use super::repository;
use crate::db::state::DbState;

pub fn list(db: &DbState) -> Result<Vec<Tag>, TagError> {
  db.with_conn(repository::list)
}

pub fn create(db: &DbState, name: &str) -> Result<Tag, TagError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::create(conn, name))
}

pub fn update(db: &DbState, id: &str, name: &str) -> Result<Tag, TagError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::update(conn, id, name))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), TagError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

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
