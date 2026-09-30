use tauri::State;

use super::constants::STATUSES;
use super::dto::{
  Plan, PlanComment, PlanGroup, PlanGroupInput, PlanGroupPatch, PlanInput, PlanPatch, PlanQuery, PlanRepeatInput,
  PlanStepInput, repeat_kind_ok,
};
use super::error::PlanError;
use super::service;
use crate::db::state::DbState;

/// 获取计划列表。条件都可省略，省略时返回全部未删除计划。
/// `from` 含、`to` 不含：某一天传当天 0 点到次日 0 点，某个月传当月 1 日 0 点到下月 1 日 0 点。
/// `group_id` 只返回该清单里的计划，可与时间一起用。
#[tauri::command]
pub async fn plan_list(state: State<'_, DbState>, query: Option<PlanQuery>) -> Result<Vec<Plan>, PlanError> {
  let query = prepare_query(query.unwrap_or_default());
  service::list(&state, &query)
}

/// 获取计划详情
/// #### Arguments
/// * `id` - 计划ID
/// #### Returns
/// * `Plan` - 计划
#[tauri::command]
pub async fn plan_get(state: State<'_, DbState>, id: String) -> Result<Plan, PlanError> {
  service::get(&state, id.as_str())
}

/// 创建计划
/// #### Arguments
/// * `input` - 计划输入
/// #### Returns
/// * `Plan` - 计划
#[tauri::command]
pub async fn plan_create(state: State<'_, DbState>, input: PlanInput) -> Result<Plan, PlanError> {
  let input = prepare_input(input)?;
  service::create(&state, input)
}

/// 更新计划。只改传来的字段。
/// #### Arguments
/// * `id` - 计划ID
/// * `patch` - 要改的字段
/// #### Returns
/// * `Plan` - 计划
#[tauri::command]
pub async fn plan_update(state: State<'_, DbState>, id: String, patch: PlanPatch) -> Result<Plan, PlanError> {
  let patch = prepare_patch(patch)?;
  reject_self_parent(id.as_str(), &patch)?;
  service::update(&state, id.as_str(), patch)
}

/// 完成计划
/// #### Arguments
/// * `id` - 计划ID
/// * `result` - 计划结果
/// #### Returns
/// * `Plan` - 计划
#[tauri::command]
pub async fn plan_complete(state: State<'_, DbState>, id: String, result: String) -> Result<Plan, PlanError> {
  service::complete(&state, id.as_str(), result.trim())
}

/// 删除计划
/// #### Arguments
/// * `id` - 计划ID
/// #### Returns
/// * `()` - 空
#[tauri::command]
pub async fn plan_delete(state: State<'_, DbState>, id: String) -> Result<(), PlanError> {
  service::delete(&state, id.as_str())
}

/// 获取清单列表
#[tauri::command]
pub async fn plan_group_list(state: State<'_, DbState>) -> Result<Vec<PlanGroup>, PlanError> {
  service::group_list(&state)
}

/// 创建清单
/// #### Arguments
/// * `input` - 清单输入
/// #### Returns
/// * `PlanGroup` - 清单
#[tauri::command]
pub async fn plan_group_create(state: State<'_, DbState>, input: PlanGroupInput) -> Result<PlanGroup, PlanError> {
  let input = prepare_group(input)?;
  service::group_create(&state, input)
}

/// 更新清单。只改传来的名称、颜色和排序。
/// #### Arguments
/// * `id` - 清单ID
/// * `patch` - 要改的字段
/// #### Returns
/// * `PlanGroup` - 清单
#[tauri::command]
pub async fn plan_group_update(
  state: State<'_, DbState>,
  id: String,
  patch: PlanGroupPatch,
) -> Result<PlanGroup, PlanError> {
  let patch = prepare_group_patch(patch)?;
  service::group_update(&state, id.as_str(), patch)
}

/// 删除清单。挂在上面的计划会失去归属。
/// #### Arguments
/// * `id` - 清单ID
#[tauri::command]
pub async fn plan_group_delete(state: State<'_, DbState>, id: String) -> Result<(), PlanError> {
  service::group_delete(&state, id.as_str())
}

