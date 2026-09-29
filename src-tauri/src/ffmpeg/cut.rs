//! 视频裁剪：按时间段切割后合并

use std::fs::File;
use std::io::Write;

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::models::TimeSegment;
use crate::ffmpeg::utils::{parse_duration_str, parse_time_from_ffmpeg_output, select_best_encoder};
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;
use crate::utils::cache::{clear_cache_temp_dir, get_cache_temp_dir};
use crate::utils::gpu::get_gpu_info;

/// 解析片段时长，非法输入按错误返回而不是中断进程
fn segment_duration(raw: &str) -> SidecarResult<f64> {
  parse_duration_str(raw).ok_or_else(|| SidecarError::invalid(format!("无效的片段时长: {raw}")))
}

pub async fn run_cut(
  app: &tauri::AppHandle,
  job: &JobCtx,
  video_path: &str,
  output_path: &str,
  segments: Vec<TimeSegment>,
) -> SidecarResult<()> {
  let mut temp_files = Vec::new();
  let temp_dir = get_cache_temp_dir(app)?;

  let gpus = get_gpu_info().await;
  let best = select_best_encoder(&gpus);
  let best_args = best.to_ffmpeg_args();
  let mut duration: f64 = 0.0;
  let mut cut_time: f64 = 0.0;

  for seg in &segments {
    duration += segment_duration(&seg.duration)?;
  }

  for (i, seg) in segments.iter().enumerate() {
    if job.is_canceled().await {
      let _ = clear_cache_temp_dir(app);
      return Err(SidecarError::canceled());
    }

    let current_temp = temp_dir.join(format!("part_{}.mp4", i));
    let temp_name = current_temp.to_string_lossy().into_owned();

    let mut args = Vec::with_capacity(9 + best_args.len());
    args.push("-ss");
    args.push(&seg.start);
    args.push("-i");
    args.push(video_path);
    args.push("-t");
    args.push(&seg.duration);
    args.extend_from_slice(&best_args);
    args.push("-y");
    args.push(&temp_name);
    args.push("-hide_banner");

    tauri_plugin_log::log::info!("ffmpeg {}", args.join(" "));
    let shell = app.shell();
    let (mut rx, child) = shell
      .sidecar("ffmpeg")
      .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?
      .args(args)
      .spawn()
      .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?;

    secure_spawn_step(app, job, child).await?;

    let mut segment_done = false;
    while !segment_done {
      tokio::select! {
        event = rx.recv() => {
          match event {
            Some(CommandEvent::Stderr(data)) => {
              if let Some(current_time) = parse_time_from_ffmpeg_output(&data) {
                job.report_ratio(cut_time + current_time, duration, "转码中...").await;
              }
            }
            Some(CommandEvent::Terminated(status)) => {
              clear_current_child(app).await;
              if status.code == Some(0) {
                temp_files.push(temp_name.clone());
                segment_done = true;
              } else {
                let _ = clear_cache_temp_dir(app);
                return Err(SidecarError::sidecar_exit("ffmpeg", format!("退出代码 {:?}", status.code)));
              }
            }
            None => segment_done = true,
            _ => {}
          }
        }
        _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
          if job.is_canceled().await {
            clear_current_child(app).await;
            let _ = clear_cache_temp_dir(app);
            return Err(SidecarError::canceled());
          }
        }
      }
    }

    cut_time += segment_duration(&seg.duration)?;
  }

  if temp_files.is_empty() {
    let _ = clear_cache_temp_dir(app);
    return Err(SidecarError::invalid("没有裁出视频"));
  }

  if job.is_canceled().await {
    let _ = clear_cache_temp_dir(app);
    return Err(SidecarError::canceled());
  }

  // 创建 concat 列表文件
  let list_file_name = temp_dir.join("concat_list.txt");
  let mut list_file = File::create(&list_file_name)?;
  for path in &temp_files {
    writeln!(list_file, "file '{}'", path)?;
  }
  list_file.flush()?;

  let shell = app.shell();
  let file_path = list_file_name.to_string_lossy().into_owned();
  let args =
    Vec::from(["-f", "concat", "-safe", "0", "-i", &file_path, "-c", "copy", "-y", output_path, "-hide_banner"]);
  let (mut rx, child) = shell
    .sidecar("ffmpeg")
    .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?
    .args(args)
    .spawn()
    .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?;

  secure_spawn_step(app, job, child).await?;

  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Terminated(status) => {
        clear_current_child(app).await;
        let _ = clear_cache_temp_dir(app);

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

  let _ = clear_cache_temp_dir(app);
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  Ok(())
}
