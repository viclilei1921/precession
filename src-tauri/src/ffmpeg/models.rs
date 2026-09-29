use serde::{Deserialize, Serialize};

/// 裁剪片段。`start` / `duration` 为 `HH:MM:SS` 或秒数。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSegment {
  pub start: String,
  pub duration: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

/// 编码器预设
#[derive(Debug)]
pub enum EncoderPreset {
  Nvidia(String), // hevc_nvenc
  Intel(String),  // hevc_qsv
  Amd(String),    // hevc_amf
  Apple(String),  // hevc_videotoolbox
  Cpu(String),    // libx265
}

impl EncoderPreset {
  /// 将编码器预设转换为 FFmpeg 参数
  pub fn to_ffmpeg_args(&self) -> Vec<&str> {
    // 1. 基础兼容性参数 (所有编码器通用)
    // -pix_fmt yuv420p: 强制 8位 色深，防止转码成 10位 导致浏览器黑屏
    // -tag:v hvc1: 苹果生态 (Safari/Finder) 识别 HEVC 的必要标签
    let common_args = vec!["-c:a", "aac", "-ac", "2", "-pix_fmt", "yuv420p", "-tag:v", "hvc1"];

    // 2. 根据硬件添加特定编码参数
    let encoder_args = match self {
      EncoderPreset::Nvidia(name) => vec![
        "-c:v",
        name.as_str(),
        // -cq: 恒定质量模式 (Constant Quality), 范围 1-51, 越小越清晰
        "-cq",
        "25",
        // -preset: p1(最快)-p7(最慢/质量最好), p4 是平衡点
        "-preset",
        "p5",
      ],
      EncoderPreset::Intel(name) => vec![
        "-c:v",
        name.as_str(),
        // -global_quality: ICQ 模式, 类似 CRF
        "-global_quality",
        "23",
        "-load_plugin",
        "hevc_hw", // 显式加载插件有时能避免报错
      ],
      EncoderPreset::Apple(name) => vec![
        "-c:v",
        name.as_str(),
        // -q:v: 质量控制, 0-100, 这里的 60 大约对应 CRF 26-28
        "-q:v",
        "68",
        // 确保 Apple 编码器不自动用 10bit
        "-profile:v",
        "main",
      ],
      EncoderPreset::Amd(name) => vec![
        "-c:v",
        name.as_str(),
        // AMD AMF 比较特殊，通常用 -rc cqp 来控制质量
        "-usage",
        "transcoding",
        "-rc",
        "cqp",
        "-qp_i",
        "24",
        "-qp_p",
        "24",
        "-quality",
        "quality",
      ],
      EncoderPreset::Cpu(name) => vec![
        "-c:v",
        name.as_str(),
        // -crf: 软件编码标准质量控制
        "-crf",
        "24",
        // -preset: medium 是默认, fast 编码更快
        "-preset",
        "medium",
      ],
    };

    // 3. 转换 Vec<&str> 为 Vec<String> 并合并

    [&common_args[..], &encoder_args[..]].concat()
  }
}
