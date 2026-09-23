use tauri::State;

use super::dto::{PlanItem, PlanWrite};
use super::error::PlanError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub fn plan_list(state: State<'_, DbState>) -> Result<Vec<PlanItem>, PlanError> {
  service::list(&state)
}

#[tauri::command]
pub fn plan_get(state: State<'_, DbState>, id: String) -> Result<PlanItem, PlanError> {
  service::get(&state, id.as_str())
}

#[tauri::command]
pub fn plan_create(state: State<'_, DbState>, input: PlanWrite) -> Result<PlanItem, PlanError> {
  service::create(&state, input)
}

#[tauri::command]
pub fn plan_update(state: State<'_, DbState>, id: String, input: PlanWrite) -> Result<PlanItem, PlanError> {
  service::update(&state, id.as_str(), input)
}

#[tauri::command]
pub fn plan_complete(state: State<'_, DbState>, id: String, result: String) -> Result<PlanItem, PlanError> {
  service::complete(&state, id.as_str(), result.as_str())
}

#[tauri::command]
pub fn plan_delete(state: State<'_, DbState>, id: String) -> Result<(), PlanError> {
  service::delete(&state, id.as_str())
}
