//! 取视频中间一小段，按当前画面参数转码，再同时算 VMAF 和 SSIM。
//!
//! 不进任务队列。标题、字幕和音轨不参与这次预估。临时文件放在缓存目录的 `preview` 下，结束或取消就删。

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use super::models::{AudioMode, AudioPlan, VideoContainer, VideoEncode, even_dim, normalize_encode};
use crate::task::error::TaskError;

/// 中间一段的起点和时长
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampleWindow {
  /// 相对源视频的起点，单位秒
  pub start: f64,
  /// 取样时长，单位秒
  pub seconds: f64,
  /// 短于 3 秒时用整段
  pub whole: bool,
}

/// 主结论来自哪一个分数
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum QualityBasis {
  /// 按 VMAF 分档
  Vmaf,
  /// VMAF 没有数，按 SSIM 分档
  Ssim,
}

/// 给人看的画质档
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum QualityBand {
  /// 几乎看不出差别
  Indistinguishable,
  /// 看起来一样
  Same,
  /// 略微变软
  Slight,
  /// 能看出损失
  Visible,
  /// 明显变差。只有 VMAF 会落到这一档
  Degraded,
}

/// 一次画质预估的结果
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityPreview {
  /// 取样起点，单位秒
  pub start: f64,
  /// 取样时长，单位秒
  pub seconds: f64,
  /// 短于 3 秒时为真，表示用的是整段
  pub whole: bool,
  /// VMAF。没算出来时为空
  pub vmaf: Option<f64>,
  /// SSIM。没算出来时为空
  pub ssim: Option<f64>,
  /// 分档用的是哪一个分数
  pub basis: QualityBasis,
  /// 分档
  pub band: QualityBand,
  /// 预览文件的字节数
  pub preview_bytes: u64,
  /// 按这段画面码率推算的整片字节数，不含音频
  pub estimated_bytes: u64,
}

/// 正在跑的预估子进程。和任务队列的进程槽分开
#[cfg(desktop)]
#[derive(Default)]
pub struct PreviewSlot {
  child: Mutex<Option<tauri_plugin_shell::process::CommandChild>>,
  generation: AtomicU64,
}

#[cfg(desktop)]
impl PreviewSlot {
  /// 开始新的一轮，并停掉上一轮
  fn begin(&self) -> u64 {
    let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
    self.kill();
    generation
  }

  /// 让当前这一轮失效，并杀掉子进程。更新的一轮已经开始时什么都不做
  fn cancel(&self) {
    let current = self.generation.load(Ordering::SeqCst);
    if self.generation.compare_exchange(current, current + 1, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
      self.kill();
    }
  }

  fn is_current(&self, generation: u64) -> bool {
    self.generation.load(Ordering::SeqCst) == generation
  }

  /// 记下子进程。这一轮已经失效时立刻杀掉，返回 false
  fn store(&self, generation: u64, child: tauri_plugin_shell::process::CommandChild) -> bool {
    if !self.is_current(generation) {
      kill_child(child);
      return false;
    }
    let mut guard = self.child.lock().unwrap_or_else(|err| err.into_inner());
    if !self.is_current(generation) {
      drop(guard);
      kill_child(child);
      return false;
    }
    if let Some(previous) = guard.replace(child) {
      kill_child(previous);
    }
    true
  }

  /// 这一轮还有效时清掉槽里的句柄。进程已经结束，不再杀一次
  fn release(&self, generation: u64) {
    if !self.is_current(generation) {
      return;
    }
    let mut guard = self.child.lock().unwrap_or_else(|err| err.into_inner());
    if self.is_current(generation) {
      guard.take();
    }
  }

  fn kill(&self) {
    if let Some(child) = self.child.lock().unwrap_or_else(|err| err.into_inner()).take() {
      kill_child(child);
    }
  }
}

#[cfg(desktop)]
fn kill_child(child: tauri_plugin_shell::process::CommandChild) {
  // macOS 上在异步运行时里直接杀可能崩溃，放到阻塞线程。
  tauri::async_runtime::spawn_blocking(move || {
    let _ = child.kill();
  });
}

