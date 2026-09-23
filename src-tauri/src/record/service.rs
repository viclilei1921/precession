//! 手记和成长记录。计划、书摘由各自模块写入同一张 `record`。

use super::constants::{
  LINK_EXCERPT_TO_JOURNAL, LINK_SPARK_TO_WRITING, LINK_WRITING_TO_DIARY, MEDIA_IMAGE, MEDIA_VIDEO, TYPE_DIARY,
  TYPE_MILESTONE, TYPE_MOMENT, TYPE_SPARK, TYPE_WRITING,
};
use super::dto::{MediaItem, RecordCard, RecordDetail, RecordLink, RecordWrite};
use super::error::RecordError;
use super::repository;
use crate::db::state::DbState;

const OWNED_TYPES: &[&str] = &[TYPE_DIARY, TYPE_SPARK, TYPE_WRITING, TYPE_MILESTONE, TYPE_MOMENT];
const GROWTH_TYPES: &[&str] = &[TYPE_MILESTONE, TYPE_MOMENT];
const LINK_KINDS: &[&str] = &[LINK_SPARK_TO_WRITING, LINK_WRITING_TO_DIARY, LINK_EXCERPT_TO_JOURNAL];
const MEDIA_KINDS: &[&str] = &[MEDIA_IMAGE, MEDIA_VIDEO];

pub fn list(
  db: &DbState,
  record_type: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<RecordCard>, RecordError> {
  if let Some(record_type) = record_type {
    validate_any_type(record_type)?;
  }
  db.with_conn(|conn| repository::list(conn, record_type, from, to))
}

pub fn get(db: &DbState, id: &str) -> Result<RecordDetail, RecordError> {
  db.with_conn(|conn| repository::get(conn, id))
}

pub fn create(db: &DbState, input: RecordWrite) -> Result<RecordDetail, RecordError> {
  let input = prepare_write(input)?;
  db.with_conn(|conn| repository::create(conn, &input))
}

pub fn update(db: &DbState, id: &str, input: RecordWrite) -> Result<RecordDetail, RecordError> {
  let input = prepare_write(input)?;
  db.with_conn(|conn| repository::update(conn, id, &input))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), RecordError> {
  db.with_conn(|conn| {
    let detail = repository::get(conn, id)?;
    if !OWNED_TYPES.contains(&detail.record.r#type.as_str()) {
      return Err(RecordError::TypeInvalid);
    }
    repository::delete(conn, id)
  })
}

pub fn media_list(db: &DbState, record_id: &str) -> Result<Vec<MediaItem>, RecordError> {
  db.with_conn(|conn| repository::media_list(conn, record_id))
}

pub fn media_create(
  db: &DbState,
  record_id: &str,
  media_kind: &str,
  rel_path: &str,
  mime: &str,
  sort: i64,
  locked: bool,
) -> Result<MediaItem, RecordError> {
  if !MEDIA_KINDS.contains(&media_kind) {
    return Err(RecordError::TypeInvalid);
  }
  db.with_conn(|conn| repository::media_create(conn, record_id, media_kind, rel_path.trim(), mime.trim(), sort, locked))
}

pub fn media_delete(db: &DbState, id: &str) -> Result<(), RecordError> {
  db.with_conn(|conn| repository::media_delete(conn, id))
}

pub fn link_list(db: &DbState, record_id: &str) -> Result<Vec<RecordLink>, RecordError> {
  db.with_conn(|conn| repository::link_list(conn, record_id))
}

pub fn link_create(db: &DbState, from_id: &str, to_id: &str, kind: &str) -> Result<RecordLink, RecordError> {
  if !LINK_KINDS.contains(&kind) {
    return Err(RecordError::TypeInvalid);
  }
  db.with_conn(|conn| repository::link_create(conn, from_id, to_id, kind))
}

pub fn link_delete(db: &DbState, id: &str) -> Result<(), RecordError> {
  db.with_conn(|conn| repository::link_delete(conn, id))
}

fn prepare_write(mut input: RecordWrite) -> Result<RecordWrite, RecordError> {
  input.r#type = input.r#type.trim().to_string();
  input.title = input.title.trim().to_string();
  input.body = input.body.trim().to_string();
  if input.title.is_empty() {
    return Err(RecordError::TitleEmpty);
  }
  if !OWNED_TYPES.contains(&input.r#type.as_str()) {
    return Err(RecordError::TypeInvalid);
  }
  let subject = input.subject_member_id.take().map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
  let growth = GROWTH_TYPES.contains(&input.r#type.as_str());
  if growth && subject.is_none() {
    return Err(RecordError::SubjectRequired);
  }
  if !growth && subject.is_some() {
    return Err(RecordError::SubjectNotAllowed);
  }
  input.subject_member_id = subject;
  Ok(input)
}

fn validate_any_type(record_type: &str) -> Result<(), RecordError> {
  const ALL: &[&str] = &[TYPE_DIARY, TYPE_SPARK, TYPE_WRITING, TYPE_MILESTONE, TYPE_MOMENT, "plan", "excerpt", "note"];
  if ALL.contains(&record_type) { Ok(()) } else { Err(RecordError::TypeInvalid) }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db =
      DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::catalog::migrate, crate::record::migrate]);
    db.create("test-password-123").expect("create db");
    (dir, db)
  }

  fn diary(title: &str, occurred_at: i64) -> RecordWrite {
    RecordWrite {
      r#type: TYPE_DIARY.into(),
      occurred_at,
      title: title.into(),
      body: String::new(),
      subject_member_id: None,
      locked: false,
      highlight: false,
      member_ids: Vec::new(),
      tag_ids: Vec::new(),
      place_ids: Vec::new(),
    }
  }

  #[test]
  fn journal_growth_and_lock() {
    let (_dir, db) = setup();
    assert!(matches!(create(&db, diary("  ", 1)), Err(RecordError::TitleEmpty)));

    let mut milestone = diary("学会站", 10);
    milestone.r#type = TYPE_MILESTONE.into();
    assert!(matches!(create(&db, milestone), Err(RecordError::SubjectRequired)));

    let member_id = crate::utils::id::new_uuid_v4();
    db.with_conn(|conn| {
      conn
        .execute(
          "INSERT INTO member (id, name, relation, gender, created_at, updated_at) VALUES (?1, '宝宝', 'child', '', ?2, ?2)",
          rusqlite::params![member_id, 1_i64],
        )
        .map_err(|_| RecordError::Internal)
    })
    .expect("member");

    let mut milestone = diary("学会站", 10);
    milestone.r#type = TYPE_MILESTONE.into();
    milestone.subject_member_id = Some(member_id.clone());
    milestone.highlight = true;
    let created = create(&db, milestone).expect("milestone");
    assert!(created.record.highlight);
    assert_eq!(created.record.subject_member_id.as_deref(), Some(member_id.as_str()));

    let note = create(&db, diary("公园", 20)).expect("diary");
    let day = list(&db, Some(TYPE_DIARY), Some(20), Some(21)).expect("list");
    assert_eq!(day.len(), 1);
    assert_eq!(day[0].id, note.record.id);

    db.lock().expect("lock");
    assert!(matches!(list(&db, None, None, None), Err(RecordError::Locked)));
  }
}
