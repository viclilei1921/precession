use tauri::State;

use super::dto::{Almanac, CalendarDayInput, CalendarOverride, CalendarRange, OfficialImport, OfficialImportInput};
use super::error::CalendarError;
use super::service;
use crate::db::state::DbState;

#[tauri::command]
pub async fn calendar_list(
  state: State<'_, DbState>,
  from: String,
  to: String,
) -> Result<CalendarRange, CalendarError> {
  service::list(&state, from.as_str(), to.as_str())
}

#[tauri::command]
pub async fn calendar_day_upsert(
  state: State<'_, DbState>,
  date: String,
  input: CalendarDayInput,
) -> Result<CalendarOverride, CalendarError> {
  service::upsert_day(&state, date.as_str(), input)
}

#[tauri::command]
pub async fn calendar_day_delete(state: State<'_, DbState>, date: String) -> Result<(), CalendarError> {
  service::delete_day(&state, date.as_str())
}

#[tauri::command]
pub async fn calendar_official_import(
  state: State<'_, DbState>,
  input: OfficialImportInput,
) -> Result<OfficialImport, CalendarError> {
  service::import_official(&state, input)
}

#[tauri::command]
pub async fn calendar_almanac(date: String) -> Result<Almanac, CalendarError> {
  service::almanac(date.as_str())
}
