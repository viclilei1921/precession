//! 视频追加：先把各段收成同一种编码、同尺寸、同帧率、同像素格式，再用 MPEG-TS 拼好，最后复制进目标容器。
//! VP9 的中间文件用 MP4，因为 VP9 不能写入 MPEG-TS。
//! 帧率跟基准，忽略传入值。尺寸空则跟基准。音轨选择和合并一样，每段各选一条。
//! 基准已经和目标一致，而且不用在基准上烧标题或字幕时，基准直接复制，音频也复制。
//! 否则基准也走同一套参数重编码。默认 AV1 时，常见的 H.264 基准会重编码。这一步不单独做开关。
//! 新片段各烧标题和字幕。基准要烧就必须重编码。
//! 采样率在基准被复制时跟基准，基准被重编码时用 48000。

use std::fs::File;
use std::io::Write;

use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::convert::push_maps;
use crate::ffmpeg::graph::{GraphRequest, build_video_filter, explicit_size, output_size};
use crate::ffmpeg::info::video_info;
use crate::ffmpeg::models::{
  AudioMode, AudioPlan, RatePolicy, VideoContainer, VideoEncode, VideoInfo, VideoParams, encode_args, plan_audio,
  push_output, source_codec_matches,
};
use crate::ffmpeg::utils::{FfmpegNotes, parse_time_from_ffmpeg_output};
use crate::sidecar::cache::{clear_cache_temp_dir, get_cache_temp_dir};
use crate::sidecar::{SidecarError, SidecarResult, clear_current_child, secure_spawn_step};
use crate::task::JobCtx;

struct CopyPlan {
  keep_audio: bool,
  sample_rate: u32,
  channels: u8,
  stream_index: Option<u32>,
}

