use tauri::State;

use super::constants::STATUSES;
use super::dto::{
  Plan, PlanComment, PlanGroup, PlanGroupInput, PlanGroupPatch, PlanInput, PlanPatch, PlanQuery, PlanRepeatInput,
  PlanStepInput, repeat_kind_ok,
};
use super::error::PlanError;
use super::service;
use crate::db::state::DbState;

/// 整理查询条件后列出计划
///
/// # 参数
/// - `state`: 数据库状态
/// - `query`: 查询条件，空则使用默认条件
///
/// # 返回值
/// 返回计划列表
///
/// # 错误
/// 查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plans = plan_list(state, query).await?;
/// ```
#[tauri::command]
pub async fn plan_list(state: State<'_, DbState>, query: Option<PlanQuery>) -> Result<Vec<Plan>, PlanError> {
  let query = prepare_query(query.unwrap_or_default());
  service::list(&state, &query)
}

/// 按主键读取计划
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 计划主键
///
/// # 返回值
/// 返回计划
///
/// # 错误
/// 查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plan = plan_get(state, id).await?;
/// ```
#[tauri::command]
pub async fn plan_get(state: State<'_, DbState>, id: String) -> Result<Plan, PlanError> {
  service::get(&state, id.as_str())
}

/// 整理输入后创建计划
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 新建计划内容
///
/// # 返回值
/// 返回新建的计划
///
/// # 错误
/// 输入不合法或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plan = plan_create(state, input).await?;
/// ```
#[tauri::command]
pub async fn plan_create(state: State<'_, DbState>, input: PlanInput) -> Result<Plan, PlanError> {
  let input = prepare_input(input)?;
  service::create(&state, input)
}

/// 整理补丁并拒绝自父后更新计划
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 计划主键
/// - `patch`: 要改的字段
///
/// # 返回值
/// 返回更新后的计划
///
/// # 错误
/// 输入不合法或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plan = plan_update(state, id, patch).await?;
/// ```
#[tauri::command]
pub async fn plan_update(state: State<'_, DbState>, id: String, patch: PlanPatch) -> Result<Plan, PlanError> {
  let patch = prepare_patch(patch)?;
  reject_self_parent(id.as_str(), &patch)?;
  service::update(&state, id.as_str(), patch)
}

/// 去掉结果首尾空白后完成计划
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 计划主键
/// - `result`: 完成结果
///
/// # 返回值
/// 返回完成后的计划
///
/// # 错误
/// 写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plan = plan_complete(state, id, result).await?;
/// ```
#[tauri::command]
pub async fn plan_complete(state: State<'_, DbState>, id: String, result: String) -> Result<Plan, PlanError> {
  service::complete(&state, id.as_str(), result.trim())
}

/// 按主键删除计划
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 计划主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// plan_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn plan_delete(state: State<'_, DbState>, id: String) -> Result<(), PlanError> {
  service::delete(&state, id.as_str())
}

/// 列出清单
///
/// # 参数
/// - `state`: 数据库状态
///
/// # 返回值
/// 返回清单列表
///
/// # 错误
/// 查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let groups = plan_group_list(state).await?;
/// ```
#[tauri::command]
pub async fn plan_group_list(state: State<'_, DbState>) -> Result<Vec<PlanGroup>, PlanError> {
  service::group_list(&state)
}

/// 整理输入后创建清单
///
/// # 参数
/// - `state`: 数据库状态
/// - `input`: 新建清单内容
///
/// # 返回值
/// 返回新建的清单
///
/// # 错误
/// 输入不合法或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let group = plan_group_create(state, input).await?;
/// ```
#[tauri::command]
pub async fn plan_group_create(state: State<'_, DbState>, input: PlanGroupInput) -> Result<PlanGroup, PlanError> {
  let input = prepare_group(input)?;
  service::group_create(&state, input)
}

/// 整理补丁后更新清单
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 清单主键
/// - `patch`: 要改的名称、颜色和排序
///
/// # 返回值
/// 返回更新后的清单
///
/// # 错误
/// 输入不合法或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let group = plan_group_update(state, id, patch).await?;
/// ```
#[tauri::command]
pub async fn plan_group_update(
  state: State<'_, DbState>,
  id: String,
  patch: PlanGroupPatch,
) -> Result<PlanGroup, PlanError> {
  let patch = prepare_group_patch(patch)?;
  service::group_update(&state, id.as_str(), patch)
}

/// 按主键删除清单
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 清单主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// plan_group_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn plan_group_delete(state: State<'_, DbState>, id: String) -> Result<(), PlanError> {
  service::group_delete(&state, id.as_str())
}

/// 整理正文后创建评论
///
/// # 参数
/// - `state`: 数据库状态
/// - `plan_id`: 计划主键
/// - `body`: 评论正文
///
/// # 返回值
/// 返回新建的评论
///
/// # 错误
/// 正文为空或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let comment = plan_comment_create(state, plan_id, body).await?;
/// ```
#[tauri::command]
pub async fn plan_comment_create(
  state: State<'_, DbState>,
  plan_id: String,
  body: String,
) -> Result<PlanComment, PlanError> {
  let body = prepare_comment(&body)?;
  service::comment_create(&state, plan_id.as_str(), &body)
}

