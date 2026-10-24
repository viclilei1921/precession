//! 内存队列。载荷里有文件密码，不能入库。

use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use zeroize::Zeroize;

use super::constants::{STATUS_CANCELED, STATUS_COMPLETED, STATUS_FAILED, STATUS_PENDING, STATUS_PROCESSING};
use super::dto::{Task, TaskDraft};
use super::error::TaskError;
#[cfg(test)]
use super::service;
use super::service::{Prepared, Spec};
#[cfg(test)]
use crate::db::state::DbState;

/// 队列里的一条任务
///
/// 载荷里有文件密码，随 `Drop` 清零，不能入库。
///
/// # 字段
/// - `id`: 任务 ID
/// - `kind`: 任务类型，`encryptFile`、`decryptFile`、`importUpload`、`encryptUpload`、`decryptUpload`、`convertVideo`、`cutVideo`、`mergeVideo`、`appendVideo`、`convertAvif`、`convertJxl`
/// - `spec`: 任务规格。工人取走后为空
/// - `password`: 任务密码，丢弃时清零
/// - `status`: 任务状态
/// - `progress`: 任务进度，0.0 到 1.0
/// - `message`: 任务消息。任务处理中时才有
/// - `cancel`: 取消标志
/// - `upload_id`: 上传文件 ID。上传文件任务时才有
/// - `target`: 目标路径，用来拒绝同一目标同时跑两条
/// - `title`: 任务标题
/// - `output`: 输出文件
/// - `draft`: 任务草稿。任务取消时才有
///
/// # 示例
/// 见 [`TaskQueue::push`]
struct Job {
  /// 任务 ID
  id: String,
  /// 任务类型: encryptFile, decryptFile, importUpload, encryptUpload, decryptUpload, convertVideo, cutVideo, mergeVideo, appendVideo, convertAvif, convertJxl
  kind: String,
  /// 任务规格。工人取走后为空
  spec: Option<Spec>,
  /// 任务密码，丢弃时清零
  password: String,
  /// 任务状态: pending, processing, completed, failed, canceled
  status: Status,
  /// 任务进度: 0.0 - 1.0
  progress: f64,
  /// 任务消息。任务处理中时才有。
  message: String,
  /// 取消标志
  cancel: Arc<AtomicBool>,
  /// 上传文件 ID。上传文件任务时才有。
  upload_id: Option<String>,
  /// 目标路径，用来拒绝同一目标同时跑两条
  target: String,
  /// 任务标题
  title: String,
  /// 输出文件
  output: String,
  /// 任务草稿。任务取消时才有。
  draft: Option<TaskDraft>,
}

impl Drop for Job {
  /// 清零任务密码
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// drop(job);
  /// ```
  fn drop(&mut self) {
    self.password.zeroize();
  }
}

/// 队列内部的任务状态
///
/// # 变体
/// - `Pending`: 待处理
/// - `Processing`: 处理中
/// - `Completed`: 完成
/// - `Failed`: 失败
/// - `Canceled`: 取消
#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
  /// 待处理
  Pending,
  /// 处理中
  Processing,
  /// 完成
  Completed,
  /// 失败
  Failed,
  /// 取消
  Canceled,
}

/// 工人从队列取出的一张票
///
/// 密码随 `Drop` 清零。
///
/// # 字段
/// - `id`: 任务 ID
/// - `spec`: 任务规格
/// - `password`: 任务密码
/// - `cancel`: 取消标志
///
/// # 示例
/// 见 [`TaskQueue::wait_ticket`]
pub(super) struct Ticket {
  /// 任务 ID
  pub id: String,
  /// 任务规格
  pub spec: Spec,
  /// 任务密码
  pub password: String,
  /// 取消标志
  pub cancel: Arc<AtomicBool>,
}

impl Drop for Ticket {
  /// 清零任务票里的密码
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// drop(ticket);
  /// ```
  fn drop(&mut self) {
    self.password.zeroize();
  }
}

/// 任务队列
///
/// 内存队列。载荷里有文件密码，不能入库。锁中毒时操作返回 `TaskError::Internal`。
///
/// # 字段
/// - `jobs`: 任务列表
/// - `wake`: 有新任务或状态变化时唤醒工人
///
/// # 示例
/// 见 [`TaskQueue::default`]
pub struct TaskQueue {
  /// 任务列表
  jobs: Mutex<Vec<Job>>,
  /// 有新任务或状态变化时唤醒工人
  wake: Condvar,
}

impl Default for TaskQueue {
  /// 创建空的任务队列
  ///
  /// # 返回值
  /// 没有任务的 `TaskQueue`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let queue = TaskQueue::default();
  /// assert!(queue.list().unwrap().is_empty());
  /// ```
  fn default() -> Self {
    Self { jobs: Mutex::new(Vec::new()), wake: Condvar::new() }
  }
}

