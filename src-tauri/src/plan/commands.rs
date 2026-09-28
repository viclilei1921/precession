use tauri::State;

use super::dto::{Plan, PlanInput};
use super::error::PlanError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub async fn plan_list(state: State<'_, DbState>) -> Result<Vec<Plan>, PlanError> {
  service::list(&state)
}

#[tauri::command]
pub async fn plan_get(state: State<'_, DbState>, id: String) -> Result<Plan, PlanError> {
  service::get(&state, id.as_str())
}

#[tauri::command]
pub async fn plan_create(state: State<'_, DbState>, input: PlanInput) -> Result<Plan, PlanError> {
  service::create(&state, input)
}

#[tauri::command]
pub async fn plan_update(state: State<'_, DbState>, id: String, input: PlanInput) -> Result<Plan, PlanError> {
  service::update(&state, id.as_str(), input)
}

#[tauri::command]
pub async fn plan_complete(state: State<'_, DbState>, id: String, result: String) -> Result<Plan, PlanError> {
  service::complete(&state, id.as_str(), result.as_str())
}

#[tauri::command]
pub async fn plan_delete(state: State<'_, DbState>, id: String) -> Result<(), PlanError> {
  service::delete(&state, id.as_str())
}
