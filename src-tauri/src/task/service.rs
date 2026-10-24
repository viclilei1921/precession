//! 校验入队参数，并按步骤跑完一条任务。

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use tauri::AppHandle;
use zeroize::Zeroize;

use super::constants::{
  KIND_APPEND_VIDEO, KIND_CONVERT_AVIF, KIND_CONVERT_JXL, KIND_CONVERT_VIDEO, KIND_CUT_VIDEO, KIND_DECRYPT_FILE,
  KIND_DECRYPT_UPLOAD, KIND_ENCRYPT_FILE, KIND_ENCRYPT_UPLOAD, KIND_IMPORT_UPLOAD, KIND_MERGE_VIDEO, PROGRESS_STEP,
};
use super::dto::{Task, TaskDraft, TaskInput};
use super::error::TaskError;
use super::platform;
use super::queue::{TaskQueue, Ticket};
use super::step::{self, Step};
use crate::db::state::DbState;
use crate::ffmpeg::models::{TimeSegment, VideoEncode};
use crate::image::models::{AvifEncodeParams, JxlEncodeParams};
use crate::upload;
use crate::upload::{KIND_IMAGE, KIND_VIDEO, KINDS};
use crate::utils::id::new_uuid_v4;

/// 校验后的任务规格
///
/// 密码不在这里，跟 [`Prepared`] 分开放。
///
/// # 变体
/// - `File`: 加密或解密一个文件，载荷是输入、输出和是否加密
/// - `Import`: 导入上传文件
/// - `Upload`: 加密或解密已上传文件
/// - `ConvertVideo`: 转换视频
/// - `CutVideo`: 按片段裁剪视频
/// - `MergeVideo`: 合并多段视频
/// - `AppendVideo`: 把多段视频追加到基准片
/// - `ConvertAvif`: 转成 AVIF
/// - `ConvertJxl`: 转成 JPEG XL
pub(super) enum Spec {
  /// 加密或解密一个文件，载荷是输入、输出和是否加密
  File { input: PathBuf, output: PathBuf, encrypt: bool },
  /// 导入上传文件
  Import { source: PathBuf, owner: String, owner_id: String, upload_kind: String, encrypt: bool },
  /// 加密或解密已上传文件
  Upload { upload_id: String, encrypt: bool },
  /// 转换视频
  ConvertVideo { input: String, output: String, encode: VideoEncode },
  /// 按片段裁剪视频
  CutVideo { input: String, output: String, segments: Vec<TimeSegment>, encode: VideoEncode },
  /// 合并多段视频
  MergeVideo { inputs: Vec<String>, output: String, encode: VideoEncode },
  /// 把多段视频追加到基准片
  AppendVideo { base: String, inputs: Vec<String>, output: String, encode: VideoEncode },
  /// 转成 AVIF
  ConvertAvif { input: String, output: String, params: AvifEncodeParams },
  /// 转成 JPEG XL
  ConvertJxl { input: String, output: String, params: JxlEncodeParams },
}

/// 校验通过、尚未入队的任务
///
/// # 字段
/// - `kind`: 任务类型，`encryptFile`、`decryptFile`、`importUpload`、`encryptUpload`、`decryptUpload`、`convertVideo`、`cutVideo`、`mergeVideo`、`appendVideo`、`convertAvif`、`convertJxl`
/// - `spec`: 任务规格
/// - `password`: 任务密码
/// - `upload_id`: 上传文件 ID。上传文件任务时才有
/// - `target`: 目标路径
/// - `title`: 任务标题
/// - `output`: 输出文件
/// - `draft`: 任务草稿。任务取消时才有
///
/// # 示例
/// 见 [`enqueue`]
pub(super) struct Prepared {
  /// 任务类型: encryptFile, decryptFile, importUpload, encryptUpload, decryptUpload, convertVideo, cutVideo, mergeVideo, appendVideo, convertAvif, convertJxl
  pub kind: String,
  /// 任务规格
  pub spec: Spec,
  /// 任务密码
  pub password: String,
  /// 上传文件 ID。上传文件任务时才有。
  pub upload_id: Option<String>,
  /// 目标路径
  pub target: String,
  /// 任务标题
  pub title: String,
  /// 输出文件
  pub output: String,
  /// 任务草稿。任务取消时才有。
  pub draft: Option<TaskDraft>,
}

/// 校验入队参数后放进队列
///
/// # 参数
/// - `db`: 数据库状态
/// - `queue`: 任务队列
/// - `input`: 入队参数
///
/// # 返回值
/// 成功时返回已入队的任务
///
/// # 错误
/// `prepare` 或 `push` 失败时返回 `TaskError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let task = enqueue(db, queue, input)?;
/// ```
pub(super) fn enqueue(db: &DbState, queue: &TaskQueue, input: TaskInput) -> Result<Task, TaskError> {
  queue.push(prepare(db, input)?)
}

/// 按规格走侧车或加解密步骤
///
/// 转码、裁剪、合并、追加、AVIF、JXL 在桌面走侧车。这些规格在非桌面返回 `DesktopOnly`。其余规格走 `run`。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `db`: 数据库状态
/// - `ticket`: 任务票
/// - `report`: 进度回调。侧车路径不使用它
///
/// # 返回值
/// 成功时返回可选的上传文件 ID。侧车成功时是 `None`
///
/// # 错误
/// 非桌面的侧车规格返回 `TaskError::DesktopOnly`。其余失败返回 `TaskError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let upload_id = run_job(app, db, ticket, &mut report)?;
/// ```
pub(super) fn run_job(
  app: &AppHandle,
  db: &DbState,
  ticket: &Ticket,
  report: &mut dyn FnMut(f64, &str),
) -> Result<Option<String>, TaskError> {
  if is_sidecar(&ticket.spec) {
    return platform::run_sidecar(app, ticket);
  }
  run(db, ticket, report)
}

