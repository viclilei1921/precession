//! 校验入队参数，并按步骤跑完一条任务。

use std::fs;
use std::path::{Path, PathBuf};
#[cfg(desktop)]
use std::sync::Arc;
use std::sync::atomic::Ordering;

use tauri::AppHandle;
use zeroize::Zeroize;

use super::constants::{
  KIND_APPEND_VIDEO, KIND_CONVERT_AVIF, KIND_CONVERT_JXL, KIND_CONVERT_VIDEO, KIND_CUT_VIDEO, KIND_DECRYPT_FILE,
  KIND_DECRYPT_MEDIA, KIND_ENCRYPT_FILE, KIND_ENCRYPT_MEDIA, KIND_IMPORT_MEDIA, KIND_MERGE_VIDEO, PROGRESS_STEP,
};
use super::dto::{Task, TaskInput};
use super::error::TaskError;
#[cfg(desktop)]
use super::job::JobCtx;
use super::queue::{TaskQueue, Ticket};
use super::step::{self, Step};
use crate::db::state::DbState;
use crate::ffmpeg::models::TimeSegment;
use crate::image::params::{AvifEncodeParams, JxlEncodeParams};
use crate::media::{KIND_IMAGE, KIND_VIDEO};
use crate::utils::id::new_uuid_v4;

pub(super) enum Spec {
  File { input: PathBuf, output: PathBuf, encrypt: bool },
  Import { source: PathBuf, owner: String, owner_id: String, media_kind: String, encrypt: bool },
  Media { media_id: String, encrypt: bool },
  ConvertVideo { input: String, output: String, target_fps: Option<u32> },
  CutVideo { input: String, output: String, segments: Vec<TimeSegment> },
  MergeVideo { inputs: Vec<String>, output: String, draw_filename: bool },
  AppendVideo { base: String, inputs: Vec<String>, output: String, draw_filename: bool },
  ConvertAvif { input: String, output: String, params: AvifEncodeParams },
  ConvertJxl { input: String, output: String, params: JxlEncodeParams },
}

pub(super) struct Prepared {
  pub kind: String,
  pub spec: Spec,
  pub password: String,
  pub media_id: Option<String>,
  pub target: String,
}

pub(super) fn enqueue(db: &DbState, queue: &TaskQueue, input: TaskInput) -> Result<Task, TaskError> {
  queue.push(prepare(db, input)?)
}

pub(super) fn run_job(
  app: &AppHandle,
  db: &DbState,
  ticket: &Ticket,
  report: &mut dyn FnMut(f64, &str),
) -> Result<Option<String>, TaskError> {
  if is_sidecar(&ticket.spec) {
    #[cfg(desktop)]
    {
      return run_sidecar(app, ticket);
    }
    #[cfg(not(desktop))]
    {
      let _ = app;
      return Err(TaskError::DesktopOnly);
    }
  }
  run(db, ticket, report)
}

pub(super) fn run(
  db: &DbState,
  ticket: &Ticket,
  report: &mut dyn FnMut(f64, &str),
) -> Result<Option<String>, TaskError> {
  let built = build(db, ticket)?;
  let weights: Vec<f64> = built.steps.iter().map(Step::weight).collect();
  let total = weights.iter().copied().sum::<f64>().max(1.0);
  let mut done = 0.0;
  for (step, weight) in built.steps.iter().zip(weights) {
    let mut marked = 0u64;
    let result = step.run(&ticket.password, &ticket.cancel, &mut |current, file_total| {
      if ticket.cancel.load(Ordering::Relaxed) {
        return Err(TaskError::Canceled);
      }
      let due = current == file_total || current.saturating_sub(marked) >= PROGRESS_STEP;
      if due {
        marked = current;
        let frac = if file_total == 0 { 1.0 } else { current as f64 / file_total as f64 };
        let percent = (((done + frac * weight) / total) * 100.0).round().min(100.0);
        report(percent, "正在处理...");
      }
      Ok(())
    });
    if let Err(err) = result {
      cleanup(&built.temps);
      return Err(err);
    }
    done += weight;
  }
  if ticket.cancel.load(Ordering::Relaxed) {
    cleanup(&built.temps);
    return Err(TaskError::Canceled);
  }
  match commit(db, &built) {
    Ok(media_id) => {
      cleanup(&built.temps);
      report(100.0, "完成");
      Ok(media_id)
    }
    Err(err) => {
      cleanup(&built.temps);
      if let Some(dest) = &built.dest {
        let _ = fs::remove_file(dest);
      }
      Err(err)
    }
  }
}

