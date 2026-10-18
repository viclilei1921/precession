//! 视频裁剪：按时间段编码后无损拼接

use std::fs::File;
use std::io::Write;

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::convert::push_maps;
use crate::ffmpeg::graph::{GraphRequest, build_video_filter, explicit_size};
use crate::ffmpeg::info::video_info;
use crate::ffmpeg::models::{
  RatePolicy, TimeSegment, VideoContainer, VideoEncode, VideoParams, encode_args, ensure_audio, plan_audio, push_output,
};
use crate::ffmpeg::utils::{FfmpegNotes, parse_duration_str, parse_time_from_ffmpeg_output};
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;
use crate::utils::cache::{clear_cache_temp_dir, get_cache_temp_dir};

fn segment_seconds(raw: &str) -> SidecarResult<f64> {
  parse_duration_str(raw).ok_or_else(|| SidecarError::invalid(format!("无效的片段时间: {raw}")))
}

pub async fn run_cut(
  app: &tauri::AppHandle,
  job: &JobCtx,
  video_path: &str,
  output_path: &str,
  segments: Vec<TimeSegment>,
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
  let mut segment_encode = encode.clone();
  segment_encode.faststart = false;

  let mut pieces = Vec::new();
  for segment in &segments {
    pieces.push((segment_seconds(&segment.start)?, segment_seconds(&segment.duration)?));
  }
  let total: f64 = pieces.iter().map(|(_, duration)| *duration).sum();
  let temp_dir = get_cache_temp_dir(app)?;
  let mut temp_files = Vec::new();
  let mut offset = 0.0;

  for (index, ((start, duration), segment)) in pieces.iter().zip(&segments).enumerate() {
    if job.is_canceled().await {
      let _ = clear_cache_temp_dir(app);
      return Err(SidecarError::canceled());
    }

    let current_temp = temp_dir.join(format!("part_{index}.{}", extension(encode.container)));
    let temp_name = current_temp.to_string_lossy().into_owned();
    let filter = build_video_filter(&GraphRequest {
      encode: &segment_encode,
      path: video_path,
      width: info.width,
      fps: None,
      size,
      offset,
      duration: *duration,
      source_start: Some(*start),
    });

    let mut args = vec![
      "-ss".to_string(),
      segment.start.clone(),
      "-i".to_string(),
      video_path.to_string(),
      "-t".to_string(),
      segment.duration.clone(),
    ];
    if let Some(filter) = &filter {
      args.push("-vf".to_string());
      args.push(filter.clone());
    }
    push_maps(&mut args, &audio);
    args.extend(encode_args(&segment_encode, &audio, width));
    push_output(&mut args, &segment_encode, &temp_name);

    tauri_plugin_log::log::info!("ffmpeg {}", args.join(" "));
    let shell = app.shell();
    let (mut rx, child) = shell
      .sidecar("ffmpeg")
      .map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?
      .args(&args)
      .spawn()
      .map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?;

    secure_spawn_step(app, job, child).await?;

    let mut notes = FfmpegNotes::default();
    let mut segment_done = false;
    while !segment_done {
      tokio::select! {
        event = rx.recv() => {
          match event {
            Some(CommandEvent::Stderr(data)) => {
              notes.push(&data);
              if let Some(current_time) = parse_time_from_ffmpeg_output(&data) {
                job.report_ratio(offset + current_time, total, "转码中...").await;
              }
            }
            Some(CommandEvent::Terminated(status)) => {
              clear_current_child(app).await;
              if status.code == Some(0) {
                temp_files.push(temp_name.clone());
                segment_done = true;
              } else {
                let _ = clear_cache_temp_dir(app);
                let detail = status.code.map(|code| notes.failure(code)).unwrap_or_else(|| "异常退出".to_string());
                return Err(SidecarError::sidecar_exit("ffmpeg", detail));
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

    offset += duration;
  }

  if temp_files.is_empty() {
    let _ = clear_cache_temp_dir(app);
    return Err(SidecarError::invalid("没有裁出视频"));
  }
  if job.is_canceled().await {
    let _ = clear_cache_temp_dir(app);
    return Err(SidecarError::canceled());
  }

  let list_file_name = temp_dir.join("concat_list.txt");
  let mut list_file = File::create(&list_file_name)?;
  for path in &temp_files {
    writeln!(list_file, "file '{}'", path.replace('\'', "'\\''"))?;
  }
  list_file.flush()?;

  let file_path = list_file_name.to_string_lossy().into_owned();
  let mut args = vec![
    "-f".to_string(),
    "concat".to_string(),
    "-safe".to_string(),
    "0".to_string(),
    "-i".to_string(),
    file_path,
    "-c".to_string(),
    "copy".to_string(),
  ];
  if matches!(encode.video, VideoParams::Hevc { .. })
    && matches!(encode.container, VideoContainer::Mp4 | VideoContainer::Mov)
  {
    args.push("-tag:v".to_string());
    args.push("hvc1".to_string());
  }
  if encode.faststart && matches!(encode.container, VideoContainer::Mp4 | VideoContainer::Mov) {
    args.push("-movflags".to_string());
    args.push("+faststart".to_string());
  }
  push_output(&mut args, encode, output_path);

  let shell = app.shell();
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
      CommandEvent::Stderr(line) | CommandEvent::Stdout(line) => notes.push(&line),
      CommandEvent::Terminated(status) => {
        clear_current_child(app).await;
        let _ = clear_cache_temp_dir(app);
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

  let _ = clear_cache_temp_dir(app);
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }
  Ok(())
}

fn extension(container: VideoContainer) -> &'static str {
  match container {
    VideoContainer::Mp4 => "mp4",
    VideoContainer::Mov => "mov",
    VideoContainer::MpegTs => "ts",
  }
}
