//! 视频信息解析：从 FFmpeg 输出中提取画面和全部音轨。

use std::sync::OnceLock;

use regex::Regex;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::models::{AudioTrack, VideoInfo};
use crate::ffmpeg::utils::parse_duration_str;
use crate::sidecar::{SidecarError, SidecarResult};

/// 画面宽高
static RE_RES: OnceLock<Regex> = OnceLock::new();
/// 帧率
static RE_FPS: OnceLock<Regex> = OnceLock::new();
/// 时间基帧率。比平均帧率更接近编码器实际使用的帧率
static RE_TBR: OnceLock<Regex> = OnceLock::new();
/// 时长
static RE_DURATION: OnceLock<Regex> = OnceLock::new();
/// 比特率
static RE_BITRATE_KB: OnceLock<Regex> = OnceLock::new();
/// 视频编码
static RE_VCODEC: OnceLock<Regex> = OnceLock::new();
/// 像素格式
static RE_PIX: OnceLock<Regex> = OnceLock::new();
/// 音频
static RE_AUDIO: OnceLock<Regex> = OnceLock::new();
/// 音频比特率
static RE_AUDIO_BITRATE: OnceLock<Regex> = OnceLock::new();
/// 音轨标题
static RE_TITLE: OnceLock<Regex> = OnceLock::new();

/// 取出或编译这条正则
///
/// # 参数
/// - `slot`: 缓存槽
/// - `pattern`: 正则文本
///
/// # 返回值
/// 编译好的正则
///
/// # Panics
/// `Regex::new` 失败时，`expect` 会 panic
///
/// # 示例
/// ```ignore
/// let re = regex(&RE_RES, r"(\d{2,5})x(\d{2,5})");
/// ```
fn regex(slot: &'static OnceLock<Regex>, pattern: &'static str) -> &'static Regex {
  slot.get_or_init(|| Regex::new(pattern).expect("ffmpeg video_info regex"))
}

/// 按声道布局字符串估算声道数
///
/// 识别 mono、7.1、6.1、5.1、5.0、4.0、quad、2.1。都对不上时返回 2。
///
/// # 参数
/// - `layout`: 声道布局
///
/// # 返回值
/// 声道数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(channels_of("5.1(side)"), 6);
/// assert_eq!(channels_of("stereo"), 2);
/// ```
fn channels_of(layout: &str) -> u32 {
  let layout = layout.to_ascii_lowercase();
  if layout.contains("mono") {
    1
  } else if layout.contains("7.1") {
    8
  } else if layout.contains("6.1") {
    7
  } else if layout.contains("5.1") {
    6
  } else if layout.contains("5.0") {
    5
  } else if layout.contains("4.0") || layout.contains("quad") {
    4
  } else if layout.contains("2.1") {
    3
  } else {
    2
  }
}

/// 这一行是不是可以当作画面的视频流
///
/// 只认 `Stream #` 流描述。探测警告里也会出现 `Video:`，例如封面尺寸读不到，那些行不是流描述。
/// 带 `attached pic` 的是封面图，不算画面。
///
/// # 参数
/// - `line`: 一行 ffmpeg 输出
///
/// # 返回值
/// 是画面流时返回 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(is_picture_stream("Stream #0:0: Video: h264, yuv420p, 1280x720, 25 fps"));
/// assert!(!is_picture_stream("Could not find codec parameters for stream 2 (Video: mjpeg, none)"));
/// assert!(!is_picture_stream("Stream #0:2: Video: mjpeg, none (attached pic)"));
/// ```
fn is_picture_stream(line: &str) -> bool {
  line.contains("Stream #") && line.contains("Video:") && !line.contains("attached pic")
}

