use tauri::State;

use super::dto::Place;
use super::error::PlaceError;
use super::service;
use crate::db::state::DbState;

/// 列出地点
///
/// # 参数
/// - `state`: 数据库状态
///
/// # 返回值
/// 返回地点列表
///
/// # 错误
/// 查询失败时返回地点错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = place_list(state).await?;
/// ```
#[tauri::command]
pub async fn place_list(state: State<'_, DbState>) -> Result<Vec<Place>, PlaceError> {
  service::list(&state)
}

/// 创建地点
///
/// # 参数
/// - `state`: 数据库状态
/// - `name`: 地点名称
///
/// # 返回值
/// 返回新建的地点
///
/// # 错误
/// 创建失败时返回地点错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = place_create(state, name).await?;
/// ```
#[tauri::command]
pub async fn place_create(state: State<'_, DbState>, name: String) -> Result<Place, PlaceError> {
  service::create(&state, name.as_str())
}

/// 更新地点
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 地点标识
/// - `name`: 地点名称
///
/// # 返回值
/// 返回更新后的地点
///
/// # 错误
/// 更新失败时返回地点错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = place_update(state, id, name).await?;
/// ```
#[tauri::command]
pub async fn place_update(state: State<'_, DbState>, id: String, name: String) -> Result<Place, PlaceError> {
  service::update(&state, id.as_str(), name.as_str())
}

/// 删除地点
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 地点标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回地点错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// place_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn place_delete(state: State<'_, DbState>, id: String) -> Result<(), PlaceError> {
  service::delete(&state, id.as_str())
}
