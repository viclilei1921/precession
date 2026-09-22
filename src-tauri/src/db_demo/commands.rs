use tauri::State;

use super::dto::DemoItem;
use super::error::DemoError;
use super::service;
use crate::db::state::DbState;

/// 列出示例表全部行
#[tauri::command]
pub fn demo_list(state: State<'_, DbState>) -> Result<Vec<DemoItem>, DemoError> {
  service::list(&state)
}

/// 按 id 读取一行
#[tauri::command]
pub fn demo_get(state: State<'_, DbState>, id: String) -> Result<DemoItem, DemoError> {
  service::get(&state, id.as_str())
}

/// 新建一行
#[tauri::command]
pub fn demo_create(state: State<'_, DbState>, title: String, body: String) -> Result<DemoItem, DemoError> {
  service::create(&state, title.as_str(), body.as_str())
}

/// 更新一行
#[tauri::command]
pub fn demo_update(state: State<'_, DbState>, id: String, title: String, body: String) -> Result<DemoItem, DemoError> {
  service::update(&state, id.as_str(), title.as_str(), body.as_str())
}

/// 删除一行
#[tauri::command]
pub fn demo_delete(state: State<'_, DbState>, id: String) -> Result<(), DemoError> {
  service::delete(&state, id.as_str())
}
