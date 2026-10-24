use tauri::State;

use super::dto::{Upload, UploadImage, UploadInput};
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

/// 读出属于这条记录的未加密图片
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 文件标识
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// MIME 和 base64 字节
///
/// # 错误
/// 记录不存在、所属不匹配、已加密或类型不是图片时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let image = upload_image(state, id, owner, owner_id).await?;
/// ```
#[tauri::command]
pub async fn upload_image(
  state: State<'_, DbState>,
  id: String,
  owner: String,
  owner_id: String,
) -> Result<UploadImage, UploadError> {
  service::read_image(&state, id.as_str(), owner.as_str(), owner_id.as_str())
}

/// 把粘贴的图片字节暂存到导入临时目录
///
/// # 参数
/// - `state`: 数据库状态
/// - `ext`: 不带点的扩展名
/// - `data`: 标准 base64
///
/// # 返回值
/// 临时文件绝对路径
///
/// # 错误
/// 扩展名或字节不合法时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let path = upload_stage(state, ext, data).await?;
/// ```
#[tauri::command]
pub async fn upload_stage(state: State<'_, DbState>, ext: String, data: String) -> Result<String, UploadError> {
  service::stage(&state, ext.as_str(), data.as_str())
}

/// 分配一个转码输出用的临时路径
///
/// # 参数
/// - `state`: 数据库状态
/// - `ext`: 不带点的扩展名
///
/// # 返回值
/// 临时文件绝对路径，文件本身还不存在
///
/// # 错误
/// 扩展名不支持时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let path = upload_tmp(state, ext).await?;
/// ```
#[tauri::command]
pub async fn upload_tmp(state: State<'_, DbState>, ext: String) -> Result<String, UploadError> {
  service::tmp(&state, ext.as_str())
}

/// 删掉导入临时目录里的文件
///
/// # 参数
/// - `state`: 数据库状态
/// - `path`: 临时文件绝对路径
///
/// # 返回值
/// 删除完成时返回空成功
///
/// # 错误
/// 路径不在临时目录里时返回不存在
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// upload_discard(state, path).await?;
/// ```
#[tauri::command]
pub async fn upload_discard(state: State<'_, DbState>, path: String) -> Result<(), UploadError> {
  service::discard(&state, path.as_str())
}