/// 从一行 ffmpeg 输出里填画面、时长或音轨
///
/// 遇到新的 `Stream #` 就结束上一条音轨的标题收集。音轨行匹配后，后续行里的 `title` 写进这条音轨，直到下一条流。
/// 画面只取第一条带宽高的视频流。封面和探测警告不算。前面的流没有宽高时，后面带尺寸的流可以补上。
///
/// # 参数
/// - `info`: 正在填充的视频信息
/// - `audio_open`: 是否还在收集当前音轨标题
/// - `line`: 一行输出
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut info = VideoInfo::default();
/// let mut audio_open = false;
/// parse_line(&mut info, &mut audio_open, "Duration: 00:01:00.00, bitrate: 100 kb/s");
/// assert_eq!(info.duration, 60.0);
/// ```
fn parse_line(info: &mut VideoInfo, audio_open: &mut bool, line: &str) {
  // 遇到新的流就结束音轨标题收集
  if line.contains("Stream #") {
    *audio_open = false;
  }

  // 已经读到宽高后，后面的视频流不再改画面。还没有宽高时，继续找带尺寸的流。
  if is_picture_stream(line) && (info.width == 0 || info.height == 0) {
    let mut width = 0;
    let mut height = 0;
    if let Some(caps) = regex(&RE_RES, r"(\d{2,5})x(\d{2,5})").captures(line) {
      width = caps[1].parse().unwrap_or(0);
      height = caps[2].parse().unwrap_or(0);
    }
    let has_size = width > 0 && height > 0;
    if info.video_codec.is_empty() || has_size {
      info.width = width;
      info.height = height;
      info.fps = 0.0;
      if let Some(caps) = regex(&RE_TBR, r",\s*(\d+(?:\.\d+)?)\s*tbr").captures(line) {
        info.fps = caps[1].parse().unwrap_or(0.0);
      } else if let Some(caps) = regex(&RE_FPS, r",\s*(\d+(?:\.\d+)?)\s*fps").captures(line) {
        info.fps = caps[1].parse().unwrap_or(0.0);
      }
      if let Some(caps) = regex(&RE_VCODEC, r"Video:\s*([A-Za-z0-9_]+)").captures(line) {
        info.video_codec = caps[1].to_string();
      }
      if let Some(caps) = regex(&RE_PIX, r"Video:.*?,\s*([a-z0-9]+)").captures(line) {
        info.pix_fmt = caps[1].to_string();
      } else {
        info.pix_fmt.clear();
      }
    }
  }

  if line.contains("Audio:") {
    if let Some(caps) = regex(
      &RE_AUDIO,
      r"Stream #\d+:(\d+)(?:\[[^\]]*\])?(?:\(([^)]*)\))?: Audio:\s*([A-Za-z0-9_]+).*?,\s*(\d+)\s*Hz,\s*([^,]+)",
    )
    .captures(line)
    {
      let layout = caps[5].trim().to_string();
      let bitrate =
        regex(&RE_AUDIO_BITRATE, r"(\d+)\s*kb/s").captures(line).and_then(|caps| caps[1].parse().ok()).unwrap_or(0);
      info.audio_tracks.push(AudioTrack {
        index: caps[1].parse().unwrap_or(0),
        codec: caps[3].to_string(),
        sample_rate: caps[4].parse().unwrap_or(0),
        channels: channels_of(&layout),
        channel_layout: layout,
        bitrate_kbps: bitrate,
        language: caps.get(2).map(|item| item.as_str().to_string()).unwrap_or_default(),
        title: String::new(),
      });
      *audio_open = true;
    }
  }

  if *audio_open {
    if let Some(caps) = regex(&RE_TITLE, r"title\s*:\s*(.+)$").captures(line) {
      if let Some(track) = info.audio_tracks.last_mut() {
        if track.title.is_empty() {
          track.title = caps[1].trim().to_string();
        }
      }
    }
  }

  if line.contains("Duration:") {
    if let Some(caps) = regex(&RE_DURATION, r"Duration:\s*(\d+:\d{2}:\d{2}\.\d+)").captures(line) {
      if let Some(duration) = parse_duration_str(&caps[1]) {
        info.duration = duration;
      }
    }
    if let Some(caps) = regex(&RE_BITRATE_KB, r"bitrate:\s*(\d+)\s*kb/s").captures(line) {
      info.bitrate_kbps = caps[1].parse().unwrap_or(0);
    }
  }
}

/// 用 ffmpeg `-i` 读取视频信息
///
/// 退出码 0 或 1 时停止读取。读完后时长、宽度或视频编码至少有一项才算成功。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `path`: 视频路径
///
/// # 返回值
/// 成功时返回视频信息，路径写入 `path`
///
/// # 错误
/// 启动失败返回 `SidecarError::Spawn`。退出码不是 0 或 1 时返回 `SidecarError::Exit`。没有读到时长、宽度和视频编码时返回 `SidecarError::Invalid`，消息是「无法读取视频时长」
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let info = video_info(app, path).await?;
/// ```
pub async fn video_info(app: &tauri::AppHandle, path: &str) -> SidecarResult<VideoInfo> {
  let shell = app.shell();
  let (mut rx, _child) = shell
    .sidecar("ffmpeg")
    .map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?
    .args(["-i", path, "-hide_banner"])
    .spawn()
    .map_err(|err| SidecarError::sidecar_spawn("ffmpeg", err))?;

  let mut info = VideoInfo { path: path.to_string(), ..Default::default() };
  let mut audio_open = false;
  let mut pending = Vec::new();

  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stderr(data) => consume_stderr(&mut pending, &data, &mut info, &mut audio_open),
      CommandEvent::Terminated(status) => {
        finish_stderr(&mut pending, &mut info, &mut audio_open);
        if let Some(code) = status.code {
          if code == 0 || code == 1 {
            break;
          }
          return Err(SidecarError::sidecar_exit("ffmpeg", format!("退出代码 {code}")));
        }
        break;
      }
      _ => {}
    }
  }
  finish_stderr(&mut pending, &mut info, &mut audio_open);

  if info.duration > 0.0 || info.width > 0 || !info.video_codec.is_empty() {
    return Ok(info);
  }

  Err(SidecarError::invalid("无法读取视频时长"))
}

