//! 手记。校验放在进库之前。

use super::dto::{JournalCitation, JournalEntry, JournalEntryInput, JournalLink};
use super::error::JournalError;
use super::repository;
use crate::db::state::DbState;

pub fn list(
  db: &DbState,
  kind: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<JournalEntry>, JournalError> {
  if let Some(kind) = kind
    && !repository::allowed_kind(kind)
  {
    return Err(JournalError::KindInvalid);
  }
  db.with_conn(move |conn| repository::list(conn, kind, from, to))
}

pub fn get(db: &DbState, id: &str) -> Result<JournalEntry, JournalError> {
  db.with_conn(|conn| repository::get(conn, id))
}

pub fn create(db: &DbState, input: JournalEntryInput) -> Result<JournalEntry, JournalError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::create(conn, &input))
}

pub fn update(db: &DbState, id: &str, input: JournalEntryInput) -> Result<JournalEntry, JournalError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::update(conn, id, &input))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), JournalError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

pub fn link_list(db: &DbState, entry_id: &str) -> Result<Vec<JournalLink>, JournalError> {
  db.with_conn(|conn| repository::link_list(conn, entry_id))
}

pub fn link_create(db: &DbState, from_id: &str, to_id: &str, kind: &str) -> Result<JournalLink, JournalError> {
  let kind = kind.to_string();
  db.with_conn(move |conn| repository::link_create(conn, from_id, to_id, &kind))
}

pub fn link_delete(db: &DbState, id: &str) -> Result<(), JournalError> {
  db.with_conn(|conn| repository::link_delete(conn, id))
}

pub fn citation_list(db: &DbState, entry_id: &str) -> Result<Vec<JournalCitation>, JournalError> {
  db.with_conn(|conn| repository::citation_list(conn, entry_id))
}

pub fn citation_create(db: &DbState, entry_id: &str, book_note_id: &str) -> Result<JournalCitation, JournalError> {
  db.with_conn(|conn| repository::citation_create(conn, entry_id, book_note_id))
}

pub fn citation_delete(db: &DbState, id: &str) -> Result<(), JournalError> {
  db.with_conn(|conn| repository::citation_delete(conn, id))
}

fn prepare(mut input: JournalEntryInput) -> Result<JournalEntryInput, JournalError> {
  input.kind = input.kind.trim().to_string();
  input.title = input.title.trim().to_string();
  input.body = input.body.trim().to_string();
  if input.title.is_empty() {
    return Err(JournalError::TitleEmpty);
  }
  if !repository::allowed_kind(&input.kind) {
    return Err(JournalError::KindInvalid);
  }
  Ok(input)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::journal::constants::KIND_DIARY;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![
      crate::member::migrate,
      crate::tag::migrate,
      crate::place::migrate,
      crate::media::migrate,
      crate::journal::migrate,
    ]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn diary_and_lock() {
    let (_dir, db) = setup();
    let mut empty = sample();
    empty.title = " ".into();
    assert!(matches!(create(&db, empty), Err(JournalError::TitleEmpty)));

    let entry = create(&db, sample()).expect("create");
    let day = list(&db, Some(KIND_DIARY), Some(20), Some(21)).expect("list");
    assert_eq!(day.len(), 1);
    assert_eq!(day[0].id, entry.id);

    db.lock().expect("lock");
    assert!(matches!(list(&db, None, None, None), Err(JournalError::Locked)));
  }

  fn sample() -> JournalEntryInput {
    JournalEntryInput {
      kind: KIND_DIARY.into(),
      occurred_at: 20,
      title: "公园".into(),
      body: String::new(),
      locked: false,
      highlight: false,
      member_ids: Vec::new(),
      tag_ids: Vec::new(),
      place_ids: Vec::new(),
    }
  }
}
