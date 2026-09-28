//! 内存队列。载荷里有文件密码，不能入库。

use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use zeroize::Zeroize;

use super::constants::{STATUS_CANCELED, STATUS_COMPLETED, STATUS_FAILED, STATUS_PENDING, STATUS_PROCESSING};
use super::dto::Task;
use super::error::TaskError;
#[cfg(test)]
use super::service;
use super::service::{Prepared, Spec};
#[cfg(test)]
use crate::db::state::DbState;

struct Job {
  id: String,
  kind: String,
  spec: Option<Spec>,
  password: String,
  status: Status,
  progress: f64,
  message: String,
  cancel: Arc<AtomicBool>,
  media_id: Option<String>,
  target: String,
}

impl Drop for Job {
  fn drop(&mut self) {
    self.password.zeroize();
  }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
  Pending,
  Processing,
  Completed,
  Failed,
  Canceled,
}

pub(super) struct Ticket {
  pub id: String,
  pub spec: Spec,
  pub password: String,
  pub cancel: Arc<AtomicBool>,
}

impl Drop for Ticket {
  fn drop(&mut self) {
    self.password.zeroize();
  }
}

pub struct TaskQueue {
  jobs: Mutex<Vec<Job>>,
  wake: Condvar,
}

impl Default for TaskQueue {
  fn default() -> Self {
    Self { jobs: Mutex::new(Vec::new()), wake: Condvar::new() }
  }
}

impl TaskQueue {
  fn lock(&self) -> Result<std::sync::MutexGuard<'_, Vec<Job>>, TaskError> {
    self.jobs.lock().map_err(|_| TaskError::Internal)
  }

  pub(super) fn push(&self, prepared: Prepared) -> Result<Task, TaskError> {
    let Prepared { kind, spec, mut password, media_id, target } = prepared;
    let mut jobs = match self.lock() {
      Ok(jobs) => jobs,
      Err(err) => {
        password.zeroize();
        return Err(err);
      }
    };
    let busy =
      jobs.iter().any(|job| job.target == target && matches!(job.status, Status::Pending | Status::Processing));
    if busy {
      password.zeroize();
      return Err(TaskError::Busy);
    }
    let job = Job {
      id: crate::utils::id::new_uuid_v4(),
      kind,
      spec: Some(spec),
      password,
      status: Status::Pending,
      progress: 0.0,
      message: "准备开始...".to_string(),
      cancel: Arc::new(AtomicBool::new(false)),
      media_id,
      target,
    };
    let view = view(&job);
    jobs.push(job);
    self.wake.notify_one();
    Ok(view)
  }

  pub(super) fn list(&self) -> Result<Vec<Task>, TaskError> {
    Ok(self.lock()?.iter().map(view).collect())
  }

  pub(super) fn cancel(&self, id: &str) -> Result<Task, TaskError> {
    let mut jobs = self.lock()?;
    let Some(job) = jobs.iter_mut().find(|job| job.id == id) else {
      return Err(TaskError::NotFound);
    };
    match job.status {
      Status::Pending => {
        job.cancel.store(true, Ordering::Relaxed);
        job.password.zeroize();
        job.status = Status::Canceled;
        job.message = "已取消".to_string();
      }
      Status::Processing => {
        job.cancel.store(true, Ordering::Relaxed);
      }
      Status::Completed | Status::Failed | Status::Canceled => {}
    }
    Ok(view(job))
  }

  pub(super) fn wait_ticket(&self) -> Result<Ticket, TaskError> {
    let mut jobs = self.lock()?;
    loop {
      if let Some(job) = jobs.iter_mut().find(|job| job.status == Status::Pending) {
        job.status = Status::Processing;
        job.message = "准备开始...".to_string();
        return Ok(take_ticket(job));
      }
      jobs = self.wake.wait(jobs).map_err(|_| TaskError::Internal)?;
    }
  }

  #[cfg(test)]
  pub(super) fn try_ticket(&self) -> Result<Option<Ticket>, TaskError> {
    let mut jobs = self.lock()?;
    let Some(job) = jobs.iter_mut().find(|job| job.status == Status::Pending) else {
      return Ok(None);
    };
    job.status = Status::Processing;
    job.message = "准备开始...".to_string();
    Ok(Some(take_ticket(job)))
  }

  pub(super) fn note(&self, id: &str, progress: f64, message: &str) -> Result<Option<Task>, TaskError> {
    let mut jobs = self.lock()?;
    let Some(job) = jobs.iter_mut().find(|job| job.id == id) else {
      return Ok(None);
    };
    if job.status != Status::Processing {
      return Ok(None);
    }
    job.progress = progress;
    job.message = message.to_string();
    Ok(Some(view(job)))
  }

  pub(super) fn finish(&self, id: &str, result: &Result<Option<String>, TaskError>) -> Result<Task, TaskError> {
    let mut jobs = self.lock()?;
    let Some(job) = jobs.iter_mut().find(|job| job.id == id) else {
      return Err(TaskError::NotFound);
    };
    job.password.zeroize();
    match result {
      Ok(media_id) => {
        if let Some(media_id) = media_id {
          job.media_id = Some(media_id.clone());
        }
        job.status = Status::Completed;
        job.progress = 100.0;
        job.message = "完成".to_string();
      }
      Err(TaskError::Canceled) => {
        job.status = Status::Canceled;
        job.message = "已取消".to_string();
      }
      Err(err) => {
        job.status = Status::Failed;
        job.message = err.to_string();
      }
    }
    Ok(view(job))
  }

  /// 测试里同步跑掉队列头上的一个任务。
  #[cfg(test)]
  pub(super) fn drive(&self, db: &DbState) -> Result<Task, TaskError> {
    let Some(ticket) = self.try_ticket()? else {
      return Err(TaskError::NotFound);
    };
    let id = ticket.id.clone();
    let mut reached = 0.0f64;
    let result = service::run(db, &ticket, &mut |progress, message| {
      reached = reached.max(progress);
      let _ = self.note(&id, progress, message);
    });
    let task = self.finish(&id, &result)?;
    if result.is_ok() {
      assert_eq!(reached, 100.0);
    }
    Ok(task)
  }
}

fn take_ticket(job: &mut Job) -> Ticket {
  Ticket {
    id: job.id.clone(),
    spec: job.spec.take().expect("task spec"),
    password: mem::take(&mut job.password),
    cancel: Arc::clone(&job.cancel),
  }
}

fn view(job: &Job) -> Task {
  Task {
    id: job.id.clone(),
    kind: job.kind.clone(),
    status: match job.status {
      Status::Pending => STATUS_PENDING,
      Status::Processing => STATUS_PROCESSING,
      Status::Completed => STATUS_COMPLETED,
      Status::Failed => STATUS_FAILED,
      Status::Canceled => STATUS_CANCELED,
    }
    .to_string(),
    progress: job.progress,
    message: job.message.clone(),
    media_id: job.media_id.clone(),
  }
}