/// 添加评论
/// #### Arguments
/// * `plan_id` - 计划ID
/// * `body` - 评论正文
/// #### Returns
/// * `PlanComment` - 评论
#[tauri::command]
pub async fn plan_comment_create(
  state: State<'_, DbState>,
  plan_id: String,
  body: String,
) -> Result<PlanComment, PlanError> {
  let body = prepare_comment(&body)?;
  service::comment_create(&state, plan_id.as_str(), &body)
}

/// 修改评论
/// #### Arguments
/// * `id` - 评论ID
/// * `body` - 评论正文
/// #### Returns
/// * `PlanComment` - 评论
#[tauri::command]
pub async fn plan_comment_update(
  state: State<'_, DbState>,
  id: String,
  body: String,
) -> Result<PlanComment, PlanError> {
  let body = prepare_comment(&body)?;
  service::comment_update(&state, id.as_str(), &body)
}

/// 删除评论
/// #### Arguments
/// * `id` - 评论ID
#[tauri::command]
pub async fn plan_comment_delete(state: State<'_, DbState>, id: String) -> Result<(), PlanError> {
  service::comment_delete(&state, id.as_str())
}

/// 准备计划输入。trim 前后空白，转换空字符串为 None。
fn prepare_input(mut input: PlanInput) -> Result<PlanInput, PlanError> {
  input.title = input.title.trim().to_string();
  input.body = input.body.trim().to_string();
  input.result = input.result.trim().to_string();
  input.status = input.status.trim().to_string();
  input.time_zone = input.time_zone.trim().to_string();
  input.group_id = blank_to_none(input.group_id);
  input.parent_id = blank_to_none(input.parent_id);
  if input.title.is_empty() {
    return Err(PlanError::TitleEmpty);
  }
  if !STATUSES.contains(&input.status.as_str()) {
    return Err(PlanError::StatusInvalid);
  }
  prepare_steps(&mut input.steps)?;
  if let Some(repeat) = &mut input.repeat {
    prepare_repeat(repeat)?;
  }
  for reminder in &mut input.reminders {
    reminder.note = reminder.note.trim().to_string();
  }
  Ok(input)
}

/// 准备计划更新。trim 前后空白，转换空字符串为 None。
fn prepare_patch(mut patch: PlanPatch) -> Result<PlanPatch, PlanError> {
  if let Some(title) = &mut patch.title {
    *title = title.trim().to_string();
    if title.is_empty() {
      return Err(PlanError::TitleEmpty);
    }
  }
  if let Some(body) = &mut patch.body {
    *body = body.trim().to_string();
  }
  if let Some(result) = &mut patch.result {
    *result = result.trim().to_string();
  }
  if let Some(status) = &mut patch.status {
    *status = status.trim().to_string();
    if !STATUSES.contains(&status.as_str()) {
      return Err(PlanError::StatusInvalid);
    }
  }
  if let Some(time_zone) = &mut patch.time_zone {
    *time_zone = time_zone.trim().to_string();
  }
  patch.group_id = blank_to_none(patch.group_id);
  patch.parent_id = blank_to_none(patch.parent_id);
  if let Some(steps) = &mut patch.steps {
    prepare_steps(steps)?;
  }
  if let Some(repeat) = &mut patch.repeat {
    prepare_repeat(repeat)?;
  }
  if let Some(reminders) = &mut patch.reminders {
    for reminder in reminders {
      reminder.note = reminder.note.trim().to_string();
    }
  }
  Ok(patch)
}

/// 拒绝计划把自己设成父计划。
fn reject_self_parent(id: &str, patch: &PlanPatch) -> Result<(), PlanError> {
  if let Some(parent_id) = &patch.parent_id
    && parent_id == id
  {
    return Err(PlanError::ParentInvalid);
  }
  Ok(())
}

/// 准备计划步骤。trim 前后空白，转换空字符串为 None。
fn prepare_steps(steps: &mut [PlanStepInput]) -> Result<(), PlanError> {
  for step in steps {
    step.title = step.title.trim().to_string();
    if step.title.is_empty() {
      return Err(PlanError::StepTitleEmpty);
    }
  }
  Ok(())
}

