use serde::Serialize;

/// 今天和回顾看到的一条记录
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
  pub id: String,
  pub kind: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub locked: bool,
  pub highlight: bool,
}
