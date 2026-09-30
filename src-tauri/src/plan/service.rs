//! 计划的业务判断。参数整理在 commands，读写在 repository。

use rusqlite::Connection;

use super::constants::STATUS_DONE;
use super::dto::{Plan, PlanComment, PlanGroup, PlanGroupInput, PlanGroupPatch, PlanInput, PlanPatch, PlanQuery};
use super::error::PlanError;
use super::repository;
use crate::db::state::DbState;
use crate::utils::time::now_unix_ms;

/// 列出计划
/// #### Arguments
/// * `query` - 查询条件
/// #### Returns
/// * 计划列表
pub fn list(db: &DbState, query: &PlanQuery) -> Result<Vec<Plan>, PlanError> {
  db.with_conn(|conn| repository::list(conn, query))
}

/// 获取计划详情
/// #### Arguments
/// * `id` - 计划ID
/// #### Returns
/// * 计划详情
pub fn get(db: &DbState, id: &str) -> Result<Plan, PlanError> {
  db.with_conn(|conn| repository::get(conn, id))
}

/// 创建计划
/// #### Arguments
/// * `input` - 计划输入
/// #### Returns
/// * 计划
pub fn create(db: &DbState, input: PlanInput) -> Result<Plan, PlanError> {
  db.with_conn(|conn| {
    if let Some(group_id) = &input.group_id {
      ensure_group(conn, group_id)?;
    }
    if let Some(parent_id) = &input.parent_id {
      ensure_parent(conn, None, parent_id)?;
    }
    repository::create(conn, &input)
  })
}

/// 更新计划
/// #### Arguments
/// * `id` - 计划ID
/// * `patch` - 要改的字段
/// #### Returns
/// * 计划
pub fn update(db: &DbState, id: &str, patch: PlanPatch) -> Result<Plan, PlanError> {
  db.with_conn(|conn| {
    if patch.is_empty() {
      return repository::get(conn, id);
    }
    if let Some(group_id) = &patch.group_id {
      ensure_group(conn, group_id)?;
    }
    if let Some(parent_id) = &patch.parent_id {
      ensure_parent(conn, Some(id), parent_id)?;
    }
    repository::update(conn, id, &patch)
  })
}

/// 完成计划
/// #### Arguments
/// * `id` - 计划ID
/// * `result` - 完成结果
/// #### Returns
/// * 计划
pub fn complete(db: &DbState, id: &str, result: &str) -> Result<Plan, PlanError> {
  let now = now_unix_ms();
  db.with_conn(|conn| repository::complete(conn, id, STATUS_DONE, result, now))
}

/// 删除计划
/// #### Arguments
/// * `id` - 计划ID
/// #### Returns
/// * 结果
pub fn delete(db: &DbState, id: &str) -> Result<(), PlanError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

/// 获取清单列表
/// #### Returns
/// * 清单列表
pub fn group_list(db: &DbState) -> Result<Vec<PlanGroup>, PlanError> {
  db.with_conn(repository::group_list)
}

/// 创建清单
/// #### Arguments
/// * `input` - 清单输入
/// #### Returns
/// * 清单
pub fn group_create(db: &DbState, input: PlanGroupInput) -> Result<PlanGroup, PlanError> {
  db.with_conn(|conn| repository::group_create(conn, &input))
}

/// 更新清单
/// #### Arguments
/// * `id` - 清单ID
/// * `patch` - 要改的字段
/// #### Returns
/// * 清单
pub fn group_update(db: &DbState, id: &str, patch: PlanGroupPatch) -> Result<PlanGroup, PlanError> {
  db.with_conn(|conn| {
    // 如果更新内容为空，则返回当前清单
    if patch.is_empty() {
      return repository::group_get(conn, id);
    }

    // 更新清单
    repository::group_update(conn, id, &patch)
  })
}

