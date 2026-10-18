use serde::{Deserialize, Serialize};

/// 裁剪片段。`start` / `duration` 为 `HH:MM:SS` 或秒数。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSegment {
  pub start: String,
  pub duration: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
  pub path: String,
  pub width: u32,
  pub height: u32,
  pub fps: f64,
  pub duration: f64,
  pub video_codec: String,
  pub audio_codec: String,
  pub audio_sample_rate: u32,
  pub bitrate_kbps: u32,
}

/// 写出容器。侧车只打开了这三种封装。
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VideoContainer {
  Mp4,
  Mov,
  MpegTs,
}

/// 视频编码器。都是软件编码。
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VideoCodec {
  Av1,
  Hevc,
  Vp9,
}

/// 处理速度。各编码器再映射到自己的 preset。
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EncodeSpeed {
  Fast,
  Balanced,
  Slow,
}

/// 一次转码用的编码配置。缺省就是默认模式：MP4、AV1、均衡。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct VideoEncode {
  pub container: VideoContainer,
  pub codec: VideoCodec,
  pub speed: EncodeSpeed,
  /// 空则按编码器取默认 CRF。
  pub crf: Option<u8>,
  /// 空则保留源帧率。合并和追加在滤镜里决定帧率，调用前会清掉这个字段。
  pub fps: Option<u32>,
}

impl Default for VideoEncode {
  fn default() -> Self {
    Self { container: VideoContainer::Mp4, codec: VideoCodec::Av1, speed: EncodeSpeed::Balanced, crf: None, fps: None }
  }
}

impl Default for VideoContainer {
  fn default() -> Self {
    Self::Mp4
  }
}

impl Default for VideoCodec {
  fn default() -> Self {
    Self::Av1
  }
}

impl Default for EncodeSpeed {
  fn default() -> Self {
    Self::Balanced
  }
}

fn default_crf(codec: VideoCodec) -> u8 {
  match codec {
    VideoCodec::Av1 => 32,
    VideoCodec::Hevc => 24,
    VideoCodec::Vp9 => 32,
  }
}

fn push_pair(args: &mut Vec<String>, key: &str, value: impl Into<String>) {
  args.push(key.to_string());
  args.push(value.into());
}

/// 把编码配置变成 ffmpeg 参数。`audio` 为假时只写 `-an`，不再改返回值。
pub fn encode_args(profile: &VideoEncode, audio: bool) -> Vec<String> {
  let mut args = Vec::new();
  let crf = profile.crf.unwrap_or(default_crf(profile.codec)).min(63);

  match profile.codec {
    VideoCodec::Av1 => {
      let preset = match profile.speed {
        EncodeSpeed::Fast => "8",
        EncodeSpeed::Balanced => "6",
        EncodeSpeed::Slow => "4",
      };
      push_pair(&mut args, "-c:v", "libsvtav1");
      push_pair(&mut args, "-crf", crf.to_string());
      push_pair(&mut args, "-preset", preset);
    }
    VideoCodec::Hevc => {
      let preset = match profile.speed {
        EncodeSpeed::Fast => "fast",
        EncodeSpeed::Balanced => "medium",
        EncodeSpeed::Slow => "slow",
      };
      push_pair(&mut args, "-c:v", "libx265");
      push_pair(&mut args, "-crf", crf.to_string());
      push_pair(&mut args, "-preset", preset);
    }
    VideoCodec::Vp9 => {
      let cpu_used = match profile.speed {
        EncodeSpeed::Fast => "4",
        EncodeSpeed::Balanced => "2",
        EncodeSpeed::Slow => "1",
      };
      push_pair(&mut args, "-c:v", "libvpx-vp9");
      push_pair(&mut args, "-crf", crf.to_string());
      push_pair(&mut args, "-b:v", "0");
      push_pair(&mut args, "-deadline", "good");
      push_pair(&mut args, "-cpu-used", cpu_used);
    }
  }

  push_pair(&mut args, "-pix_fmt", "yuv420p");

  if audio {
    push_pair(&mut args, "-c:a", "aac");
    push_pair(&mut args, "-ac", "2");
  } else {
    args.push("-an".to_string());
  }

  if let Some(fps) = profile.fps.filter(|fps| *fps > 0 && *fps <= 240) {
    push_pair(&mut args, "-fps_mode", "cfr");
    push_pair(&mut args, "-r", fps.to_string());
  }

  if profile.codec == VideoCodec::Hevc && matches!(profile.container, VideoContainer::Mp4 | VideoContainer::Mov) {
    push_pair(&mut args, "-tag:v", "hvc1");
  }

  args
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_encode_is_av1_without_hardware() {
    let joined = encode_args(&VideoEncode::default(), true).join(" ");
    assert!(joined.contains("libsvtav1"), "{joined}");
    assert!(joined.contains("-preset 6"), "{joined}");
    assert!(joined.contains("-crf 32"), "{joined}");
    assert!(joined.contains("-c:a aac"), "{joined}");
    assert!(joined.contains("-pix_fmt yuv420p"), "{joined}");
    for banned in ["nvenc", "qsv", "amf", "videotoolbox", "hvc1", "libx265", "libvpx"] {
      assert!(!joined.contains(banned), "{joined} 含有 {banned}");
    }
  }

  #[test]
  fn hevc_mp4_adds_hvc1_and_can_drop_audio() {
    let profile = VideoEncode {
      container: VideoContainer::Mp4,
      codec: VideoCodec::Hevc,
      speed: EncodeSpeed::Fast,
      crf: None,
      fps: Some(30),
    };
    let joined = encode_args(&profile, false).join(" ");
    assert!(joined.contains("libx265"), "{joined}");
    assert!(joined.contains("-preset fast"), "{joined}");
    assert!(joined.contains("-crf 24"), "{joined}");
    assert!(joined.contains("-tag:v hvc1"), "{joined}");
    assert!(joined.contains("-an"), "{joined}");
    assert!(joined.contains("-r 30"), "{joined}");
    assert!(!joined.contains("-c:a"), "{joined}");
  }

  #[test]
  fn hevc_mpegts_skips_hvc1() {
    let profile = VideoEncode { container: VideoContainer::MpegTs, codec: VideoCodec::Hevc, ..VideoEncode::default() };
    let joined = encode_args(&profile, true).join(" ");
    assert!(!joined.contains("hvc1"), "{joined}");
  }

  #[test]
  fn vp9_slow_uses_cpu_used() {
    let profile = VideoEncode {
      container: VideoContainer::MpegTs,
      codec: VideoCodec::Vp9,
      speed: EncodeSpeed::Slow,
      crf: Some(40),
      fps: None,
    };
    let joined = encode_args(&profile, true).join(" ");
    assert!(joined.contains("libvpx-vp9"), "{joined}");
    assert!(joined.contains("-cpu-used 1"), "{joined}");
    assert!(joined.contains("-b:v 0"), "{joined}");
    assert!(joined.contains("-crf 40"), "{joined}");
    assert!(!joined.contains("hvc1"), "{joined}");
  }
}
