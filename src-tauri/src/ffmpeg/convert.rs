//! 视频转码：单文件转换。
//! 容器、编码、音频码率、faststart、帧率、尺寸、音轨都可改。帧率和尺寸空则跟源。标题和字幕烧整片。
//! 采样率没指定时，44100 和 48000 保持原样，其他值改成 48000。

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::graph::{GraphRequest, build_video_filter, explicit_size};
use crate::ffmpeg::info::video_info;
use crate::ffmpeg::models::{AudioPlan, RatePolicy, VideoEncode, encode_args, ensure_audio, plan_audio, push_output};
use crate::ffmpeg::utils::{FfmpegNotes, parse_time_from_ffmpeg_output};
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;

/// 把单个视频按编码配置转成输出文件
///
/// 音轨序号不存在时改用第一条并记警告。进度按输出里的 `time=` 相对时长上报。退出码 0 视为成功。通道结束且未取消时也返回成功。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `job`: 任务执行期句柄
/// - `video_path`: 源视频
/// - `output_path`: 输出路径
/// - `encode`: 编码配置
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 读视频、要保留音频但没有音轨、启动失败、非 0 退出或取消时返回 `SidecarError`。没有音轨是 `Invalid`，启动失败是 `Spawn`，非 0 退出是 `Exit`，取消是 `Canceled`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// run_convert(app, job, video_path, output_path, encode).await?;
/// ```
pub async fn run_convert(
  app: &tauri::AppHandle,
  job: &JobCtx,
  video_path: &str,
  output_path: &str,
  encode: &VideoEncode,
) -> SidecarResult<()> {
  let info = video_info(app, video_path).await?;
  let audio = plan_audio(&encode.audio, &info.audio_tracks, RatePolicy::Source);
  if audio.fallback {
    tauri_plugin_log::log::warn!("音轨序号不存在，改用第一条: {video_path}");
  }
  ensure_audio(encode.audio.mode, &audio, video_path).map_err(SidecarError::invalid)?;
  let size = explicit_size(encode);
  let width = size.map(|item| item.width).unwrap_or(info.width);
  let filter = build_video_filter(&GraphRequest {
    encode,
    path: video_path,
    width: info.width,
    fps: None,
    size,
    offset: 0.0,
    duration: info.duration,
    source_start: None,
  });

  let mut args = vec!["-i".to_string(), video_path.to_string()];
  if let Some(filter) = &filter {
    args.push("-vf".to_string());
    args.push(filter.clone());
  }
  push_maps(&mut args, &audio);
  args.extend(encode_args(encode, &audio, width));
  push_output(&mut args, encode, output_path);

  let shell = app.shell();
  tauri_plugin_log::log::info!("ffmpeg {}", args.join(" "));
  let (mut rx, child) = shell
    .sidecar("ffmpeg")
    .map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?
    .args(&args)
    .spawn()
    .map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?;

  secure_spawn_step(app, job, child).await?;

  let mut notes = FfmpegNotes::default();
  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stderr(line) => {
        notes.push(&line);
        if job.is_canceled().await {
          break;
        }
        if let Some(current_time) = parse_time_from_ffmpeg_output(&line) {
          job.report_ratio(current_time, info.duration, "转码中...").await;
        }
      }
      CommandEvent::Terminated(status) => {
        clear_current_child(app).await;
        if let Some(code) = status.code {
          if code == 0 {
            return Ok(());
          }
          return Err(SidecarError::sidecar_exit("ffmpeg", notes.failure(code)));
        }
      }
      _ => {}
    }
  }

  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  Ok(())
}

/// 写入视频流映射，有音轨序号时再映射那一条
///
/// 视频固定 `-map 0:v:0`。音轨写成 `-map 0:{index}?`。
///
/// # 参数
/// - `args`: 参数列表
/// - `audio`: 音轨计划
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut args = Vec::new();
/// push_maps(&mut args, &audio);
/// ```
pub(crate) fn push_maps(args: &mut Vec<String>, audio: &AudioPlan) {
  args.push("-map".to_string());
  args.push("0:v:0".to_string());
  if let Some(index) = audio.stream_index {
    args.push("-map".to_string());
    args.push(format!("0:{index}?"));
  }
}
