use tauri::State;
use zeroize::Zeroize;

use super::dto::{JournalCitation, JournalEntry, JournalEntryInput, JournalLink};
use super::error::JournalError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub async fn journal_entry_list(
  state: State<'_, DbState>,
  kind: Option<String>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<JournalEntry>, JournalError> {
  service::list(&state, kind.as_deref(), from, to)
}

#[tauri::command]
pub async fn journal_entry_get(state: State<'_, DbState>, id: String) -> Result<JournalEntry, JournalError> {
  service::get(&state, id.as_str())
}

#[tauri::command]
pub async fn journal_entry_create(
  state: State<'_, DbState>,
  input: JournalEntryInput,
) -> Result<JournalEntry, JournalError> {
  service::create(&state, input)
}

#[tauri::command]
pub async fn journal_entry_update(
  state: State<'_, DbState>,
  id: String,
  input: JournalEntryInput,
) -> Result<JournalEntry, JournalError> {
  service::update(&state, id.as_str(), input)
}

#[tauri::command]
pub async fn journal_entry_delete(state: State<'_, DbState>, id: String) -> Result<(), JournalError> {
  service::delete(&state, id.as_str())
}

#[tauri::command]
pub async fn journal_link_list(state: State<'_, DbState>, entry_id: String) -> Result<Vec<JournalLink>, JournalError> {
  service::link_list(&state, entry_id.as_str())
}

#[tauri::command]
pub async fn journal_link_create(
  state: State<'_, DbState>,
  from_id: String,
  to_id: String,
  kind: String,
) -> Result<JournalLink, JournalError> {
  service::link_create(&state, from_id.as_str(), to_id.as_str(), kind.as_str())
}

#[tauri::command]
pub async fn journal_link_delete(state: State<'_, DbState>, id: String) -> Result<(), JournalError> {
  service::link_delete(&state, id.as_str())
}

#[tauri::command]
pub async fn journal_citation_list(
  state: State<'_, DbState>,
  entry_id: String,
) -> Result<Vec<JournalCitation>, JournalError> {
  service::citation_list(&state, entry_id.as_str())
}

#[tauri::command]
pub async fn journal_citation_create(
  state: State<'_, DbState>,
  entry_id: String,
  book_note_id: String,
) -> Result<JournalCitation, JournalError> {
  service::citation_create(&state, entry_id.as_str(), book_note_id.as_str())
}

#[tauri::command]
pub async fn journal_citation_delete(state: State<'_, DbState>, id: String) -> Result<(), JournalError> {
  service::citation_delete(&state, id.as_str())
}

#[tauri::command]
pub async fn journal_entry_seal(body: String, password: String) -> Result<String, JournalError> {
  spawn_cipher(move || {
    let result = service::seal_body(&body, &password);
    let mut password = password;
    password.zeroize();
    result
  })
  .await
}

#[tauri::command]
pub async fn journal_entry_open(body: String, password: String) -> Result<String, JournalError> {
  spawn_cipher(move || {
    let result = service::open_body(&body, &password);
    let mut password = password;
    password.zeroize();
    result
  })
  .await
}

async fn spawn_cipher<T>(work: impl FnOnce() -> Result<T, JournalError> + Send + 'static) -> Result<T, JournalError>
where
  T: Send + 'static,
{
  match tauri::async_runtime::spawn_blocking(work).await {
    Ok(result) => result,
    Err(err) => {
      tauri_plugin_log::log::error!("journal cipher: {err}");
      Err(JournalError::Internal)
    }
  }
}
