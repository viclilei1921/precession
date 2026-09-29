//! 计划的业务判断。参数整理在 commands，读写在 repository。

use rusqlite::Connection;

use super::constants::STATUS_DONE;
use super::dto::{Plan, PlanComment, PlanGroup, PlanGroupInput, PlanGroupPatch, PlanInput, PlanPatch};
use super::error::PlanError;
use super::repository;
use crate::db::state::DbState;
use crate::utils::time::now_unix_ms;

pub fn list(db: &DbState) -> Result<Vec<Plan>, PlanError> {
  db.with_conn(repository::list)
}

pub fn get(db: &DbState, id: &str) -> Result<Plan, PlanError> {
  db.with_conn(|conn| repository::get(conn, id))
}

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

pub fn complete(db: &DbState, id: &str, result: &str) -> Result<Plan, PlanError> {
  let now = now_unix_ms();
  db.with_conn(|conn| repository::complete(conn, id, STATUS_DONE, result, now))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), PlanError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

pub fn group_list(db: &DbState) -> Result<Vec<PlanGroup>, PlanError> {
  db.with_conn(repository::group_list)
}

pub fn group_create(db: &DbState, input: PlanGroupInput) -> Result<PlanGroup, PlanError> {
  db.with_conn(|conn| repository::group_create(conn, &input))
}

pub fn group_update(db: &DbState, id: &str, patch: PlanGroupPatch) -> Result<PlanGroup, PlanError> {
  db.with_conn(|conn| {
    if patch.is_empty() {
      return repository::group_get(conn, id);
    }
    repository::group_update(conn, id, &patch)
  })
}

pub fn group_delete(db: &DbState, id: &str) -> Result<(), PlanError> {
  db.with_conn(|conn| {
    let group = repository::group_get(conn, id)?;
    if group.system {
      return Err(PlanError::GroupProtected);
    }
    repository::group_delete(conn, id)
  })
}

pub fn comment_create(db: &DbState, plan_id: &str, body: &str) -> Result<PlanComment, PlanError> {
  db.with_conn(|conn| {
    if !repository::plan_exists(conn, plan_id)? {
      return Err(PlanError::NotFound);
    }
    repository::comment_create(conn, plan_id, body)
  })
}

pub fn comment_update(db: &DbState, id: &str, body: &str) -> Result<PlanComment, PlanError> {
  db.with_conn(|conn| repository::comment_update(conn, id, body))
}

pub fn comment_delete(db: &DbState, id: &str) -> Result<(), PlanError> {
  db.with_conn(|conn| repository::comment_delete(conn, id))
}

fn ensure_group(conn: &Connection, id: &str) -> Result<(), PlanError> {
  if repository::group_exists(conn, id)? { Ok(()) } else { Err(PlanError::ReferencedMissing) }
}

fn ensure_parent(conn: &Connection, plan_id: Option<&str>, parent_id: &str) -> Result<(), PlanError> {
  let mut cursor = parent_id.to_string();
  for _ in 0..64 {
    if plan_id.is_some_and(|id| cursor == id) {
      return Err(PlanError::ParentInvalid);
    }
    match repository::parent_of(conn, &cursor)? {
      None => return Err(PlanError::ReferencedMissing),
      Some(None) => return Ok(()),
      Some(Some(next)) => cursor = next,
    }
  }
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
    assert!(matches!(list(&db), Err(PlanError::Locked)));
  }

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
    let listed = list(&db).expect("list");
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