/// 把新片段接到基准视频后面
///
/// 基准与目标一致且不用烧标题或字幕时直接复制。否则基准也重编码。VP9 的中间文件用 MP4，其他用 MPEG-TS。重编码且未指定声道时，全是单声道就用 1，否则用 2。最后用 concat 复制进目标容器。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `job`: 任务执行期句柄
/// - `base`: 基准视频
/// - `inputs`: 要追加的视频
/// - `output_path`: 输出路径
/// - `encode`: 编码配置
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 没有要追加的视频、取消、没有音轨、写清单失败、启动失败、封装或编码非 0 退出时返回 `SidecarError`。空列表和没有音轨是 `Invalid`，取消是 `Canceled`，写清单失败是 `Io`，启动失败是 `Spawn`，非 0 退出是 `Exit`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// run_append_smart(app, job, base, inputs, output_path, encode).await?;
/// ```
pub async fn run_append_smart(
  app: &tauri::AppHandle,
  job: &JobCtx,
  base: &str,
  inputs: Vec<String>,
  output_path: &str,
  encode: &VideoEncode,
) -> SidecarResult<()> {
  if inputs.is_empty() {
    return Err(SidecarError::invalid("没有要追加的视频"));
  }
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  let base_info = video_info(app, base).await?;
  let (width, height) = output_size(encode, &base_info);
  let fps = if base_info.fps > 0.0 { base_info.fps } else { 30.0 };
  let copy = copy_plan(&base_info, encode, width, height);
  let size = explicit_size(encode).or_else(|| {
    if copy.is_some() { None } else { Some(crate::ffmpeg::graph::canvas_size(width, height)) }
  });
  let temp_dir = get_cache_temp_dir(app)?;
  let (muxer, extension) = intermediate(encode);
  let mut parts: Vec<String> = Vec::new();
  let mut segment_encode = encode.clone();
  segment_encode.faststart = false;
  segment_encode.container = if muxer == "mp4" { VideoContainer::Mp4 } else { VideoContainer::MpegTs };

  let base_path = temp_dir.join(format!("part_base.{extension}")).to_string_lossy().into_owned();
  let mut planned: Vec<(String, VideoInfo, AudioPlan, String)> = Vec::new();
  if let Some(plan) = &copy {
    remux_base(app, base, &base_info, plan, muxer, &base_path).await?;
    parts.push(base_path);
  } else {
    let audio = segment_audio(encode, &base_info, None)?;
    planned.push((base.to_string(), base_info.clone(), audio, base_path));
  }

  for (index, input) in inputs.iter().enumerate() {
    if job.is_canceled().await {
      let _ = clear_cache_temp_dir(app);
      return Err(SidecarError::canceled());
    }
    let info = video_info(app, input).await?;
    let audio = segment_audio(encode, &info, copy.as_ref())?;
    let part = temp_dir.join(format!("part_new_{index}.{extension}")).to_string_lossy().into_owned();
    planned.push((input.clone(), info, audio, part));
  }

  // 重编码时各段声道必须一致，否则后面的无损拼接对不上。
  if copy.is_none() && encode.audio.channels.is_none() && !matches!(encode.audio.mode, AudioMode::Drop) {
    let channels = if planned.iter().all(|(_, _, audio, _)| audio.channels <= 1) { 1 } else { 2 };
    for (_, _, audio, _) in &mut planned {
      audio.channels = channels;
    }
  }

  let mut offset = if copy.is_some() { base_info.duration } else { 0.0 };
  let new_count = inputs.len();
  for (index, (input, info, audio, part)) in planned.into_iter().enumerate() {
    if job.is_canceled().await {
      let _ = clear_cache_temp_dir(app);
      return Err(SidecarError::canceled());
    }
    let message = if copy.is_none() && index == 0 {
      "正在处理基准".to_string()
    } else {
      let shown = if copy.is_none() { index } else { index + 1 };
      format!("正在处理第 {shown}/{new_count} 段")
    };
    encode_part(
      app,
      job,
      &input,
      &info,
      &segment_encode,
      &audio,
      size.or_else(|| Some(crate::ffmpeg::graph::canvas_size(width, height))),
      fps,
      offset,
      None,
      muxer,
      &part,
      &message,
    )
    .await?;
    parts.push(part);
    offset += info.duration;
  }

  let list_path = temp_dir.join("ts_concat_list.txt");
  let mut list_file = File::create(&list_path)?;
  for path in &parts {
    writeln!(list_file, "file '{}'", path.replace('\'', "'\\''"))?;
  }
  list_file.flush()?;

  let list = list_path.to_string_lossy().into_owned();
  let mut args = vec![
    "-f".to_string(),
    "concat".to_string(),
    "-safe".to_string(),
    "0".to_string(),
    "-i".to_string(),
    list,
    "-c".to_string(),
    "copy".to_string(),
  ];
  if muxer == "mpegts"
    && matches!(encode.container, VideoContainer::Mp4 | VideoContainer::Mov)
    && copy.as_ref().is_none_or(|plan| plan.keep_audio)
    && !matches!(encode.audio.mode, AudioMode::Drop)
  {
    args.extend(["-bsf:a".to_string(), "aac_adtstoasc".to_string()]);
  }
  if matches!(encode.video, VideoParams::Hevc { .. })
    && matches!(encode.container, VideoContainer::Mp4 | VideoContainer::Mov)
  {
    args.extend(["-tag:v".to_string(), "hvc1".to_string()]);
  }
  if encode.faststart && matches!(encode.container, VideoContainer::Mp4 | VideoContainer::Mov) {
    args.extend(["-movflags".to_string(), "+faststart".to_string()]);
  }
  push_output(&mut args, encode, output_path);

  tauri_plugin_log::log::info!("ffmpeg {}", args.join(" "));
  let shell = app.shell();
  let (mut rx, _) = shell
    .sidecar("ffmpeg")
    .map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?
    .args(&args)
    .spawn()
    .map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?;

  let mut notes = FfmpegNotes::default();
  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stderr(line) | CommandEvent::Stdout(line) => notes.push(&line),
      CommandEvent::Terminated(status) => {
        let _ = clear_cache_temp_dir(app);
        if status.code == Some(0) {
          return Ok(());
        }
        let detail = status.code.map(|code| notes.failure(code)).unwrap_or_else(|| "异常退出".to_string());
        return Err(SidecarError::sidecar_exit("ffmpeg", detail));
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

/// 选择中间容器和扩展名
///
/// VP9 用 `mp4` / `mp4`，其他用 `mpegts` / `ts`。
///
/// # 参数
/// - `encode`: 编码配置
///
/// # 返回值
/// `(封装名, 扩展名)`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let (muxer, ext) = intermediate(&VideoEncode::default());
/// assert_eq!((muxer, ext), ("mpegts", "ts"));
/// ```
fn intermediate(encode: &VideoEncode) -> (&'static str, &'static str) {
  match encode.video {
    VideoParams::Vp9 { .. } => ("mp4", "mp4"),
    _ => ("mpegts", "ts"),
  }
}

/// 判断基准能不能直接复制
///
/// 有标题或字幕、编码或像素格式对不上、宽高对不上时返回 `None`。丢掉音频时可以复制且不保留音频。保留音频时要求选中轨是 AAC、1 或 2 声道、44100 或 48000，并且和指定的声道、采样率一致。
///
/// # 参数
/// - `info`: 基准视频信息
/// - `encode`: 编码配置
/// - `width`: 输出宽度
/// - `height`: 输出高度
///
/// # 返回值
/// 可以复制时返回复制计划
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let plan = copy_plan(&info, &encode, width, height);
/// ```
fn copy_plan(info: &VideoInfo, encode: &VideoEncode, width: u32, height: u32) -> Option<CopyPlan> {
  if encode.title.is_some() || encode.subtitle.is_some() {
    return None;
  }
  if !source_codec_matches(&info.video_codec, &encode.video) || info.pix_fmt != encode.video.pix_fmt() {
    return None;
  }
  if info.width != width || info.height != height {
    return None;
  }
  if matches!(encode.audio.mode, AudioMode::Drop) {
    return Some(CopyPlan { keep_audio: false, sample_rate: 0, channels: 2, stream_index: None });
  }
  let plan = plan_audio(&encode.audio, &info.audio_tracks, RatePolicy::Source);
  let track = info.audio_tracks.iter().find(|track| Some(track.index) == plan.stream_index)?;
  let aac = track.codec.eq_ignore_ascii_case("aac");
  let channels_ok = track.channels == 1 || track.channels == 2;
  let rate_ok = track.sample_rate == 44100 || track.sample_rate == 48000;
  let channels_match = encode.audio.channels.is_none_or(|channels| channels == track.channels as u8);
  let rate_match = encode.audio.sample_rate.is_none_or(|rate| rate == track.sample_rate);
  if aac && channels_ok && rate_ok && channels_match && rate_match {
    Some(CopyPlan {
      keep_audio: true,
      sample_rate: track.sample_rate,
      channels: track.channels as u8,
      stream_index: Some(track.index),
    })
  } else {
    None
  }
}

/// 为一段视频选定音轨计划
///
/// 基准复制并保留音频时，强制使用那条的声道和采样率。丢掉音频时按 48000 规划。否则强制 48000；序号不存在时改用第一条并记警告。
///
/// # 参数
/// - `encode`: 编码配置
/// - `info`: 这一段的视频信息
/// - `copy`: 基准复制计划。新片段或基准要重编码时为空
///
/// # 返回值
/// 成功时返回音轨计划
///
/// # 错误
/// 要保留音频但没有可选音轨时返回 `SidecarError::Invalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let audio = segment_audio(&encode, &info, None)?;
/// ```
fn segment_audio(encode: &VideoEncode, info: &VideoInfo, copy: Option<&CopyPlan>) -> SidecarResult<AudioPlan> {
  if let Some(plan) = copy.filter(|plan| plan.keep_audio) {
    let mut selection = encode.audio.clone();
    selection.channels = Some(plan.channels);
    let audio = plan_audio(&selection, &info.audio_tracks, RatePolicy::Force(plan.sample_rate));
    if audio.stream_index.is_none() {
      return Err(SidecarError::invalid(format!("没有音轨: {}", info.path)));
    }
    return Ok(audio);
  }
  if matches!(encode.audio.mode, AudioMode::Drop) {
    return Ok(plan_audio(&encode.audio, &info.audio_tracks, RatePolicy::Force(48000)));
  }
  let audio = plan_audio(&encode.audio, &info.audio_tracks, RatePolicy::Force(48000));
  if audio.fallback {
    tauri_plugin_log::log::warn!("音轨序号不存在，改用第一条: {}", info.path);
  }
  if audio.stream_index.is_none() {
    return Err(SidecarError::invalid(format!("没有音轨: {}", info.path)));
  }
  Ok(audio)
}

/// 把基准视频复制进中间容器
///
/// 保留音频时映射那条流，否则加 `-an`。中间容器是 MPEG-TS 时，HEVC 或 H.264 加上对应的 annexb 比特流滤镜。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `base`: 基准视频
/// - `info`: 基准视频信息
/// - `plan`: 复制计划
/// - `muxer`: 中间封装名
/// - `output`: 中间文件路径
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 启动失败返回 `SidecarError::Spawn`。退出码不是 0 或通道结束时返回 `SidecarError::Exit`，消息以「基准视频封装失败」开头
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// remux_base(app, base, info, plan, muxer, output).await?;
/// ```
async fn remux_base(
  app: &tauri::AppHandle,
  base: &str,
  info: &VideoInfo,
  plan: &CopyPlan,
  muxer: &str,
  output: &str,
) -> SidecarResult<()> {
  let mut args = vec!["-i".to_string(), base.to_string(), "-map".to_string(), "0:v:0".to_string()];
  if plan.keep_audio {
    if let Some(index) = plan.stream_index {
      args.push("-map".to_string());
      args.push(format!("0:{index}"));
    }
    args.extend(["-c".to_string(), "copy".to_string()]);
  } else {
    args.extend(["-an".to_string(), "-c".to_string(), "copy".to_string()]);
  }
  if muxer == "mpegts" {
    if let Some(bsf) = source_annexb(&info.video_codec) {
      args.extend(["-bsf:v".to_string(), bsf.to_string()]);
    }
  }
  args.extend(["-f".to_string(), muxer.to_string(), "-y".to_string(), output.to_string(), "-hide_banner".to_string()]);

  tauri_plugin_log::log::info!("ffmpeg {}", args.join(" "));
  let (mut rx, _) = app
    .shell()
    .sidecar("ffmpeg")
    .map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?
    .args(&args)
    .spawn()
    .map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?;
  let mut notes = FfmpegNotes::default();
  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stderr(line) | CommandEvent::Stdout(line) => notes.push(&line),
      CommandEvent::Terminated(status) => {
        if status.code != Some(0) {
          let _ = clear_cache_temp_dir(app);
          let detail = status.code.map(|code| notes.failure(code)).unwrap_or_else(|| "异常退出".to_string());
          return Err(SidecarError::sidecar_exit("ffmpeg", format!("基准视频封装失败：{detail}")));
        }
        return Ok(());
      }
      _ => {}
    }
  }
  Err(SidecarError::sidecar_exit("ffmpeg", "基准视频封装失败"))
}

