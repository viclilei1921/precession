use crate::ffmpeg::models::VideoInfo;

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

/// 留下 ffmpeg 的报错，丢掉进度和片头信息。
#[derive(Default)]
pub struct FfmpegNotes {
  text: String,
}

impl FfmpegNotes {
  pub fn push(&mut self, chunk: &[u8]) {
    let raw = String::from_utf8_lossy(chunk);
    for line in raw.lines() {
      let line = line.trim();
      if line.is_empty() || is_progress(line) {
        continue;
      }
      if !self.text.is_empty() {
        self.text.push('\n');
      }
      self.text.push_str(line);
      if self.text.len() > 1200 {
        let extra = self.text.len() - 800;
        let cut = self.text[extra..].find('\n').map(|pos| extra + pos + 1).unwrap_or(extra);
        self.text.drain(..cut);
      }
    }
  }

  pub fn failure(&self, code: i32) -> String {
    let detail = self.detail();
    if detail.is_empty() { format!("退出代码 {code}") } else { format!("退出代码 {code}：{detail}") }
  }

  fn detail(&self) -> String {
    let reasons: Vec<&str> = self.text.lines().filter(|line| is_reason(line)).collect();
    let chosen: Vec<&str> = if reasons.is_empty() {
      self.text.lines().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect()
    } else {
      reasons.into_iter().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect()
    };
    chosen.join(" ")
  }
}

fn is_progress(line: &str) -> bool {
  line.contains("time=")
    || line.starts_with("frame=")
    || line.starts_with("size=")
    || line.starts_with("Exiting with exit code")
    || line.starts_with("Input #")
    || line.starts_with("Output #")
    || line.starts_with("Stream #")
    || line.starts_with("Metadata:")
    || line.starts_with("Duration:")
    || line.starts_with("Side data:")
    || line.starts_with("major_brand")
    || line.starts_with("minor_version")
    || line.starts_with("compatible_brands")
    || line.starts_with("creation_time")
    || line.starts_with("handler_name")
}

fn is_reason(line: &str) -> bool {
  let lower = line.to_ascii_lowercase();
  lower.contains("error")
    || lower.contains("invalid")
    || lower.contains("not found")
    || lower.contains("no decoder")
    || lower.contains("failed")
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

#[cfg(test)]
mod tests {
  use super::FfmpegNotes;

  #[test]
  fn failure_keeps_decoder_reason() {
    let mut notes = FfmpegNotes::default();
    notes.push(b"Stream #0:1(eng): Audio: ac3, 48000 Hz, 5.1(side)\n");
    notes.push(b"frame=  12 fps=3.0 time=00:00:01.00 bitrate=100.0kbits/s\n");
    notes.push(b"[aist#0:1/ac3] Decoding requested, but no decoder found for: ac3\n");
    notes.push(b"Error opening output files: Invalid argument\n");
    notes.push(b"Exiting with exit code 234\n");
    let message = notes.failure(234);
    assert!(message.contains("no decoder found for: ac3"), "{message}");
    assert!(message.contains("Invalid argument"), "{message}");
    assert!(!message.contains("frame="), "{message}");
    assert!(!message.contains("Stream #"), "{message}");
  }
}
