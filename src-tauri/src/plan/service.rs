//! 计划。同一次 `with_conn` 里写 `record` 和计划明细。

use super::constants::{STATUS_DONE, STATUS_INBOX, STATUS_SCHEDULED};
use super::dto::{PlanItem, PlanWrite};
use super::error::PlanError;
use super::repository;
use crate::db::state::DbState;

const STATUSES: &[&str] = &[STATUS_INBOX, STATUS_SCHEDULED, STATUS_DONE];

pub fn list(db: &DbState) -> Result<Vec<PlanItem>, PlanError> {
  db.with_conn(repository::list)
}

pub fn get(db: &DbState, id: &str) -> Result<PlanItem, PlanError> {
  db.with_conn(|conn| repository::get(conn, id))
}

pub fn create(db: &DbState, input: PlanWrite) -> Result<PlanItem, PlanError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::create(conn, &input))
}

pub fn update(db: &DbState, id: &str, input: PlanWrite) -> Result<PlanItem, PlanError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::update(conn, id, &input))
}

pub fn complete(db: &DbState, id: &str, result: &str) -> Result<PlanItem, PlanError> {
  let result = result.trim();
  db.with_conn(|conn| repository::complete(conn, id, result))
}

pub fn delete(db: &DbState, id: &str) -> Result<(), PlanError> {
  db.with_conn(|conn| repository::delete(conn, id))
}

fn prepare(mut input: PlanWrite) -> Result<PlanWrite, PlanError> {
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
  use crate::plan::dto::PlanStepWrite;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::record::migrate, crate::plan::migrate]);
    db.create("test-password-123").expect("create db");
    (dir, db)
  }

  fn sample() -> PlanWrite {
    PlanWrite {
      occurred_at: 10,
      title: "晨跑".into(),
      body: String::new(),
      locked: false,
      highlight: false,
      status: STATUS_SCHEDULED.into(),
      priority: 1,
      due_at: Some(10),
      result: String::new(),
      steps: vec![PlanStepWrite { title: "热身".into(), done: false }],
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
    assert_eq!(list(&db).expect("list").len(), 1);

    let done = complete(&db, &item.id, "到了公园").expect("complete");
    assert_eq!(done.status, STATUS_DONE);
    assert_eq!(done.result, "到了公园");

    db.lock().expect("lock");
    assert!(matches!(list(&db), Err(PlanError::Locked)));
  }
}
