use tauri::State;

use super::dto::{Member, MemberInput};
use super::error::MemberError;
use super::service;
use crate::db::state::DbState;

/// 列出成员
///
/// # 参数
/// - `state`: 数据库状态
///
/// # 返回值
/// 返回成员列表
///
/// # 错误
/// 查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let members = member_list(state).await?;
/// ```
#[tauri::command]
pub async fn member_list(state: State<'_, DbState>) -> Result<Vec<Member>, MemberError> {
  service::list(&state)
}

/// 按主键读取成员
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 成员主键
///
/// # 返回值
/// 返回成员
///
/// # 错误
/// 查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let member = member_get(state, id).await?;
/// ```
#[tauri::command]
pub async fn member_get(state: State<'_, DbState>, id: String) -> Result<Member, MemberError> {
  service::get(&state, id.as_str())
}

/// 创建成员
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 成员内容
///
/// # 返回值
/// 返回新建的成员
///
/// # 错误
/// 写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let member = member_create(state, input).await?;
/// ```
#[tauri::command]
pub async fn member_create(state: State<'_, DbState>, input: MemberInput) -> Result<Member, MemberError> {
  service::create(&state, input)
}

/// 按主键更新成员
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 成员主键
/// - `input`: 成员内容
///
/// # 返回值
/// 返回更新后的成员
///
/// # 错误
/// 写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let member = member_update(state, id, input).await?;
/// ```
#[tauri::command]
pub async fn member_update(state: State<'_, DbState>, id: String, input: MemberInput) -> Result<Member, MemberError> {
  service::update(&state, id.as_str(), input)
}

/// 按主键删除成员
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 成员主键
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
/// member_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn member_delete(state: State<'_, DbState>, id: String) -> Result<(), MemberError> {
  service::delete(&state, id.as_str())
}