/// 按时长决定取样窗口
///
/// 短于 0.5 秒无法预估。0.5 秒到 3 秒用整段。更长的取正中间 2.5 秒。
pub fn sample_window(duration: f64) -> Result<SampleWindow, &'static str> {
  if !duration.is_finite() || duration < 0.5 {
    return Err("视频太短，无法预估");
  }
  if duration < 3.0 {
    return Ok(SampleWindow { start: 0.0, seconds: duration, whole: true });
  }
  let seconds = 2.5_f64;
  let start = (duration / 2.0 - seconds / 2.0).max(0.0);
  let seconds = seconds.min((duration - start).max(0.0));
  Ok(SampleWindow { start, seconds, whole: false })
}

/// 丢掉首尾各 2 帧。片段不长于 0.5 秒，或丢掉之后剩不下帧时不裁
pub fn edge_trim(seconds: f64, fps: f64) -> Option<(u32, u32)> {
  if seconds <= 0.5 || !fps.is_finite() || fps <= 0.0 {
    return None;
  }
  let frames = (seconds * fps).round();
  if !frames.is_finite() || frames <= 6.0 {
    return None;
  }
  Some((2, frames as u32 - 2))
}

/// 按 VMAF 分档。没有 VMAF 时退回 SSIM。两个都没有则没有结论
pub fn quality_band(vmaf: Option<f64>, ssim: Option<f64>) -> Option<(QualityBasis, QualityBand)> {
  if let Some(vmaf) = vmaf.filter(|score| score.is_finite()) {
    let band = if vmaf >= 95.0 {
      QualityBand::Indistinguishable
    } else if vmaf >= 90.0 {
      QualityBand::Same
    } else if vmaf >= 80.0 {
      QualityBand::Slight
    } else if vmaf >= 70.0 {
      QualityBand::Visible
    } else {
      QualityBand::Degraded
    };
    return Some((QualityBasis::Vmaf, band));
  }
  if let Some(ssim) = ssim.filter(|score| score.is_finite()) {
    let band = if ssim >= 0.98 {
      QualityBand::Indistinguishable
    } else if ssim >= 0.95 {
      QualityBand::Same
    } else if ssim >= 0.90 {
      QualityBand::Slight
    } else {
      QualityBand::Visible
    };
    return Some((QualityBasis::Ssim, band));
  }
  None
}

/// 按预览字节数和时长推算整片体积
pub fn estimate_bytes(preview_bytes: u64, sample_seconds: f64, duration: f64) -> u64 {
  if preview_bytes == 0
    || sample_seconds <= 0.0
    || !sample_seconds.is_finite()
    || duration <= 0.0
    || !duration.is_finite()
  {
    return preview_bytes;
  }
  let value = preview_bytes as f64 * duration / sample_seconds;
  if value.is_finite() && value > 0.0 { value.round() as u64 } else { preview_bytes }
}

/// 只保留画面参数。标题、字幕和音轨不参与预估，容器固定 MP4
pub fn picture_encode(mut encode: VideoEncode) -> Result<VideoEncode, String> {
  encode.title = None;
  encode.subtitle = None;
  encode.audio.mode = AudioMode::Drop;
  encode.container = VideoContainer::Mp4;
  encode.faststart = false;
  normalize_encode(&mut encode)?;
  Ok(encode)
}

/// 拼出同时算 SSIM 和 VMAF 的滤镜。宽高是源画面收成偶数后的尺寸
pub fn score_filter(width: u32, height: u32, fps: Option<u32>, trim: Option<(u32, u32)>) -> String {
  // 裁剪构建没有 setpts。两边裁同一段帧号，时间戳仍然对齐
  let trim = trim.map(|(start, end)| format!(",trim=start_frame={start}:end_frame={end}")).unwrap_or_default();
  let reference_fps = fps.map(|fps| format!("fps={fps},")).unwrap_or_default();
  format!(
    "[0:v]scale={width}:{height}:flags=bicubic,format=yuv420p{trim},split[d1][d2];\
     [1:v]{reference_fps}scale={width}:{height}:flags=bicubic,format=yuv420p{trim},split[r1][r2];\
     [d1][r1]ssim[ss];\
     [d2][r2]libvmaf=model=path=vmaf_v0.6.1.json[vm]"
  )
}

