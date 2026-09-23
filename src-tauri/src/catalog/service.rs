//! 成员、标签、地点。不持有连接，一律走 `DbState::with_conn`。

use super::dto::{Member, MemberWrite, Place, Tag};
use super::error::CatalogError;
use super::repository;
use crate::db::state::DbState;

pub fn member_list(db: &DbState) -> Result<Vec<Member>, CatalogError> {
  db.with_conn(repository::member_list)
}

pub fn member_get(db: &DbState, id: &str) -> Result<Member, CatalogError> {
  db.with_conn(|conn| repository::member_get(conn, id))
}

pub fn member_create(db: &DbState, input: MemberWrite) -> Result<Member, CatalogError> {
  let input = trim_member(input)?;
  db.with_conn(|conn| repository::member_create(conn, &input))
}

pub fn member_update(db: &DbState, id: &str, input: MemberWrite) -> Result<Member, CatalogError> {
  let input = trim_member(input)?;
  db.with_conn(|conn| repository::member_update(conn, id, &input))
}

pub fn member_delete(db: &DbState, id: &str) -> Result<(), CatalogError> {
  db.with_conn(|conn| repository::member_delete(conn, id))
}

pub fn tag_list(db: &DbState) -> Result<Vec<Tag>, CatalogError> {
  db.with_conn(repository::tag_list)
}

pub fn tag_create(db: &DbState, name: &str) -> Result<Tag, CatalogError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::tag_create(conn, name))
}

pub fn tag_update(db: &DbState, id: &str, name: &str) -> Result<Tag, CatalogError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::tag_update(conn, id, name))
}

pub fn tag_delete(db: &DbState, id: &str) -> Result<(), CatalogError> {
  db.with_conn(|conn| repository::tag_delete(conn, id))
}

pub fn place_list(db: &DbState) -> Result<Vec<Place>, CatalogError> {
  db.with_conn(repository::place_list)
}

pub fn place_create(db: &DbState, name: &str) -> Result<Place, CatalogError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::place_create(conn, name))
}

pub fn place_update(db: &DbState, id: &str, name: &str) -> Result<Place, CatalogError> {
  let name = trim_name(name)?;
  db.with_conn(|conn| repository::place_update(conn, id, name))
}

pub fn place_delete(db: &DbState, id: &str) -> Result<(), CatalogError> {
  db.with_conn(|conn| repository::place_delete(conn, id))
}

fn trim_name(name: &str) -> Result<&str, CatalogError> {
  let name = name.trim();
  if name.is_empty() {
    return Err(CatalogError::NameEmpty);
  }
  Ok(name)
}

fn trim_member(mut input: MemberWrite) -> Result<MemberWrite, CatalogError> {
  input.name = input.name.trim().to_string();
  if input.name.is_empty() {
    return Err(CatalogError::NameEmpty);
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
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::catalog::migrate]);
    db.create("test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn crud_soft_delete_and_lock() {
    let (_dir, db) = setup();

    assert!(matches!(
      member_create(
        &db,
        MemberWrite { name: "  ".into(), relation: String::new(), gender: String::new(), birthday: None }
      ),
      Err(CatalogError::NameEmpty)
    ));

    let member = member_create(
      &db,
      MemberWrite {
        name: " 宝宝 ".into(),
        relation: "child".into(),
        gender: "男".into(),
        birthday: Some(1_741_363_200_000),
      },
    )
    .expect("create member");
    assert_eq!(member.name, "宝宝");
    assert_eq!(member_list(&db).expect("list").len(), 1);

    member_delete(&db, &member.id).expect("delete");
    assert!(member_list(&db).expect("list empty").is_empty());
    assert!(matches!(member_get(&db, &member.id), Err(CatalogError::RecordNotFound)));

    let tag = tag_create(&db, "成长").expect("tag");
    assert!(matches!(tag_create(&db, "成长"), Err(CatalogError::NameTaken)));
    tag_delete(&db, &tag.id).expect("delete tag");
    let again = tag_create(&db, "成长").expect("reuse name");
    assert_ne!(again.id, tag.id);

    let place = place_create(&db, "公园").expect("place");
    let renamed = place_update(&db, &place.id, "小区花园").expect("rename");
    assert_eq!(renamed.name, "小区花园");

    db.lock().expect("lock");
    assert!(matches!(member_list(&db), Err(CatalogError::Locked)));
  }
}