#[cfg(desktop)]
fn run_sidecar(app: &AppHandle, ticket: &Ticket) -> Result<Option<String>, TaskError> {
  let job = JobCtx::new(app.clone(), ticket.id.clone(), Arc::clone(&ticket.cancel));
  tauri::async_runtime::block_on(dispatch_sidecar(app, &job, &ticket.spec)).map(|()| None).map_err(TaskError::from)
}

#[cfg(desktop)]
async fn dispatch_sidecar(app: &AppHandle, job: &JobCtx, spec: &Spec) -> Result<(), crate::sidecar::SidecarError> {
  match spec {
    Spec::ConvertVideo { input, output, target_fps } => {
      crate::ffmpeg::run_convert(app, job, input, output, *target_fps).await
    }
    Spec::CutVideo { input, output, segments } => {
      crate::ffmpeg::run_cut(app, job, input, output, segments.clone()).await
    }
    Spec::MergeVideo { inputs, output, draw_filename } => {
      crate::ffmpeg::run_merge_smart(app, job, inputs.clone(), output, *draw_filename).await
    }
    Spec::AppendVideo { base, inputs, output, draw_filename } => {
      crate::ffmpeg::run_append_smart(app, job, base, inputs.clone(), output, *draw_filename).await
    }
    Spec::ConvertAvif { input, output, params } => {
      crate::image::run_convert_avif(app, job, input, output, params.clone()).await
    }
    Spec::ConvertJxl { input, output, params } => {
      crate::image::run_convert_jxl(app, job, input, output, params.clone()).await
    }
    Spec::File { .. } | Spec::Import { .. } | Spec::Media { .. } => {
      Err(crate::sidecar::SidecarError::invalid("不是转码任务"))
    }
  }
}

fn is_sidecar(spec: &Spec) -> bool {
  matches!(
    spec,
    Spec::ConvertVideo { .. }
      | Spec::CutVideo { .. }
      | Spec::MergeVideo { .. }
      | Spec::AppendVideo { .. }
      | Spec::ConvertAvif { .. }
      | Spec::ConvertJxl { .. }
  )
}

struct Built {
  steps: Vec<Step>,
  temps: Vec<PathBuf>,
  dest: Option<PathBuf>,
  after: After,
}

enum After {
  None,
  Import { id: String, owner: String, owner_id: String, kind: String, rel_path: String, mime: String, encrypted: bool },
  Replace { media_id: String, source: PathBuf, temp: PathBuf, encrypted: bool },
}