/// 按步骤跑完加解密，再提交数据库
///
/// 进度按步骤权重分摊。字节增量达到 `PROGRESS_STEP`，或已经读完该文件时才回调。任一步失败或取消时清掉临时文件。提交失败时再删掉 `dest`。
///
/// # 参数
/// - `db`: 数据库状态
/// - `ticket`: 任务票
/// - `report`: 进度和说明回调
///
/// # 返回值
/// 成功时返回可选的上传文件 ID，并回调 100 和「完成」
///
/// # 错误
/// 步骤失败、取消或提交失败时返回 `TaskError`。取消为 `TaskError::Canceled`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let upload_id = run(db, ticket, &mut report)?;
/// ```
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
    Ok(upload_id) => {
      cleanup(&built.temps);
      report(100.0, "完成");
      Ok(upload_id)
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

/// 判断规格是否走视频或图片侧车
///
/// # 参数
/// - `spec`: 任务规格
///
/// # 返回值
/// 转码、裁剪、合并、追加、AVIF、JXL 时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let side = is_sidecar(&spec);
/// ```
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

/// 展开后的步骤和提交动作
///
/// # 字段
/// - `steps`: 按顺序执行的步骤
/// - `temps`: 临时文件
/// - `dest`: 最终目标路径，可以没有
/// - `after`: 步骤跑完后的提交动作
///
/// # 示例
/// 见 [`build`]
struct Built {
  /// 按顺序执行的步骤
  steps: Vec<Step>,
  /// 临时文件
  temps: Vec<PathBuf>,
  /// 最终目标路径，可以没有
  dest: Option<PathBuf>,
  /// 步骤跑完后的提交动作
  after: After,
}

/// 步骤跑完后怎么提交
///
/// # 变体
/// - `None`: 不额外提交
/// - `Import`: 把临时文件改名到上传目录，再登记成上传文件
/// - `Replace`: 用旁路文件换上原上传文件
enum After {
  /// 不额外提交
  None,
  /// 把临时文件改名到上传目录，再登记成上传文件
  Import(Box<Imported>),
  /// 用旁路文件换上原上传文件
  Replace { upload_id: String, source: PathBuf, temp: PathBuf, encrypted: bool },
}

/// 导入跑完步骤后要提交的内容
///
/// # 字段
/// - `staged`: 临时目录里写好的文件
/// - `dest`: 上传目录里的最终位置
/// - `id`: 上传文件 ID
/// - `owner`: 归属类型
/// - `owner_id`: 归属 ID
/// - `kind`: 上传文件类型
/// - `rel_path`: `dest` 相对数据根目录的路径
/// - `mime`: MIME 类型
/// - `name`: 源文件名
/// - `size`: 源文件字节数
/// - `encrypted`: 是否已加密
///
/// # 示例
/// 见 [`build`]
struct Imported {
  /// 临时目录里写好的文件
  staged: PathBuf,
  /// 上传目录里的最终位置
  dest: PathBuf,
  /// 上传文件 ID
  id: String,
  /// 归属类型
  owner: String,
  /// 归属 ID
  owner_id: String,
  /// 上传文件类型
  kind: String,
  /// `dest` 相对数据根目录的路径
  rel_path: String,
  /// MIME 类型
  mime: String,
  /// 源文件名
  name: String,
  /// 源文件字节数
  size: i64,
  /// 是否已加密
  encrypted: bool,
}

/// 把任务票展开成步骤、临时文件和提交动作
///
/// 导入先写到上传临时目录，加密时先复制再加密，提交时才改名到 `upload/{kind}/{id}.{ext}`。上传文件加解密写到旁路文件，提交时再换上。转码类规格在这里直接拒绝。
///
/// # 参数
/// - `db`: 数据库状态
/// - `ticket`: 任务票
///
/// # 返回值
/// 成功时返回步骤和提交计划
///
/// # 错误
/// 上传文件已加密还要加密时返回 `AlreadyEncrypted`，未加密还要解密时返回 `NotEncrypted`。转码类返回 `Failed`，消息是「转码任务不能走加解密步骤」。读上传文件失败时返回对应的 `TaskError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let built = build(db, ticket)?;
/// ```
fn build(db: &DbState, ticket: &Ticket) -> Result<Built, TaskError> {
  match &ticket.spec {
    Spec::File { input, output, encrypt } => Ok(Built {
      steps: vec![cipher_step(*encrypt, input.clone(), output.clone())],
      temps: Vec::new(),
      dest: None,
      after: After::None,
    }),
    Spec::Import { source, owner, owner_id, upload_kind, encrypt } => {
      let id = new_uuid_v4();
      let rel_path = upload::rel_path(upload_kind, &id, upload::ext_of(source).as_deref());
      let dest = upload::resolve(db, &rel_path);
      let staged = upload::tmp_path(db, &format!("{id}.part"));
      let size = fs::metadata(source).map(|meta| meta.len() as i64).unwrap_or(0);
      let (steps, temps) = if *encrypt {
        let plain = upload::tmp_path(db, &format!("{id}.plain"));
        (
          vec![
            Step::Copy { from: source.clone(), to: plain.clone() },
            cipher_step(true, plain.clone(), staged.clone()),
          ],
          vec![plain, staged.clone()],
        )
      } else {
        (
          vec![Step::Copy { from: source.clone(), to: staged.clone() }],
          vec![staged.clone()],
        )
      };
      Ok(Built {
        steps,
        temps,
        dest: None,
        after: After::Import(Box::new(Imported {
          staged,
          dest,
          id,
          owner: owner.clone(),
          owner_id: owner_id.clone(),
          kind: upload_kind.clone(),
          rel_path,
          mime: mime_of(source, upload_kind),
          name: file_name(&source.display().to_string()),
          size,
          encrypted: *encrypt,
        })),
      })
    }
    Spec::Upload { upload_id, encrypt } => {
      let file = db.with_conn(|conn| crate::upload::get(conn, upload_id))?;
      if *encrypt && file.encrypted {
        return Err(TaskError::AlreadyEncrypted);
      }
      if !*encrypt && !file.encrypted {
        return Err(TaskError::NotEncrypted);
      }
      let source = upload::resolve(db, &file.rel_path);
      let temp = step::side_path(&source, "part");
      Ok(Built {
        steps: vec![cipher_step(*encrypt, source.clone(), temp.clone())],
        temps: vec![temp.clone()],
        dest: None,
        after: After::Replace { upload_id: upload_id.clone(), source, temp, encrypted: *encrypt },
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

/// 按是否加密选择加密或解密步骤
///
/// # 参数
/// - `encrypt`: 为真时加密
/// - `from`: 源路径
/// - `to`: 目标路径
///
/// # 返回值
/// `Step::Encrypt` 或 `Step::Decrypt`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let step = cipher_step(true, from, to);
/// ```
fn cipher_step(encrypt: bool, from: PathBuf, to: PathBuf) -> Step {
  if encrypt { Step::Encrypt { from, to } } else { Step::Decrypt { from, to } }
}

/// 按提交动作写入上传文件记录，或换上加解密后的文件
///
/// 导入先确认数据库可写，再在 [`upload::commit_guard`] 下把临时文件改名到上传目录并插入记录；插入失败时删掉改名后的文件。替换时先换文件再改加密标志；改标志失败会把备份恢复回去。换文件之前数据库不可写，则删掉临时文件。
///
/// # 参数
/// - `db`: 数据库状态
/// - `built`: 步骤跑完后的提交计划
///
/// # 返回值
/// 没有后续动作时返回 `None`，导入或替换成功时返回上传文件 ID
///
/// # 错误
/// 数据库锁定返回 `TaskError::Locked`，其他数据库失败返回 `TaskError::Internal`。导入建目录或改名失败返回 `TaskError::Internal`。换文件失败返回 `replace_with_backup` 的错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let upload_id = commit(db, &built)?;
/// ```
fn commit(db: &DbState, built: &Built) -> Result<Option<String>, TaskError> {
  match &built.after {
    After::None => Ok(None),
    After::Import(imported) => {
      db.with_conn(|_| Ok::<(), TaskError>(()))?;
      let _guard = upload::commit_guard();
      if let Some(parent) = imported.dest.parent() {
        fs::create_dir_all(parent).map_err(|err| io_fail("upload dir", err))?;
      }
      fs::rename(&imported.staged, &imported.dest).map_err(|err| io_fail("upload rename", err))?;
      let inserted = db.with_conn(|conn| {
        crate::upload::insert_imported(
          conn,
          &imported.id,
          crate::upload::dto::UploadInput {
            owner: imported.owner.clone(),
            owner_id: imported.owner_id.clone(),
            kind: imported.kind.clone(),
            rel_path: imported.rel_path.clone(),
            mime: imported.mime.clone(),
            name: imported.name.clone(),
            size: imported.size,
            sort: 0,
            locked: false,
          },
          imported.encrypted,
        )
        .map(|_| ())
      });
      if let Err(err) = inserted {
        let _ = fs::remove_file(&imported.dest);
        return Err(err.into());
      }
      Ok(Some(imported.id.clone()))
    }
    After::Replace { upload_id, source, temp, encrypted } => {
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
      if let Err(err) = db.with_conn(|conn| crate::upload::set_encrypted(conn, upload_id, *encrypted)) {
        step::restore(&backup, source);
        return Err(err.into());
      }
      let _ = fs::remove_file(&backup);
      Ok(Some(upload_id.clone()))
    }
  }
}

/// 删掉临时文件，删除失败时忽略
///
/// # 参数
/// - `temps`: 临时路径
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// cleanup(&temps);
/// ```
fn cleanup(temps: &[PathBuf]) {
  for path in temps {
    let _ = fs::remove_file(path);
  }
}

/// 按入队参数分发到对应的准备函数
///
/// # 参数
/// - `db`: 数据库状态
/// - `input`: 入队参数
///
/// # 返回值
/// 成功时返回可入队的任务
///
/// # 错误
/// 校验失败时返回对应的 `TaskError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let prepared = prepare(db, input)?;
/// ```
fn prepare(db: &DbState, input: TaskInput) -> Result<Prepared, TaskError> {
  match input {
    TaskInput::EncryptFile { input, output, password } => {
      prepare_file(KIND_ENCRYPT_FILE, input, output, password, true)
    }
    TaskInput::DecryptFile { input, output, password } => {
      prepare_file(KIND_DECRYPT_FILE, input, output, password, false)
    }
    TaskInput::ImportUpload { source, owner, owner_id, upload_kind, encrypt, password } => {
      prepare_import(db, source, owner, owner_id, upload_kind, encrypt, password)
    }
    TaskInput::EncryptUpload { upload_id, password } => {
      prepare_upload(db, KIND_ENCRYPT_UPLOAD, upload_id, password, true)
    }
    TaskInput::DecryptUpload { upload_id, password } => {
      prepare_upload(db, KIND_DECRYPT_UPLOAD, upload_id, password, false)
    }
    TaskInput::ConvertVideo { input, output, encode } => prepare_convert_video(input, output, encode),
    TaskInput::CutVideo { input, output, segments, encode } => prepare_cut_video(input, output, segments, encode),
    TaskInput::MergeVideo { inputs, output, encode } => prepare_merge_video(inputs, output, encode),
    TaskInput::AppendVideo { base, inputs, output, encode } => prepare_append_video(base, inputs, output, encode),
    TaskInput::ConvertAvif { input, output, params } => prepare_convert_avif(input, output, params),
    TaskInput::ConvertJxl { input, output, params } => prepare_convert_jxl(input, output, params),
  }
}

/// 准备视频转码任务
///
/// 非桌面直接返回 `DesktopOnly`。桌面先收紧编码参数，再检查输入文件和输出路径。
///
/// # 参数
/// - `input`: 源视频
/// - `output`: 输出路径
/// - `encode`: 编码配置
///
/// # 返回值
/// 成功时返回侧车任务，密码为空
///
/// # 错误
/// 非桌面返回 `TaskError::DesktopOnly`。编码参数不合法时返回 `TaskError::Failed`。路径为空返回 `PathEmpty`，源不是文件返回 `Missing`，输入输出相同返回 `SamePath`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let prepared = prepare_convert_video(input, output, encode)?;
/// ```
fn prepare_convert_video(input: String, output: String, mut encode: VideoEncode) -> Result<Prepared, TaskError> {
  platform::ensure_desktop()?;
  crate::ffmpeg::models::normalize_encode(&mut encode).map_err(TaskError::Failed)?;
  let input = require_file(input)?;
  let output = require_output(&input, output)?;
  Ok(sidecar_job(
    KIND_CONVERT_VIDEO,
    output.clone(),
    Spec::ConvertVideo { input, output, encode },
  ))
}

/// 准备视频裁剪任务
///
/// 非桌面直接返回 `DesktopOnly`。桌面要求片段非空，再收紧编码参数并检查路径。
///
/// # 参数
/// - `input`: 源视频
/// - `output`: 输出路径
/// - `segments`: 时间片段
/// - `encode`: 编码配置
///
/// # 返回值
/// 成功时返回侧车任务，密码为空
///
/// # 错误
/// 非桌面返回 `TaskError::DesktopOnly`。片段为空或编码参数不合法时返回 `TaskError::Failed`。路径为空返回 `PathEmpty`，源不是文件返回 `Missing`，输入输出相同返回 `SamePath`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let prepared = prepare_cut_video(input, output, segments, encode)?;
/// ```
fn prepare_cut_video(
  input: String,
  output: String,
  segments: Vec<TimeSegment>,
  mut encode: VideoEncode,
) -> Result<Prepared, TaskError> {
  platform::ensure_desktop()?;
  if segments.is_empty() {
    return Err(TaskError::Failed("时间片段不能为空".into()));
  }
  crate::ffmpeg::models::normalize_encode(&mut encode).map_err(TaskError::Failed)?;
  let input = require_file(input)?;
  let output = require_output(&input, output)?;
  Ok(sidecar_job(
    KIND_CUT_VIDEO,
    output.clone(),
    Spec::CutVideo { input, output, segments, encode },
  ))
}

/// 准备视频合并任务
///
/// 非桌面直接返回 `DesktopOnly`。桌面收紧编码参数，并检查每一段输入和输出路径。
///
/// # 参数
/// - `inputs`: 源视频列表
/// - `output`: 输出路径
/// - `encode`: 编码配置
///
/// # 返回值
/// 成功时返回侧车任务，密码为空
///
/// # 错误
/// 非桌面返回 `TaskError::DesktopOnly`。编码参数不合法时返回 `TaskError::Failed`。列表或路径为空返回 `PathEmpty`，某个源不是文件返回 `Missing`，输出与任一输入相同返回 `SamePath`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let prepared = prepare_merge_video(inputs, output, encode)?;
/// ```
fn prepare_merge_video(inputs: Vec<String>, output: String, mut encode: VideoEncode) -> Result<Prepared, TaskError> {
  platform::ensure_desktop()?;
  crate::ffmpeg::models::normalize_encode(&mut encode).map_err(TaskError::Failed)?;
  let inputs = require_files(inputs)?;
  let output = require_output_many(&inputs, output)?;
  Ok(sidecar_job(
    KIND_MERGE_VIDEO,
    output.clone(),
    Spec::MergeVideo { inputs, output, encode },
  ))
}

/// 准备视频追加任务
///
/// 非桌面直接返回 `DesktopOnly`。桌面收紧编码参数，并把基准和各段一起拿来检查输出是否撞路径。
///
/// # 参数
/// - `base`: 基准视频
/// - `inputs`: 要追加的视频
/// - `output`: 输出路径
/// - `encode`: 编码配置
///
/// # 返回值
/// 成功时返回侧车任务，密码为空
///
/// # 错误
/// 非桌面返回 `TaskError::DesktopOnly`。编码参数不合法时返回 `TaskError::Failed`。路径或追加列表为空返回 `PathEmpty`，源不是文件返回 `Missing`，输出与基准或任一段相同返回 `SamePath`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let prepared = prepare_append_video(base, inputs, output, encode)?;
/// ```
fn prepare_append_video(
  base: String,
  inputs: Vec<String>,
  output: String,
  mut encode: VideoEncode,
) -> Result<Prepared, TaskError> {
  platform::ensure_desktop()?;
  crate::ffmpeg::models::normalize_encode(&mut encode).map_err(TaskError::Failed)?;
  let base = require_file(base)?;
  let inputs = require_files(inputs)?;
  let mut all = Vec::with_capacity(inputs.len() + 1);
  all.push(base.clone());
  all.extend(inputs.iter().cloned());
  let output = require_output_many(&all, output)?;
  Ok(sidecar_job(
    KIND_APPEND_VIDEO,
    output.clone(),
    Spec::AppendVideo { base, inputs, output, encode },
  ))
}

/// 准备 AVIF 转码任务
///
/// 非桌面直接返回 `DesktopOnly`。桌面先收紧编码参数，再检查输入文件和输出路径。
///
/// # 参数
/// - `input`: 源图片
/// - `output`: 输出路径
/// - `params`: AVIF 参数
///
/// # 返回值
/// 成功时返回侧车任务，密码为空。草稿里的参数已经收紧
///
/// # 错误
/// 非桌面返回 `TaskError::DesktopOnly`。参数不合法返回 `Failed`。路径为空返回 `PathEmpty`，源不是文件返回 `Missing`，输入输出相同返回 `SamePath`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let prepared = prepare_convert_avif(input, output, params)?;
/// ```
fn prepare_convert_avif(input: String, output: String, mut params: AvifEncodeParams) -> Result<Prepared, TaskError> {
  platform::ensure_desktop()?;
  crate::image::models::normalize_avif(&mut params).map_err(TaskError::Failed)?;
  let input = require_file(input)?;
  let output = require_output(&input, output)?;
  Ok(sidecar_job(
    KIND_CONVERT_AVIF,
    output.clone(),
    Spec::ConvertAvif { input, output, params },
  ))
}

/// 准备 JXL 转码任务
///
/// 非桌面直接返回 `DesktopOnly`。桌面先收紧编码参数，再检查输入文件和输出路径。
///
/// # 参数
/// - `input`: 源图片
/// - `output`: 输出路径
/// - `params`: JXL 参数
///
/// # 返回值
/// 成功时返回侧车任务，密码为空。草稿里的参数已经收紧
///
/// # 错误
/// 非桌面返回 `TaskError::DesktopOnly`。参数不合法返回 `Failed`。路径为空返回 `PathEmpty`，源不是文件返回 `Missing`，输入输出相同返回 `SamePath`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let prepared = prepare_convert_jxl(input, output, params)?;
/// ```
fn prepare_convert_jxl(input: String, output: String, mut params: JxlEncodeParams) -> Result<Prepared, TaskError> {
  platform::ensure_desktop()?;
  crate::image::models::normalize_jxl(&mut params).map_err(TaskError::Failed)?;
  let input = require_file(input)?;
  let output = require_output(&input, output)?;
  Ok(sidecar_job(
    KIND_CONVERT_JXL,
    output.clone(),
    Spec::ConvertJxl { input, output, params },
  ))
}

/// 组装一条没有密码、没有上传文件 ID 的侧车任务
///
/// # 参数
/// - `kind`: 任务类型
/// - `target`: 用来判断忙闲的目标路径
/// - `spec`: 任务规格
///
/// # 返回值
/// 可入队的任务
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let prepared = sidecar_job(kind, target, spec);
/// ```
fn sidecar_job(kind: &str, target: String, spec: Spec) -> Prepared {
  finish(kind, target, spec, String::new(), None)
}

/// 补上标题、输出和草稿，组装可入队任务
///
/// # 参数
/// - `kind`: 任务类型
/// - `target`: 用来判断忙闲的目标
/// - `spec`: 任务规格
/// - `password`: 口令。侧车任务为空
/// - `upload_id`: 上传文件 ID
///
/// # 返回值
/// 可入队的任务
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let prepared = finish(kind, target, spec, password, upload_id);
/// ```
fn finish(kind: &str, target: String, spec: Spec, password: String, upload_id: Option<String>) -> Prepared {
  let (title, output, draft) = describe(&spec);
  Prepared { kind: kind.to_string(), spec, password, upload_id, target, title, output, draft }
}

/// 从规格取出标题、输出路径和可还原的草稿
///
/// 导入没有输出和草稿。已上传文件的加解密标题固定为「上传文件」，也没有输出和草稿。多文件标题带上数量。
///
/// # 参数
/// - `spec`: 任务规格
///
/// # 返回值
/// 标题、输出路径、草稿
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let (title, output, draft) = describe(spec);
/// ```
fn describe(spec: &Spec) -> (String, String, Option<TaskDraft>) {
  match spec {
    Spec::File { input, output, encrypt } => {
      let input = input.display().to_string();
      let output = output.display().to_string();
      let draft = if *encrypt {
        TaskDraft::EncryptFile { input: input.clone(), output: output.clone() }
      } else {
        TaskDraft::DecryptFile { input: input.clone(), output: output.clone() }
      };
      (file_name(&input), output, Some(draft))
    }
    Spec::ConvertVideo { input, output, encode } => (
      file_name(input),
      output.clone(),
      Some(TaskDraft::ConvertVideo { input: input.clone(), output: output.clone(), encode: encode.clone() }),
    ),
    Spec::CutVideo { input, output, segments, encode } => (
      file_name(input),
      output.clone(),
      Some(TaskDraft::CutVideo {
        input: input.clone(),
        output: output.clone(),
        segments: segments.clone(),
        encode: encode.clone(),
      }),
    ),
    Spec::MergeVideo { inputs, output, encode } => (
      file_names(inputs),
      output.clone(),
      Some(TaskDraft::MergeVideo { inputs: inputs.clone(), output: output.clone(), encode: encode.clone() }),
    ),
    Spec::AppendVideo { base, inputs, output, encode } => {
      let mut all = Vec::with_capacity(inputs.len() + 1);
      all.push(base.clone());
      all.extend(inputs.iter().cloned());
      (
        file_names(&all),
        output.clone(),
        Some(TaskDraft::AppendVideo {
          base: base.clone(),
          inputs: inputs.clone(),
          output: output.clone(),
          encode: encode.clone(),
        }),
      )
    }
    Spec::ConvertAvif { input, output, params } => (
      file_name(input),
      output.clone(),
      Some(TaskDraft::ConvertAvif { input: input.clone(), output: output.clone(), params: params.clone() }),
    ),
    Spec::ConvertJxl { input, output, params } => (
      file_name(input),
      output.clone(),
      Some(TaskDraft::ConvertJxl { input: input.clone(), output: output.clone(), params: params.clone() }),
    ),
    Spec::Import { source, .. } => (file_name(&source.display().to_string()), String::new(), None),
    Spec::Upload { .. } => ("上传文件".to_string(), String::new(), None),
  }
}

/// 取路径里的文件名，空则返回「未命名」
///
/// # 参数
/// - `path`: 文件路径
///
/// # 返回值
/// 文件名
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(file_name("a/b.mp4"), "b.mp4");
/// assert_eq!(file_name(""), "未命名");
/// ```
fn file_name(path: &str) -> String {
  Path::new(path)
    .file_name()
    .map(|name| name.to_string_lossy().into_owned())
    .filter(|name| !name.is_empty())
    .unwrap_or_else(|| "未命名".to_string())
}

/// 用第一个文件名代表一组路径
///
/// 没有路径时返回「未命名」。多于一个时接上「等 N 个」。
///
/// # 参数
/// - `paths`: 路径列表
///
/// # 返回值
/// 标题文本
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let title = file_names(&["a.mp4".into(), "b.mp4".into()]);
/// assert_eq!(title, "a.mp4 等 2 个");
/// ```
fn file_names(paths: &[String]) -> String {
  let Some(first) = paths.first() else {
    return "未命名".to_string();
  };
  let name = file_name(first);
  if paths.len() <= 1 { name } else { format!("{name} 等 {} 个", paths.len()) }
}

/// 去掉两端空白并确认路径指向一个文件
///
/// # 参数
/// - `path`: 待检查的路径
///
/// # 返回值
/// 成功时返回去掉空白后的路径
///
/// # 错误
/// 空白后为空返回 `TaskError::PathEmpty`，不是文件返回 `TaskError::Missing`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let path = require_file(path)?;
/// ```
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

/// 确认列表非空，并逐个确认是文件
///
/// # 参数
/// - `paths`: 路径列表
///
/// # 返回值
/// 成功时返回去掉空白后的路径
///
/// # 错误
/// 列表为空返回 `TaskError::PathEmpty`。某一项为空返回 `PathEmpty`，不是文件返回 `Missing`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let paths = require_files(paths)?;
/// ```
fn require_files(paths: Vec<String>) -> Result<Vec<String>, TaskError> {
  if paths.is_empty() {
    return Err(TaskError::PathEmpty);
  }
  paths.into_iter().map(require_file).collect()
}

/// 去掉输出路径两端空白，并拒绝和输入相同
///
/// # 参数
/// - `input`: 输入路径
/// - `output`: 输出路径
///
/// # 返回值
/// 成功时返回去掉空白后的输出路径
///
/// # 错误
/// 空白后为空返回 `TaskError::PathEmpty`，与输入相同返回 `TaskError::SamePath`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let output = require_output("in.mp4", " out.mp4 ".into()).unwrap();
/// assert_eq!(output, "out.mp4");
/// ```
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

/// 去掉输出路径两端空白，并拒绝和任一输入相同
///
/// # 参数
/// - `inputs`: 输入路径
/// - `output`: 输出路径
///
/// # 返回值
/// 成功时返回去掉空白后的输出路径
///
/// # 错误
/// 空白后为空返回 `TaskError::PathEmpty`，与任一输入相同返回 `TaskError::SamePath`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let output = require_output_many(&["a.mp4".into()], "b.mp4".into()).unwrap();
/// assert_eq!(output, "b.mp4");
/// ```
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

/// 准备文件加密或解密任务
///
/// 失败时清零密码。输入输出会去掉两端空白。
///
/// # 参数
/// - `kind`: 任务类型
/// - `input`: 源文件
/// - `output`: 输出路径
/// - `password`: 口令
/// - `encrypt`: 为真时加密
///
/// # 返回值
/// 成功时返回可入队任务。目标是输出路径
///
/// # 错误
/// 口令为空返回 `PasswordEmpty`，路径空白后为空返回 `PathEmpty`，输入输出相同返回 `SamePath`，源不是文件返回 `Missing`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let prepared = prepare_file(kind, input, output, password, true)?;
/// ```
fn prepare_file(
  kind: &str,
  input: String,
  output: String,
  password: String,
  encrypt: bool,
) -> Result<Prepared, TaskError> {
  // 失败时清零密码
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

  let target = output.display().to_string();
  Ok(finish(kind, target, Spec::File { input, output, encrypt }, password, None))
}

/// 准备导入上传文件任务
///
/// 不加密时清零密码。加密时口令不能为空。归属必须能解析，并且在库里存在。失败时清零密码。
///
/// # 参数
/// - `db`: 数据库状态
/// - `source`: 源文件
/// - `owner`: 归属类型
/// - `owner_id`: 归属 ID
/// - `upload_kind`: 上传文件类型，只接受图片、视频、书或文件
/// - `encrypt`: 是否加密后入库
/// - `password`: 口令
///
/// # 返回值
/// 成功时返回可入队任务。目标是源路径
///
/// # 错误
/// 加密但口令为空返回 `PasswordEmpty`。源、归属或归属 ID 空白后为空返回 `PathEmpty`。类型不是图片、视频、书或文件，或归属解析失败，返回 `KindInvalid`。源不是文件返回 `Missing`。查询归属失败返回 `Internal`。归属不存在返回 `ReferencedMissing`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let prepared = prepare_import(db, source, owner, owner_id, upload_kind, encrypt, password)?;
/// ```
fn prepare_import(
  db: &DbState,
  source: String,
  owner: String,
  owner_id: String,
  upload_kind: String,
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
  let upload_kind = upload_kind.trim().to_string();
  if source.is_empty() || owner.is_empty() || owner_id.is_empty() {
    return Err(reject(password, TaskError::PathEmpty));
  }
  if !KINDS.contains(&upload_kind.as_str()) {
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
  Ok(finish(
    KIND_IMPORT_UPLOAD,
    source,
    Spec::Import { source: source_path, owner, owner_id, upload_kind, encrypt },
    password,
    None,
  ))
}

/// 准备上传文件加密或解密任务
///
/// 失败时清零密码。已是目标加密状态时拒绝。
///
/// # 参数
/// - `db`: 数据库状态
/// - `kind`: 任务类型
/// - `upload_id`: 上传文件 ID
/// - `password`: 口令
/// - `encrypt`: 为真时加密
///
/// # 返回值
/// 成功时返回可入队任务，并带上上传文件 ID
///
/// # 错误
/// 口令为空返回 `PasswordEmpty`，ID 空白后为空返回 `PathEmpty`。读上传文件失败时返回映射后的 `TaskError`。已加密还要加密返回 `AlreadyEncrypted`，未加密还要解密返回 `NotEncrypted`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let prepared = prepare_upload(db, kind, upload_id, password, true)?;
/// ```
fn prepare_upload(
  db: &DbState,
  kind: &str,
  upload_id: String,
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
  let upload_id = upload_id.trim().to_string();
  if upload_id.is_empty() {
    return Err(reject(password, TaskError::PathEmpty));
  }
  let file = match db.with_conn(|conn| crate::upload::get(conn, &upload_id)) {
    Ok(file) => file,
    Err(err) => return Err(reject(password, err.into())),
  };
  if encrypt && file.encrypted {
    return Err(reject(password, TaskError::AlreadyEncrypted));
  }
  if !encrypt && !file.encrypted {
    return Err(reject(password, TaskError::NotEncrypted));
  }
  Ok(finish(
    kind,
    upload_id.clone(),
    Spec::Upload { upload_id: upload_id.clone(), encrypt },
    password,
    Some(upload_id),
  ))
}

/// 记下 I/O 错误并返回 `Internal`
///
/// # 参数
/// - `context`: 出错位置说明
/// - `err`: 标准库 I/O 错误
///
/// # 返回值
/// `TaskError::Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let err = io_fail("upload rename", err);
/// ```
fn io_fail(context: &str, err: std::io::Error) -> TaskError {
  tauri_plugin_log::log::error!("{context}: {err}");
  TaskError::Internal
}

/// 按扩展名给出 MIME，认不出时按上传文件类型回退
///
/// 图片回退 `image/jpeg`，视频回退 `video/mp4`，其余回退 `application/octet-stream`。
///
/// # 参数
/// - `path`: 文件路径
/// - `kind`: 上传文件类型
///
/// # 返回值
/// MIME 字符串
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(mime_of(Path::new("a.png"), "image"), "image/png");
/// assert_eq!(mime_of(Path::new("a"), "video"), "video/mp4");
/// ```
fn mime_of(path: &Path, kind: &str) -> String {
  if let Some(mime) = mime_guess::from_path(path).first_raw() {
    return mime.to_string();
  }
  match kind {
    KIND_IMAGE => "image/jpeg".to_string(),
    KIND_VIDEO => "video/mp4".to_string(),
    _ => "application/octet-stream".to_string(),
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::sync::atomic::Ordering;

  use rusqlite::params;

  use super::super::constants::{KIND_IMPORT_UPLOAD, STATUS_CANCELED, STATUS_COMPLETED, STATUS_PENDING};
  use super::super::dto::TaskDraft;
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
      crate::upload::migrate,
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
  fn import_input_reads_camel_case_fields() {
    let input: TaskInput = serde_json::from_str(
      r#"{"kind":"importUpload","source":"a.jpg","owner":"plan","ownerId":"p1","uploadKind":"image","encrypt":false,"password":""}"#,
    )
    .expect("parse");
    assert!(matches!(
      input,
      TaskInput::ImportUpload { ref owner_id, ref upload_kind, .. } if owner_id == "p1" && upload_kind == "image"
    ));
    let input: TaskInput =
      serde_json::from_str(r#"{"kind":"encryptUpload","uploadId":"u1","password":"pw"}"#).expect("parse");
    assert!(matches!(input, TaskInput::EncryptUpload { ref upload_id, .. } if upload_id == "u1"));
  }

  #[test]
  fn import_book_lands_in_upload_with_name_and_size() {
    let (dir, db) = setup();
    let plan_id = seed_plan(&db);
    let source = dir.path().join("三体.EPUB");
    fs::write(&source, b"PK-book-bytes").expect("source");
    let queue = TaskQueue::default();
    enqueue(
      &db,
      &queue,
      TaskInput::ImportUpload {
        source: source.display().to_string(),
        owner: Owner::Plan.as_str().into(),
        owner_id: plan_id,
        upload_kind: "book".into(),
        encrypt: false,
        password: String::new(),
      },
    )
    .expect("enqueue");
    let done = queue.drive(&db).expect("drive");
    assert_eq!(done.status, STATUS_COMPLETED);
    let upload_id = done.upload_id.expect("upload id");
    let file = db.with_conn(|conn| crate::upload::get(conn, &upload_id)).expect("get");
    assert_eq!(file.rel_path, format!("upload/book/{upload_id}.epub"));
    assert_eq!(file.name, "三体.EPUB");
    assert_eq!(file.size, 13);
    assert_eq!(file.mime, "application/epub+zip");
    assert_eq!(
      fs::read(upload::resolve(&db, &file.rel_path)).expect("stored"),
      b"PK-book-bytes"
    );
    let tmp = upload::tmp_path(&db, "");
    assert_eq!(fs::read_dir(tmp).map(|entries| entries.count()).unwrap_or(0), 0);
  }

  #[test]
  fn import_encrypts_into_upload() {
    let (dir, db) = setup();
    let plan_id = seed_plan(&db);
    let source = dir.path().join("shot.jpg");
    fs::write(&source, b"plain-image-bytes").expect("source");
    let queue = TaskQueue::default();
    let pending = enqueue(
      &db,
      &queue,
      TaskInput::ImportUpload {
        source: source.display().to_string(),
        owner: Owner::Plan.as_str().into(),
        owner_id: plan_id.clone(),
        upload_kind: KIND_IMAGE.into(),
        encrypt: true,
        password: "file-password".into(),
      },
    )
    .expect("enqueue");
    assert_eq!(pending.kind, KIND_IMPORT_UPLOAD);
    assert_eq!(pending.status, STATUS_PENDING);
    let done = queue.drive(&db).expect("drive");
    assert_eq!(done.status, STATUS_COMPLETED);
    assert_eq!(done.progress, 100.0);
    let upload_id = done.upload_id.expect("upload id");
    let file = db.with_conn(|conn| crate::upload::get(conn, &upload_id)).expect("get");
    assert!(file.encrypted);
    assert_eq!(file.rel_path, format!("upload/image/{upload_id}.jpg"));
    assert_eq!(file.name, "shot.jpg");
    let stored = upload::resolve(&db, &file.rel_path);
    let sealed = fs::read(&stored).expect("stored");
    assert_eq!(&sealed[..8], b"chacha20");

    assert!(matches!(
      enqueue(
        &db,
        &queue,
        TaskInput::EncryptUpload { upload_id: upload_id.clone(), password: "file-password".into() },
      ),
      Err(TaskError::AlreadyEncrypted)
    ));

    enqueue(
      &db,
      &queue,
      TaskInput::DecryptUpload { upload_id, password: "file-password".into() },
    )
    .expect("decrypt");
    let opened = queue.drive(&db).expect("decrypt drive");
    assert_eq!(opened.status, STATUS_COMPLETED);
    assert_eq!(fs::read(stored).expect("plain"), b"plain-image-bytes");
    let file = db.with_conn(|conn| crate::upload::get(conn, opened.upload_id.as_deref().unwrap())).expect("get");
    assert!(!file.encrypted);
  }

  #[test]
  fn empty_password_is_rejected_and_file_job_skips_upload() {
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
        TaskInput::ImportUpload {
          source: input.display().to_string(),
          owner: Owner::Plan.as_str().into(),
          owner_id: "missing".into(),
          upload_kind: KIND_IMAGE.into(),
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
    assert!(done.upload_id.is_none());
    assert_eq!(&fs::read(&output).expect("enc")[..8], b"chacha20");
    let count = db
      .with_conn(|conn| {
        conn.query_row("SELECT COUNT(*) FROM upload", [], |row| row.get::<_, i64>(0)).map_err(|_| TaskError::Internal)
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
    let upload_id = new_uuid_v4();
    let file = db
      .with_conn(|conn| {
        crate::upload::insert_imported(
          conn,
          &upload_id,
          crate::upload::dto::UploadInput {
            owner: Owner::Plan.as_str().into(),
            owner_id: plan_id.clone(),
            kind: KIND_IMAGE.into(),
            rel_path: path.display().to_string(),
            mime: "image/jpeg".into(),
            name: String::new(),
            size: 0,
            sort: 0,
            locked: false,
          },
          false,
        )
      })
      .expect("upload");
    let queue = TaskQueue::default();
    let pending = enqueue(
      &db,
      &queue,
      TaskInput::EncryptUpload { upload_id: file.id.clone(), password: "file-password".into() },
    )
    .expect("enqueue");
    let canceled = queue.cancel(&pending.id).expect("cancel");
    assert_eq!(canceled.task.status, STATUS_CANCELED);
    assert!(matches!(queue.drive(&db), Err(TaskError::NotFound)));
    assert_eq!(fs::read(&path).expect("still"), original);
    let row = db.with_conn(|conn| crate::upload::get(conn, &file.id)).expect("get");
    assert!(!row.encrypted);

    let again = enqueue(
      &db,
      &queue,
      TaskInput::EncryptUpload { upload_id: file.id.clone(), password: "file-password".into() },
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
    let row = db.with_conn(|conn| crate::upload::get(conn, &file.id)).expect("get");
    assert!(!row.encrypted);
  }

  #[test]
  fn locked_archive_keeps_original() {
    let (dir, db) = setup();
    let plan_id = seed_plan(&db);
    let path = dir.path().join("still.jpg");
    fs::write(&path, b"still-plain").expect("file");
    let upload_id = new_uuid_v4();
    let file = db
      .with_conn(|conn| {
        crate::upload::insert_imported(
          conn,
          &upload_id,
          crate::upload::dto::UploadInput {
            owner: Owner::Plan.as_str().into(),
            owner_id: plan_id.clone(),
            kind: KIND_IMAGE.into(),
            rel_path: path.display().to_string(),
            mime: "image/jpeg".into(),
            name: String::new(),
            size: 0,
            sort: 0,
            locked: false,
          },
          false,
        )
      })
      .expect("upload");
    let queue = TaskQueue::default();
    enqueue(
      &db,
      &queue,
      TaskInput::EncryptUpload { upload_id: file.id.clone(), password: "file-password".into() },
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
    let row = db.with_conn(|conn| crate::upload::get(conn, &file.id)).expect("get");
    assert!(!row.encrypted);
  }

  #[cfg(desktop)]
  #[test]
  fn convert_keeps_name_and_draft() {
    let (dir, db) = setup();
    let input = dir.path().join("clip.mp4");
    fs::write(&input, b"not-a-video").expect("source");
    let output = dir.path().join("clip_converted.mp4");
    let queue = TaskQueue::default();
    let task = enqueue(
      &db,
      &queue,
      TaskInput::ConvertVideo {
        input: input.display().to_string(),
        output: output.display().to_string(),
        encode: crate::ffmpeg::models::VideoEncode::default(),
      },
    )
    .expect("enqueue");
    assert_eq!(task.title, "clip.mp4");
    assert_eq!(task.output, output.display().to_string());
    assert!(matches!(task.draft, Some(TaskDraft::ConvertVideo { .. })));
  }
}
