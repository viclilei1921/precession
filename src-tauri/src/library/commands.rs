use tauri::State;

use super::dto::{Book, BookWrite, QuoteItem, QuoteWrite};
use super::error::LibraryError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub fn book_list(state: State<'_, DbState>) -> Result<Vec<Book>, LibraryError> {
  service::book_list(&state)
}

#[tauri::command]
pub fn book_get(state: State<'_, DbState>, id: String) -> Result<Book, LibraryError> {
  service::book_get(&state, id.as_str())
}

#[tauri::command]
pub fn book_create(state: State<'_, DbState>, input: BookWrite) -> Result<Book, LibraryError> {
  service::book_create(&state, input)
}

#[tauri::command]
pub fn book_update(state: State<'_, DbState>, id: String, input: BookWrite) -> Result<Book, LibraryError> {
  service::book_update(&state, id.as_str(), input)
}

#[tauri::command]
pub fn book_delete(state: State<'_, DbState>, id: String) -> Result<(), LibraryError> {
  service::book_delete(&state, id.as_str())
}

#[tauri::command]
pub fn quote_list(state: State<'_, DbState>, book_id: String) -> Result<Vec<QuoteItem>, LibraryError> {
  service::quote_list(&state, book_id.as_str())
}

#[tauri::command]
pub fn quote_get(state: State<'_, DbState>, id: String) -> Result<QuoteItem, LibraryError> {
  service::quote_get(&state, id.as_str())
}

#[tauri::command]
pub fn quote_create(state: State<'_, DbState>, input: QuoteWrite) -> Result<QuoteItem, LibraryError> {
  service::quote_create(&state, input)
}

#[tauri::command]
pub fn quote_update(state: State<'_, DbState>, id: String, input: QuoteWrite) -> Result<QuoteItem, LibraryError> {
  service::quote_update(&state, id.as_str(), input)
}

#[tauri::command]
pub fn quote_delete(state: State<'_, DbState>, id: String) -> Result<(), LibraryError> {
  service::quote_delete(&state, id.as_str())
}
