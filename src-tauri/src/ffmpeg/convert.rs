//! 视频转码：单文件转换

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::utils::{parse_time_from_ffmpeg_output, select_best_encoder};
use crate::ffmpeg::video_info;
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;
use crate::utils::gpu::get_gpu_info;

pub async fn run_convert(
  app: &tauri::AppHandle,
  job: &JobCtx,
  video_path: &str,
  output_path: &str,
  target_fps: Option<u32>,
) -> SidecarResult<()> {
  let video_info = video_info(app, video_path).await?;
  let total_duration = video_info.duration;

  let gpus = get_gpu_info().await;
  let best = select_best_encoder(&gpus);
  let best_args = best.to_ffmpeg_args();

  let mut args: Vec<String> = Vec::with_capacity(8 + best_args.len());
  args.push("-i".to_string());
  args.push(video_path.to_string());
  for a in &best_args {
    args.push((*a).to_string());
  }
  if let Some(fps) = target_fps.filter(|f| *f > 0 && *f <= 240) {
    // WebM（MediaRecorder）常见 VFR/PTS 异常，不指定时 HEVC 封装后常表现为约 23fps
    args.push("-fps_mode".to_string());
    args.push("cfr".to_string());
    args.push("-r".to_string());
    args.push(fps.to_string());
  }
  args.push("-y".to_string());
  args.push(output_path.to_string());
  args.push("-hide_banner".to_string());

  let shell = app.shell();
  tauri_plugin_log::log::info!("ffmpeg {}", args.join(" "));
  let (mut rx, child) = shell
    .sidecar("ffmpeg")
    .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?
    .args(&args)
    .spawn()
    .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?;

  secure_spawn_step(app, job, child).await?;

  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stderr(line) => {
        if job.is_canceled().await {
          break;
        }
        if let Some(current_time) = parse_time_from_ffmpeg_output(&line) {
          job.report_ratio(current_time, total_duration, "转码中...").await;
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