/// 从 ffmpeg 标准错误里取出最后一次 VMAF 和 SSIM
pub fn parse_scores(stderr: &str) -> (Option<f64>, Option<f64>) {
  let mut vmaf = None;
  let mut ssim = None;
  for line in stderr.lines() {
    if line.contains("VMAF score:") {
      if let Some(value) = last_number(line, "VMAF score:") {
        vmaf = Some(value);
      }
    }
    if line.contains("SSIM") {
      if let Some(value) = last_number(line, "All:") {
        ssim = Some(value);
      }
    }
  }
  (vmaf, ssim)
}

fn last_number(text: &str, label: &str) -> Option<f64> {
  let mut found = None;
  let mut rest = text;
  while let Some(index) = rest.find(label) {
    let after = &rest[index + label.len()..];
    let number: String = after.trim_start().chars().take_while(|ch| ch.is_ascii_digit() || *ch == '.').collect();
    if let Ok(value) = number.parse::<f64>() {
      if value.is_finite() {
        found = Some(value);
      }
    }
    rest = &rest[index + label.len()..];
  }
  found
}

/// 滤镜或模型缺失时给出明确原因，否则带回 ffmpeg 的退出说明
pub fn metric_failure(stderr: &str, code: Option<i32>) -> String {
  let lower = stderr.to_ascii_lowercase();
  if lower.contains("no such filter") || lower.contains("filter not found") {
    return "当前 FFmpeg 没有画质滤镜，需要按文档重新编译侧车".to_string();
  }
  if lower.contains("hardware accelerated av1") {
    return "当前 FFmpeg 不能做画质预估，需要按文档重新编译侧车".to_string();
  }
  if lower.contains("null") && (lower.contains("output format") || lower.contains("suitable output format"))
    || lower.contains("wrapped_avframe")
    || lower.contains("encoder not found")
  {
    return "当前 FFmpeg 不能做画质预估，需要按文档重新编译侧车".to_string();
  }
  if lower.contains("could not read model")
    || lower.contains("could not load libvmaf model")
    || lower.contains("no such built-in model")
  {
    return "找不到 VMAF 模型，需要更新应用资源".to_string();
  }
  #[cfg(desktop)]
  {
    let mut notes = super::utils::FfmpegNotes::default();
    notes.push(stderr.as_bytes());
    return notes.failure(code.unwrap_or(-1));
  }
  #[cfg(not(desktop))]
  {
    let _ = stderr;
    format!("退出代码 {}", code.unwrap_or(-1))
  }
}

/// 停掉正在进行的预估
pub fn preview_cancel(app: &tauri::AppHandle) {
  #[cfg(desktop)]
  {
    use tauri::Manager;
    app.state::<PreviewSlot>().cancel();
  }
  #[cfg(not(desktop))]
  {
    let _ = app;
  }
}

/// 预估当前画面参数的画质
pub async fn preview_quality(
  app: &tauri::AppHandle,
  path: String,
  encode: Option<VideoEncode>,
) -> Result<QualityPreview, TaskError> {
  #[cfg(desktop)]
  {
    run_preview(app, path, encode.unwrap_or_default()).await.map_err(TaskError::from)
  }
  #[cfg(not(desktop))]
  {
    let _ = (app, path, encode);
    Err(TaskError::DesktopOnly)
  }
}

