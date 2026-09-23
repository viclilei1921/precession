use tauri::State;

use super::dto::{MediaItem, RecordCard, RecordDetail, RecordLink, RecordWrite};
use super::error::RecordError;
use super::service;
use crate::db::state::DbState;

/// 按类型和时间列出卡片。`from` 含，`to` 不含。
#[tauri::command]
pub fn record_list(
  state: State<'_, DbState>,
  record_type: Option<String>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<RecordCard>, RecordError> {
  service::list(&state, record_type.as_deref(), from, to)
}

#[tauri::command]
pub fn record_get(state: State<'_, DbState>, id: String) -> Result<RecordDetail, RecordError> {
  service::get(&state, id.as_str())
}

#[tauri::command]
pub fn record_create(state: State<'_, DbState>, input: RecordWrite) -> Result<RecordDetail, RecordError> {
  service::create(&state, input)
}

#[tauri::command]
pub fn record_update(state: State<'_, DbState>, id: String, input: RecordWrite) -> Result<RecordDetail, RecordError> {
  service::update(&state, id.as_str(), input)
}

#[tauri::command]
pub fn record_delete(state: State<'_, DbState>, id: String) -> Result<(), RecordError> {
  service::delete(&state, id.as_str())
}

#[tauri::command]
pub fn media_list(state: State<'_, DbState>, record_id: String) -> Result<Vec<MediaItem>, RecordError> {
  service::media_list(&state, record_id.as_str())
}

#[tauri::command]
pub fn media_create(
  state: State<'_, DbState>,
  record_id: String,
  media_kind: String,
  rel_path: String,
  mime: String,
  sort: i64,
  locked: bool,
) -> Result<MediaItem, RecordError> {
  service::media_create(
    &state,
    record_id.as_str(),
    media_kind.as_str(),
    rel_path.as_str(),
    mime.as_str(),
    sort,
    locked,
  )
}

#[tauri::command]
pub fn media_delete(state: State<'_, DbState>, id: String) -> Result<(), RecordError> {
  service::media_delete(&state, id.as_str())
}

#[tauri::command]
pub fn record_link_list(state: State<'_, DbState>, record_id: String) -> Result<Vec<RecordLink>, RecordError> {
  service::link_list(&state, record_id.as_str())
}

#[tauri::command]
pub fn record_link_create(
  state: State<'_, DbState>,
  from_id: String,
  to_id: String,
  kind: String,
) -> Result<RecordLink, RecordError> {
  service::link_create(&state, from_id.as_str(), to_id.as_str(), kind.as_str())
}

#[tauri::command]
pub fn record_link_delete(state: State<'_, DbState>, id: String) -> Result<(), RecordError> {
  service::link_delete(&state, id.as_str())
}