/// 删除清单
/// #### Arguments
/// * `id` - 清单ID
/// #### Returns
/// * 结果
pub fn group_delete(db: &DbState, id: &str) -> Result<(), PlanError> {
  db.with_conn(|conn| {
    // 获取清单
    let group = repository::group_get(conn, id)?;

    // 如果清单是系统清单，则返回错误
    if group.system {
      return Err(PlanError::GroupProtected);
    }

    // 删除清单
    repository::group_delete(conn, id)
  })
}

/// 创建评论
/// #### Arguments
/// * `plan_id` - 计划ID
/// * `body` - 评论正文
/// #### Returns
/// * 评论
pub fn comment_create(db: &DbState, plan_id: &str, body: &str) -> Result<PlanComment, PlanError> {
  db.with_conn(|conn| {
    // 确保计划存在
    if !repository::plan_exists(conn, plan_id)? {
      return Err(PlanError::NotFound);
    }

    // 创建评论
    repository::comment_create(conn, plan_id, body)
  })
}

/// 更新评论
/// #### Arguments
/// * `id` - 评论ID
/// * `body` - 评论正文
/// #### Returns
/// * 评论
pub fn comment_update(db: &DbState, id: &str, body: &str) -> Result<PlanComment, PlanError> {
  db.with_conn(|conn| repository::comment_update(conn, id, body))
}

/// 删除评论
/// #### Arguments
/// * `id` - 评论ID
/// #### Returns
/// * 结果
pub fn comment_delete(db: &DbState, id: &str) -> Result<(), PlanError> {
  db.with_conn(|conn| repository::comment_delete(conn, id))
}

/// 确保清单存在
/// #### Arguments
/// * `conn` - 数据库连接
/// * `id` - 清单ID
/// #### Returns
/// * 结果
fn ensure_group(conn: &Connection, id: &str) -> Result<(), PlanError> {
  if repository::group_exists(conn, id)? { Ok(()) } else { Err(PlanError::ReferencedMissing) }
}

