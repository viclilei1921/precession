use serde::{Deserialize, Serialize};

use crate::media::dto::Media;
use crate::plan::constants::{REPEAT_KINDS, STATUS_DRAFT};

/// 默认计划状态 草稿
fn default_status() -> String {
  STATUS_DRAFT.to_string()
}

/// 默认重复间隔 1
fn default_interval() -> i64 {
  1
}

/// 计划步骤(返回时包含 ID)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
  pub id: String,
  pub title: String,
  pub done: bool,
  pub sort: i64,
}

/// 写入步骤
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStepInput {
  pub title: String,
  pub done: bool,
}

/// 重复规则(返回时包含 ID)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRepeat {
  pub id: String,
  pub kind: String,
  pub interval: i64,
  pub weekdays: String,
  pub until_at: Option<i64>,
}

/// 写入重复规则
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRepeatInput {
  pub kind: String,
  #[serde(default = "default_interval")]
  pub interval: i64,
  #[serde(default)]
  pub weekdays: String,
  #[serde(default)]
  pub until_at: Option<i64>,
}

/// 提醒(返回时包含 ID)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanReminder {
  pub id: String,
  pub remind_at: i64,
  pub note: String,
  pub triggered: bool,
  pub triggered_at: Option<i64>,
  pub created_at: i64,
}

/// 写入提醒
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanReminderInput {
  pub remind_at: i64,
  #[serde(default)]
  pub note: String,
}

/// 评论(返回时包含 ID)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanComment {
  pub id: String,
  pub body: String,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 一条计划(返回时包含 ID)
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
  pub group_id: Option<String>,
  pub parent_id: Option<String>,
  pub all_day: bool,
  pub archived: bool,
  pub sort: i64,
  pub time_zone: String,
  pub steps: Vec<PlanStep>,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
  pub repeat: Option<PlanRepeat>,
  pub reminders: Vec<PlanReminder>,
  pub media: Vec<Media>,
  pub comments: Vec<PlanComment>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 列出计划的条件。都可省略，省略即不限制。
/// `from` 含、`to` 不含，比较安排时间；没有安排时间则用创建时间。
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlanQuery {
  pub from: Option<i64>,
  pub to: Option<i64>,
  pub group_id: Option<String>,
}

/// 新建计划。标题必填，其余都可省略，省略时用默认值。
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanInput {
  pub title: String,
  #[serde(default)]
  pub body: String,
  #[serde(default = "default_status")]
  pub status: String,
  #[serde(default)]
  pub priority: i64,
  #[serde(default)]
  pub scheduled_at: Option<i64>,
  #[serde(default)]
  pub due_at: Option<i64>,
  #[serde(default)]
  pub result: String,
  #[serde(default)]
  pub locked: bool,
  #[serde(default)]
  pub highlight: bool,
  #[serde(default)]
  pub group_id: Option<String>,
  #[serde(default)]
  pub parent_id: Option<String>,
  #[serde(default)]
  pub all_day: bool,
  #[serde(default)]
  pub archived: bool,
  #[serde(default)]
  pub sort: i64,
  #[serde(default)]
  pub time_zone: String,
  #[serde(default)]
  pub steps: Vec<PlanStepInput>,
  #[serde(default)]
  pub member_ids: Vec<String>,
  #[serde(default)]
  pub tag_ids: Vec<String>,
  #[serde(default)]
  pub place_ids: Vec<String>,
  #[serde(default)]
  pub repeat: Option<PlanRepeatInput>,
  #[serde(default)]
  pub reminders: Vec<PlanReminderInput>,
}

/// 更新计划。没传的字段和 JSON `null` 都保持原值。
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlanPatch {
  pub title: Option<String>,
  pub body: Option<String>,
  pub status: Option<String>,
  pub priority: Option<i64>,
  pub scheduled_at: Option<i64>,
  pub due_at: Option<i64>,
  pub result: Option<String>,
  pub locked: Option<bool>,
  pub highlight: Option<bool>,
  pub group_id: Option<String>,
  pub parent_id: Option<String>,
  pub all_day: Option<bool>,
  pub archived: Option<bool>,
  pub sort: Option<i64>,
  pub time_zone: Option<String>,
  pub steps: Option<Vec<PlanStepInput>>,
  pub member_ids: Option<Vec<String>>,
  pub tag_ids: Option<Vec<String>>,
  pub place_ids: Option<Vec<String>>,
  pub repeat: Option<PlanRepeatInput>,
  pub reminders: Option<Vec<PlanReminderInput>>,
}

impl PlanPatch {
  pub(super) fn is_empty(&self) -> bool {
    self == &Self::default()
  }
}

/// 清单
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanGroup {
  pub id: String,
  pub name: String,
  pub color: String,
  pub system: bool,
  pub sort: i64,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 新建清单
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanGroupInput {
  pub name: String,
  #[serde(default)]
  pub color: String,
  #[serde(default)]
  pub system: bool,
  #[serde(default)]
  pub sort: i64,
}

/// 更新清单。没传的字段和 JSON `null` 都保持原值。
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlanGroupPatch {
  pub name: Option<String>,
  pub color: Option<String>,
  pub sort: Option<i64>,
}

impl PlanGroupPatch {
  pub(super) fn is_empty(&self) -> bool {
    self == &Self::default()
  }
}

/// 重复规则是否有效
pub(super) fn repeat_kind_ok(kind: &str) -> bool {
  REPEAT_KINDS.contains(&kind)
}