fn build(db: &DbState, ticket: &Ticket) -> Result<Built, TaskError> {
  match &ticket.spec {
    Spec::File { input, output, encrypt } => Ok(Built {
      steps: vec![cipher_step(*encrypt, input.clone(), output.clone())],
      temps: Vec::new(),
      dest: None,
      after: After::None,
    }),
    Spec::Import { source, owner, owner_id, media_kind, encrypt } => {
      let id = new_uuid_v4();
      let dest = data_root(db).join("media").join(&id);
      let rel_path = format!("media/{id}");
      let mime = mime_of(source, media_kind);
      if *encrypt {
        let temp = step::side_path(&dest, "part");
        Ok(Built {
          steps: vec![
            Step::Copy { from: source.clone(), to: temp.clone() },
            cipher_step(true, temp.clone(), dest.clone()),
          ],
          temps: vec![temp],
          dest: Some(dest),
          after: After::Import {
            id,
            owner: owner.clone(),
            owner_id: owner_id.clone(),
            kind: media_kind.clone(),
            rel_path,
            mime,
            encrypted: true,
          },
        })
      } else {
        Ok(Built {
          steps: vec![Step::Copy { from: source.clone(), to: dest.clone() }],
          temps: Vec::new(),
          dest: Some(dest),
          after: After::Import {
            id,
            owner: owner.clone(),
            owner_id: owner_id.clone(),
            kind: media_kind.clone(),
            rel_path,
            mime,
            encrypted: false,
          },
        })
      }
    }
    Spec::Media { media_id, encrypt } => {
      let media = db.with_conn(|conn| crate::media::get(conn, media_id))?;
      if *encrypt && media.encrypted {
        return Err(TaskError::AlreadyEncrypted);
      }
      if !*encrypt && !media.encrypted {
        return Err(TaskError::NotEncrypted);
      }
      let source = resolve(&data_root(db), &media.rel_path);
      let temp = step::side_path(&source, "part");
      Ok(Built {
        steps: vec![cipher_step(*encrypt, source.clone(), temp.clone())],
        temps: vec![temp.clone()],
        dest: None,
        after: After::Replace { media_id: media_id.clone(), source, temp, encrypted: *encrypt },
      })
    }
    Spec::ConvertVideo { .. }
    | Spec::CutVideo { .. }
    | Spec::MergeVideo { .. }
    | Spec::AppendVideo { .. }
    | Spec::ConvertAvif { .. }
    | Spec::ConvertJxl { .. } => Err(TaskError::Failed("转码任务不能走加解密步骤".into())),
  }
}

fn cipher_step(encrypt: bool, from: PathBuf, to: PathBuf) -> Step {
  if encrypt { Step::Encrypt { from, to } } else { Step::Decrypt { from, to } }
}

fn commit(db: &DbState, built: &Built) -> Result<Option<String>, TaskError> {
  match &built.after {
    After::None => Ok(None),
    After::Import { id, owner, owner_id, kind, rel_path, mime, encrypted } => {
      db.with_conn(|_| Ok::<(), TaskError>(()))?;
      db.with_conn(|conn| {
        crate::media::insert_imported(
          conn,
          id,
          crate::media::dto::MediaInput {
            owner: owner.clone(),
            owner_id: owner_id.clone(),
            kind: kind.clone(),
            rel_path: rel_path.clone(),
            mime: mime.clone(),
            sort: 0,
            locked: false,
          },
          *encrypted,
        )
        .map(|_| ())
      })?;
      Ok(Some(id.clone()))
    }
    After::Replace { media_id, source, temp, encrypted } => {
      if let Err(err) = db.with_conn(|_| Ok::<(), TaskError>(())) {
        let _ = fs::remove_file(temp);
        return Err(err);
      }
      let backup = match step::replace_with_backup(temp, source) {
        Ok(path) => path,
        Err(err) => {
          let _ = fs::remove_file(temp);
          return Err(err);
        }
      };
      if let Err(err) = db.with_conn(|conn| crate::media::set_encrypted(conn, media_id, *encrypted)) {
        step::restore(&backup, source);
        return Err(err.into());
      }
      let _ = fs::remove_file(&backup);
      Ok(Some(media_id.clone()))
    }
  }
}

fn cleanup(temps: &[PathBuf]) {
  for path in temps {
    let _ = fs::remove_file(path);
  }
}