/// 把一块标准错误接上未写完的半行，遇到换行再解析
///
/// ffmpeg 一次写出的字节不一定对齐到行尾。半行先留着，避免把 `Video:` 和后面的宽高拆成两行。
///
/// # 参数
/// - `pending`: 上一块留下的半行
/// - `chunk`: 新读到的标准错误
/// - `info`: 正在填充的视频信息
/// - `audio_open`: 是否还在收集当前音轨标题
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let mut pending = Vec::new();
/// consume_stderr(&mut pending, b"Stream #0:0: Video: h264, 1280x720\n", &mut info, &mut audio_open);
/// ```
fn consume_stderr(pending: &mut Vec<u8>, chunk: &[u8], info: &mut VideoInfo, audio_open: &mut bool) {
  pending.extend_from_slice(chunk);
  while let Some(index) = pending.iter().position(|byte| *byte == b'\n') {
    let line: Vec<u8> = pending.drain(..=index).collect();
    apply_stderr_line(info, audio_open, &line);
  }
}

/// 进程结束时把最后一行没有换行的输出也解析掉
///
/// # 参数
/// - `pending`: 还没遇到换行的剩余字节
/// - `info`: 正在填充的视频信息
/// - `audio_open`: 是否还在收集当前音轨标题
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// finish_stderr(&mut pending, &mut info, &mut audio_open);
/// ```
fn finish_stderr(pending: &mut Vec<u8>, info: &mut VideoInfo, audio_open: &mut bool) {
  if pending.is_empty() {
    return;
  }
  let line = std::mem::take(pending);
  apply_stderr_line(info, audio_open, &line);
}

