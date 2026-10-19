use tauri::State;

use super::dto::{Almanac, CalendarDayInput, CalendarOverride, CalendarRange, OfficialImport, OfficialImportInput};
use super::error::CalendarError;
use super::service;
use crate::db::state::DbState;

/// 按日期区间列出日历
///
/// # 参数
/// - `state`: 数据库状态
/// - `from`: 开始日期
/// - `to`: 结束日期
///
/// # 返回值
/// 返回区间内每一天
///
/// # 错误
/// 日期不合法或查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let range = calendar_list(state, from, to).await?;
/// ```
#[tauri::command]
pub async fn calendar_list(
  state: State<'_, DbState>,
  from: String,
  to: String,
) -> Result<CalendarRange, CalendarError> {
  service::list(&state, from.as_str(), to.as_str())
}

/// 写入或覆盖某一天的个人安排
///
/// # 参数
/// - `state`: 数据库状态
/// - `date`: 日期
/// - `input`: 是否上班和备注
///
/// # 返回值
/// 返回保存后的个人覆盖
///
/// # 错误
/// 日期不合法或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let day = calendar_day_upsert(state, date, input).await?;
/// ```
#[tauri::command]
pub async fn calendar_day_upsert(
  state: State<'_, DbState>,
  date: String,
  input: CalendarDayInput,
) -> Result<CalendarOverride, CalendarError> {
  service::upsert_day(&state, date.as_str(), input)
}

/// 删除某一天的个人覆盖
///
/// # 参数
/// - `state`: 数据库状态
/// - `date`: 日期
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 日期不合法或删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// calendar_day_delete(state, date).await?;
/// ```
#[tauri::command]
pub async fn calendar_day_delete(state: State<'_, DbState>, date: String) -> Result<(), CalendarError> {
  service::delete_day(&state, date.as_str())
}

/// 按年导入官方节假日
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 年份和官方日期
///
/// # 返回值
/// 返回年份和写入条数
///
/// # 错误
/// 输入不合法或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let imported = calendar_official_import(state, input).await?;
/// ```
#[tauri::command]
pub async fn calendar_official_import(
  state: State<'_, DbState>,
  input: OfficialImportInput,
) -> Result<OfficialImport, CalendarError> {
  service::import_official(&state, input)
}

/// 按日期生成黄历
///
/// # 参数
/// - `date`: 日期
///
/// # 返回值
/// 返回单日黄历
///
/// # 错误
/// 日期不合法或不在支持年份时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let book = calendar_almanac(date).await?;
/// ```
#[tauri::command]
pub async fn calendar_almanac(date: String) -> Result<Almanac, CalendarError> {
  service::almanac(date.as_str())
}
