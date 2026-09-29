use crate::{
  ffmpeg::models::{EncoderPreset, VideoInfo},
  utils::gpu::GpuInfo,
};

/// 将 HH:MM:SS.ms 转换为秒
pub fn parse_duration_str(duration_str: &str) -> Option<f64> {
  let parts: Vec<&str> = duration_str.split(':').collect();
  match parts.len() {
    4 => {
      let hours: f64 = parts[0].parse().ok()?;
      let minutes: f64 = parts[1].parse().ok()?;
      let seconds: f64 = parts[2].parse().ok()?;
      let millisecond: f64 = parts[3].parse().ok()?;
      Some(hours * 3600.0 + minutes * 60.0 + seconds + millisecond / 1000.0)
    }
    3 => {
      // 小时:分钟:秒
      let hours: f64 = parts[0].parse().ok()?;
      let minutes: f64 = parts[1].parse().ok()?;
      let seconds: f64 = parts[2].parse().ok()?;
      Some(hours * 3600.0 + minutes * 60.0 + seconds)
    }
    2 => {
      // 分钟:秒
      let minutes: f64 = parts[0].parse().ok()?;
      let seconds: f64 = parts[1].parse().ok()?;
      Some(minutes * 60.0 + seconds)
    }
    _ => None,
  }
}

/// 从 FFmpeg 输出解析当前时间（秒）
pub fn parse_time_from_ffmpeg_output(line: &Vec<u8>) -> Option<f64> {
  let line_str = String::from_utf8_lossy(&line);

  // 查找 time= 格式的时间（如 time=00:01:23.45）
  if let Some(start) = line_str.find("time=") {
    let time_part = &line_str[start + 5..];
    let end = time_part.find(' ').unwrap_or(time_part.len());
    let time_str = &time_part[..end];

    parse_duration_str(time_str)
  } else {
    None
  }
}

/// 匹配厂商到编码器预设
pub fn match_vendor_to_encoder(name: &str) -> EncoderPreset {
  let name_lower = name.to_lowercase();

  if name_lower.contains("nvidia") {
    EncoderPreset::Nvidia("hevc_nvenc".to_string())
  } else if name_lower.contains("intel") {
    EncoderPreset::Intel("hevc_qsv".to_string())
  } else if name_lower.contains("amd") || name_lower.contains("radeon") {
    EncoderPreset::Amd("hevc_amf".to_string())
  } else if name_lower.contains("apple") {
    EncoderPreset::Apple("hevc_videotoolbox".to_string())
  } else {
    // 未知厂商，兜底回 CPU
    EncoderPreset::Cpu("libx265".to_string())
  }
}

/// 选择最佳编码器
pub fn select_best_encoder(gpus: &[GpuInfo]) -> EncoderPreset {
  // 1. 优先寻找独立显卡 (DiscreteGpu)
  if let Some(gpu) = gpus.iter().find(|g| g.device_type == "DiscreteGpu") {
    return match_vendor_to_encoder(&gpu.name);
  }

  // 2. 如果没有独显，寻找集成显卡 (IntegratedGpu)
  if let Some(gpu) = gpus.iter().find(|g| g.device_type == "IntegratedGpu") {
    return match_vendor_to_encoder(&gpu.name);
  }

  // 3. 都没有，返回 CPU 软解
  EncoderPreset::Cpu("libx265".to_string())
}

/// 用于计算多个视频合并后的目标参数 <br>
/// return: (目标宽度, 目标高度, 目标帧率)
pub fn calculate_target_params(videos_info: &[VideoInfo]) -> (u32, u32, f64) {
  // A. 计算最大分辨率 (画布能够包容所有视频)
  let max_w = videos_info.iter().map(|m| m.width).max().unwrap_or(1920);
  let max_h = videos_info.iter().map(|m| m.height).max().unwrap_or(1080);

  // B. 计算帧率中位数 (取大者)
  let mut fps_list: Vec<f64> = videos_info.iter().map(|m| m.fps).collect();
  // 排序
  fps_list.sort_by(|a, b| a.partial_cmp(b).unwrap());

  let len = fps_list.len();
  let median_fps = if len == 0 {
    30.0
  } else {
    // 如果是偶数长度 (例如 [24, 25, 30, 60])，len/2 索引是 2 (数值30)。
    // 这符合你要求的 "取最大者" (Upper Median)
    fps_list[len / 2]
  };

  (max_w, max_h, median_fps)
}
