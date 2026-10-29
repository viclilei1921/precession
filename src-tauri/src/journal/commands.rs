use tauri::State;
use zeroize::Zeroize;

use super::dto::{JournalCitation, JournalEntry, JournalEntryInput, JournalLink};
use super::error::JournalError;
use super::service;
use crate::db::state::DbState;

/// 按类型和时间列出手记
///
/// # 参数
/// - `state`: 数据库状态
/// - `kind`: 手记类型，空则不限
/// - `from`: 起始发生时间，空则不限
/// - `to`: 结束发生时间，空则不限
///
/// # 返回值
/// 返回手记列表
///
/// # 错误
/// 查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entries = journal_entry_list(state, kind, from, to).await?;
/// ```
#[tauri::command]
pub async fn journal_entry_list(
  state: State<'_, DbState>,
  kind: Option<String>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<JournalEntry>, JournalError> {
  service::list(&state, kind.as_deref(), from, to)
}

/// 按主键读取手记
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 手记主键
///
/// # 返回值
/// 返回手记
///
/// # 错误
/// 查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entry = journal_entry_get(state, id).await?;
/// ```
#[tauri::command]
pub async fn journal_entry_get(state: State<'_, DbState>, id: String) -> Result<JournalEntry, JournalError> {
  service::get(&state, id.as_str())
}

/// 创建手记
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 手记内容
///
/// # 返回值
/// 返回新建的手记
///
/// # 错误
/// 写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entry = journal_entry_create(state, input).await?;
/// ```
#[tauri::command]
pub async fn journal_entry_create(
  state: State<'_, DbState>,
  input: JournalEntryInput,
) -> Result<JournalEntry, JournalError> {
  service::create(&state, input)
}

/// 按主键更新手记
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 手记主键
/// - `input`: 手记内容
///
/// # 返回值
/// 返回更新后的手记
///
/// # 错误
/// 写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entry = journal_entry_update(state, id, input).await?;
/// ```
#[tauri::command]
pub async fn journal_entry_update(
  state: State<'_, DbState>,
  id: String,
  input: JournalEntryInput,
) -> Result<JournalEntry, JournalError> {
  service::update(&state, id.as_str(), input)
}

/// 按主键删除手记
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 手记主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// journal_entry_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn journal_entry_delete(state: State<'_, DbState>, id: String) -> Result<(), JournalError> {
  service::delete(&state, id.as_str())
}

/// 列出与手记相关的流转
///
/// # 参数
/// - `state`: 数据库状态
/// - `entry_id`: 手记主键
///
/// # 返回值
/// 返回流转列表
///
/// # 错误
/// 查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let links = journal_link_list(state, entry_id).await?;
/// ```
#[tauri::command]
pub async fn journal_link_list(state: State<'_, DbState>, entry_id: String) -> Result<Vec<JournalLink>, JournalError> {
  service::link_list(&state, entry_id.as_str())
}

/// 创建手记流转
///
/// # 参数
/// - `state`: 数据库状态
/// - `from_id`: 起点手记主键
/// - `to_id`: 终点手记主键
/// - `kind`: 流转类型
///
/// # 返回值
/// 返回新建的流转
///
/// # 错误
/// 写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let link = journal_link_create(state, from_id, to_id, kind).await?;
/// ```
#[tauri::command]
pub async fn journal_link_create(
  state: State<'_, DbState>,
  from_id: String,
  to_id: String,
  kind: String,
) -> Result<JournalLink, JournalError> {
  service::link_create(&state, from_id.as_str(), to_id.as_str(), kind.as_str())
}

/// 按主键删除流转
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 流转主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// journal_link_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn journal_link_delete(state: State<'_, DbState>, id: String) -> Result<(), JournalError> {
  service::link_delete(&state, id.as_str())
}

/// 列出手记的书摘引用
///
/// # 参数
/// - `state`: 数据库状态
/// - `entry_id`: 手记主键
///
/// # 返回值
/// 返回书摘引用列表
///
/// # 错误
/// 查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let citations = journal_citation_list(state, entry_id).await?;
/// ```
#[tauri::command]
pub async fn journal_citation_list(
  state: State<'_, DbState>,
  entry_id: String,
) -> Result<Vec<JournalCitation>, JournalError> {
  service::citation_list(&state, entry_id.as_str())
}

/// 创建书摘引用
///
/// # 参数
/// - `state`: 数据库状态
/// - `entry_id`: 手记主键
/// - `book_note_id`: 书摘主键
///
/// # 返回值
/// 返回新建的引用
///
/// # 错误
/// 写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let citation = journal_citation_create(state, entry_id, book_note_id).await?;
/// ```
#[tauri::command]
pub async fn journal_citation_create(
  state: State<'_, DbState>,
  entry_id: String,
  book_note_id: String,
) -> Result<JournalCitation, JournalError> {
  service::citation_create(&state, entry_id.as_str(), book_note_id.as_str())
}

/// 按主键删除书摘引用
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 引用主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// journal_citation_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn journal_citation_delete(state: State<'_, DbState>, id: String) -> Result<(), JournalError> {
  service::citation_delete(&state, id.as_str())
}

/// 提交加密到阻塞线程，并在任务里清掉密码
///
/// # 参数
/// - `body`: 正文
/// - `password`: 密码
///
/// # 返回值
/// 返回 Base64 密文
///
/// # 错误
/// 加密失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let sealed = journal_entry_seal(body, password).await?;
/// ```
#[tauri::command]
pub async fn journal_entry_seal(body: String, password: String) -> Result<String, JournalError> {
  spawn_cipher(move || {
    let result = service::seal_body(&body, &password);
    let mut password = password;
    password.zeroize();
    result
  })
  .await
}

/// 提交解密到阻塞线程，并在任务里清掉密码
///
/// # 参数
/// - `body`: Base64 密文
/// - `password`: 密码
///
/// # 返回值
/// 返回正文
///
/// # 错误
/// 解密失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plain = journal_entry_open(body, password).await?;
/// ```
#[tauri::command]
pub async fn journal_entry_open(body: String, password: String) -> Result<String, JournalError> {
  spawn_cipher(move || {
    let result = service::open_body(&body, &password);
    let mut password = password;
    password.zeroize();
    result
  })
  .await
}

/// 提交任务到阻塞线程并等待结果
///
/// # 参数
/// - `work`: 返回手记结果的任务
///
/// # 返回值
/// 返回任务结果
///
/// # 错误
/// 线程异常时记录日志并返回 `JournalError::Internal`，任务自身的错误原样返回
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let value = spawn_cipher(work).await?;
/// ```
async fn spawn_cipher<T>(work: impl FnOnce() -> Result<T, JournalError> + Send + 'static) -> Result<T, JournalError>
where
  T: Send + 'static,
{
  match tauri::async_runtime::spawn_blocking(work).await {
    Ok(result) => result,
    Err(err) => {
      tauri_plugin_log::log::error!("journal cipher: {err}");
      Err(JournalError::Internal)
    }
  }
}