#[cfg(desktop)]
async fn run_preview(
  app: &tauri::AppHandle,
  path: String,
  encode: VideoEncode,
) -> Result<QualityPreview, crate::sidecar::SidecarError> {
  use tauri::Manager;

  use super::graph::{GraphRequest, build_video_filter, explicit_size};
  use super::info::video_info;
  use crate::sidecar::SidecarError;
  use crate::sidecar::cache::get_cache_dir;

  if path.trim().is_empty() || !Path::new(path.trim()).is_file() {
    return Err(SidecarError::invalid("文件不存在"));
  }
  let model = model_source(app)?;
  let encode = picture_encode(encode).map_err(SidecarError::invalid)?;
  let slot = app.state::<PreviewSlot>();
  let generation = slot.begin();
  let info = video_info(app, path.trim()).await?;
  if !slot.is_current(generation) {
    return Err(SidecarError::canceled());
  }
  if info.width == 0 || info.height == 0 {
    return Err(SidecarError::invalid("无法读取视频画面"));
  }
  let window = sample_window(info.duration).map_err(SidecarError::invalid)?;
  let width = even_dim(info.width);
  let height = even_dim(info.height);
  let dir = get_cache_dir(app)?.join("preview").join(generation.to_string());
  std::fs::create_dir_all(&dir)?;
  let _cleanup = WorkDir(dir.clone());
  std::fs::copy(&model, dir.join("vmaf_v0.6.1.json"))?;
  if !slot.is_current(generation) {
    return Err(SidecarError::canceled());
  }

  let output = dir.join("preview.mp4");
  let output_text = output.to_string_lossy().to_string();
  let filter = build_video_filter(&GraphRequest {
    encode: &encode,
    path: path.trim(),
    width: info.width,
    fps: None,
    size: explicit_size(&encode),
    offset: 0.0,
    duration: window.seconds,
    source_start: Some(window.start),
  });
  let mut encode_args = Vec::new();
  encode_args.push("-ss".to_string());
  encode_args.push(seconds_arg(window.start));
  encode_args.push("-t".to_string());
  encode_args.push(seconds_arg(window.seconds));
  encode_args.push("-i".to_string());
  encode_args.push(path.trim().to_string());
  if let Some(filter) = filter {
    encode_args.push("-vf".to_string());
    encode_args.push(filter);
  }
  encode_args.push("-map".to_string());
  encode_args.push("0:v:0".to_string());
  encode_args.extend(super::models::encode_args(
    &encode,
    &AudioPlan {
      stream_index: None,
      channels: 2,
      sample_rate: 0,
      bitrate_kbps: encode.audio.bitrate_kbps,
      fallback: false,
    },
    width,
  ));
  super::models::push_output(&mut encode_args, &encode, &output_text);
  let encoded = run_ffmpeg(app, &slot, generation, &encode_args, Some(&dir)).await?;
  if encoded.code != Some(0) {
    return Err(SidecarError::sidecar_exit(
      "ffmpeg",
      metric_failure(&encoded.stderr, encoded.code),
    ));
  }
  if !slot.is_current(generation) {
    return Err(SidecarError::canceled());
  }

  let preview_bytes = std::fs::metadata(&output)?.len();
  let fps = encode.fps.filter(|fps| *fps > 0);
  let trim_fps = fps.map(|fps| fps as f64).filter(|fps| *fps > 0.0).unwrap_or(info.fps);
  let filter = score_filter(width, height, fps, edge_trim(window.seconds, trim_fps));
  let score_args = vec![
    "-hide_banner".to_string(),
    "-nostats".to_string(),
    "-i".to_string(),
    output_text,
    "-ss".to_string(),
    seconds_arg(window.start),
    "-t".to_string(),
    seconds_arg(window.seconds),
    "-i".to_string(),
    path.trim().to_string(),
    "-filter_complex".to_string(),
    filter,
    "-map".to_string(),
    "[ss]".to_string(),
    "-map".to_string(),
    "[vm]".to_string(),
    "-c:v".to_string(),
    "wrapped_avframe".to_string(),
    "-f".to_string(),
    "null".to_string(),
    "-".to_string(),
  ];
  let scored = run_ffmpeg(app, &slot, generation, &score_args, Some(&dir)).await?;
  if !slot.is_current(generation) {
    return Err(SidecarError::canceled());
  }
  let (vmaf, ssim) = parse_scores(&scored.stderr);
  let Some((basis, band)) = quality_band(vmaf, ssim) else {
    return Err(SidecarError::invalid(metric_failure(&scored.stderr, scored.code)));
  };
  if scored.code != Some(0) && vmaf.is_none() {
    tauri_plugin_log::log::warn!("VMAF 未测，退回 SSIM: {}", metric_failure(&scored.stderr, scored.code));
  }
  Ok(QualityPreview {
    start: window.start,
    seconds: window.seconds,
    whole: window.whole,
    vmaf,
    ssim,
    basis,
    band,
    preview_bytes,
    estimated_bytes: estimate_bytes(preview_bytes, window.seconds, info.duration),
  })
}