/// 把一段视频编码进中间容器
///
/// 中间容器是 MPEG-TS 且视频是 HEVC 时加上 `hevc_mp4toannexb`。进度按这一段自己的时长上报。取消或失败时清掉临时目录。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `job`: 任务执行期句柄
/// - `input`: 源视频
/// - `info`: 源视频信息
/// - `encode`: 这一段使用的编码配置
/// - `audio`: 音轨计划
/// - `size`: 目标画面
/// - `fps`: 目标帧率
/// - `offset`: 这一段在成片上的起点
/// - `source_start`: 源视频上的起点
/// - `muxer`: 中间封装名
/// - `output`: 中间文件路径
/// - `message`: 进度说明
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 取消返回 `SidecarError::Canceled`。启动失败返回 `Spawn`。非 0 退出返回 `Exit`。通道结束且未取消时返回成功
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// encode_part(app, job, input, info, encode, audio, size, fps, offset, None, muxer, output, message).await?;
/// ```
async fn encode_part(
  app: &tauri::AppHandle,
  job: &JobCtx,
  input: &str,
  info: &VideoInfo,
  encode: &VideoEncode,
  audio: &AudioPlan,
  size: Option<crate::ffmpeg::models::VideoSize>,
  fps: f64,
  offset: f64,
  source_start: Option<f64>,
  muxer: &str,
  output: &str,
  message: &str,
) -> SidecarResult<()> {
  let filter = build_video_filter(&GraphRequest {
    encode,
    path: input,
    width: info.width,
    fps: Some(fps),
    size,
    offset,
    duration: info.duration,
    source_start,
  });
  let mut args = vec!["-i".to_string(), input.to_string()];
  if let Some(filter) = &filter {
    args.push("-vf".to_string());
    args.push(filter.clone());
  }
  push_maps(&mut args, audio);
  let width = size.map(|item| item.width).unwrap_or(info.width);
  args.extend(encode_args(encode, audio, width));
  if muxer == "mpegts" {
    if let Some(bsf) = output_annexb(&encode.video) {
      args.extend(["-bsf:v".to_string(), bsf.to_string()]);
    }
  }
  args.extend(["-f".to_string(), muxer.to_string(), "-y".to_string(), output.to_string(), "-hide_banner".to_string()]);

  tauri_plugin_log::log::info!("ffmpeg {}", args.join(" "));
  let (mut rx, child) = app
    .shell()
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
        if let Some(current) = parse_time_from_ffmpeg_output(&line) {
          job.report_ratio(current, info.duration, message).await;
        }
      }
      CommandEvent::Terminated(status) => {
        clear_current_child(app).await;
        if status.code == Some(0) {
          return Ok(());
        }
        let _ = clear_cache_temp_dir(app);
        let detail = status.code.map(|code| notes.failure(code)).unwrap_or_else(|| "异常退出".to_string());
        return Err(SidecarError::sidecar_exit("ffmpeg", detail));
      }
      _ => {}
    }
  }
  if job.is_canceled().await {
    let _ = clear_cache_temp_dir(app);
    return Err(SidecarError::canceled());
  }
  Ok(())
}

