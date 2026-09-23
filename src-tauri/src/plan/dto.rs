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
pub struct PlanStepWrite {
  pub title: String,
  pub done: bool,
}

/// 一条计划，含记录卡片上的字段
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanItem {
  pub id: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub locked: bool,
  pub highlight: bool,
  pub status: String,
  pub priority: i64,
  pub due_at: Option<i64>,
  pub result: String,
  pub steps: Vec<PlanStep>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 新建或修改计划
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanWrite {
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub locked: bool,
  pub highlight: bool,
  pub status: String,
  pub priority: i64,
  pub due_at: Option<i64>,
  pub result: String,
  pub steps: Vec<PlanStepWrite>,
}
