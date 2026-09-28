use serde::{Deserialize, Serialize};

/// 计划步骤
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
  pub id: String,
  pub title: String,
  pub done: bool,
  pub sort: i64,
}

/// 写入步骤
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStepInput {
  pub title: String,
  pub done: bool,
}

/// 一条计划
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
  pub id: String,
  pub title: String,
  pub body: String,
  pub status: String,
  pub priority: i64,
  pub scheduled_at: Option<i64>,
  pub due_at: Option<i64>,
  pub completed_at: Option<i64>,
  pub result: String,
  pub locked: bool,
  pub highlight: bool,
  pub steps: Vec<PlanStep>,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 新建或修改计划
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanInput {
  pub title: String,
  pub body: String,
  pub status: String,
  pub priority: i64,
  pub scheduled_at: Option<i64>,
  pub due_at: Option<i64>,
  pub result: String,
  pub locked: bool,
  pub highlight: bool,
  pub steps: Vec<PlanStepInput>,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
}
