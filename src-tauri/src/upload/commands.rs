use tauri::State;

use super::dto::{Upload, UploadInput};
use super::error::UploadError;
use super::service;
use crate::db::state::DbState;

/// 列出所属记录上的文件
///
/// # 参数
/// - `state`: 数据库状态
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 返回文件记录列表
///
/// # 错误
/// 查询失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = upload_list(state, owner, owner_id).await?;
/// ```
#[tauri::command]
pub async fn upload_list(
  state: State<'_, DbState>,
  owner: String,
  owner_id: String,
) -> Result<Vec<Upload>, UploadError> {
  service::list(&state, owner.as_str(), owner_id.as_str())
}

/// 创建文件记录
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 文件输入
///
/// # 返回值
/// 返回新建的文件记录
///
/// # 错误
/// 创建失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = upload_create(state, input).await?;
/// ```
#[tauri::command]
pub async fn upload_create(state: State<'_, DbState>, input: UploadInput) -> Result<Upload, UploadError> {
  service::create(&state, input)
}

/// 删除文件记录和上传目录里的文件
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 文件标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// upload_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn upload_delete(state: State<'_, DbState>, id: String) -> Result<(), UploadError> {
  service::delete(&state, id.as_str())
}