fn prepare(db: &DbState, input: TaskInput) -> Result<Prepared, TaskError> {
  match input {
    TaskInput::EncryptFile { input, output, password } => {
      prepare_file(KIND_ENCRYPT_FILE, input, output, password, true)
    }
    TaskInput::DecryptFile { input, output, password } => {
      prepare_file(KIND_DECRYPT_FILE, input, output, password, false)
    }
    TaskInput::ImportMedia { source, owner, owner_id, media_kind, encrypt, password } => {
      prepare_import(db, source, owner, owner_id, media_kind, encrypt, password)
    }
    TaskInput::EncryptMedia { media_id, password } => prepare_media(db, KIND_ENCRYPT_MEDIA, media_id, password, true),
    TaskInput::DecryptMedia { media_id, password } => prepare_media(db, KIND_DECRYPT_MEDIA, media_id, password, false),
    TaskInput::ConvertVideo { input, output, target_fps } => prepare_convert_video(input, output, target_fps),
    TaskInput::CutVideo { input, output, segments } => prepare_cut_video(input, output, segments),
    TaskInput::MergeVideo { inputs, output, draw_filename } => prepare_merge_video(inputs, output, draw_filename),
    TaskInput::AppendVideo { base, inputs, output, draw_filename } => {
      prepare_append_video(base, inputs, output, draw_filename)
    }
    TaskInput::ConvertAvif { input, output, params } => prepare_convert_avif(input, output, params),
    TaskInput::ConvertJxl { input, output, params } => prepare_convert_jxl(input, output, params),
  }
}

fn prepare_convert_video(input: String, output: String, target_fps: Option<u32>) -> Result<Prepared, TaskError> {
  #[cfg(not(desktop))]
  {
    let _ = (input, output, target_fps);
    return Err(TaskError::DesktopOnly);
  }
  #[cfg(desktop)]
  {
    let input = require_file(input)?;
    let output = require_output(&input, output)?;
    Ok(sidecar_job(
      KIND_CONVERT_VIDEO,
      output.clone(),
      Spec::ConvertVideo { input, output, target_fps },
    ))
  }
}

fn prepare_cut_video(input: String, output: String, segments: Vec<TimeSegment>) -> Result<Prepared, TaskError> {
  #[cfg(not(desktop))]
  {
    let _ = (input, output, segments);
    return Err(TaskError::DesktopOnly);
  }
  #[cfg(desktop)]
  {
    if segments.is_empty() {
      return Err(TaskError::Failed("时间片段不能为空".into()));
    }
    let input = require_file(input)?;
    let output = require_output(&input, output)?;
    Ok(sidecar_job(
      KIND_CUT_VIDEO,
      output.clone(),
      Spec::CutVideo { input, output, segments },
    ))
  }
}

fn prepare_merge_video(inputs: Vec<String>, output: String, draw_filename: bool) -> Result<Prepared, TaskError> {
  #[cfg(not(desktop))]
  {
    let _ = (inputs, output, draw_filename);
    return Err(TaskError::DesktopOnly);
  }
  #[cfg(desktop)]
  {
    let inputs = require_files(inputs)?;
    let output = require_output_many(&inputs, output)?;
    Ok(sidecar_job(
      KIND_MERGE_VIDEO,
      output.clone(),
      Spec::MergeVideo { inputs, output, draw_filename },
    ))
  }
}

fn prepare_append_video(
  base: String,
  inputs: Vec<String>,
  output: String,
  draw_filename: bool,
) -> Result<Prepared, TaskError> {
  #[cfg(not(desktop))]
  {
    let _ = (base, inputs, output, draw_filename);
    return Err(TaskError::DesktopOnly);
  }
  #[cfg(desktop)]
  {
    let base = require_file(base)?;
    let inputs = require_files(inputs)?;
    let mut all = Vec::with_capacity(inputs.len() + 1);
    all.push(base.clone());
    all.extend(inputs.iter().cloned());
    let output = require_output_many(&all, output)?;
    Ok(sidecar_job(
      KIND_APPEND_VIDEO,
      output.clone(),
      Spec::AppendVideo { base, inputs, output, draw_filename },
    ))
  }
}

