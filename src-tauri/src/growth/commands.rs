use tauri::State;

use super::dto::{GrowthEntry, GrowthEntryInput, GrowthMeasure, GrowthMeasureInput};
use super::error::GrowthError;
use super::service;
use crate::db::state::DbState;

/// 按成员、类型和时间范围列出成长记录
///
/// # 参数
/// - `state`: 数据库状态
/// - `member_id`: 成员标识
/// - `kind`: 记录类型
/// - `from`: 起始发生时间
/// - `to`: 结束发生时间
///
/// # 返回值
/// 返回符合条件的成长记录
///
/// # 错误
/// 查询失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = growth_entry_list(state, member_id, kind, from, to).await?;
/// ```
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

/// 按标识读取成长记录
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 成长记录标识
///
/// # 返回值
/// 返回对应的成长记录
///
/// # 错误
/// 读取失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = growth_entry_get(state, id).await?;
/// ```
#[tauri::command]
pub async fn growth_entry_get(state: State<'_, DbState>, id: String) -> Result<GrowthEntry, GrowthError> {
  service::get(&state, id.as_str())
}

/// 创建成长记录
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 成长记录输入
///
/// # 返回值
/// 返回新建的成长记录
///
/// # 错误
/// 创建失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = growth_entry_create(state, input).await?;
/// ```
#[tauri::command]
pub async fn growth_entry_create(
  state: State<'_, DbState>,
  input: GrowthEntryInput,
) -> Result<GrowthEntry, GrowthError> {
  service::create(&state, input)
}

/// 更新成长记录
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 成长记录标识
/// - `input`: 成长记录输入
///
/// # 返回值
/// 返回更新后的成长记录
///
/// # 错误
/// 更新失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = growth_entry_update(state, id, input).await?;
/// ```
#[tauri::command]
pub async fn growth_entry_update(
  state: State<'_, DbState>,
  id: String,
  input: GrowthEntryInput,
) -> Result<GrowthEntry, GrowthError> {
  service::update(&state, id.as_str(), input)
}

/// 删除成长记录
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 成长记录标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// growth_entry_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn growth_entry_delete(state: State<'_, DbState>, id: String) -> Result<(), GrowthError> {
  service::delete(&state, id.as_str())
}

/// 按成员列出身高体重
///
/// # 参数
/// - `state`: 数据库状态
/// - `member_id`: 成员标识
///
/// # 返回值
/// 返回该成员的测量，发生时间晚的在前
///
/// # 错误
/// 查询失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = growth_measure_list(state, member_id).await?;
/// ```
#[tauri::command]
pub async fn growth_measure_list(
  state: State<'_, DbState>,
  member_id: String,
) -> Result<Vec<GrowthMeasure>, GrowthError> {
  service::list_measures(&state, member_id.as_str())
}

/// 创建身高体重
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 测量输入
///
/// # 返回值
/// 返回新建的测量
///
/// # 错误
/// 创建失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = growth_measure_create(state, input).await?;
/// ```
#[tauri::command]
pub async fn growth_measure_create(
  state: State<'_, DbState>,
  input: GrowthMeasureInput,
) -> Result<GrowthMeasure, GrowthError> {
  service::create_measure(&state, input)
}

/// 更新身高体重
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 测量标识
/// - `input`: 测量输入
///
/// # 返回值
/// 返回更新后的测量
///
/// # 错误
/// 更新失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = growth_measure_update(state, id, input).await?;
/// ```
#[tauri::command]
pub async fn growth_measure_update(
  state: State<'_, DbState>,
  id: String,
  input: GrowthMeasureInput,
) -> Result<GrowthMeasure, GrowthError> {
  service::update_measure(&state, id.as_str(), input)
}

/// 删除身高体重
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 测量标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// growth_measure_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn growth_measure_delete(state: State<'_, DbState>, id: String) -> Result<(), GrowthError> {
  service::delete_measure(&state, id.as_str())
}
