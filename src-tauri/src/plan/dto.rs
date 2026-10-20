use serde::{Deserialize, Serialize};

use crate::media::dto::Media;
use crate::plan::constants::{REPEAT_KINDS, STATUS_DRAFT};

/// 返回默认计划状态
///
/// # 返回值
/// 返回草稿状态文本
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let status = default_status();
/// ```
fn default_status() -> String {
  STATUS_DRAFT.to_string()
}

/// 返回默认重复间隔
///
/// # 返回值
/// 返回 1
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let interval = default_interval();
/// ```
fn default_interval() -> i64 {
  1
}

/// 计划步骤(返回时包含 ID)
///
/// 线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 步骤 ID
/// - `title`: 标题
/// - `done`: 是否完成
/// - `sort`: 排序
///
/// # 示例
/// 见 [`plan_get`](super::commands::plan_get)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
  /// 步骤 ID
  pub id: String,
  /// 标题
  pub title: String,
  /// 是否完成
  pub done: bool,
  /// 排序
  pub sort: i64,
}

/// 写入步骤
///
/// 标题不能为空。
///
/// # 字段
/// - `title`: 标题，不能为空
/// - `done`: 是否完成
///
/// # 示例
/// 见 [`plan_create`](super::commands::plan_create)
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStepInput {
  /// 标题，不能为空
  pub title: String,
  /// 是否完成
  pub done: bool,
}

/// 重复规则(返回时包含 ID)
///
/// `kind` 为 `daily`、`weekly`、`monthly` 或 `yearly`。`interval` 至少为 1。
///
/// # 字段
/// - `id`: 规则 ID
/// - `kind`: 重复种类，`daily`、`weekly`、`monthly` 或 `yearly`
/// - `interval`: 间隔，至少为 1
/// - `weekdays`: 星期，写入前去掉首尾空白
/// - `until_at`: 重复截止时间，可以没有
///
/// # 示例
/// 见 [`plan_get`](super::commands::plan_get)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRepeat {
  /// 规则 ID
  pub id: String,
  /// 重复种类，`daily`、`weekly`、`monthly` 或 `yearly`
  pub kind: String,
  /// 间隔，至少为 1
  pub interval: i64,
  /// 星期，写入前去掉首尾空白
  pub weekdays: String,
  /// 重复截止时间，可以没有
  pub until_at: Option<i64>,
}

/// 写入重复规则
///
/// 种类必须是 `daily`、`weekly`、`monthly` 或 `yearly`，间隔至少为 1。省略的间隔默认为 1。
///
/// # 字段
/// - `kind`: 重复种类，`daily`、`weekly`、`monthly` 或 `yearly`
/// - `interval`: 间隔，至少为 1，省略时为 1
/// - `weekdays`: 星期，写入前去掉首尾空白
/// - `until_at`: 重复截止时间，可以没有
///
/// # 示例
/// 见 [`plan_create`](super::commands::plan_create)
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRepeatInput {
  /// 重复种类，`daily`、`weekly`、`monthly` 或 `yearly`
  pub kind: String,
  /// 间隔，至少为 1，省略时为 1
  #[serde(default = "default_interval")]
  pub interval: i64,
  /// 星期，写入前去掉首尾空白
  #[serde(default)]
  pub weekdays: String,
  /// 重复截止时间，可以没有
  #[serde(default)]
  pub until_at: Option<i64>,
}

/// 提醒(返回时包含 ID)
///
/// # 字段
/// - `id`: 提醒 ID
/// - `remind_at`: 提醒时间
/// - `note`: 备注
/// - `triggered`: 是否已触发
/// - `triggered_at`: 触发时间，可以没有
/// - `created_at`: 创建时间
///
/// # 示例
/// 见 [`plan_get`](super::commands::plan_get)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanReminder {
  /// 提醒 ID
  pub id: String,
  /// 提醒时间
  pub remind_at: i64,
  /// 备注
  pub note: String,
  /// 是否已触发
  pub triggered: bool,
  /// 触发时间，可以没有
  pub triggered_at: Option<i64>,
  /// 创建时间
  pub created_at: i64,
}

/// 写入提醒
///
/// # 字段
/// - `remind_at`: 提醒时间
/// - `note`: 备注，省略时为空
///
/// # 示例
/// 见 [`plan_create`](super::commands::plan_create)
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanReminderInput {
  /// 提醒时间
  pub remind_at: i64,
  /// 备注，省略时为空
  #[serde(default)]
  pub note: String,
}

