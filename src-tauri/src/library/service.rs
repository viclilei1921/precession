//! 书库。书摘和笔记写入 `book_note`。

use super::constants::{STATUS_FINISHED, STATUS_READING, STATUS_WANT};
use super::dto::{Book, BookInput, BookNote, BookNoteInput};
use super::error::LibraryError;
use super::repository;
use crate::db::state::DbState;

const STATUSES: &[&str] = &[STATUS_WANT, STATUS_READING, STATUS_FINISHED];

pub fn book_list(db: &DbState) -> Result<Vec<Book>, LibraryError> {
  db.with_conn(repository::book_list)
}

pub fn book_get(db: &DbState, id: &str) -> Result<Book, LibraryError> {
  db.with_conn(|conn| repository::book_get(conn, id))
}

pub fn book_create(db: &DbState, input: BookInput) -> Result<Book, LibraryError> {
  let input = prepare_book(input)?;
  db.with_conn(|conn| repository::book_create(conn, &input))
}

pub fn book_update(db: &DbState, id: &str, input: BookInput) -> Result<Book, LibraryError> {
  let input = prepare_book(input)?;
  db.with_conn(|conn| repository::book_update(conn, id, &input))
}

pub fn book_delete(db: &DbState, id: &str) -> Result<(), LibraryError> {
  db.with_conn(|conn| repository::book_delete(conn, id))
}

pub fn note_list(db: &DbState, book_id: &str) -> Result<Vec<BookNote>, LibraryError> {
  db.with_conn(|conn| repository::note_list(conn, book_id))
}

pub fn note_get(db: &DbState, id: &str) -> Result<BookNote, LibraryError> {
  db.with_conn(|conn| repository::note_get(conn, id))
}

pub fn note_create(db: &DbState, input: BookNoteInput) -> Result<BookNote, LibraryError> {
  let input = prepare_note(input)?;
  db.with_conn(|conn| repository::note_create(conn, &input))
}

pub fn note_update(db: &DbState, id: &str, input: BookNoteInput) -> Result<BookNote, LibraryError> {
  let input = prepare_note(input)?;
  db.with_conn(|conn| repository::note_update(conn, id, &input))
}

pub fn note_delete(db: &DbState, id: &str) -> Result<(), LibraryError> {
  db.with_conn(|conn| repository::note_delete(conn, id))
}

fn prepare_book(mut input: BookInput) -> Result<BookInput, LibraryError> {
  input.title = input.title.trim().to_string();
  input.author = input.author.trim().to_string();
  input.cover_path = input.cover_path.trim().to_string();
  input.status = input.status.trim().to_string();
  if input.title.is_empty() {
    return Err(LibraryError::TitleEmpty);
  }
  if !STATUSES.contains(&input.status.as_str()) {
    return Err(LibraryError::StatusInvalid);
  }
  if !(0.0..=1.0).contains(&input.progress) {
    return Err(LibraryError::ProgressInvalid);
  }
  Ok(input)
}

fn prepare_note(mut input: BookNoteInput) -> Result<BookNoteInput, LibraryError> {
  input.book_id = input.book_id.trim().to_string();
  input.kind = input.kind.trim().to_string();
  input.title = input.title.trim().to_string();
  input.body = input.body.trim().to_string();
  input.chapter = input.chapter.trim().to_string();
  input.location = input.location.trim().to_string();
  if input.title.is_empty() {
    return Err(LibraryError::TitleEmpty);
  }
  if !repository::allowed_kind(&input.kind) {
    return Err(LibraryError::KindInvalid);
  }
  Ok(input)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::library::constants::KIND_EXCERPT;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![
      crate::member::migrate,
      crate::tag::migrate,
      crate::place::migrate,
      crate::media::migrate,
      crate::journal::migrate,
      crate::library::migrate,
    ]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  fn book() -> BookInput {
    BookInput {
      title: "置身事内".into(),
      author: "兰小欢".into(),
      cover_path: String::new(),
      status: STATUS_READING.into(),
      progress: 0.62,
      rating: None,
      started_at: Some(1),
      finished_at: None,
    }
  }

  #[test]
  fn book_note_and_lock() {
    let (_dir, db) = setup();
    let mut bad = book();
    bad.progress = 1.2;
    assert!(matches!(book_create(&db, bad), Err(LibraryError::ProgressInvalid)));

    let saved = book_create(&db, book()).expect("book");
    let note = note_create(
      &db,
      BookNoteInput {
        book_id: saved.id.clone(),
        kind: KIND_EXCERPT.into(),
        occurred_at: 20,
        title: "事权与财权".into(),
        body: "划分与再平衡".into(),
        chapter: "第 4 章".into(),
        location: "P301".into(),
        locked: false,
        highlight: false,
        member_ids: Vec::new(),
        tag_ids: Vec::new(),
        place_ids: Vec::new(),
      },
    )
    .expect("note");
    assert_eq!(note_list(&db, &saved.id).expect("notes").len(), 1);
    assert_eq!(note.location, "P301");

    db.lock().expect("lock");
    assert!(matches!(book_list(&db), Err(LibraryError::Locked)));
  }
}
