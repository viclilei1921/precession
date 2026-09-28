//! 只读聚合。各业务模块仍只写自己的表。

use super::dto::TimelineItem;
use super::error::TimelineError;
use crate::db::state::DbState;

const KINDS: &[&str] = &["plan", "diary", "spark", "writing", "milestone", "moment", "excerpt", "note"];

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
