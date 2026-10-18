//! 视频转码：单文件转换

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::graph::{GraphRequest, build_video_filter, explicit_size};
use crate::ffmpeg::info::video_info;
use crate::ffmpeg::models::{AudioPlan, RatePolicy, VideoEncode, encode_args, ensure_audio, plan_audio, push_output};
use crate::ffmpeg::utils::parse_time_from_ffmpeg_output;
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;

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

  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stderr(line) => {
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
          return Err(SidecarError::sidecar_exit("ffmpeg", format!("退出代码 {code}")));
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

pub(crate) fn push_maps(args: &mut Vec<String>, audio: &AudioPlan) {
  args.push("-map".to_string());
  args.push("0:v:0".to_string());
  if let Some(index) = audio.stream_index {
    args.push("-map".to_string());
    args.push(format!("0:{index}?"));
  }
}