/// 确保父计划存在
/// #### Arguments
/// * `conn` - 数据库连接
/// * `plan_id` - 计划ID
/// * `parent_id` - 父计划ID
/// #### Returns
/// * 结果
fn ensure_parent(conn: &Connection, plan_id: Option<&str>, parent_id: &str) -> Result<(), PlanError> {
  let mut cursor = parent_id.to_string();

  // 循环查找父计划，最多64层，避免无限嵌套
  for _ in 0..64 {
    // 如果父计划是自己，则返回错误
    if plan_id.is_some_and(|id| cursor == id) {
      return Err(PlanError::ParentInvalid);
    }
    // 获取父计划
    match repository::parent_of(conn, &cursor)? {
      // 父计划不存在
      None => return Err(PlanError::ReferencedMissing),
      // 父计划是空，则返回成功
      Some(None) => return Ok(()),
      // 父计划是另一个计划，则继续查找
      Some(Some(next)) => cursor = next,
    }
  }

  // 父计划不存在
  Err(PlanError::ParentInvalid)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::db::error::DbError;
  use crate::media::KIND_IMAGE;
  use crate::media::dto::MediaInput;
  use crate::owner::Owner;
  use crate::plan::{
    constants::{STATUS_DONE, STATUS_SCHEDULED},
    dto::{PlanRepeatInput, PlanStepInput},
  };
  use crate::utils::id::new_uuid_v4;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![
      crate::member::migrate,
      crate::tag::migrate,
      crate::place::migrate,
      crate::media::migrate,
      crate::plan::migrate,
    ]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  fn sample() -> PlanInput {
    PlanInput {
      title: "晨跑".into(),
      status: STATUS_SCHEDULED.into(),
      priority: 1,
      scheduled_at: Some(10),
      due_at: Some(10),
      steps: vec![PlanStepInput { title: "热身".into(), done: false }],
      ..PlanInput::default()
    }
  }

  /// 创建、完成和锁定
  #[test]
  fn create_complete_and_lock() {
    let (_dir, db) = setup();
    let item = create(&db, sample()).expect("create");
    assert_eq!(item.steps.len(), 1);

    let done = complete(&db, &item.id, "到了公园").expect("complete");
    assert_eq!(done.status, STATUS_DONE);
    assert_eq!(done.result, "到了公园");
    assert!(done.completed_at.is_some());

    db.lock().expect("lock");
    assert!(matches!(list(&db, &PlanQuery::default()), Err(PlanError::Locked)));
  }

  /// 关系、更新和清理
  #[test]
  fn relations_patch_and_cleanup() {
    let (_dir, db) = setup();
    let group = group_create(
      &db,
      PlanGroupInput { name: "生活".into(), color: String::new(), system: false, sort: 0 },
    )
    .expect("group");
    assert!(matches!(
      group_create(
        &db,
        PlanGroupInput { name: "生活".into(), color: String::new(), system: false, sort: 1 }
      ),
      Err(PlanError::NameTaken)
    ));
    let protected = group_create(
      &db,
      PlanGroupInput { name: "系统".into(), color: String::new(), system: true, sort: 0 },
    )
    .expect("system");
    assert!(matches!(group_delete(&db, &protected.id), Err(PlanError::GroupProtected)));

    let member_id = new_uuid_v4();
    db.with_conn(|conn| {
      conn
        .execute(
          "INSERT INTO member (id, name, relation, gender, created_at, updated_at) VALUES (?1, '自己', '', '', 1, 1)",
          [&member_id],
        )
        .expect("member");
      Ok::<(), DbError>(())
    })
    .expect("seed member");

    let mut input = sample();
    input.group_id = Some(group.id.clone());
    input.member_ids = vec![member_id.clone()];
    input.repeat = Some(PlanRepeatInput { kind: "weekly".into(), interval: 1, weekdays: "1,3".into(), until_at: None });
    input.reminders = vec![crate::plan::dto::PlanReminderInput { remind_at: 20, note: "出门".into() }];
    let item = create(&db, input).expect("plan");
    assert_eq!(item.group_id.as_deref(), Some(group.id.as_str()));
    assert_eq!(item.member_ids, vec![member_id.clone()]);
    assert_eq!(item.repeat.as_ref().map(|item| item.kind.as_str()), Some("weekly"));
    assert_eq!(item.reminders.len(), 1);

    let mut child_input = sample();
    child_input.title = "拉伸".into();
    child_input.parent_id = Some(item.id.clone());
    let child = create(&db, child_input).expect("child");
    assert_eq!(child.parent_id.as_deref(), Some(item.id.as_str()));

    let mut cycle = PlanPatch::default();
    cycle.parent_id = Some(child.id.clone());
    assert!(matches!(update(&db, &item.id, cycle), Err(PlanError::ParentInvalid)));

    comment_create(&db, &item.id, "记得带水").expect("comment");
    let detail = get(&db, &item.id).expect("detail");
    assert_eq!(detail.comments.len(), 1);
    let listed = list(&db, &PlanQuery::default()).expect("list");
    let listed_item = listed.iter().find(|plan| plan.id == item.id).expect("listed");
    assert!(listed_item.comments.is_empty());

    let media_id = new_uuid_v4();
    db.with_conn(|conn| {
      crate::media::insert_imported(
        conn,
        &media_id,
        MediaInput {
          owner: Owner::Plan.as_str().into(),
          owner_id: item.id.clone(),
          kind: KIND_IMAGE.into(),
          rel_path: "a.jpg".into(),
          mime: "image/jpeg".into(),
          sort: 0,
          locked: false,
        },
        false,
      )
      .expect("media");
      Ok::<(), DbError>(())
    })
    .expect("seed media");

    let renamed =
      update(&db, &item.id, PlanPatch { title: Some("夜跑".into()), ..PlanPatch::default() }).expect("patch");
    assert_eq!(renamed.title, "夜跑");
    assert_eq!(renamed.member_ids, vec![member_id]);
    assert!(renamed.repeat.is_some());
    assert_eq!(renamed.reminders.len(), 1);
    assert_eq!(renamed.media.len(), 1);
    assert_eq!(renamed.comments.len(), 1);

    let untouched = update(&db, &item.id, PlanPatch::default()).expect("empty patch");
    assert_eq!(untouched.updated_at, renamed.updated_at);

    delete(&db, &item.id).expect("delete plan");
    assert!(matches!(get(&db, &item.id), Err(PlanError::NotFound)));
    let child = get(&db, &child.id).expect("child remains");
    assert!(child.parent_id.is_none());
    db.with_conn(|conn| {
      let left = crate::media::list_for(conn, Owner::Plan.as_str(), &item.id).expect("media");
      assert!(left.is_empty());
      Ok::<(), DbError>(())
    })
    .expect("media cleared");

    let mut parked = sample();
    parked.title = "买菜".into();
    parked.group_id = Some(group.id.clone());
    let parked = create(&db, parked).expect("parked");
    group_delete(&db, &group.id).expect("delete group");
    assert!(matches!(
      group_list(&db).expect("groups").iter().find(|item| item.id == group.id),
      None
    ));
    assert!(get(&db, &parked.id).expect("parked").group_id.is_none());
  }

  /// 列表过滤
  #[test]
  fn list_filters_by_range_and_group() {
    let (_dir, db) = setup();
    let life = group_create(
      &db,
      PlanGroupInput { name: "生活".into(), color: String::new(), system: false, sort: 0 },
    )
    .expect("life");
    let work = group_create(
      &db,
      PlanGroupInput { name: "工作".into(), color: String::new(), system: false, sort: 1 },
    )
    .expect("work");

    let mut morning = sample();
    morning.scheduled_at = Some(100);
    morning.group_id = Some(life.id.clone());
    create(&db, morning).expect("morning");

    let mut noon = sample();
    noon.title = "午饭".into();
    noon.scheduled_at = Some(200);
    noon.group_id = Some(life.id.clone());
    create(&db, noon).expect("noon");

    let mut meeting = sample();
    meeting.title = "开会".into();
    meeting.scheduled_at = Some(150);
    meeting.group_id = Some(work.id.clone());
    create(&db, meeting).expect("meeting");

    let mut loose = sample();
    loose.title = "随手".into();
    loose.scheduled_at = Some(150);
    create(&db, loose).expect("loose");

    let mut inbox = sample();
    inbox.title = "收集".into();
    inbox.scheduled_at = None;
    create(&db, inbox).expect("inbox");

    let names = |query: PlanQuery| {
      let mut titles = list(&db, &query).expect("list").into_iter().map(|plan| plan.title).collect::<Vec<_>>();
      titles.sort();
      titles
    };

    assert_eq!(names(PlanQuery::default()).len(), 5);
    assert_eq!(
      names(PlanQuery { from: Some(100), to: Some(200), ..PlanQuery::default() }),
      ["开会", "晨跑", "随手"]
    );
    assert_eq!(
      names(PlanQuery { from: Some(100), to: Some(200), group_id: Some(life.id.clone()) }),
      ["晨跑"]
    );
    assert_eq!(
      names(PlanQuery { group_id: Some(life.id), ..PlanQuery::default() }),
      ["午饭", "晨跑"]
    );
    assert_eq!(
      names(PlanQuery { from: Some(200), to: Some(300), ..PlanQuery::default() }),
      ["午饭"]
    );
  }

  /// 补丁 JSON 空值处理
  #[test]
  fn patch_json_treats_null_as_missing() {
    let patch: PlanPatch =
      serde_json::from_str(r#"{"title":"夜跑","body":"","scheduledAt":null,"groupId":null,"repeat":null}"#)
        .expect("patch");
    assert_eq!(patch.title.as_deref(), Some("夜跑"));
    assert_eq!(patch.body.as_deref(), Some(""));
    assert!(patch.scheduled_at.is_none());
    assert!(patch.group_id.is_none());
    assert!(patch.repeat.is_none());
    assert!(patch.due_at.is_none());
    let title_null: PlanPatch = serde_json::from_str(r#"{"title":null}"#).expect("null title");
    assert!(title_null.title.is_none());
  }
}
