use serde::{Deserialize, Serialize};

/// 一条成长记录：里程碑或精彩瞬间
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthEntry {
  pub id: String,
  pub kind: String,
  pub member_id: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub locked: bool,
  pub highlight: bool,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 新建或修改成长记录
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthEntryInput {
  pub kind: String,
  pub member_id: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub locked: bool,
  pub highlight: bool,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
}