/// 解析一行标准错误。空行跳过
///
/// # 参数
/// - `info`: 正在填充的视频信息
/// - `audio_open`: 是否还在收集当前音轨标题
/// - `line`: 一行字节，可以带换行
///
/// # 返回值
/// 无返回值
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// apply_stderr_line(&mut info, &mut audio_open, b"Duration: 00:01:00.00\n");
/// ```
fn apply_stderr_line(info: &mut VideoInfo, audio_open: &mut bool, line: &[u8]) {
  let text = String::from_utf8_lossy(line);
  let text = text.trim();
  if !text.is_empty() {
    parse_line(info, audio_open, text);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn audio_line_with_program_id_is_kept() {
    let sample = "\
Stream #0:0[0x1](und): Video: hevc (hvc1 / 0x31637668), yuv420p(tv, bt709), 1920x1080, 3601 kb/s, 29.96 fps
Stream #0:1[0x2](und): Audio: aac (mp4a / 0x6134706D), 48000 Hz, stereo, fltp, 127 kb/s (default)
    Metadata:
      handler_name    : SoundHandler
";
    let mut info = VideoInfo::default();
    let mut audio_open = false;
    for line in sample.lines() {
      parse_line(&mut info, &mut audio_open, line.trim());
    }
    assert!((info.fps - 29.96).abs() < 1e-9);
    assert_eq!(info.video_codec, "hevc");
    assert_eq!(info.pix_fmt, "yuv420p");
    assert_eq!(info.audio_tracks.len(), 1);
    let track = &info.audio_tracks[0];
    assert_eq!(track.index, 1);
    assert_eq!(track.codec, "aac");
    assert_eq!(track.sample_rate, 48000);
    assert_eq!(track.channels, 2);
    assert_eq!(track.language, "und");
    assert!(track.title.is_empty());
  }

  #[test]
  fn tbr_wins_over_the_average_fps() {
    let line = "Stream #0:0[0x1](und): Video: h264 (Main), yuv420p(tv, bt709), 1920x1080, 5985 kb/s, 29.94 fps, 29.97 tbr, 90k tbn";
    let mut info = VideoInfo::default();
    let mut audio_open = false;
    parse_line(&mut info, &mut audio_open, line);
    assert!((info.fps - 29.97).abs() < 1e-9);
  }

  fn parse_sample(sample: &str) -> VideoInfo {
    let mut info = VideoInfo::default();
    let mut audio_open = false;
    for line in sample.lines() {
      parse_line(&mut info, &mut audio_open, line.trim());
    }
    info
  }

  #[test]
  fn cover_warning_does_not_hide_the_picture() {
    let sample = "\
[in#0/mov,mp4,m4a,3gp,3g2,mj2 @ 000002305e5ece00] Could not find codec parameters for stream 2 (Video: mjpeg, none): unspecified size
Consider increasing the value for the 'analyzeduration' (0) and 'probesize' (5000000) options
Duration: 00:15:32.75, start: 0.000000, bitrate: 295 kb/s
Stream #0:0[0x1](und): Video: h264 (avc1 / 0x31637661), yuv420p(tv, bt709, progressive), 1280x720 [SAR 1:1 DAR 16:9], 160 kb/s, 25 fps, 25 tbr, 12800 tbn (default)
Stream #0:1[0x2](und): Audio: aac (mp4a / 0x6134706D), 44100 Hz, stereo, fltp, 127 kb/s (default)
Stream #0:2[0x0]: Video: mjpeg, none, 90k tbr, 90k tbn (attached pic)
";
    let info = parse_sample(sample);
    assert_eq!(info.video_codec, "h264");
    assert_eq!(info.width, 1280);
    assert_eq!(info.height, 720);
    assert!((info.fps - 25.0).abs() < 1e-9);
    assert_eq!(info.pix_fmt, "yuv420p");
    assert!((info.duration - 932.75).abs() < 1e-9);
    assert_eq!(info.audio_tracks.len(), 1);
    assert_eq!(info.audio_tracks[0].codec, "aac");
  }

  #[test]
  fn attached_picture_before_the_picture_is_skipped() {
    let sample = "\
Stream #0:0: Video: mjpeg, yuvj420p(pc, bt470bg/unknown/unknown), 600x600 [SAR 1:1 DAR 1:1], 90k tbr, 90k tbn (attached pic)
Stream #0:1: Video: h264 (High), yuv420p(tv, bt709), 1920x1080, 25 fps, 25 tbr
";
    let info = parse_sample(sample);
    assert_eq!(info.video_codec, "h264");
    assert_eq!(info.width, 1920);
    assert_eq!(info.height, 1080);
    assert_eq!(info.pix_fmt, "yuv420p");
    assert!((info.fps - 25.0).abs() < 1e-9);
  }

  #[test]
  fn later_sized_stream_fills_a_sizeless_one() {
    let sample = "\
Stream #0:0: Video: mjpeg, none
Stream #0:1: Video: hevc (Main), yuv420p, 1280x720, 30 fps, 30 tbr
";
    let info = parse_sample(sample);
    assert_eq!(info.video_codec, "hevc");
    assert_eq!(info.width, 1280);
    assert_eq!(info.height, 720);
    assert_eq!(info.pix_fmt, "yuv420p");
    assert!((info.fps - 30.0).abs() < 1e-9);
  }

  #[test]
  fn sized_picture_is_not_replaced_by_a_later_stream() {
    let sample = "\
Stream #0:0: Video: h264, yuv420p, 1280x720, 25 fps, 25 tbr
Stream #0:1: Video: hevc, yuv420p, 640x360, 30 fps, 30 tbr
";
    let info = parse_sample(sample);
    assert_eq!(info.video_codec, "h264");
    assert_eq!(info.width, 1280);
    assert_eq!(info.height, 720);
    assert!((info.fps - 25.0).abs() < 1e-9);
  }

  #[test]
  fn split_stderr_still_reads_the_picture() {
    let sample = b"Stream #0:0[0x1](und): Video: h264 (avc1 / 0x31637661), yuv420p(tv, bt709, progressive), 1280x720, 25 fps, 25 tbr\n";
    let marker = b"1280x720";
    let cut = sample.windows(marker.len()).position(|window| window == marker).unwrap();
    let mut info = VideoInfo::default();
    let mut audio_open = false;
    let mut pending = Vec::new();
    consume_stderr(&mut pending, &sample[..cut], &mut info, &mut audio_open);
    assert!(info.video_codec.is_empty());
    assert_eq!(info.width, 0);
    consume_stderr(&mut pending, &sample[cut..], &mut info, &mut audio_open);
    assert_eq!(info.video_codec, "h264");
    assert_eq!(info.width, 1280);
    assert_eq!(info.height, 720);
    assert_eq!(info.pix_fmt, "yuv420p");
    assert!((info.fps - 25.0).abs() < 1e-9);
  }
}