/// 评论(返回时包含 ID)
///
/// 正文不能为空。
///
/// # 字段
/// - `id`: 评论 ID
/// - `body`: 正文，不能为空
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
///
/// # 示例
/// 见 [`plan_comment_create`](super::commands::plan_comment_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanComment {
  /// 评论 ID
  pub id: String,
  /// 正文，不能为空
  pub body: String,
  /// 创建时间
  pub created_at: i64,
  /// 更新时间
  pub updated_at: i64,
}

/// 一条计划(返回时包含 ID)
///
/// `status` 为 `draft`、`scheduled` 或 `done`。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 计划 ID
/// - `title`: 标题
/// - `body`: 正文
/// - `status`: 状态，`draft`、`scheduled` 或 `done`
/// - `priority`: 优先级
/// - `scheduled_at`: 安排时间，可以没有
/// - `due_at`: 截止时间，可以没有
/// - `completed_at`: 完成时间，可以没有
/// - `result`: 完成结果
/// - `locked`: 是否锁定
/// - `highlight`: 是否高亮
/// - `group_id`: 清单 ID，可以没有
/// - `parent_id`: 父计划 ID，可以没有
/// - `all_day`: 是否全天
/// - `archived`: 是否归档
/// - `sort`: 排序
/// - `time_zone`: 时区
/// - `steps`: 步骤
/// - `member_ids`: 成员 ID 列表
/// - `tag_ids`: 标签 ID 列表
/// - `place_ids`: 地点 ID 列表
/// - `repeat`: 重复规则，可以没有
/// - `reminders`: 提醒
/// - `media`: 挂上的媒体
/// - `comments`: 评论
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
///
/// # 示例
/// 见 [`plan_get`](super::commands::plan_get)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
  /// 计划 ID
  pub id: String,
  /// 标题
  pub title: String,
  /// 正文
  pub body: String,
  /// 状态，`draft`、`scheduled` 或 `done`
  pub status: String,
  /// 优先级
  pub priority: i64,
  /// 安排时间，可以没有
  pub scheduled_at: Option<i64>,
  /// 截止时间，可以没有
  pub due_at: Option<i64>,
  /// 完成时间，可以没有
  pub completed_at: Option<i64>,
  /// 完成结果
  pub result: String,
  /// 是否锁定
  pub locked: bool,
  /// 是否高亮
  pub highlight: bool,
  /// 清单 ID，可以没有
  pub group_id: Option<String>,
  /// 父计划 ID，可以没有
  pub parent_id: Option<String>,
  /// 是否全天
  pub all_day: bool,
  /// 是否归档
  pub archived: bool,
  /// 排序
  pub sort: i64,
  /// 时区
  pub time_zone: String,
  /// 步骤
  pub steps: Vec<PlanStep>,
  /// 成员 ID 列表
  pub member_ids: Vec<String>,
  /// 标签 ID 列表
  pub tag_ids: Vec<String>,
  /// 地点 ID 列表
  pub place_ids: Vec<String>,
  /// 重复规则，可以没有
  pub repeat: Option<PlanRepeat>,
  /// 提醒
  pub reminders: Vec<PlanReminder>,
  /// 挂上的媒体
  pub media: Vec<Media>,
  /// 评论
  pub comments: Vec<PlanComment>,
  /// 创建时间
  pub created_at: i64,
  /// 更新时间
  pub updated_at: i64,
}

/// 列出计划的条件
///
/// 都可省略，省略即不限制。`from` 含、`to` 不含，比较安排时间；没有安排时间则用创建时间。
///
/// # 字段
/// - `from`: 起始时间，含。省略则不限制
/// - `to`: 结束时间，不含。省略则不限制
/// - `group_id`: 清单 ID。省略则不限制
///
/// # 示例
/// 见 [`plan_list`](super::commands::plan_list)
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlanQuery {
  /// 起始时间，含。省略则不限制
  pub from: Option<i64>,
  /// 结束时间，不含。省略则不限制
  pub to: Option<i64>,
  /// 清单 ID。省略则不限制
  pub group_id: Option<String>,
}