/// 准备重复规则。trim 前后空白，转换空字符串为 None。
fn prepare_repeat(repeat: &mut PlanRepeatInput) -> Result<(), PlanError> {
  repeat.kind = repeat.kind.trim().to_string();
  repeat.weekdays = repeat.weekdays.trim().to_string();
  if !repeat_kind_ok(&repeat.kind) || repeat.interval < 1 {
    return Err(PlanError::RepeatInvalid);
  }
  Ok(())
}

/// 准备清单输入。trim 前后空白，转换空字符串为 None。
fn prepare_group(mut input: PlanGroupInput) -> Result<PlanGroupInput, PlanError> {
  input.name = input.name.trim().to_string();
  input.color = input.color.trim().to_string();
  if input.name.is_empty() {
    return Err(PlanError::NameEmpty);
  }
  Ok(input)
}

/// 准备清单更新。trim 前后空白，转换空字符串为 None。
fn prepare_group_patch(mut patch: PlanGroupPatch) -> Result<PlanGroupPatch, PlanError> {
  if let Some(name) = &mut patch.name {
    *name = name.trim().to_string();
    if name.is_empty() {
      return Err(PlanError::NameEmpty);
    }
  }
  if let Some(color) = &mut patch.color {
    *color = color.trim().to_string();
  }
  Ok(patch)
}

/// 准备评论正文。trim 前后空白，转换空字符串为 None。
fn prepare_comment(body: &str) -> Result<String, PlanError> {
  let body = body.trim().to_string();
  if body.is_empty() { Err(PlanError::CommentEmpty) } else { Ok(body) }
}

/// 准备计划查询条件。转换空字符串为 None。
fn prepare_query(mut query: PlanQuery) -> PlanQuery {
  query.group_id = blank_to_none(query.group_id);
  query
}

