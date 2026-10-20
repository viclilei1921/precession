//! 视频合并：统一分辨率和帧率后拼成一条。
//! 帧率取各段的上中位数，忽略传入值。尺寸空则取最大宽高并补边。
//! 每段各选一条音轨，再用 `aresample`、`aformat` 统一到 48000 和同一声道数。标题和字幕每段各烧。

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::graph::{GraphRequest, build_video_filter, canvas_size, explicit_size, file_label};
use crate::ffmpeg::info;
use crate::ffmpeg::models::{
  AudioMode, AudioPlan, RatePolicy, VideoEncode, VideoInfo, encode_args, plan_audio, push_output,
};
use crate::ffmpeg::utils::{FfmpegNotes, calculate_target_params, parse_time_from_ffmpeg_output};
use crate::sidecar::cache::clear_cache_temp_dir;
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;

/// 合并时的一段输入
///
/// 读失败的输入不会放进来。采样率按 48000 规划。
///
/// # 字段
/// - `path`: 源视频路径
/// - `info`: 探测到的视频信息
/// - `audio`: 这一段的音轨计划
///
/// # 示例
/// ```ignore
/// let piece = Piece {
///   path: "a.mp4".into(),
///   info: VideoInfo::default(),
///   audio: plan_audio(&AudioSelection::default(), &[], RatePolicy::Force(48000)),
/// };
/// assert!(piece.audio.stream_index.is_none());
/// ```
struct Piece {
  /// 源视频路径
  path: String,
  /// 探测到的视频信息
  info: VideoInfo,
  /// 这一段的音轨计划
  audio: AudioPlan,
}