#[cfg(desktop)]
struct FfmpegRun {
  code: Option<i32>,
  stderr: String,
}

#[cfg(desktop)]
async fn run_ffmpeg(
  app: &tauri::AppHandle,
  slot: &PreviewSlot,
  generation: u64,
  args: &[String],
  cwd: Option<&Path>,
) -> Result<FfmpegRun, crate::sidecar::SidecarError> {
  use tauri_plugin_shell::ShellExt;
  use tauri_plugin_shell::process::CommandEvent;

  use crate::sidecar::SidecarError;

  if !slot.is_current(generation) {
    return Err(SidecarError::canceled());
  }
  tauri_plugin_log::log::info!("ffmpeg {}", args.join(" "));
  let mut command = app.shell().sidecar("ffmpeg").map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?.args(args);
  if let Some(cwd) = cwd {
    command = command.current_dir(cwd);
  }
  let (mut rx, child) = command.spawn().map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?;
  if !slot.store(generation, child) {
    return Err(SidecarError::canceled());
  }

  let mut stderr = String::new();
  let mut code = None;
  while let Some(event) = rx.recv().await {
    if !slot.is_current(generation) {
      break;
    }
    match event {
      CommandEvent::Stderr(chunk) => {
        stderr.push_str(&String::from_utf8_lossy(&chunk));
        if stderr.len() > 512_000 {
          let cut = stderr.ceil_char_boundary(stderr.len() - 256_000);
          stderr.drain(..cut);
        }
      }
      CommandEvent::Terminated(status) => {
        code = status.code;
        break;
      }
      _ => {}
    }
  }
  slot.release(generation);
  if !slot.is_current(generation) {
    return Err(SidecarError::canceled());
  }
  Ok(FfmpegRun { code, stderr })
}

#[cfg(desktop)]
fn model_source(app: &tauri::AppHandle) -> Result<PathBuf, crate::sidecar::SidecarError> {
  use tauri::Manager;
  use tauri::path::BaseDirectory;

  use crate::sidecar::SidecarError;

  if let Ok(path) = app.path().resolve("resources/vmaf/vmaf_v0.6.1.json", BaseDirectory::Resource) {
    if path.is_file() {
      return Ok(path);
    }
  }
  let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/vmaf/vmaf_v0.6.1.json");
  if dev.is_file() {
    return Ok(dev);
  }
  Err(SidecarError::invalid("找不到 VMAF 模型，需要更新应用资源"))
}

#[cfg(desktop)]
fn seconds_arg(value: f64) -> String {
  format!("{value:.3}")
}

#[cfg(desktop)]
struct WorkDir(PathBuf);

