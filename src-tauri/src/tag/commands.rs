use tauri::State;

use super::dto::Tag;
use super::error::TagError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub async fn tag_list(state: State<'_, DbState>) -> Result<Vec<Tag>, TagError> {
  service::list(&state)
}

#[tauri::command]
pub async fn tag_create(state: State<'_, DbState>, name: String) -> Result<Tag, TagError> {
  service::create(&state, name.as_str())
}

#[tauri::command]
pub async fn tag_update(state: State<'_, DbState>, id: String, name: String) -> Result<Tag, TagError> {
  service::update(&state, id.as_str(), name.as_str())
}

#[tauri::command]
pub async fn tag_delete(state: State<'_, DbState>, id: String) -> Result<(), TagError> {
  service::delete(&state, id.as_str())
}
