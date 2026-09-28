use tauri::State;

use super::dto::{Member, MemberInput};
use super::error::MemberError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub async fn member_list(state: State<'_, DbState>) -> Result<Vec<Member>, MemberError> {
  service::list(&state)
}

#[tauri::command]
pub async fn member_get(state: State<'_, DbState>, id: String) -> Result<Member, MemberError> {
  service::get(&state, id.as_str())
}

#[tauri::command]
pub async fn member_create(state: State<'_, DbState>, input: MemberInput) -> Result<Member, MemberError> {
  service::create(&state, input)
}

#[tauri::command]
pub async fn member_update(state: State<'_, DbState>, id: String, input: MemberInput) -> Result<Member, MemberError> {
  service::update(&state, id.as_str(), input)
}

#[tauri::command]
pub async fn member_delete(state: State<'_, DbState>, id: String) -> Result<(), MemberError> {
  service::delete(&state, id.as_str())
}
