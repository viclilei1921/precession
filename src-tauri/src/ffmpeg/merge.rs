//! 视频智能合并：多文件合并为统一分辨率/帧率

use std::fs::File;
use std::io::Write;
use std::path::Path;

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::models::VideoInfo;
use crate::ffmpeg::utils::{calculate_target_params, parse_time_from_ffmpeg_output, select_best_encoder};
use crate::ffmpeg::video_info;
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;
use crate::utils::cache::{clear_cache_temp_dir, get_cache_dir};
use crate::utils::font::get_default_font_path;
use crate::utils::gpu::get_gpu_info;

pub async fn run_merge_smart(
  app: &tauri::AppHandle,
  job: &JobCtx,
  inputs: Vec<String>,
  output_path: &str,
  draw_filename: bool,
) -> SidecarResult<()> {
  if inputs.is_empty() {
    return Err(SidecarError::invalid("没有可合并的视频"));
  }

  let mut valid_tasks: Vec<(&str, VideoInfo)> = Vec::new();
  for input in &inputs {
    match video_info(app, input).await {
      Ok(meta) => valid_tasks.push((input.as_str(), meta)),
      Err(e) => tauri_plugin_log::log::error!("读取视频失败 {input}: {e}"),
    }
  }

  if valid_tasks.is_empty() {
    return Err(SidecarError::invalid("没有可读取的视频"));
  }

  let videos_info: Vec<VideoInfo> = valid_tasks.iter().map(|(_, m)| m.clone()).collect();
  let (target_w, target_h, target_fps) = calculate_target_params(&videos_info);
  let target_duration: f64 = videos_info.iter().map(|m| m.duration).sum();

  tauri_plugin_log::log::info!("合并目标 {target_w}x{target_h} | 帧率 {target_fps}");

  let gpus = get_gpu_info().await;
  let best = select_best_encoder(&gpus);
  let mut best_args = best.to_ffmpeg_args();
  best_args.pop();
  best_args.pop();
  best_args.push("-an");

  let cache_dir = get_cache_dir(app)?;
  let temp_dir = cache_dir.join("temp");
  let _ = std::fs::create_dir(&temp_dir);
  let filter_file_name = temp_dir.join("filter.txt");
  let mut filter_file = File::create(&filter_file_name)?;

  let font_path = get_default_font_path();
  let mut filter_complex = String::new();
  let mut args = Vec::with_capacity(8 + best_args.len() + inputs.len() * 2);

  for (i, input_path) in inputs.iter().enumerate() {
    args.push("-i");
    args.push(input_path);

    let drawtext_part = if draw_filename {
      let filename = Path::new(input_path).file_name().unwrap().to_string_lossy();
      let clean_name = filename.replace(":", "\\:").replace("'", "");
      let target_w_f = target_w as f64;
      let target_h_f = target_h as f64;
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

    let chain = format!(
      "[{i}:v]fps={fps},scale='trunc(iw*sar/2)*2':'trunc(ih/2)*2',setsar=1,scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,setsar=1{text_filter}[v{i}];[{i}:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo[a{i}];",
      i = i,
      fps = target_fps,
      w = target_w,
      h = target_h,
      text_filter = drawtext_part
    );
    filter_complex.push_str(&chain);
  }

  for i in 0..valid_tasks.len() {
    filter_complex.push_str(&format!("[v{i}][a{i}]", i = i));
  }
  filter_complex.push_str(&format!("concat=n={}:v=1:a=1[outv][outa]", valid_tasks.len()));
  writeln!(filter_file, "{}", filter_complex)?;
  filter_file.flush()?;

  let filter_file_name_str = filter_file_name.to_string_lossy();
  args.push("-/filter_complex");
  args.push(filter_file_name_str.as_ref());
  args.push("-map");
  args.push("[outv]");
  args.push("-map");
  args.push("[outa]");
  args.extend_from_slice(&best_args);
  args.push("-y");
  args.push(output_path);

  if job.is_canceled().await {
    let _ = clear_cache_temp_dir(app);
    return Err(SidecarError::canceled());
  }

  tauri_plugin_log::log::info!("ffmpeg {}", args.join(" "));

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
          let video_name = segment_name_at(&valid_tasks, current_time);
          job.report_ratio(current_time, target_duration, &format!("正在合并：{video_name}")).await;
        }
      }
      CommandEvent::Terminated(status) => {
        clear_current_child(app).await;
        let _ = clear_cache_temp_dir(app);

        if let Some(code) = status.code {
          if code == 0 {
            return Ok(());
          }
          return Err(SidecarError::sidecar_exit("ffmpeg", format!("退出代码 {code}")));
        }
        break;
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

/// 按累计时长定位 `current_time` 落在哪个输入文件上，用于进度消息
fn segment_name_at(valid_tasks: &[(&str, VideoInfo)], current_time: f64) -> String {
  let mut elapsed = 0.0;
  for (_, info) in valid_tasks {
    elapsed += info.duration;
    if current_time < elapsed {
      return file_label(&info.path);
    }
  }
  valid_tasks.first().map(|(_, info)| file_label(&info.path)).unwrap_or_default()
}

fn file_label(path: &str) -> String {
  Path::new(path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string())
}
