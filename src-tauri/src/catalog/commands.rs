use tauri::State;

use super::dto::{Member, MemberWrite, Place, Tag};
use super::error::CatalogError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub fn member_list(state: State<'_, DbState>) -> Result<Vec<Member>, CatalogError> {
  service::member_list(&state)
}

#[tauri::command]
pub fn member_get(state: State<'_, DbState>, id: String) -> Result<Member, CatalogError> {
  service::member_get(&state, id.as_str())
}

#[tauri::command]
pub fn member_create(state: State<'_, DbState>, input: MemberWrite) -> Result<Member, CatalogError> {
  service::member_create(&state, input)
}

#[tauri::command]
pub fn member_update(state: State<'_, DbState>, id: String, input: MemberWrite) -> Result<Member, CatalogError> {
  service::member_update(&state, id.as_str(), input)
}

#[tauri::command]
pub fn member_delete(state: State<'_, DbState>, id: String) -> Result<(), CatalogError> {
  service::member_delete(&state, id.as_str())
}

#[tauri::command]
pub fn tag_list(state: State<'_, DbState>) -> Result<Vec<Tag>, CatalogError> {
  service::tag_list(&state)
}

#[tauri::command]
pub fn tag_create(state: State<'_, DbState>, name: String) -> Result<Tag, CatalogError> {
  service::tag_create(&state, name.as_str())
}

#[tauri::command]
pub fn tag_update(state: State<'_, DbState>, id: String, name: String) -> Result<Tag, CatalogError> {
  service::tag_update(&state, id.as_str(), name.as_str())
}

#[tauri::command]
pub fn tag_delete(state: State<'_, DbState>, id: String) -> Result<(), CatalogError> {
  service::tag_delete(&state, id.as_str())
}

#[tauri::command]
pub fn place_list(state: State<'_, DbState>) -> Result<Vec<Place>, CatalogError> {
  service::place_list(&state)
}

#[tauri::command]
pub fn place_create(state: State<'_, DbState>, name: String) -> Result<Place, CatalogError> {
  service::place_create(&state, name.as_str())
}

#[tauri::command]
pub fn place_update(state: State<'_, DbState>, id: String, name: String) -> Result<Place, CatalogError> {
  service::place_update(&state, id.as_str(), name.as_str())
}

#[tauri::command]
pub fn place_delete(state: State<'_, DbState>, id: String) -> Result<(), CatalogError> {
  service::place_delete(&state, id.as_str())
}
