//! 视频智能追加：将新视频追加到基准视频末尾

use std::fs::File;
use std::io::Write;
use std::path::Path;

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::utils::{parse_time_from_ffmpeg_output, select_best_encoder};
use crate::ffmpeg::video_info;
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;
use crate::utils::cache::{clear_cache_temp_dir, get_cache_temp_dir};
use crate::utils::font::get_default_font_path;
use crate::utils::gpu::get_gpu_info;

pub async fn run_append_smart(
  app: &tauri::AppHandle,
  job: &JobCtx,
  base: &str,
  inputs: Vec<String>,
  output_path: &str,
  draw_filename: bool,
) -> SidecarResult<()> {
  if inputs.is_empty() {
    return Err(SidecarError::invalid("没有要追加的视频"));
  }

  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  let base_info = video_info(app, base).await?;
  let temp_dir = get_cache_temp_dir(app)?;
  let mut ts_files: Vec<String> = Vec::new();

  // 步骤 1: 将基准视频无损封装为 TS
  let base_ts_path = temp_dir.join("part_base.ts").to_string_lossy().into_owned();
  let bsf_filter = if base_info.video_codec.contains("hevc") || base_info.video_codec.contains("h265") {
    "hevc_mp4toannexb"
  } else {
    "h264_mp4toannexb"
  };

  let remux_args =
    vec!["-i", base, "-c", "copy", "-bsf:v", bsf_filter, "-f", "mpegts", "-y", &base_ts_path, "-hide_banner"];

  tauri_plugin_log::log::info!("正在把基准视频封装为 TS");
  let shell = app.shell();
  let (mut rx, _) = shell
    .sidecar("ffmpeg")
    .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?
    .args(remux_args)
    .spawn()
    .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?;

  while let Some(event) = rx.recv().await {
    if let CommandEvent::Terminated(status) = event {
      if status.code != Some(0) {
        let _ = clear_cache_temp_dir(app);
        return Err(SidecarError::sidecar_exit("ffmpeg", "基准视频封装失败"));
      }
      break;
    }
  }
  ts_files.push(base_ts_path);

  // 步骤 2: 处理新视频并转码为 TS
  let gpus = get_gpu_info().await;
  let best = select_best_encoder(&gpus);
  let best_args = best.to_ffmpeg_args();
  let font_path = get_default_font_path();

  for (i, input_path) in inputs.iter().enumerate() {
    if job.is_canceled().await {
      let _ = clear_cache_temp_dir(app);
      return Err(SidecarError::canceled());
    }

    let current_ts_path = temp_dir.join(format!("part_new_{}.ts", i)).to_string_lossy().into_owned();
    let input_info = video_info(app, input_path).await.unwrap_or_default();
    let sample_rate = if base_info.audio_sample_rate > 0 { base_info.audio_sample_rate } else { 48000 };

    let drawtext = if draw_filename {
      let filename = Path::new(input_path).file_name().unwrap().to_string_lossy();
      let clean_name = filename.replace(":", "\\:").replace("'", "");
      let target_w_f = base_info.width as f64;
      let target_h_f = base_info.height as f64;
      let font_size = 24.0 * target_w_f / 1920.0;
      let x = 10.0 * target_w_f / 1920.0;
      let y = 10.0 * target_h_f / 1080.0;
      format!(
        ",drawtext=fontfile={}:text={}:fontcolor=white:fontsize={}:x={}:y={}:box=1:boxcolor=black@0.0",
        font_path, clean_name, font_size as u32, x as u32, y as u32
      )
    } else {
      String::new()
    };

    let filter_complex = format!(
      "[0:v]fps={fps},scale='trunc(iw*sar/2)*2':'trunc(ih/2)*2',setsar=1,scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,setsar=1{text}[v];[0:a]aresample={ar},aformat=sample_fmts=fltp:channel_layouts=stereo[a]",
      fps = base_info.fps,
      w = base_info.width,
      h = base_info.height,
      text = drawtext,
      ar = sample_rate
    );

    let mut args = vec!["-i", input_path, "-filter_complex", &filter_complex, "-map", "[v]", "-map", "[a]"];
    args.extend_from_slice(&best_args);
    args.push("-bsf:v");
    args.push(bsf_filter);
    args.push("-f");
    args.push("mpegts");
    args.push("-y");
    args.push(&current_ts_path);
    args.push("-hide_banner");

    tauri_plugin_log::log::info!("正在转码第 {} 段", i + 1);
    let shell = app.shell();
    let (mut rx, child) = shell
      .sidecar("ffmpeg")
      .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?
      .args(args)
      .spawn()
      .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?;

    secure_spawn_step(app, job, child).await?;

    while let Some(event) = rx.recv().await {
      match event {
        CommandEvent::Stderr(line) => {
          if job.is_canceled().await {
            let _ = clear_cache_temp_dir(app);
            return Err(SidecarError::canceled());
          }
          if let Some(current_time) = parse_time_from_ffmpeg_output(&line) {
            let message = format!("正在处理第 {}/{} 段", i + 1, inputs.len());
            job.report_ratio(current_time, input_info.duration, &message).await;
          }
        }
        CommandEvent::Terminated(status) => {
          clear_current_child(app).await;
          if status.code == Some(0) {
            ts_files.push(current_ts_path.clone());
          } else {
            let _ = clear_cache_temp_dir(app);
            return Err(SidecarError::sidecar_exit("ffmpeg", format!("退出代码 {:?}", status.code)));
          }
          break;
        }
        _ => {}
      }
    }
  }

  // 步骤 3: Concat 合并所有 TS 并转回 MP4
  let list_file_name = temp_dir.join("ts_concat_list.txt");
  let mut list_file = File::create(&list_file_name)?;
  for path in &ts_files {
    writeln!(list_file, "file '{}'", path.replace("'", "'\\''"))?;
  }
  list_file.flush()?;

  let list_file_path_str = list_file_name.to_string_lossy().into_owned();
  let concat_args = vec![
    "-f",
    "concat",
    "-safe",
    "0",
    "-i",
    &list_file_path_str,
    "-c",
    "copy",
    "-tag:v",
    "hvc1",
    "-bsf:a",
    "aac_adtstoasc",
    "-y",
    output_path,
    "-hide_banner",
  ];

  tauri_plugin_log::log::info!("正在合并为 MP4");
  let shell = app.shell();
  let (mut rx, _) = shell
    .sidecar("ffmpeg")
    .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?
    .args(concat_args)
    .spawn()
    .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?;

  while let Some(event) = rx.recv().await {
    if let CommandEvent::Terminated(status) = event {
      let _ = clear_cache_temp_dir(app);

      if let Some(code) = status.code {
        if code == 0 {
          return Ok(());
        }
        return Err(SidecarError::sidecar_exit("ffmpeg", format!("退出代码 {code}")));
      }
    }
  }

  let _ = clear_cache_temp_dir(app);
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  Ok(())
}
