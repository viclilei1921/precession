//! 媒体元数据。所属记录必须已经存在。

use super::dto::{Media, MediaInput};
use super::error::MediaError;
use super::repository;
use crate::db::state::DbState;

pub fn list(db: &DbState, owner: &str, owner_id: &str) -> Result<Vec<Media>, MediaError> {
  let owner = owner.to_string();
  let owner_id = owner_id.to_string();
  db.with_conn(move |conn| repository::list(conn, &owner, &owner_id))
}

pub fn create(db: &DbState, input: MediaInput) -> Result<Media, MediaError> {
  let (owner, input) = repository::prepare(input)?;
  db.with_conn(|conn| repository::create(conn, owner, &input))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), MediaError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

#[cfg(test)]
mod tests {
  use rusqlite::params;

  use super::*;
  use crate::db::error::DbError;
  use crate::media::constants::KIND_IMAGE;
  use crate::owner::Owner;
  use crate::utils::id::new_uuid_v4;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![
      crate::member::migrate,
      crate::tag::migrate,
      crate::place::migrate,
      crate::media::migrate,
      crate::plan::migrate,
    ]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn attach_and_lock() {
    let (_dir, db) = setup();
    assert!(matches!(
      create(
        &db,
        MediaInput {
          owner: Owner::Plan.as_str().into(),
          owner_id: "missing".into(),
          kind: KIND_IMAGE.into(),
          rel_path: "a.jpg".into(),
          mime: "image/jpeg".into(),
          sort: 0,
          locked: false,
        },
      ),
      Err(MediaError::ReferencedMissing)
    ));

    let plan_id = new_uuid_v4();
    db.with_conn(|conn| {
      conn
        .execute(
          "INSERT INTO plan (id, title, body, status, priority, result, locked, highlight, created_at, updated_at)
           VALUES (?1, '出门', '', 'inbox', 0, '', 0, 0, 1, 1)",
          params![plan_id],
        )
        .expect("plan");
      Ok::<(), DbError>(())
    })
    .expect("seed");

    let media = create(
      &db,
      MediaInput {
        owner: Owner::Plan.as_str().into(),
        owner_id: plan_id.clone(),
        kind: KIND_IMAGE.into(),
        rel_path: "a.jpg".into(),
        mime: "image/jpeg".into(),
        sort: 0,
        locked: false,
      },
    )
    .expect("media");
    assert!(!media.encrypted);
    assert_eq!(list(&db, Owner::Plan.as_str(), &plan_id).expect("list").len(), 1);

    delete(&db, &media.id).expect("delete");
    assert!(list(&db, Owner::Plan.as_str(), &plan_id).expect("empty").is_empty());

    db.lock().expect("lock");
    assert!(matches!(list(&db, Owner::Plan.as_str(), &plan_id), Err(MediaError::Locked)));
  }
}