fn prepare_convert_avif(input: String, output: String, params: AvifEncodeParams) -> Result<Prepared, TaskError> {
  #[cfg(not(desktop))]
  {
    let _ = (input, output, params);
    return Err(TaskError::DesktopOnly);
  }
  #[cfg(desktop)]
  {
    let input = require_file(input)?;
    let output = require_output(&input, output)?;
    Ok(sidecar_job(
      KIND_CONVERT_AVIF,
      output.clone(),
      Spec::ConvertAvif { input, output, params },
    ))
  }
}

fn prepare_convert_jxl(input: String, output: String, params: JxlEncodeParams) -> Result<Prepared, TaskError> {
  #[cfg(not(desktop))]
  {
    let _ = (input, output, params);
    return Err(TaskError::DesktopOnly);
  }
  #[cfg(desktop)]
  {
    let input = require_file(input)?;
    let output = require_output(&input, output)?;
    Ok(sidecar_job(
      KIND_CONVERT_JXL,
      output.clone(),
      Spec::ConvertJxl { input, output, params },
    ))
  }
}

fn sidecar_job(kind: &str, target: String, spec: Spec) -> Prepared {
  Prepared { kind: kind.to_string(), target, spec, password: String::new(), media_id: None }
}

fn require_file(path: String) -> Result<String, TaskError> {
  let path = path.trim().to_string();
  if path.is_empty() {
    return Err(TaskError::PathEmpty);
  }
  if !Path::new(&path).is_file() {
    return Err(TaskError::Missing);
  }
  Ok(path)
}

fn require_files(paths: Vec<String>) -> Result<Vec<String>, TaskError> {
  if paths.is_empty() {
    return Err(TaskError::PathEmpty);
  }
  paths.into_iter().map(require_file).collect()
}

fn require_output(input: &str, output: String) -> Result<String, TaskError> {
  let output = output.trim().to_string();
  if output.is_empty() {
    return Err(TaskError::PathEmpty);
  }
  if input == output {
    return Err(TaskError::SamePath);
  }
  Ok(output)
}

fn require_output_many(inputs: &[String], output: String) -> Result<String, TaskError> {
  let output = output.trim().to_string();
  if output.is_empty() {
    return Err(TaskError::PathEmpty);
  }
  if inputs.iter().any(|input| input == &output) {
    return Err(TaskError::SamePath);
  }
  Ok(output)
}

fn prepare_file(
  kind: &str,
  input: String,
  output: String,
  password: String,
  encrypt: bool,
) -> Result<Prepared, TaskError> {
  let reject = |mut password: String, err: TaskError| {
    password.zeroize();
    err
  };
  if password.is_empty() {
    return Err(reject(password, TaskError::PasswordEmpty));
  }
  let input = input.trim().to_string();
  let output = output.trim().to_string();
  if input.is_empty() || output.is_empty() {
    return Err(reject(password, TaskError::PathEmpty));
  }
  let input = PathBuf::from(&input);
  let output = PathBuf::from(&output);
  if input == output {
    return Err(reject(password, TaskError::SamePath));
  }
  if !input.is_file() {
    return Err(reject(password, TaskError::Missing));
  }
  Ok(Prepared {
    kind: kind.to_string(),
    target: output.display().to_string(),
    spec: Spec::File { input, output, encrypt },
    password,
    media_id: None,
  })
}