/// 新建计划
///
/// 标题必填，其余都可省略，省略时用默认值。状态只能是 `draft`、`scheduled` 或 `done`，省略时为草稿。
///
/// # 字段
/// - `title`: 标题，不能为空
/// - `body`: 正文，省略时为空
/// - `status`: 状态，`draft`、`scheduled` 或 `done`，省略时为草稿
/// - `priority`: 优先级，省略时为 0
/// - `scheduled_at`: 安排时间，可以没有
/// - `due_at`: 截止时间，可以没有
/// - `result`: 完成结果，省略时为空
/// - `locked`: 是否锁定
/// - `highlight`: 是否高亮
/// - `group_id`: 清单 ID，可以没有
/// - `parent_id`: 父计划 ID，可以没有
/// - `all_day`: 是否全天
/// - `archived`: 是否归档
/// - `sort`: 排序，省略时为 0
/// - `time_zone`: 时区，省略时为空
/// - `steps`: 步骤
/// - `member_ids`: 成员 ID 列表
/// - `tag_ids`: 标签 ID 列表
/// - `place_ids`: 地点 ID 列表
/// - `repeat`: 重复规则，可以没有
/// - `reminders`: 提醒
///
/// # 示例
/// 见 [`plan_create`](super::commands::plan_create)
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanInput {
  /// 标题，不能为空
  pub title: String,
  /// 正文，省略时为空
  #[serde(default)]
  pub body: String,
  /// 状态，`draft`、`scheduled` 或 `done`，省略时为草稿
  #[serde(default = "default_status")]
  pub status: String,
  /// 优先级，省略时为 0
  #[serde(default)]
  pub priority: i64,
  /// 安排时间，可以没有
  #[serde(default)]
  pub scheduled_at: Option<i64>,
  /// 截止时间，可以没有
  #[serde(default)]
  pub due_at: Option<i64>,
  /// 完成结果，省略时为空
  #[serde(default)]
  pub result: String,
  /// 是否锁定
  #[serde(default)]
  pub locked: bool,
  /// 是否高亮
  #[serde(default)]
  pub highlight: bool,
  /// 清单 ID，可以没有
  #[serde(default)]
  pub group_id: Option<String>,
  /// 父计划 ID，可以没有
  #[serde(default)]
  pub parent_id: Option<String>,
  /// 是否全天
  #[serde(default)]
  pub all_day: bool,
  /// 是否归档
  #[serde(default)]
  pub archived: bool,
  /// 排序，省略时为 0
  #[serde(default)]
  pub sort: i64,
  /// 时区，省略时为空
  #[serde(default)]
  pub time_zone: String,
  /// 步骤
  #[serde(default)]
  pub steps: Vec<PlanStepInput>,
  /// 成员 ID 列表
  #[serde(default)]
  pub member_ids: Vec<String>,
  /// 标签 ID 列表
  #[serde(default)]
  pub tag_ids: Vec<String>,
  /// 地点 ID 列表
  #[serde(default)]
  pub place_ids: Vec<String>,
  /// 重复规则，可以没有
  #[serde(default)]
  pub repeat: Option<PlanRepeatInput>,
  /// 提醒
  #[serde(default)]
  pub reminders: Vec<PlanReminderInput>,
}

/// 更新计划
///
/// 没传的字段和 JSON `null` 都保持原值。传入的状态仍只能是 `draft`、`scheduled` 或 `done`。
///
/// # 字段
/// - `title`: 标题。不传则保持原值
/// - `body`: 正文。不传则保持原值
/// - `status`: 状态，`draft`、`scheduled` 或 `done`。不传则保持原值
/// - `priority`: 优先级。不传则保持原值
/// - `scheduled_at`: 安排时间。不传则保持原值
/// - `due_at`: 截止时间。不传则保持原值
/// - `result`: 完成结果。不传则保持原值
/// - `locked`: 是否锁定。不传则保持原值
/// - `highlight`: 是否高亮。不传则保持原值
/// - `group_id`: 清单 ID。不传则保持原值
/// - `parent_id`: 父计划 ID。不传则保持原值
/// - `all_day`: 是否全天。不传则保持原值
/// - `archived`: 是否归档。不传则保持原值
/// - `sort`: 排序。不传则保持原值
/// - `time_zone`: 时区。不传则保持原值
/// - `steps`: 步骤。不传则保持原值
/// - `member_ids`: 成员 ID 列表。不传则保持原值
/// - `tag_ids`: 标签 ID 列表。不传则保持原值
/// - `place_ids`: 地点 ID 列表。不传则保持原值
/// - `repeat`: 重复规则。不传则保持原值
/// - `reminders`: 提醒。不传则保持原值
///
/// # 示例
/// 见 [`plan_update`](super::commands::plan_update)
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlanPatch {
  /// 标题。不传则保持原值
  pub title: Option<String>,
  /// 正文。不传则保持原值
  pub body: Option<String>,
  /// 状态，`draft`、`scheduled` 或 `done`。不传则保持原值
  pub status: Option<String>,
  /// 优先级。不传则保持原值
  pub priority: Option<i64>,
  /// 安排时间。不传则保持原值
  pub scheduled_at: Option<i64>,
  /// 截止时间。不传则保持原值
  pub due_at: Option<i64>,
  /// 完成结果。不传则保持原值
  pub result: Option<String>,
  /// 是否锁定。不传则保持原值
  pub locked: Option<bool>,
  /// 是否高亮。不传则保持原值
  pub highlight: Option<bool>,
  /// 清单 ID。不传则保持原值
  pub group_id: Option<String>,
  /// 父计划 ID。不传则保持原值
  pub parent_id: Option<String>,
  /// 是否全天。不传则保持原值
  pub all_day: Option<bool>,
  /// 是否归档。不传则保持原值
  pub archived: Option<bool>,
  /// 排序。不传则保持原值
  pub sort: Option<i64>,
  /// 时区。不传则保持原值
  pub time_zone: Option<String>,
  /// 步骤。不传则保持原值
  pub steps: Option<Vec<PlanStepInput>>,
  /// 成员 ID 列表。不传则保持原值
  pub member_ids: Option<Vec<String>>,
  /// 标签 ID 列表。不传则保持原值
  pub tag_ids: Option<Vec<String>>,
  /// 地点 ID 列表。不传则保持原值
  pub place_ids: Option<Vec<String>>,
  /// 重复规则。不传则保持原值
  pub repeat: Option<PlanRepeatInput>,
  /// 提醒。不传则保持原值
  pub reminders: Option<Vec<PlanReminderInput>>,
}

