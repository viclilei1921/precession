use tauri::State;

use super::dto::Place;
use super::error::PlaceError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub async fn place_list(state: State<'_, DbState>) -> Result<Vec<Place>, PlaceError> {
  service::list(&state)
}

#[tauri::command]
pub async fn place_create(state: State<'_, DbState>, name: String) -> Result<Place, PlaceError> {
  service::create(&state, name.as_str())
}

#[tauri::command]
pub async fn place_update(state: State<'_, DbState>, id: String, name: String) -> Result<Place, PlaceError> {
  service::update(&state, id.as_str(), name.as_str())
}

#[tauri::command]
pub async fn place_delete(state: State<'_, DbState>, id: String) -> Result<(), PlaceError> {
  service::delete(&state, id.as_str())
}
