use tauri::State;

use super::dto::{GrowthEntry, GrowthEntryInput};
use super::error::GrowthError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub async fn growth_entry_list(
  state: State<'_, DbState>,
  member_id: Option<String>,
  kind: Option<String>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<GrowthEntry>, GrowthError> {
  service::list(&state, member_id.as_deref(), kind.as_deref(), from, to)
}

#[tauri::command]
pub async fn growth_entry_get(state: State<'_, DbState>, id: String) -> Result<GrowthEntry, GrowthError> {
  service::get(&state, id.as_str())
}

#[tauri::command]
pub async fn growth_entry_create(
  state: State<'_, DbState>,
  input: GrowthEntryInput,
) -> Result<GrowthEntry, GrowthError> {
  service::create(&state, input)
}

#[tauri::command]
pub async fn growth_entry_update(
  state: State<'_, DbState>,
  id: String,
  input: GrowthEntryInput,
) -> Result<GrowthEntry, GrowthError> {
  service::update(&state, id.as_str(), input)
}

#[tauri::command]
pub async fn growth_entry_delete(state: State<'_, DbState>, id: String) -> Result<(), GrowthError> {
  service::delete(&state, id.as_str())
}