impl PlanPatch {
  /// 判断计划补丁是否没有要改的字段
  ///
  /// # 返回值
  /// 与默认值相等时返回 true
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let empty = PlanPatch::default().is_empty();
  /// ```
  pub(super) fn is_empty(&self) -> bool {
    self == &Self::default()
  }
}

/// 清单
///
/// 系统清单不能删除。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 清单 ID
/// - `name`: 名称
/// - `color`: 颜色
/// - `system`: 是否系统清单
/// - `sort`: 排序
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
///
/// # 示例
/// 见 [`plan_group_list`](super::commands::plan_group_list)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanGroup {
  /// 清单 ID
  pub id: String,
  /// 名称
  pub name: String,
  /// 颜色
  pub color: String,
  /// 是否系统清单
  pub system: bool,
  /// 排序
  pub sort: i64,
  /// 创建时间
  pub created_at: i64,
  /// 更新时间
  pub updated_at: i64,
}

/// 新建清单
///
/// 名称去空白后不能为空。
///
/// # 字段
/// - `name`: 名称，不能为空
/// - `color`: 颜色，省略时为空
/// - `system`: 是否系统清单
/// - `sort`: 排序，省略时为 0
///
/// # 示例
/// 见 [`plan_group_create`](super::commands::plan_group_create)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanGroupInput {
  /// 名称，不能为空
  pub name: String,
  /// 颜色，省略时为空
  #[serde(default)]
  pub color: String,
  /// 是否系统清单
  #[serde(default)]
  pub system: bool,
  /// 排序，省略时为 0
  #[serde(default)]
  pub sort: i64,
}

/// 更新清单
///
/// 没传的字段和 JSON `null` 都保持原值。传入的名称去空白后不能为空。
///
/// # 字段
/// - `name`: 名称。不传则保持原值
/// - `color`: 颜色。不传则保持原值
/// - `sort`: 排序。不传则保持原值
///
/// # 示例
/// 见 [`plan_group_update`](super::commands::plan_group_update)
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlanGroupPatch {
  /// 名称。不传则保持原值
  pub name: Option<String>,
  /// 颜色。不传则保持原值
  pub color: Option<String>,
  /// 排序。不传则保持原值
  pub sort: Option<i64>,
}

impl PlanGroupPatch {
  /// 判断清单补丁是否没有要改的字段
  ///
  /// # 返回值
  /// 与默认值相等时返回 true
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let empty = PlanGroupPatch::default().is_empty();
  /// ```
  pub(super) fn is_empty(&self) -> bool {
    self == &Self::default()
  }
}

/// 判断重复种类是否在允许列表中
///
/// # 参数
/// - `kind`: 重复种类
///
/// # 返回值
/// 属于每天、每周、每月或每年时返回 true
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let ok = repeat_kind_ok("daily");
/// ```
pub(super) fn repeat_kind_ok(kind: &str) -> bool {
  REPEAT_KINDS.contains(&kind)
}
