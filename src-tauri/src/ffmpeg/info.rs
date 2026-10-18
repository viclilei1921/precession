//! 视频信息解析：从 FFmpeg 输出中提取元数据

use std::sync::OnceLock;

use regex::Regex;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::ffmpeg::models::VideoInfo;
use crate::ffmpeg::utils::parse_duration_str;
use crate::sidecar::{SidecarError, SidecarResult};

// 正则表达式：模块级 OnceLock，只编译一次，避免 video_info 每次调用重复编译
static RE_RES: OnceLock<Regex> = OnceLock::new();
static RE_FPS: OnceLock<Regex> = OnceLock::new();
static RE_DURATION: OnceLock<Regex> = OnceLock::new();
static RE_BITRATE_KB: OnceLock<Regex> = OnceLock::new();
static RE_VCODEC: OnceLock<Regex> = OnceLock::new();
static RE_ACODEC: OnceLock<Regex> = OnceLock::new();

fn re_res() -> &'static Regex {
  RE_RES.get_or_init(|| Regex::new(r"Video:.*?\s(\d{3,})x(\d{3,})").expect("ffmpeg video_info regex: res"))
}
fn re_fps() -> &'static Regex {
  RE_FPS.get_or_init(|| Regex::new(r",\s*(\d+(?:\.\d+)?)\s*fps").expect("ffmpeg video_info regex: fps"))
}
fn re_duration() -> &'static Regex {
  RE_DURATION
    .get_or_init(|| Regex::new(r"Duration:\s*(\d+:\d{2}:\d{2}\.\d+)").expect("ffmpeg video_info regex: duration"))
}
fn re_bitrate_kb() -> &'static Regex {
  RE_BITRATE_KB.get_or_init(|| Regex::new(r"bitrate:\s*(\d+)\s*kb/s").expect("ffmpeg video_info regex: bitrate_kb"))
}
fn re_vcodec() -> &'static Regex {
  RE_VCODEC.get_or_init(|| Regex::new(r"Video:\s*([a-zA-Z0-9_]+)").expect("ffmpeg video_info regex: vcodec"))
}
fn re_acodec() -> &'static Regex {
  RE_ACODEC
    .get_or_init(|| Regex::new(r"Audio:\s*([a-zA-Z0-9_]+).*?(\d+)\s*Hz").expect("ffmpeg video_info regex: acodec"))
}

pub async fn video_info(app: &tauri::AppHandle, path: &str) -> SidecarResult<VideoInfo> {
  let shell = app.shell();
  let (mut rx, _child) = shell
    .sidecar("ffmpeg")
    .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?
    .args(&["-i", path, "-hide_banner"])
    .spawn()
    .map_err(|e| SidecarError::sidecar_spawn("ffmpeg", e))?;

  let mut video_info = VideoInfo { path: path.to_string(), ..Default::default() };

  while let Some(event) = rx.recv().await {
    match event {
      CommandEvent::Stderr(data) => {
        let line = String::from_utf8_lossy(&data).trim().to_string();

        // 1. 解析基础视频信息 (分辨率, 编码)
        if line.contains("Video:") {
          if let Some(caps) = re_res().captures(&line) {
            video_info.width = caps[1].parse().unwrap_or(0);
            video_info.height = caps[2].parse().unwrap_or(0);
          }
          if let Some(caps) = re_fps().captures(&line) {
            video_info.fps = caps[1].parse().unwrap_or(0.0);
          }
          if video_info.video_codec.is_empty() {
            if let Some(caps) = re_vcodec().captures(&line) {
              video_info.video_codec = caps[1].to_string();
            }
          }
        }

        // 2. 解析音频信息
        if line.contains("Audio:") && video_info.audio_codec.is_empty() {
          if let Some(caps) = re_acodec().captures(&line) {
            video_info.audio_codec = caps[1].to_string();
            video_info.audio_sample_rate = caps[2].parse().unwrap_or(0);
          }
        }

        // 3. 解析时长与码率（分开匹配：浏览器 WebM 常为 Duration 有效但 bitrate: N/A，旧正则会整行失败）
        if line.contains("Duration:") {
          if let Some(caps) = re_duration().captures(&line) {
            if let Some(duration) = parse_duration_str(&caps[1]) {
              video_info.duration = duration;
            }
          }
          if let Some(caps) = re_bitrate_kb().captures(&line) {
            video_info.bitrate_kbps = caps[1].parse().unwrap_or(0);
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

  if video_info.duration > 0.0 {
    return Ok(video_info);
  }

  // 部分 WebM（如 Chrome/MediaRecorder）容器里无总时长，ffmpeg 报 Duration: N/A，但视频流仍可读；转码可正常完成，仅进度条无百分比
  if video_info.width > 0 || !video_info.video_codec.is_empty() {
    return Ok(video_info);
  }

  Err(SidecarError::invalid("无法读取视频时长"))
}
