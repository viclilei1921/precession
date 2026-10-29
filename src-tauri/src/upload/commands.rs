use tauri::State;

use super::dto::{Upload, UploadCatalog, UploadImage, UploadInput};
use super::error::UploadError;
use super::media::{self, MediaEndpoint, MediaState};
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

/// 列出全部文件及其所属记录
///
/// # 参数
/// - `state`: 数据库状态
///
/// # 返回值
/// 返回文件目录
///
/// # 错误
/// 查询失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = upload_catalog(state).await?;
/// ```
#[tauri::command]
pub async fn upload_catalog(state: State<'_, DbState>) -> Result<Vec<UploadCatalog>, UploadError> {
  service::catalog(&state)
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
/// - `media`: 播放地址和文件密钥
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
/// upload_delete(state, media, id).await?;
/// ```
#[tauri::command]
pub async fn upload_delete(
  state: State<'_, DbState>,
  media: State<'_, MediaState>,
  id: String,
) -> Result<(), UploadError> {
  service::delete(&state, id.as_str())?;
  media.forget(id.as_str());
  Ok(())
}

/// 把未挂载的文件改挂到一条已存在的记录上
///
/// # 参数
/// - `state`: 数据库状态
/// - `ids`: 要改挂的文件标识
/// - `owner`: 目标所属类型
/// - `owner_id`: 目标记录标识
///
/// # 返回值
/// 改挂完成时返回空成功
///
/// # 错误
/// 目标不合法、记录不存在或已封存时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// upload_attach(state, ids, owner, owner_id).await?;
/// ```
#[tauri::command]
pub async fn upload_attach(
  state: State<'_, DbState>,
  ids: Vec<String>,
  owner: String,
  owner_id: String,
) -> Result<(), UploadError> {
  service::attach(&state, &ids, owner.as_str(), owner_id.as_str())
}

/// 用文件密码准备预览
///
/// 未加密的文件直接成功。加密文件会记下密钥，后面的区间请求不再跑 Argon2id。
///
/// # 参数
/// - `state`: 数据库状态
/// - `media`: 播放地址和密钥
/// - `id`: 文件标识
/// - `password`: 文件密码。未加密或已经解开过时可以是空字符串
///
/// # 返回值
/// 可以预览时返回空成功
///
/// # 错误
/// 记录不存在、已封存、没填密码或密码不对时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// upload_open(state, media, id, password).await?;
/// ```
#[tauri::command]
pub async fn upload_open(
  state: State<'_, DbState>,
  media: State<'_, MediaState>,
  id: String,
  password: String,
) -> Result<(), UploadError> {
  let db = (*state).clone();
  let media = (*media).clone();
  tauri::async_runtime::spawn_blocking(move || media::open_media(&db, &media, id.as_str(), password))
    .await
    .map_err(|_| UploadError::Internal)?
}

/// 把文件的明文写到用户选的文件夹
///
/// # 参数
/// - `state`: 数据库状态
/// - `media`: 播放地址和文件密钥
/// - `id`: 文件标识
/// - `dir`: 用户选择的文件夹
/// - `password`: 文件密码。未加密或已经解开过时可以是空字符串
///
/// # 返回值
/// 写好的文件绝对路径
///
/// # 错误
/// 记录不存在、已封存、没填密码、密码不对或写盘失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let saved = upload_export(state, media, id, dir, password).await?;
/// ```
#[tauri::command]
pub async fn upload_export(
  state: State<'_, DbState>,
  media: State<'_, MediaState>,
  id: String,
  dir: String,
  password: String,
) -> Result<String, UploadError> {
  let db = (*state).clone();
  let media = (*media).clone();
  tauri::async_runtime::spawn_blocking(move || media::export_file(&db, &media, id.as_str(), dir.as_str(), password))
    .await
    .map_err(|_| UploadError::Internal)?
}

/// 返回本机播放地址
///
/// 图片和视频的地址是 `{origin}/m/{token}/{id}`。
///
/// # 参数
/// - `media`: 播放地址和密钥
///
/// # 返回值
/// 本机入口。绑定失败时 `origin` 为空
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let endpoint = media_info(media).await?;
/// ```
#[tauri::command]
pub async fn media_info(media: State<'_, MediaState>) -> Result<MediaEndpoint, UploadError> {
  Ok(media.endpoint())
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
