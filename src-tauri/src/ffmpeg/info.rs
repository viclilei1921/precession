//! 视频信息解析：从 FFmpeg 输出中提取画面和全部音轨。

use std::sync::OnceLock;

use regex::Regex;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::models::{AudioTrack, VideoInfo};
use crate::ffmpeg::utils::parse_duration_str;
use crate::sidecar::{SidecarError, SidecarResult};

static RE_RES: OnceLock<Regex> = OnceLock::new();
static RE_FPS: OnceLock<Regex> = OnceLock::new();
static RE_DURATION: OnceLock<Regex> = OnceLock::new();
static RE_BITRATE_KB: OnceLock<Regex> = OnceLock::new();
static RE_VCODEC: OnceLock<Regex> = OnceLock::new();
static RE_PIX: OnceLock<Regex> = OnceLock::new();
static RE_AUDIO: OnceLock<Regex> = OnceLock::new();
static RE_AUDIO_BITRATE: OnceLock<Regex> = OnceLock::new();
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

/// 从一行 ffmpeg 输出里填画面、时长或音轨
///
/// 遇到新的 `Stream #` 就结束上一条音轨的标题收集。音轨行匹配后，后续行里的 `title` 写进这条音轨，直到下一条流。
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
  if line.contains("Stream #") {
    *audio_open = false;
  }

  if line.contains("Video:") && info.video_codec.is_empty() {
    if let Some(caps) = regex(&RE_RES, r"(\d{2,5})x(\d{2,5})").captures(line) {
      info.width = caps[1].parse().unwrap_or(0);
      info.height = caps[2].parse().unwrap_or(0);
    }
    if let Some(caps) = regex(&RE_FPS, r",\s*(\d+(?:\.\d+)?)\s*fps").captures(line) {
      info.fps = caps[1].parse().unwrap_or(0.0);
    }
    if let Some(caps) = regex(&RE_VCODEC, r"Video:\s*([A-Za-z0-9_]+)").captures(line) {
      info.video_codec = caps[1].to_string();
    }
    if let Some(caps) = regex(&RE_PIX, r"Video:.*?,\s*([a-z0-9]+)").captures(line) {
      info.pix_fmt = caps[1].to_string();
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

  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stderr(data) => {
        let chunk = String::from_utf8_lossy(&data);
        for raw in chunk.lines() {
          let line = raw.trim();
          if !line.is_empty() {
            parse_line(&mut info, &mut audio_open, line);
          }
        }
      }
      CommandEvent::Terminated(status) => {
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

  if info.duration > 0.0 || info.width > 0 || !info.video_codec.is_empty() {
    return Ok(info);
  }

  Err(SidecarError::invalid("无法读取视频时长"))
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
}
