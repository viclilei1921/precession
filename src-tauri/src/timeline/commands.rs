use tauri::State;

use super::dto::TimelineItem;
use super::error::TimelineError;
use super::service;
use crate::db::state::DbState;

/// 按时间范围和类型列出时间线条目
///
/// # 参数
/// - `state`: 数据库状态
/// - `from`: 起始发生时间
/// - `to`: 结束发生时间
/// - `kinds`: 要包含的类型
///
/// # 返回值
/// 返回聚合后的时间线条目
///
/// # 错误
/// 查询失败时返回时间线错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = timeline_list(state, from, to, kinds).await?;
/// ```
#[tauri::command]
pub async fn timeline_list(
  state: State<'_, DbState>,
  from: Option<i64>,
  to: Option<i64>,
  kinds: Option<Vec<String>>,
) -> Result<Vec<TimelineItem>, TimelineError> {
  service::list(&state, from, to, kinds.as_deref())
}
