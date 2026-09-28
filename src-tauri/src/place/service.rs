//! 地点。不持有连接，一律走 `DbState::with_conn`。

use super::dto::Place;
use super::error::PlaceError;
use super::repository;
use crate::db::state::DbState;

pub fn list(db: &DbState) -> Result<Vec<Place>, PlaceError> {
  db.with_conn(repository::list)
}

pub fn create(db: &DbState, name: &str) -> Result<Place, PlaceError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::create(conn, name))
}

pub fn update(db: &DbState, id: &str, name: &str) -> Result<Place, PlaceError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::update(conn, id, name))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), PlaceError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

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
