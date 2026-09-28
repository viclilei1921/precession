use tauri::State;

use super::dto::{Book, BookInput, BookNote, BookNoteInput};
use super::error::LibraryError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub async fn book_list(state: State<'_, DbState>) -> Result<Vec<Book>, LibraryError> {
  service::book_list(&state)
}

#[tauri::command]
pub async fn book_get(state: State<'_, DbState>, id: String) -> Result<Book, LibraryError> {
  service::book_get(&state, id.as_str())
}

#[tauri::command]
pub async fn book_create(state: State<'_, DbState>, input: BookInput) -> Result<Book, LibraryError> {
  service::book_create(&state, input)
}

#[tauri::command]
pub async fn book_update(state: State<'_, DbState>, id: String, input: BookInput) -> Result<Book, LibraryError> {
  service::book_update(&state, id.as_str(), input)
}

#[tauri::command]
pub async fn book_delete(state: State<'_, DbState>, id: String) -> Result<(), LibraryError> {
  service::book_delete(&state, id.as_str())
}

#[tauri::command]
pub async fn book_note_list(state: State<'_, DbState>, book_id: String) -> Result<Vec<BookNote>, LibraryError> {
  service::note_list(&state, book_id.as_str())
}

#[tauri::command]
pub async fn book_note_get(state: State<'_, DbState>, id: String) -> Result<BookNote, LibraryError> {
  service::note_get(&state, id.as_str())
}

#[tauri::command]
pub async fn book_note_create(state: State<'_, DbState>, input: BookNoteInput) -> Result<BookNote, LibraryError> {
  service::note_create(&state, input)
}

#[tauri::command]
pub async fn book_note_update(
  state: State<'_, DbState>,
  id: String,
  input: BookNoteInput,
) -> Result<BookNote, LibraryError> {
  service::note_update(&state, id.as_str(), input)
}

#[tauri::command]
pub async fn book_note_delete(state: State<'_, DbState>, id: String) -> Result<(), LibraryError> {
  service::note_delete(&state, id.as_str())
}