/// 把多段视频收成同一画面和帧率后拼成一条
///
/// 读失败的输入只记日志并跳过。未指定声道且不是丢掉音频时，全是单声道就用 1，否则用 2。帧率用上中位数，不大于 0 时用 30。进度消息带上当前段的文件名。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `job`: 任务执行期句柄
/// - `inputs`: 源视频
/// - `output_path`: 输出路径
/// - `encode`: 编码配置
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 输入为空、没有可读取的视频、要保留音频但某段没有音轨、滤镜为空、取消、启动失败或非 0 退出时返回 `SidecarError`。前四项是 `Invalid`，取消是 `Canceled`，启动失败是 `Spawn`，非 0 退出是 `Exit`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// run_merge_smart(app, job, inputs, output_path, encode).await?;
/// ```
pub async fn run_merge_smart(
  app: &tauri::AppHandle,
  job: &JobCtx,
  inputs: Vec<String>,
  output_path: &str,
  encode: &VideoEncode,
) -> SidecarResult<()> {
  if inputs.is_empty() {
    return Err(SidecarError::invalid("没有可合并的视频"));
  }

  let mut pieces = Vec::new();
  for input in &inputs {
    match info::video_info(app, input).await {
      Ok(meta) => {
        let audio = plan_audio(&encode.audio, &meta.audio_tracks, RatePolicy::Force(48000));
        if audio.fallback {
          tauri_plugin_log::log::warn!("音轨序号不存在，改用第一条: {input}");
        }
        if !matches!(encode.audio.mode, AudioMode::Drop) && audio.stream_index.is_none() {
          return Err(SidecarError::invalid(format!("没有音轨: {input}")));
        }
        pieces.push(Piece { path: input.clone(), info: meta, audio });
      }
      Err(err) => tauri_plugin_log::log::error!("读取视频失败 {input}: {err}"),
    }
  }
  if pieces.is_empty() {
    return Err(SidecarError::invalid("没有可读取的视频"));
  }

  if !matches!(encode.audio.mode, AudioMode::Drop) && encode.audio.channels.is_none() {
    let channels = if pieces.iter().all(|piece| piece.audio.channels <= 1) { 1 } else { 2 };
    for piece in &mut pieces {
      piece.audio.channels = channels;
    }
  }

  let infos: Vec<VideoInfo> = pieces.iter().map(|piece| piece.info.clone()).collect();
  let (max_w, max_h, median_fps) = calculate_target_params(&infos);
  let size = explicit_size(encode).unwrap_or_else(|| canvas_size(max_w, max_h));
  let fps = if median_fps > 0.0 { median_fps } else { 30.0 };
  let total: f64 = pieces.iter().map(|piece| piece.info.duration).sum();
  tauri_plugin_log::log::info!("合并目标 {}x{} | 帧率 {fps}", size.width, size.height);

  let mut filter = String::new();
  let mut offset = 0.0;
  let keep_audio = pieces.iter().any(|piece| piece.audio.stream_index.is_some());

  for (index, piece) in pieces.iter().enumerate() {
    let chain = build_video_filter(&GraphRequest {
      encode,
      path: &piece.path,
      width: piece.info.width,
      fps: Some(fps),
      size: Some(size),
      offset,
      duration: piece.info.duration,
      source_start: None,
    })
    .ok_or_else(|| SidecarError::invalid("合并滤镜为空"))?;
    filter.push_str(&format!("[{index}:v]{chain}[v{index}];"));
    if let Some(stream) = piece.audio.stream_index {
      let layout = if piece.audio.channels <= 1 { "mono" } else { "stereo" };
      filter.push_str(&format!(
        "[{index}:{stream}]aresample=48000,aformat=sample_fmts=fltp:channel_layouts={layout}[a{index}];"
      ));
    }
    offset += piece.info.duration;
  }

  for index in 0..pieces.len() {
    filter.push_str(&format!("[v{index}]"));
    if keep_audio {
      filter.push_str(&format!("[a{index}]"));
    }
  }
  if keep_audio {
    filter.push_str(&format!("concat=n={}:v=1:a=1[outv][outa]", pieces.len()));
  } else {
    filter.push_str(&format!("concat=n={}:v=1:a=0[outv]", pieces.len()));
  }
  let mut args = Vec::new();
  for piece in &pieces {
    args.push("-i".to_string());
    args.push(piece.path.clone());
  }
  args.push("-filter_complex".to_string());
  args.push(filter);
  args.push("-map".to_string());
  args.push("[outv]".to_string());
  let audio = if keep_audio {
    args.push("-map".to_string());
    args.push("[outa]".to_string());
    AudioPlan {
      stream_index: Some(0),
      channels: pieces.first().map(|piece| piece.audio.channels).unwrap_or(2),
      sample_rate: 48000,
      bitrate_kbps: encode.audio.bitrate_kbps,
      fallback: false,
    }
  } else {
    AudioPlan {
      stream_index: None,
      channels: 2,
      sample_rate: 0,
      bitrate_kbps: encode.audio.bitrate_kbps,
      fallback: false,
    }
  };
  args.extend(encode_args(encode, &audio, size.width));
  push_output(&mut args, encode, output_path);

  if job.is_canceled().await {
    let _ = clear_cache_temp_dir(app);
    return Err(SidecarError::canceled());
  }

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
  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stderr(line) => {
        notes.push(&line);
        if job.is_canceled().await {
          let _ = clear_cache_temp_dir(app);
          return Err(SidecarError::canceled());
        }
        if let Some(current_time) = parse_time_from_ffmpeg_output(&line) {
          let name = name_at(&pieces, current_time);
          job.report_ratio(current_time, total, &format!("正在合并：{name}")).await;
        }
      }
      CommandEvent::Terminated(status) => {
        clear_current_child(app).await;
        let _ = clear_cache_temp_dir(app);
        if let Some(code) = status.code {
          if code == 0 {
            return Ok(());
          }
          return Err(SidecarError::sidecar_exit("ffmpeg", notes.failure(code)));
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

/// 按累计时长找出当前时间落在哪一段，返回它的文件名
///
/// 时间不落在任一段时，返回第一段的文件名；一段都没有则返回空字符串。
///
/// # 参数
/// - `pieces`: 已读到的片段
/// - `current_time`: 成片上的当前秒数
///
/// # 返回值
/// 文件名
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let name = name_at(&pieces, 0.0);
/// ```
fn name_at(pieces: &[Piece], current_time: f64) -> String {
  let mut elapsed = 0.0;
  for piece in pieces {
    elapsed += piece.info.duration;
    if current_time < elapsed {
      return file_label(&piece.path);
    }
  }
  pieces.first().map(|piece| file_label(&piece.path)).unwrap_or_default()
}