fn prepare_import(
  db: &DbState,
  source: String,
  owner: String,
  owner_id: String,
  media_kind: String,
  encrypt: bool,
  mut password: String,
) -> Result<Prepared, TaskError> {
  let reject = |mut password: String, err: TaskError| {
    password.zeroize();
    err
  };
  if encrypt && password.is_empty() {
    return Err(reject(password, TaskError::PasswordEmpty));
  }
  let source = source.trim().to_string();
  let owner = owner.trim().to_string();
  let owner_id = owner_id.trim().to_string();
  let media_kind = media_kind.trim().to_string();
  if source.is_empty() || owner.is_empty() || owner_id.is_empty() {
    return Err(reject(password, TaskError::PathEmpty));
  }
  if media_kind != KIND_IMAGE && media_kind != KIND_VIDEO {
    return Err(reject(password, TaskError::KindInvalid));
  }
  let source_path = PathBuf::from(&source);
  if !source_path.is_file() {
    return Err(reject(password, TaskError::Missing));
  }
  let alive = match db.with_conn(|conn| {
    let Some(parsed) = crate::owner::Owner::parse(&owner) else {
      return Err(TaskError::KindInvalid);
    };
    crate::owner::exists(conn, parsed, &owner_id).map_err(|_| TaskError::Internal)
  }) {
    Ok(alive) => alive,
    Err(err) => return Err(reject(password, err)),
  };
  if !alive {
    return Err(reject(password, TaskError::ReferencedMissing));
  }
  if !encrypt {
    password.zeroize();
  }
  Ok(Prepared {
    kind: KIND_IMPORT_MEDIA.to_string(),
    target: source,
    spec: Spec::Import { source: source_path, owner, owner_id, media_kind, encrypt },
    password,
    media_id: None,
  })
}

fn prepare_media(
  db: &DbState,
  kind: &str,
  media_id: String,
  password: String,
  encrypt: bool,
) -> Result<Prepared, TaskError> {
  let reject = |mut password: String, err: TaskError| {
    password.zeroize();
    err
  };
  if password.is_empty() {
    return Err(reject(password, TaskError::PasswordEmpty));
  }
  let media_id = media_id.trim().to_string();
  if media_id.is_empty() {
    return Err(reject(password, TaskError::PathEmpty));
  }
  let media = match db.with_conn(|conn| crate::media::get(conn, &media_id)) {
    Ok(media) => media,
    Err(err) => return Err(reject(password, err.into())),
  };
  if encrypt && media.encrypted {
    return Err(reject(password, TaskError::AlreadyEncrypted));
  }
  if !encrypt && !media.encrypted {
    return Err(reject(password, TaskError::NotEncrypted));
  }
  Ok(Prepared {
    kind: kind.to_string(),
    target: media_id.clone(),
    spec: Spec::Media { media_id: media_id.clone(), encrypt },
    password,
    media_id: Some(media_id),
  })
}

fn data_root(db: &DbState) -> PathBuf {
  db.dir.parent().map(Path::to_path_buf).unwrap_or_else(|| db.dir.clone())
}

fn resolve(root: &Path, rel: &str) -> PathBuf {
  let path = PathBuf::from(rel);
  if path.is_absolute() { path } else { root.join(path) }
}