/// 按源编码选择 MPEG-TS 用的 annexb 滤镜
///
/// HEVC 或 H.265 用 `hevc_mp4toannexb`。H.264 或 AVC 用 `h264_mp4toannexb`。其他返回 `None`。
///
/// # 参数
/// - `codec`: 源视频编码名
///
/// # 返回值
/// 滤镜名
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(source_annexb("hevc"), Some("hevc_mp4toannexb"));
/// assert_eq!(source_annexb("av1"), None);
/// ```
fn source_annexb(codec: &str) -> Option<&'static str> {
  let codec = codec.to_ascii_lowercase();
  if codec.contains("hevc") || codec.contains("h265") {
    Some("hevc_mp4toannexb")
  } else if codec.contains("h264") || codec.contains("avc") {
    Some("h264_mp4toannexb")
  } else {
    None
  }
}

/// 按目标编码选择 MPEG-TS 用的 annexb 滤镜
///
/// HEVC 返回 `hevc_mp4toannexb`。AV1 和 VP9 返回 `None`。
///
/// # 参数
/// - `video`: 目标视频参数
///
/// # 返回值
/// 滤镜名
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(output_annexb(&VideoParams::default()), None);
/// ```
fn output_annexb(video: &VideoParams) -> Option<&'static str> {
  match video {
    VideoParams::Hevc { .. } => Some("hevc_mp4toannexb"),
    VideoParams::Av1 { .. } | VideoParams::Vp9 { .. } => None,
  }
}
