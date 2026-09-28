use tauri::State;

use super::dto::TimelineItem;
use super::error::TimelineError;
use super::service;
use crate::db::state::DbState;

/// 按发生时间聚合计划、手记、成长和书摘。`from` 含，`to` 不含。
#[tauri::command]
pub async fn timeline_list(
  state: State<'_, DbState>,
  from: Option<i64>,
  to: Option<i64>,
  kinds: Option<Vec<String>>,
) -> Result<Vec<TimelineItem>, TimelineError> {
  service::list(&state, from, to, kinds.as_deref())
}
