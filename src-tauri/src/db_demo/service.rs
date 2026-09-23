//! 示例表业务 API。不持有连接，一律走 `DbState::with_conn`。
//!
//! 跨表写入必须在同一次 `with_conn` 里调用多个 repository，必要时开事务；
//! 不要每个 repository 自己再走一遍 `with_conn`。

use super::dto::DemoItem;
use super::error::DemoError;
use super::repository;
use crate::db::state::DbState;

pub fn list(db: &DbState) -> Result<Vec<DemoItem>, DemoError> {
  db.with_conn(repository::list)
}

pub fn get(db: &DbState, id: &str) -> Result<DemoItem, DemoError> {
  db.with_conn(|conn| repository::get(conn, id))
}

pub fn create(db: &DbState, title: &str, body: &str) -> Result<DemoItem, DemoError> {
  let title = title.trim();
  if title.is_empty() {
    return Err(DemoError::TitleEmpty);
  }
  db.with_conn(|conn| repository::create(conn, title, body))
}

pub fn update(db: &DbState, id: &str, title: &str, body: &str) -> Result<DemoItem, DemoError> {
  let title = title.trim();
  if title.is_empty() {
    return Err(DemoError::TitleEmpty);
  }
  db.with_conn(|conn| repository::update(conn, id, title, body))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), DemoError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::db_demo::migrate]);
    db.create("test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn crud_and_survives_lock() {
    let (_dir, db) = setup();

    assert!(matches!(create(&db, "  ", "x"), Err(DemoError::TitleEmpty)));

    let item = create(&db, "第一篇", "正文").expect("create");
    assert_eq!(item.title, "第一篇");
    assert_eq!(item.body, "正文");
    assert!(!item.id.is_empty());

    let listed = list(&db).expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, item.id);

    let got = get(&db, &item.id).expect("get");
    assert_eq!(got.title, "第一篇");

    let updated = update(&db, &item.id, "改过的标题", "新正文").expect("update");
    assert_eq!(updated.title, "改过的标题");
    assert_eq!(updated.body, "新正文");
    assert!(updated.updated_at >= item.updated_at);

    db.lock().expect("lock");
    assert!(matches!(list(&db), Err(DemoError::Locked)));
    db.unlock("test-password-123").expect("unlock");

    let after_unlock = list(&db).expect("list after unlock");
    assert_eq!(after_unlock.len(), 1);
    assert_eq!(after_unlock[0].title, "改过的标题");

    delete(&db, &item.id).expect("delete");
    assert!(list(&db).expect("list empty").is_empty());
    assert!(matches!(get(&db, &item.id), Err(DemoError::RecordNotFound)));
  }
}
