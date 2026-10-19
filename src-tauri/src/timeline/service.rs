//! 只读聚合。各业务模块仍只写自己的表。

use super::dto::TimelineItem;
use super::error::TimelineError;
use crate::db::state::DbState;

const KINDS: &[&str] = &["plan", "diary", "spark", "writing", "milestone", "moment", "excerpt", "note"];

/// 校验类型后按时间范围汇总时间线条目
///
/// # 参数
/// - `db`: 数据库状态
/// - `from`: 起始发生时间
/// - `to`: 结束发生时间
/// - `kinds`: 要包含的类型，未指定时不校验
///
/// # 返回值
/// 返回汇总后的时间线条目
///
/// # 错误
/// 任一类型不在允许列表中时返回类型不正确；查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list(db, from, to, kinds)?;
/// ```
pub fn list(
  db: &DbState,
  from: Option<i64>,
  to: Option<i64>,
  kinds: Option<&[String]>,
) -> Result<Vec<TimelineItem>, TimelineError> {
  if let Some(kinds) = kinds {
    for kind in kinds {
      if !KINDS.contains(&kind.as_str()) {
        return Err(TimelineError::KindInvalid);
      }
    }
  }
  let selected = kinds.map(|items| items.to_vec());
  db.with_conn(move |conn| list_between(conn, from, to, selected.as_deref()))
}

/// 按时间范围和类型汇总计划、手记、成长记录和书摘
///
/// # 参数
/// - `conn`: 数据库连接
/// - `from`: 起始发生时间
/// - `to`: 结束发生时间
/// - `kinds`: 要包含的类型，未指定时全部纳入
///
/// # 返回值
/// 返回按发生时间倒序、标识倒序排列的条目；计划的发生时间取计划时间，没有计划时间时取创建时间
///
/// # 错误
/// 任一来源查询失败时返回内部错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list_between(conn, from, to, kinds)?;
/// ```
pub(super) fn list_between(
  conn: &rusqlite::Connection,
  from: Option<i64>,
  to: Option<i64>,
  kinds: Option<&[String]>,
) -> Result<Vec<TimelineItem>, TimelineError> {
  let allow = |kind: &str| kinds.is_none_or(|items| items.iter().any(|item| item == kind));
  let mut items = Vec::new();

  if allow("plan") {
    for plan in crate::plan::list_between(conn, from, to).map_err(|_| TimelineError::Internal)? {
      items.push(TimelineItem {
        id: plan.id,
        kind: "plan".into(),
        occurred_at: plan.scheduled_at.unwrap_or(plan.created_at),
        title: plan.title,
        body: plan.body,
        locked: plan.locked,
        highlight: plan.highlight,
      });
    }
  }

  for entry in crate::journal::list_between(conn, from, to).map_err(|_| TimelineError::Internal)? {
    if allow(&entry.kind) {
      items.push(TimelineItem {
        id: entry.id,
        kind: entry.kind,
        occurred_at: entry.occurred_at,
        title: entry.title,
        body: entry.body,
        locked: entry.locked,
        highlight: entry.highlight,
      });
    }
  }

  for entry in crate::growth::list_between(conn, from, to).map_err(|_| TimelineError::Internal)? {
    if allow(&entry.kind) {
      items.push(TimelineItem {
        id: entry.id,
        kind: entry.kind,
        occurred_at: entry.occurred_at,
        title: entry.title,
        body: entry.body,
        locked: entry.locked,
        highlight: entry.highlight,
      });
    }
  }

  for note in crate::library::list_between(conn, from, to).map_err(|_| TimelineError::Internal)? {
    if allow(&note.kind) {
      items.push(TimelineItem {
        id: note.id,
        kind: note.kind,
        occurred_at: note.occurred_at,
        title: note.title,
        body: note.body,
        locked: note.locked,
        highlight: note.highlight,
      });
    }
  }

  items.sort_by(|left, right| right.occurred_at.cmp(&left.occurred_at).then(right.id.cmp(&left.id)));
  Ok(items)
}
