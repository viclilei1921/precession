use tauri::State;

use super::dto::{GrowthEntry, GrowthEntryInput};
use super::error::GrowthError;
use super::service;
use crate::db::state::DbState;

/// 获取成长记录列表
/// 参数：
/// - member_id: 会员ID
/// - kind: 成长记录类型
/// - from: 开始时间
/// - to: 结束时间
/// 返回：
/// - 成长记录列表
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

/// 获取成长记录
/// 参数：
/// - id: 成长记录ID
/// 返回：
/// - 成长记录
#[tauri::command]
pub async fn growth_entry_get(state: State<'_, DbState>, id: String) -> Result<GrowthEntry, GrowthError> {
  service::get(&state, id.as_str())
}

/// 创建成长记录
/// 参数：
/// - input: 成长记录输入
/// 返回：
/// - 成长记录
#[tauri::command]
pub async fn growth_entry_create(
  state: State<'_, DbState>,
  input: GrowthEntryInput,
) -> Result<GrowthEntry, GrowthError> {
  service::create(&state, input)
}

/// 更新成长记录
/// 参数：
/// - id: 成长记录ID
/// - input: 成长记录输入
/// 返回：
/// - 成长记录
#[tauri::command]
pub async fn growth_entry_update(
  state: State<'_, DbState>,
  id: String,
  input: GrowthEntryInput,
) -> Result<GrowthEntry, GrowthError> {
  service::update(&state, id.as_str(), input)
}

/// 删除成长记录
/// 参数：
/// - id: 成长记录ID
/// 返回：
/// - 是否成功
#[tauri::command]
pub async fn growth_entry_delete(state: State<'_, DbState>, id: String) -> Result<(), GrowthError> {
  service::delete(&state, id.as_str())
}
