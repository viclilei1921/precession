//! 计划。同一次 `with_conn` 里写计划、步骤和引用。

use super::constants::{STATUS_DONE, STATUS_INBOX, STATUS_SCHEDULED};
use super::dto::{Plan, PlanInput};
use super::error::PlanError;
use super::repository;
use crate::db::state::DbState;

const STATUSES: &[&str] = &[STATUS_INBOX, STATUS_SCHEDULED, STATUS_DONE];

pub fn list(db: &DbState) -> Result<Vec<Plan>, PlanError> {
  db.with_conn(repository::list)
}

pub fn get(db: &DbState, id: &str) -> Result<Plan, PlanError> {
  db.with_conn(|conn| repository::get(conn, id))
}

pub fn create(db: &DbState, input: PlanInput) -> Result<Plan, PlanError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::create(conn, &input))
}

pub fn update(db: &DbState, id: &str, input: PlanInput) -> Result<Plan, PlanError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::update(conn, id, &input))
}

pub fn complete(db: &DbState, id: &str, result: &str) -> Result<Plan, PlanError> {
  let result = result.trim().to_string();
  db.with_conn(move |conn| repository::complete(conn, id, &result))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), PlanError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

fn prepare(mut input: PlanInput) -> Result<PlanInput, PlanError> {
  input.title = input.title.trim().to_string();
  input.body = input.body.trim().to_string();
  input.result = input.result.trim().to_string();
  input.status = input.status.trim().to_string();
  if input.title.is_empty() {
    return Err(PlanError::TitleEmpty);
  }
  if !STATUSES.contains(&input.status.as_str()) {
    return Err(PlanError::StatusInvalid);
  }
  for step in &mut input.steps {
    step.title = step.title.trim().to_string();
    if step.title.is_empty() {
      return Err(PlanError::StepTitleEmpty);
    }
  }
  Ok(input)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::plan::dto::PlanStepInput;

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
      body: String::new(),
      status: STATUS_SCHEDULED.into(),
      priority: 1,
      scheduled_at: Some(10),
      due_at: Some(10),
      result: String::new(),
      locked: false,
      highlight: false,
      steps: vec![PlanStepInput { title: "热身".into(), done: false }],
      member_ids: Vec::new(),
      tag_ids: Vec::new(),
      place_ids: Vec::new(),
    }
  }

  #[test]
  fn create_complete_and_lock() {
    let (_dir, db) = setup();
    let mut empty = sample();
    empty.title = "  ".into();
    assert!(matches!(create(&db, empty), Err(PlanError::TitleEmpty)));

    let item = create(&db, sample()).expect("create");
    assert_eq!(item.steps.len(), 1);

    let done = complete(&db, &item.id, "到了公园").expect("complete");
    assert_eq!(done.status, STATUS_DONE);
    assert_eq!(done.result, "到了公园");
    assert!(done.completed_at.is_some());

    db.lock().expect("lock");
    assert!(matches!(list(&db), Err(PlanError::Locked)));
  }
}
