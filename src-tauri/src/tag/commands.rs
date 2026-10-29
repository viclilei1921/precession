use tauri::State;

use super::dto::Tag;
use super::error::TagError;
use super::service;
use crate::db::state::DbState;

/// 列出标签
///
/// # 参数
/// - `state`: 数据库状态
///
/// # 返回值
/// 返回标签列表
///
/// # 错误
/// 查询失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = tag_list(state).await?;
/// ```
#[tauri::command]
pub async fn tag_list(state: State<'_, DbState>) -> Result<Vec<Tag>, TagError> {
  service::list(&state)
}

/// 创建标签
///
/// # 参数
/// - `state`: 数据库状态
/// - `name`: 标签名称
///
/// # 返回值
/// 返回新建的标签
///
/// # 错误
/// 创建失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = tag_create(state, name).await?;
/// ```
#[tauri::command]
pub async fn tag_create(state: State<'_, DbState>, name: String) -> Result<Tag, TagError> {
  service::create(&state, name.as_str())
}

/// 更新标签
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 标签标识
/// - `name`: 标签名称
///
/// # 返回值
/// 返回更新后的标签
///
/// # 错误
/// 更新失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = tag_update(state, id, name).await?;
/// ```
#[tauri::command]
pub async fn tag_update(state: State<'_, DbState>, id: String, name: String) -> Result<Tag, TagError> {
  service::update(&state, id.as_str(), name.as_str())
}

/// 删除标签
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 标签标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回标签错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// tag_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn tag_delete(state: State<'_, DbState>, id: String) -> Result<(), TagError> {
  service::delete(&state, id.as_str())
}