/// 整理正文后更新评论
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 评论主键
/// - `body`: 评论正文
///
/// # 返回值
/// 返回更新后的评论
///
/// # 错误
/// 正文为空或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let comment = plan_comment_update(state, id, body).await?;
/// ```
#[tauri::command]
pub async fn plan_comment_update(
  state: State<'_, DbState>,
  id: String,
  body: String,
) -> Result<PlanComment, PlanError> {
  let body = prepare_comment(&body)?;
  service::comment_update(&state, id.as_str(), &body)
}

/// 按主键删除评论
///
/// # 参数
/// - `state`: 数据库状态
/// - `id`: 评论主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// plan_comment_delete(state, id).await?;
/// ```
#[tauri::command]
pub async fn plan_comment_delete(state: State<'_, DbState>, id: String) -> Result<(), PlanError> {
  service::comment_delete(&state, id.as_str())
}

/// 去掉文本空白并校验标题、状态、步骤和重复
///
/// # 参数
/// - `input`: 新建计划内容
///
/// # 返回值
/// 返回整理后的输入
///
/// # 错误
/// 标题为空时返回 `TitleEmpty`，状态非法时返回 `StatusInvalid`，步骤标题为空时返回 `StepTitleEmpty`，重复非法时返回 `RepeatInvalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let input = prepare_input(input)?;
/// ```
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

/// 去掉已传字段的空白并校验标题、状态、步骤和重复
///
/// # 参数
/// - `patch`: 要改的字段
///
/// # 返回值
/// 返回整理后的补丁
///
/// # 错误
/// 标题为空时返回 `TitleEmpty`，状态非法时返回 `StatusInvalid`，步骤标题为空时返回 `StepTitleEmpty`，重复非法时返回 `RepeatInvalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let patch = prepare_patch(patch)?;
/// ```
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

/// 拒绝把计划自己设成父计划
///
/// # 参数
/// - `id`: 计划主键
/// - `patch`: 要改的字段
///
/// # 返回值
/// 父计划不是自己时返回空
///
/// # 错误
/// 父计划主键与 `id` 相同时返回 `PlanError::ParentInvalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// reject_self_parent(id, patch)?;
/// ```
fn reject_self_parent(id: &str, patch: &PlanPatch) -> Result<(), PlanError> {
  if let Some(parent_id) = &patch.parent_id
    && parent_id == id
  {
    return Err(PlanError::ParentInvalid);
  }
  Ok(())
}

/// 去掉步骤标题空白并拒绝空标题
///
/// # 参数
/// - `steps`: 步骤列表
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 任一标题去空白后为空时返回 `PlanError::StepTitleEmpty`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// prepare_steps(steps)?;
/// ```
fn prepare_steps(steps: &mut [PlanStepInput]) -> Result<(), PlanError> {
  for step in steps {
    step.title = step.title.trim().to_string();
    if step.title.is_empty() {
      return Err(PlanError::StepTitleEmpty);
    }
  }
  Ok(())
}

/// 去掉重复种类和星期空白，并校验种类与间隔
///
/// # 参数
/// - `repeat`: 重复规则
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 种类不在允许列表或间隔小于 1 时返回 `PlanError::RepeatInvalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// prepare_repeat(repeat)?;
/// ```
fn prepare_repeat(repeat: &mut PlanRepeatInput) -> Result<(), PlanError> {
  repeat.kind = repeat.kind.trim().to_string();
  repeat.weekdays = repeat.weekdays.trim().to_string();
  if !repeat_kind_ok(&repeat.kind) || repeat.interval < 1 {
    return Err(PlanError::RepeatInvalid);
  }
  Ok(())
}

/// 去掉名称和颜色空白并拒绝空名称
///
/// # 参数
/// - `input`: 新建清单内容
///
/// # 返回值
/// 返回整理后的输入
///
/// # 错误
/// 名称去空白后为空时返回 `PlanError::NameEmpty`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let input = prepare_group(input)?;
/// ```
fn prepare_group(mut input: PlanGroupInput) -> Result<PlanGroupInput, PlanError> {
  input.name = input.name.trim().to_string();
  input.color = input.color.trim().to_string();
  if input.name.is_empty() {
    return Err(PlanError::NameEmpty);
  }
  Ok(input)
}

/// 去掉已传名称和颜色的空白并拒绝空名称
///
/// # 参数
/// - `patch`: 要改的字段
///
/// # 返回值
/// 返回整理后的补丁
///
/// # 错误
/// 名称去空白后为空时返回 `PlanError::NameEmpty`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let patch = prepare_group_patch(patch)?;
/// ```
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

/// 去掉评论空白并拒绝空正文
///
/// # 参数
/// - `body`: 评论正文
///
/// # 返回值
/// 返回去空白后的正文
///
/// # 错误
/// 去空白后为空时返回 `PlanError::CommentEmpty`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let body = prepare_comment(body)?;
/// ```
fn prepare_comment(body: &str) -> Result<String, PlanError> {
  let body = body.trim().to_string();
  if body.is_empty() { Err(PlanError::CommentEmpty) } else { Ok(body) }
}

/// 把空白清单主键转成空
///
/// # 参数
/// - `query`: 查询条件
///
/// # 返回值
/// 返回整理后的查询条件
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let query = prepare_query(query);
/// ```
fn prepare_query(mut query: PlanQuery) -> PlanQuery {
  query.group_id = blank_to_none(query.group_id);
  query
}

/// 去掉空白并把空字符串转成空
///
/// # 参数
/// - `value`: 可选文本
///
/// # 返回值
/// 去空白后为空或原本为空时返回 `None`，否则返回 `Some`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let value = blank_to_none(Some("  ".into()));
/// ```
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
