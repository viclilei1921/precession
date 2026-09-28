use tauri::State;

use super::dto::{Media, MediaInput};
use super::error::MediaError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub async fn media_list(state: State<'_, DbState>, owner: String, owner_id: String) -> Result<Vec<Media>, MediaError> {
  service::list(&state, owner.as_str(), owner_id.as_str())
}

#[tauri::command]
pub async fn media_create(state: State<'_, DbState>, input: MediaInput) -> Result<Media, MediaError> {
  service::create(&state, input)
}

#[tauri::command]
pub async fn media_delete(state: State<'_, DbState>, id: String) -> Result<(), MediaError> {
  service::delete(&state, id.as_str())
}