fn mime_of(path: &Path, kind: &str) -> String {
  let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_ascii_lowercase();
  match ext.as_str() {
    "jpg" | "jpeg" => "image/jpeg".to_string(),
    "png" => "image/png".to_string(),
    "webp" => "image/webp".to_string(),
    "gif" => "image/gif".to_string(),
    "mp4" => "video/mp4".to_string(),
    "webm" => "video/webm".to_string(),
    "mov" => "video/quicktime".to_string(),
    _ if kind == KIND_IMAGE => "image/jpeg".to_string(),
    _ if kind == KIND_VIDEO => "video/mp4".to_string(),
    _ => String::new(),
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::sync::atomic::Ordering;

  use rusqlite::params;

  use super::super::constants::{KIND_IMPORT_MEDIA, STATUS_CANCELED, STATUS_COMPLETED, STATUS_PENDING};
  use super::*;
  use crate::db::error::DbError;
  use crate::owner::Owner;
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

  fn seed_plan(db: &DbState) -> String {
    let plan_id = new_uuid_v4();
    db.with_conn(|conn| {
      conn
        .execute(
          "INSERT INTO plan (id, title, body, status, priority, result, locked, highlight, created_at, updated_at)
           VALUES (?1, '出门', '', 'inbox', 0, '', 0, 0, 1, 1)",
          params![plan_id],
        )
        .expect("plan");
      Ok::<(), DbError>(())
    })
    .expect("seed");
    plan_id
  }

  #[test]
  fn import_encrypts_into_data_media() {
    let (dir, db) = setup();
    let plan_id = seed_plan(&db);
    let source = dir.path().join("shot.jpg");
    fs::write(&source, b"plain-image-bytes").expect("source");
    let queue = TaskQueue::default();
    let pending = enqueue(
      &db,
      &queue,
      TaskInput::ImportMedia {
        source: source.display().to_string(),
        owner: Owner::Plan.as_str().into(),
        owner_id: plan_id.clone(),
        media_kind: KIND_IMAGE.into(),
        encrypt: true,
        password: "file-password".into(),
      },
    )
    .expect("enqueue");
    assert_eq!(pending.kind, KIND_IMPORT_MEDIA);
    assert_eq!(pending.status, STATUS_PENDING);
    let done = queue.drive(&db).expect("drive");
    assert_eq!(done.status, STATUS_COMPLETED);
    assert_eq!(done.progress, 100.0);
    let media_id = done.media_id.expect("media id");
    let media = db.with_conn(|conn| crate::media::get(conn, &media_id)).expect("get");
    assert!(media.encrypted);
    assert_eq!(media.rel_path, format!("media/{media_id}"));
    let stored = data_root(&db).join(&media.rel_path);
    let sealed = fs::read(&stored).expect("stored");
    assert_eq!(&sealed[..8], b"chacha20");

    assert!(matches!(
      enqueue(
        &db,
        &queue,
        TaskInput::EncryptMedia { media_id: media_id.clone(), password: "file-password".into() },
      ),
      Err(TaskError::AlreadyEncrypted)
    ));

    enqueue(
      &db,
      &queue,
      TaskInput::DecryptMedia { media_id, password: "file-password".into() },
    )
    .expect("decrypt");
    let opened = queue.drive(&db).expect("decrypt drive");
    assert_eq!(opened.status, STATUS_COMPLETED);
    assert_eq!(fs::read(stored).expect("plain"), b"plain-image-bytes");
    let media = db.with_conn(|conn| crate::media::get(conn, opened.media_id.as_deref().unwrap())).expect("get");
    assert!(!media.encrypted);
  }

  #[test]
  fn empty_password_is_rejected_and_file_job_skips_media() {
    let (dir, db) = setup();
    let queue = TaskQueue::default();
    let input = dir.path().join("plain.bin");
    let output = dir.path().join("plain.enc");
    fs::write(&input, b"solo").expect("write");
    assert!(matches!(
      enqueue(
        &db,
        &queue,
        TaskInput::EncryptFile {
          input: input.display().to_string(),
          output: output.display().to_string(),
          password: String::new(),
        },
      ),
      Err(TaskError::PasswordEmpty)
    ));
    assert!(matches!(
      enqueue(
        &db,
        &queue,
        TaskInput::ImportMedia {
          source: input.display().to_string(),
          owner: Owner::Plan.as_str().into(),
          owner_id: "missing".into(),
          media_kind: KIND_IMAGE.into(),
          encrypt: true,
          password: String::new(),
        },
      ),
      Err(TaskError::PasswordEmpty)
    ));

    enqueue(
      &db,
      &queue,
      TaskInput::EncryptFile {
        input: input.display().to_string(),
        output: output.display().to_string(),
        password: "file-password".into(),
      },
    )
    .expect("file");
    let done = queue.drive(&db).expect("drive");
    assert_eq!(done.status, STATUS_COMPLETED);
    assert!(done.media_id.is_none());
    assert_eq!(&fs::read(&output).expect("enc")[..8], b"chacha20");
    let count = db
      .with_conn(|conn| {
        conn.query_row("SELECT COUNT(*) FROM media", [], |row| row.get::<_, i64>(0)).map_err(|_| TaskError::Internal)
      })
      .expect("count");
    assert_eq!(count, 0i64);
  }

  #[test]
  fn cancel_leaves_file_and_flag() {
    let (dir, db) = setup();
    let plan_id = seed_plan(&db);
    let path = dir.path().join("keep.jpg");
    let original = b"keep-these-bytes";
    fs::write(&path, original).expect("file");
    let media_id = new_uuid_v4();
    let media = db
      .with_conn(|conn| {
        crate::media::insert_imported(
          conn,
          &media_id,
          crate::media::dto::MediaInput {
            owner: Owner::Plan.as_str().into(),
            owner_id: plan_id.clone(),
            kind: KIND_IMAGE.into(),
            rel_path: path.display().to_string(),
            mime: "image/jpeg".into(),
            sort: 0,
            locked: false,
          },
          false,
        )
      })
      .expect("media");
    let queue = TaskQueue::default();
    let pending = enqueue(
      &db,
      &queue,
      TaskInput::EncryptMedia { media_id: media.id.clone(), password: "file-password".into() },
    )
    .expect("enqueue");
    let canceled = queue.cancel(&pending.id).expect("cancel");
    assert_eq!(canceled.task.status, STATUS_CANCELED);
    assert!(matches!(queue.drive(&db), Err(TaskError::NotFound)));
    assert_eq!(fs::read(&path).expect("still"), original);
    let row = db.with_conn(|conn| crate::media::get(conn, &media.id)).expect("get");
    assert!(!row.encrypted);

    let again = enqueue(
      &db,
      &queue,
      TaskInput::EncryptMedia { media_id: media.id.clone(), password: "file-password".into() },
    )
    .expect("again");
    let ticket = queue.try_ticket().expect("ticket").expect("pending");
    assert_eq!(ticket.id, again.id);
    ticket.cancel.store(true, Ordering::Relaxed);
    let result = run(&db, &ticket, &mut |_, _| {});
    assert!(matches!(result, Err(TaskError::Canceled)));
    let _ = queue.finish(&again.id, &result);
    assert_eq!(fs::read(&path).expect("original"), original);
    assert!(!step::side_path(&path, "part").exists());
    let row = db.with_conn(|conn| crate::media::get(conn, &media.id)).expect("get");
    assert!(!row.encrypted);
  }

  #[test]
  fn locked_archive_keeps_original() {
    let (dir, db) = setup();
    let plan_id = seed_plan(&db);
    let path = dir.path().join("still.jpg");
    fs::write(&path, b"still-plain").expect("file");
    let media_id = new_uuid_v4();
    let media = db
      .with_conn(|conn| {
        crate::media::insert_imported(
          conn,
          &media_id,
          crate::media::dto::MediaInput {
            owner: Owner::Plan.as_str().into(),
            owner_id: plan_id.clone(),
            kind: KIND_IMAGE.into(),
            rel_path: path.display().to_string(),
            mime: "image/jpeg".into(),
            sort: 0,
            locked: false,
          },
          false,
        )
      })
      .expect("media");
    let queue = TaskQueue::default();
    enqueue(
      &db,
      &queue,
      TaskInput::EncryptMedia { media_id: media.id.clone(), password: "file-password".into() },
    )
    .expect("enqueue");
    let ticket = queue.try_ticket().expect("ticket").expect("pending");
    let result = run(&db, &ticket, &mut |_, _| {
      db.lock().expect("lock");
    });
    assert!(matches!(result, Err(TaskError::Locked)));
    assert_eq!(fs::read(&path).expect("original"), b"still-plain");
    assert!(!step::side_path(&path, "part").exists());
    db.unlock("test-password-123").expect("unlock");
    let row = db.with_conn(|conn| crate::media::get(conn, &media.id)).expect("get");
    assert!(!row.encrypted);
  }
}