#[cfg(desktop)]
impl Drop for WorkDir {
  fn drop(&mut self) {
    if let Err(err) = std::fs::remove_dir_all(&self.0) {
      tauri_plugin_log::log::warn!("清理画质预估临时目录失败: {err}");
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn window_uses_the_middle_two_and_a_half_seconds() {
    let window = sample_window(10.0).unwrap();
    assert!((window.start - 3.75).abs() < 1e-9);
    assert!((window.seconds - 2.5).abs() < 1e-9);
    assert!(!window.whole);
  }

  #[test]
  fn short_video_uses_the_whole_file() {
    let window = sample_window(2.0).unwrap();
    assert_eq!(window.start, 0.0);
    assert_eq!(window.seconds, 2.0);
    assert!(window.whole);
  }

  #[test]
  fn tiny_video_is_rejected() {
    assert_eq!(sample_window(0.4).unwrap_err(), "视频太短，无法预估");
  }

  #[test]
  fn bands_follow_vmaf_then_ssim() {
    assert_eq!(
      quality_band(Some(96.0), Some(0.5)),
      Some((QualityBasis::Vmaf, QualityBand::Indistinguishable))
    );
    assert_eq!(quality_band(Some(91.0), None), Some((QualityBasis::Vmaf, QualityBand::Same)));
    assert_eq!(quality_band(Some(80.0), None), Some((QualityBasis::Vmaf, QualityBand::Slight)));
    assert_eq!(quality_band(Some(70.0), None), Some((QualityBasis::Vmaf, QualityBand::Visible)));
    assert_eq!(
      quality_band(Some(69.9), None),
      Some((QualityBasis::Vmaf, QualityBand::Degraded))
    );
    assert_eq!(
      quality_band(None, Some(0.99)),
      Some((QualityBasis::Ssim, QualityBand::Indistinguishable))
    );
    assert_eq!(quality_band(None, Some(0.96)), Some((QualityBasis::Ssim, QualityBand::Same)));
    assert_eq!(quality_band(None, Some(0.91)), Some((QualityBasis::Ssim, QualityBand::Slight)));
    assert_eq!(quality_band(None, Some(0.5)), Some((QualityBasis::Ssim, QualityBand::Visible)));
    assert_eq!(quality_band(None, None), None);
  }

  #[test]
  fn trim_drops_two_frames_at_each_end() {
    assert_eq!(edge_trim(2.5, 24.0), Some((2, 58)));
    assert_eq!(edge_trim(0.5, 24.0), None);
    assert_eq!(edge_trim(2.5, 0.0), None);
  }

  #[test]
  fn estimate_scales_by_duration() {
    assert_eq!(estimate_bytes(1000, 2.5, 10.0), 4000);
  }

  #[test]
  fn scores_come_from_the_last_match() {
    let stderr = "SSIM Y:0.1 All:0.100 (1.00)\nVMAF score: 80.5\nSSIM Y:0.9 All:0.972 (15.5)\nVMAF score: 93.25\n";
    assert_eq!(parse_scores(stderr), (Some(93.25), Some(0.972)));
  }

  #[test]
  fn missing_filter_and_model_have_their_own_messages() {
    assert!(metric_failure("No such filter: 'libvmaf'", Some(1)).contains("画质滤镜"));
    assert!(metric_failure("Could not read model from path", Some(1)).contains("VMAF 模型"));
    assert!(
      !metric_failure(
        "No option name near 'PTS-STARTPTS'\nError parsing filterchain 'libvmaf=model=path=vmaf_v0.6.1.json'",
        Some(1)
      )
      .contains("VMAF 模型")
    );
    assert!(
      metric_failure("Your platform doesn't support hardware accelerated AV1 decoding.", Some(1)).contains("画质预估")
    );
    assert!(
      metric_failure(
        "Default encoder for format null (codec wrapped_avframe) is probably disabled. Encoder not found",
        Some(1)
      )
      .contains("画质预估")
    );
  }

  #[test]
  fn score_filter_aligns_fps_and_drops_edges() {
    let filter = score_filter(1920, 1080, Some(30), Some((2, 70)));
    assert!(filter.contains("scale=1920:1080:flags=bicubic"));
    assert!(filter.contains("fps=30,"));
    assert!(filter.contains("trim=start_frame=2:end_frame=70"));
    assert!(!filter.contains("setpts"));
    assert!(filter.contains("libvmaf=model=path=vmaf_v0.6.1.json"));
    assert!(filter.contains("[d1][r1]ssim[ss]"));
  }

  #[test]
  fn bundled_model_is_the_default_vmaf_model() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/vmaf/vmaf_v0.6.1.json");
    let text = std::fs::read_to_string(path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(value.get("model_dict").is_some());
  }
}
