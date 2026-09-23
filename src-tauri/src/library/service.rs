//! 书库。书摘和笔记同一次事务写入 `record` 与 `book_quote`。

use super::constants::{STATUS_DONE, STATUS_READING, STATUS_WANT, TYPE_EXCERPT, TYPE_NOTE};
use super::dto::{Book, BookWrite, QuoteItem, QuoteWrite};
use super::error::LibraryError;
use super::repository;
use crate::db::state::DbState;

const STATUSES: &[&str] = &[STATUS_WANT, STATUS_READING, STATUS_DONE];
const QUOTE_TYPES: &[&str] = &[TYPE_EXCERPT, TYPE_NOTE];

pub fn book_list(db: &DbState) -> Result<Vec<Book>, LibraryError> {
  db.with_conn(repository::book_list)
}

pub fn book_get(db: &DbState, id: &str) -> Result<Book, LibraryError> {
  db.with_conn(|conn| repository::book_get(conn, id))
}

pub fn book_create(db: &DbState, input: BookWrite) -> Result<Book, LibraryError> {
  let input = prepare_book(input)?;
  db.with_conn(|conn| repository::book_create(conn, &input))
}

pub fn book_update(db: &DbState, id: &str, input: BookWrite) -> Result<Book, LibraryError> {
  let input = prepare_book(input)?;
  db.with_conn(|conn| repository::book_update(conn, id, &input))
}

pub fn book_delete(db: &DbState, id: &str) -> Result<(), LibraryError> {
  db.with_conn(|conn| repository::book_delete(conn, id))
}

pub fn quote_list(db: &DbState, book_id: &str) -> Result<Vec<QuoteItem>, LibraryError> {
  db.with_conn(|conn| repository::quote_list(conn, book_id))
}

pub fn quote_get(db: &DbState, id: &str) -> Result<QuoteItem, LibraryError> {
  db.with_conn(|conn| repository::quote_get(conn, id))
}

pub fn quote_create(db: &DbState, input: QuoteWrite) -> Result<QuoteItem, LibraryError> {
  let input = prepare_quote(input)?;
  db.with_conn(|conn| repository::quote_create(conn, &input))
}

pub fn quote_update(db: &DbState, id: &str, input: QuoteWrite) -> Result<QuoteItem, LibraryError> {
  let input = prepare_quote(input)?;
  db.with_conn(|conn| repository::quote_update(conn, id, &input))
}

pub fn quote_delete(db: &DbState, id: &str) -> Result<(), LibraryError> {
  db.with_conn(|conn| repository::quote_delete(conn, id))
}

fn prepare_book(mut input: BookWrite) -> Result<BookWrite, LibraryError> {
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

fn prepare_quote(mut input: QuoteWrite) -> Result<QuoteWrite, LibraryError> {
  input.book_id = input.book_id.trim().to_string();
  input.r#type = input.r#type.trim().to_string();
  input.title = input.title.trim().to_string();
  input.body = input.body.trim().to_string();
  input.chapter = input.chapter.trim().to_string();
  input.location = input.location.trim().to_string();
  if input.title.is_empty() {
    return Err(LibraryError::TitleEmpty);
  }
  if !QUOTE_TYPES.contains(&input.r#type.as_str()) {
    return Err(LibraryError::TypeInvalid);
  }
  Ok(input)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db =
      DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::record::migrate, crate::library::migrate]);
    db.create("test-password-123").expect("create db");
    (dir, db)
  }

  fn book() -> BookWrite {
    BookWrite {
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
  fn book_quote_and_lock() {
    let (_dir, db) = setup();
    let mut bad = book();
    bad.progress = 1.2;
    assert!(matches!(book_create(&db, bad), Err(LibraryError::ProgressInvalid)));

    let saved = book_create(&db, book()).expect("book");
    let quote = quote_create(
      &db,
      QuoteWrite {
        book_id: saved.id.clone(),
        r#type: TYPE_EXCERPT.into(),
        occurred_at: 20,
        title: "事权与财权".into(),
        body: "划分与再平衡".into(),
        chapter: "第 4 章".into(),
        location: "P301".into(),
      },
    )
    .expect("quote");
    assert_eq!(quote_list(&db, &saved.id).expect("quotes").len(), 1);
    assert_eq!(quote.location, "P301");

    let touched = book_get(&db, &saved.id).expect("book");
    assert!(touched.updated_at >= saved.updated_at);

    db.lock().expect("lock");
    assert!(matches!(book_list(&db), Err(LibraryError::Locked)));
  }
}