/// 取消一条任务的结果
///
/// # 字段
/// - `task`: 取消后的任务视图
/// - `kill_child`: 是否杀死子进程
///
/// # 示例
/// 见 [`TaskQueue::cancel`]
pub(super) struct CancelOutcome {
  /// 取消后的任务视图
  pub task: Task,
  /// 是否杀死子进程
  pub kill_child: bool,
}

impl TaskQueue {
  /// 锁住任务列表
  ///
  /// # 返回值
  /// 成功时返回任务列表的互斥锁
  ///
  /// # 错误
  /// 锁中毒时返回 `TaskError::Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let jobs = queue.lock()?;
  /// ```
  fn lock(&self) -> Result<std::sync::MutexGuard<'_, Vec<Job>>, TaskError> {
    self.jobs.lock().map_err(|_| TaskError::Internal)
  }

  /// 把准备好的任务放进队列并唤醒工人
  ///
  /// 同一目标已有待处理或处理中的任务时拒绝。锁定失败或目标忙时清零密码。
  ///
  /// # 参数
  /// - `prepared`: 已校验的任务
  ///
  /// # 返回值
  /// 成功时返回刚入队的任务视图，状态为待处理，进度为 0
  ///
  /// # 错误
  /// 目标忙返回 `TaskError::Busy`，锁定失败返回 `TaskError::Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let task = queue.push(prepared)?;
  /// ```
  pub(super) fn push(&self, prepared: Prepared) -> Result<Task, TaskError> {
    let Prepared { kind, spec, mut password, upload_id, target, title, output, draft } = prepared;
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
      upload_id,
      target,
      title,
      output,
      draft,
    };
    let view = view(&job);
    jobs.push(job);
    self.wake.notify_one();
    Ok(view)
  }

  /// 列出队列里的全部任务
  ///
  /// # 返回值
  /// 成功时返回任务视图
  ///
  /// # 错误
  /// 锁定失败时返回 `TaskError::Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let tasks = queue.list()?;
  /// ```
  pub(super) fn list(&self) -> Result<Vec<Task>, TaskError> {
    Ok(self.lock()?.iter().map(view).collect())
  }

  /// 按状态取消一条任务
  ///
  /// 待处理：置取消标志、清零密码、状态改为已取消、消息写成「已取消」，不要求杀子进程。处理中：只置取消标志，并要求杀子进程。已完成、失败或已取消：不改状态，也不杀子进程。
  ///
  /// # 参数
  /// - `id`: 任务 ID
  ///
  /// # 返回值
  /// 成功时返回任务视图，以及是否要杀掉当前子进程
  ///
  /// # 错误
  /// 找不到任务时返回 `TaskError::NotFound`，锁定失败时返回 `TaskError::Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let outcome = queue.cancel(id)?;
  /// ```
  pub(super) fn cancel(&self, id: &str) -> Result<CancelOutcome, TaskError> {
    let mut jobs = self.lock()?;
    let Some(job) = jobs.iter_mut().find(|job| job.id == id) else {
      return Err(TaskError::NotFound);
    };
    let kill_child = match job.status {
      Status::Pending => {
        job.cancel.store(true, Ordering::Relaxed);
        job.password.zeroize();
        job.status = Status::Canceled;
        job.message = "已取消".to_string();
        false
      }
      Status::Processing => {
        job.cancel.store(true, Ordering::Relaxed);
        true
      }
      Status::Completed | Status::Failed | Status::Canceled => false,
    };
    Ok(CancelOutcome { task: view(job), kill_child })
  }

  /// 从队列拿掉一条
  ///
  /// 正在处理的不删，避免工人还拿着这条的载荷。
  ///
  /// # 参数
  /// - `id`: 任务 ID
  ///
  /// # 返回值
  /// 成功时无额外值
  ///
  /// # 错误
  /// 找不到任务时返回 `TaskError::NotFound`，正在处理时返回 `TaskError::Running`，锁定失败时返回 `TaskError::Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// queue.remove(id)?;
  /// ```
  pub(super) fn remove(&self, id: &str) -> Result<(), TaskError> {
    let mut jobs = self.lock()?;
    let Some(index) = jobs.iter().position(|job| job.id == id) else {
      return Err(TaskError::NotFound);
    };
    if jobs[index].status == Status::Processing {
      return Err(TaskError::Running);
    }
    jobs.remove(index);
    Ok(())
  }

  /// 等到一条待处理任务，标成处理中并取出任务票
  ///
  /// 没有待处理任务时睡在条件变量上。取出后消息写成「准备开始...」。
  ///
  /// # 返回值
  /// 成功时返回任务票
  ///
  /// # 错误
  /// 锁定失败或等待时锁中毒返回 `TaskError::Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let ticket = queue.wait_ticket()?;
  /// ```
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

  /// 有待处理任务就取出任务票，没有就返回空
  ///
  /// 取到时把状态改成处理中，消息写成「准备开始...」。
  ///
  /// # 返回值
  /// 成功且有待处理任务时返回任务票，否则返回 `None`
  ///
  /// # 错误
  /// 锁定失败时返回 `TaskError::Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let ticket = queue.try_ticket()?;
  /// ```
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

  /// 给处理中的任务写下进度和消息
  ///
  /// # 参数
  /// - `id`: 任务 ID
  /// - `progress`: 进度
  /// - `message`: 说明
  ///
  /// # 返回值
  /// 任务存在且正在处理时返回更新后的视图，否则返回 `None`
  ///
  /// # 错误
  /// 锁定失败时返回 `TaskError::Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let task = queue.note(id, progress, message)?;
  /// ```
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

  /// 按执行结果收尾，并清零密码
  ///
  /// 成功时若带上传文件 ID 就写上，状态改为完成、进度 100、消息「完成」。`Canceled` 改为已取消、消息「已取消」。其他错误改为失败，消息用错误的显示文本。
  ///
  /// # 参数
  /// - `id`: 任务 ID
  /// - `result`: 执行结果。`Ok` 里是可选的上传文件 ID
  ///
  /// # 返回值
  /// 成功时返回收尾后的任务视图
  ///
  /// # 错误
  /// 找不到任务时返回 `TaskError::NotFound`，锁定失败时返回 `TaskError::Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let task = queue.finish(id, &result)?;
  /// ```
  pub(super) fn finish(&self, id: &str, result: &Result<Option<String>, TaskError>) -> Result<Task, TaskError> {
    let mut jobs = self.lock()?;
    let Some(job) = jobs.iter_mut().find(|job| job.id == id) else {
      return Err(TaskError::NotFound);
    };
    job.password.zeroize();
    match result {
      Ok(upload_id) => {
        if let Some(upload_id) = upload_id {
          job.upload_id = Some(upload_id.clone());
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

  /// 同步跑掉队列头上的一个任务
  ///
  /// 执行过程中记下出现过的最大进度，成功时要求它等于 100。
  ///
  /// # 参数
  /// - `db`: 数据库状态
  ///
  /// # 返回值
  /// 成功时返回收尾后的任务
  ///
  /// # 错误
  /// 没有待处理任务时返回 `TaskError::NotFound`。取票、执行或收尾失败时返回对应的 `TaskError`
  ///
  /// # Panics
  /// 执行结果为 `Ok` 且记录到的最大进度不是 `100.0` 时，`assert_eq!` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// let task = queue.drive(&db)?;
  /// ```
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

/// 从任务上拿走规格、密码和取消标志，组成任务票
///
/// # 参数
/// - `job`: 状态已经改成处理中的任务
///
/// # 返回值
/// 任务票。任务上的密码被取走，规格变成 `None`
///
/// # Panics
/// `job.spec` 已经是 `None` 时，`expect` 会 panic
///
/// # 示例
/// ```ignore
/// let ticket = take_ticket(job);
/// ```
fn take_ticket(job: &mut Job) -> Ticket {
  Ticket {
    id: job.id.clone(),
    spec: job.spec.take().expect("task spec"),
    password: mem::take(&mut job.password),
    cancel: Arc::clone(&job.cancel),
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::super::constants::KIND_ENCRYPT_FILE;
  use super::super::service::{Prepared, Spec};
  use super::{Status, TaskError, TaskQueue};

  fn prepared(target: &str) -> Prepared {
    Prepared {
      kind: KIND_ENCRYPT_FILE.into(),
      spec: Spec::File { input: PathBuf::from("in"), output: PathBuf::from(target), encrypt: true },
      password: "secret".into(),
      upload_id: None,
      target: target.into(),
      title: "in".into(),
      output: target.into(),
      draft: None,
    }
  }

  #[test]
  fn remove_drops_pending() {
    let queue = TaskQueue::default();
    let pending = queue.push(prepared("a")).expect("push");
    queue.remove(&pending.id).expect("remove");
    assert!(queue.list().expect("list").is_empty());
  }

  #[test]
  fn remove_rejects_processing() {
    let queue = TaskQueue::default();
    let pending = queue.push(prepared("a")).expect("push");
    let _ticket = queue.try_ticket().expect("ticket").expect("pending");
    assert!(matches!(queue.remove(&pending.id), Err(TaskError::Running)));
    let jobs = queue.lock().expect("lock");
    assert_eq!(jobs.len(), 1);
    assert!(matches!(jobs[0].status, Status::Processing));
  }
}

/// 把队列里的任务收成对外的 `Task`
///
/// 状态写成 pending、processing、completed、failed 或 canceled。密码不出现在结果里。
///
/// # 参数
/// - `job`: 队列中的任务
///
/// # 返回值
/// 对外任务视图
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let task = view(job);
/// ```
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
    upload_id: job.upload_id.clone(),
    title: job.title.clone(),
    output: job.output.clone(),
    draft: job.draft.clone(),
  }
}
