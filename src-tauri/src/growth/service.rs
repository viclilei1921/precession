//! 成长记录。里程碑和精彩瞬间都必须指定成员。

use super::dto::{GrowthEntry, GrowthEntryInput};
use super::error::GrowthError;
use super::repository;
use crate::db::state::DbState;

pub fn list(
  db: &DbState,
  member_id: Option<&str>,
  kind: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<GrowthEntry>, GrowthError> {
  if let Some(kind) = kind
    && !repository::allowed_kind(kind)
  {
    return Err(GrowthError::KindInvalid);
  }
  db.with_conn(move |conn| repository::list(conn, member_id, kind, from, to))
}

pub fn get(db: &DbState, id: &str) -> Result<GrowthEntry, GrowthError> {
  db.with_conn(|conn| repository::get(conn, id))
}

pub fn create(db: &DbState, input: GrowthEntryInput) -> Result<GrowthEntry, GrowthError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::create(conn, &input))
}

pub fn update(db: &DbState, id: &str, input: GrowthEntryInput) -> Result<GrowthEntry, GrowthError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::update(conn, id, &input))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), GrowthError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

fn prepare(mut input: GrowthEntryInput) -> Result<GrowthEntryInput, GrowthError> {
  input.kind = input.kind.trim().to_string();
  input.member_id = input.member_id.trim().to_string();
  input.title = input.title.trim().to_string();
  input.body = input.body.trim().to_string();
  if input.title.is_empty() {
    return Err(GrowthError::TitleEmpty);
  }
  if !repository::allowed_kind(&input.kind) {
    return Err(GrowthError::KindInvalid);
  }
  if input.member_id.is_empty() {
    return Err(GrowthError::MemberRequired);
  }
  Ok(input)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::growth::constants::KIND_MILESTONE;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![
      crate::member::migrate,
      crate::tag::migrate,
      crate::place::migrate,
      crate::media::migrate,
      crate::growth::migrate,
    ]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn milestone_requires_member_and_lock() {
    let (_dir, db) = setup();
    let mut input = sample("missing");
    input.member_id.clear();
    assert!(matches!(create(&db, input), Err(GrowthError::MemberRequired)));

    let member_id = crate::utils::id::new_uuid_v4();
    db.with_conn(|conn| {
      conn
        .execute(
          "INSERT INTO member (id, name, relation, gender, created_at, updated_at) VALUES (?1, '宝宝', 'child', '', ?2, ?2)",
          rusqlite::params![member_id, 1_i64],
        )
        .map_err(|_| GrowthError::Internal)
    })
    .expect("member");

    let created = create(&db, sample(&member_id)).expect("milestone");
    assert!(created.highlight);
    assert_eq!(created.member_id, member_id);

    db.lock().expect("lock");
    assert!(matches!(list(&db, None, None, None, None), Err(GrowthError::Locked)));
  }

  fn sample(member_id: &str) -> GrowthEntryInput {
    GrowthEntryInput {
      kind: KIND_MILESTONE.into(),
      member_id: member_id.into(),
      occurred_at: 10,
      title: "学会站".into(),
      body: String::new(),
      locked: false,
      highlight: true,
      member_ids: Vec::new(),
      tag_ids: Vec::new(),
      place_ids: Vec::new(),
    }
  }
}
