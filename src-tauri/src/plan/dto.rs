use serde::{Deserialize, Serialize};

use crate::media::dto::Media;
use crate::plan::constants::{REPEAT_KINDS, STATUS_DRAFT};

fn default_status() -> String {
  STATUS_DRAFT.to_string()
}

fn default_interval() -> i64 {
  1
}

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

/// 重复规则
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
#[derive(Clone, Debug, Deserialize)]
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

/// 提醒
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
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanReminderInput {
  pub remind_at: i64,
  #[serde(default)]
  pub note: String,
}

/// 评论
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanComment {
  pub id: String,
  pub body: String,
  pub created_at: i64,
  pub updated_at: i64,
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

/// 新建计划。标题必填，其余省略时用默认值。
#[derive(Debug, Deserialize)]
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

impl Default for PlanInput {
  fn default() -> Self {
    Self {
      title: String::new(),
      body: String::new(),
      status: default_status(),
      priority: 0,
      scheduled_at: None,
      due_at: None,
      result: String::new(),
      locked: false,
      highlight: false,
      group_id: None,
      parent_id: None,
      all_day: false,
      archived: false,
      sort: 0,
      time_zone: String::new(),
      steps: Vec::new(),
      member_ids: Vec::new(),
      tag_ids: Vec::new(),
      place_ids: Vec::new(),
      repeat: None,
      reminders: Vec::new(),
    }
  }
}

/// 更新计划。没传的字段和 JSON `null` 都保持原值。
#[derive(Debug, Default, Deserialize)]
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
    self.title.is_none()
      && self.body.is_none()
      && self.status.is_none()
      && self.priority.is_none()
      && self.scheduled_at.is_none()
      && self.due_at.is_none()
      && self.result.is_none()
      && self.locked.is_none()
      && self.highlight.is_none()
      && self.group_id.is_none()
      && self.parent_id.is_none()
      && self.all_day.is_none()
      && self.archived.is_none()
      && self.sort.is_none()
      && self.time_zone.is_none()
      && self.steps.is_none()
      && self.member_ids.is_none()
      && self.tag_ids.is_none()
      && self.place_ids.is_none()
      && self.repeat.is_none()
      && self.reminders.is_none()
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
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlanGroupPatch {
  pub name: Option<String>,
  pub color: Option<String>,
  pub sort: Option<i64>,
}

impl PlanGroupPatch {
  pub(super) fn is_empty(&self) -> bool {
    self.name.is_none() && self.color.is_none() && self.sort.is_none()
  }
}

pub(super) fn repeat_kind_ok(kind: &str) -> bool {
  REPEAT_KINDS.contains(&kind)
}