/// 转换空字符串为 None
fn blank_to_none(value: Option<String>) -> Option<String> {
  value.and_then(|text| {
    let text = text.trim().to_string();
    if text.is_empty() { None } else { Some(text) }
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::plan::constants::STATUS_SCHEDULED;
  use crate::plan::dto::PlanReminderInput;

  fn sample() -> PlanInput {
    PlanInput {
      title: "晨跑".into(),
      status: STATUS_SCHEDULED.into(),
      steps: vec![PlanStepInput { title: "热身".into(), done: false }],
      ..PlanInput::default()
    }
  }

  /// 创建、更新、查询、评论和清单都会去掉前后空白；纯空白的可选 ID 会变成「未设置」。
  #[test]
  fn trims_fields() {
    // 创建：标题、正文、步骤、重复规则、提醒备注去空白；空白清单 ID 视为无归属。
    let mut input = sample();
    input.title = " 晨跑 ".into();
    input.body = " 正文 ".into();
    input.group_id = Some("  ".into());
    input.parent_id = Some(" parent ".into());
    input.steps = vec![PlanStepInput { title: " 热身 ".into(), done: false }];
    input.repeat =
      Some(PlanRepeatInput { kind: " weekly ".into(), interval: 1, weekdays: " 1,3 ".into(), until_at: None });
    input.reminders = vec![PlanReminderInput { remind_at: 20, note: " 出门 ".into() }];
    let prepared = prepare_input(input).expect("prepare");
    assert_eq!(prepared.title, "晨跑");
    assert_eq!(prepared.body, "正文");
    assert!(prepared.group_id.is_none());
    assert_eq!(prepared.parent_id.as_deref(), Some("parent"));
    assert_eq!(prepared.steps[0].title, "热身");
    assert_eq!(prepared.repeat.as_ref().map(|item| item.kind.as_str()), Some("weekly"));
    assert_eq!(prepared.repeat.as_ref().map(|item| item.weekdays.as_str()), Some("1,3"));
    assert_eq!(prepared.reminders[0].note, "出门");

    // 更新：只处理传来的字段，空白清单 ID 同样视为不改归属。
    let patch = prepare_patch(PlanPatch {
      title: Some(" 夜跑 ".into()),
      parent_id: Some(" parent ".into()),
      group_id: Some("  ".into()),
      ..PlanPatch::default()
    })
    .expect("patch");
    assert_eq!(patch.title.as_deref(), Some("夜跑"));
    assert_eq!(patch.parent_id.as_deref(), Some("parent"));
    assert!(patch.group_id.is_none());

    // 查询：清单 ID 去空白；纯空白则不按清单筛选。
    let query = prepare_query(PlanQuery { group_id: Some("  life ".into()), ..PlanQuery::default() });
    assert_eq!(query.group_id.as_deref(), Some("life"));
    assert!(prepare_query(PlanQuery { group_id: Some("  ".into()), ..PlanQuery::default() }).group_id.is_none());

    // 评论正文、清单名称和颜色同样去空白。
    assert_eq!(prepare_comment(" 记得 ").expect("comment"), "记得");
    let group =
      prepare_group(PlanGroupInput { name: " 生活 ".into(), color: " #fff ".into(), system: false, sort: 0 })
        .expect("group");
    assert_eq!(group.name, "生活");
    assert_eq!(group.color, "#fff");
  }

  /// 必填项为空、状态不在允许列表、重复类型非法或间隔小于 1 时，准备阶段直接拒绝。
  #[test]
  fn rejects_bad_parameters() {
    // 创建：标题去掉空白后为空。
    let mut empty = sample();
    empty.title = "  ".into();
    assert!(matches!(prepare_input(empty), Err(PlanError::TitleEmpty)));

    // 创建：步骤标题去掉空白后为空。
    let mut steps = sample();
    steps.steps = vec![PlanStepInput { title: "  ".into(), done: false }];
    assert!(matches!(prepare_input(steps), Err(PlanError::StepTitleEmpty)));

    // 创建：状态不在允许列表里。
    let mut status = sample();
    status.status = "nope".into();
    assert!(matches!(prepare_input(status), Err(PlanError::StatusInvalid)));

    // 创建：重复类型不是 daily / weekly / monthly / yearly。
    let mut repeat = sample();
    repeat.repeat =
      Some(PlanRepeatInput { kind: "hourly".into(), interval: 1, weekdays: String::new(), until_at: None });
    assert!(matches!(prepare_input(repeat), Err(PlanError::RepeatInvalid)));

    // 创建：重复间隔必须至少为 1。
    let mut interval = sample();
    interval.repeat =
      Some(PlanRepeatInput { kind: "daily".into(), interval: 0, weekdays: String::new(), until_at: None });
    assert!(matches!(prepare_input(interval), Err(PlanError::RepeatInvalid)));

    // 更新：传来的标题为空、状态非法时同样拒绝。
    assert!(matches!(
      prepare_patch(PlanPatch { title: Some("  ".into()), ..PlanPatch::default() }),
      Err(PlanError::TitleEmpty)
    ));
    assert!(matches!(
      prepare_patch(PlanPatch { status: Some("nope".into()), ..PlanPatch::default() }),
      Err(PlanError::StatusInvalid)
    ));

    // 评论正文、清单名称（创建和更新）去掉空白后为空。
    assert!(matches!(prepare_comment("  "), Err(PlanError::CommentEmpty)));
    assert!(matches!(
      prepare_group(PlanGroupInput { name: "  ".into(), color: String::new(), system: false, sort: 0 }),
      Err(PlanError::NameEmpty)
    ));
    assert!(matches!(
      prepare_group_patch(PlanGroupPatch { name: Some("  ".into()), ..PlanGroupPatch::default() }),
      Err(PlanError::NameEmpty)
    ));
  }

  /// 计划不能把自己设成父计划；父计划是别的计划、或这次没改父计划时可以通过。
  #[test]
  fn rejects_self_parent() {
    // 父计划 ID 去掉空白后与自身相同。
    let patch = prepare_patch(PlanPatch { parent_id: Some(" same ".into()), ..PlanPatch::default() }).expect("patch");
    assert!(matches!(reject_self_parent("same", &patch), Err(PlanError::ParentInvalid)));

    // 父计划是别的计划，或补丁里没有父计划字段。
    let other = PlanPatch { parent_id: Some("other".into()), ..PlanPatch::default() };
    assert!(reject_self_parent("same", &other).is_ok());
    assert!(reject_self_parent("same", &PlanPatch::default()).is_ok());
  }
}
