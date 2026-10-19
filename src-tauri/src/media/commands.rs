use tauri::State;

use super::dto::{Media, MediaInput};
use super::error::MediaError;
use super::service;
use crate::db::state::DbState;

/// 列出所属记录上的媒体
///
/// # 参数
/// - `state`: 数据库状态
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 返回媒体列表
///
/// # 错误
/// 查询失败时返回媒体错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = media_list(state, owner, owner_id).await?;
/// ```
#[tauri::command]
pub async fn media_list(state: State<'_, DbState>, owner: String, owner_id: String) -> Result<Vec<Media>, MediaError> {
  service::list(&state, owner.as_str(), owner_id.as_str())
}

/// 创建媒体
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 媒体输入
///
/// # 返回值
/// 返回新建的媒体
///
/// # 错误
/// 创建失败时返回媒体错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = media_create(state, input).await?;
/// ```
#[tauri::command]
pub async fn media_create(state: State<'_, DbState>, input: MediaInput) -> Result<Media, MediaError> {
  service::create(&state, input)
}

/// 删除媒体
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 媒体标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回媒体错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// media_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn media_delete(state: State<'_, DbState>, id: String) -> Result<(), MediaError> {
  service::delete(&state, id.as_str())
}
