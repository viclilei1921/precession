use crate::ffmpeg::models::VideoInfo;

/// 把冒号分隔的时间换成秒
///
/// 四段按时、分、秒、毫秒，毫秒除以 1000。三段按时、分、秒。两段按分、秒。其他段数返回 `None`。任一段不是数字也返回 `None`。
///
/// # 参数
/// - `duration_str`: 时间文本
///
/// # 返回值
/// 解析成功时的秒数，否则 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(parse_duration_str("00:01:30"), Some(90.0));
/// assert_eq!(parse_duration_str("1:02:03:004"), Some(3723.004));
/// ```
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
  /// 把这段输出里的非进度行追加进来
  ///
  /// 空行和进度、片头行丢掉。文本超过 1200 字节时，从前面裁到大约 800 字节，尽量从换行处切开。
  ///
  /// # 参数
  /// - `chunk`: ffmpeg 输出的一块字节
  ///
  /// # 返回值
  /// 无返回值
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let mut notes = FfmpegNotes::default();
  /// notes.push(b"Error opening output files: Invalid argument\n");
  /// ```
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

  /// 拼出带退出码的失败说明
  ///
  /// 没有挑出的详情时只写退出码。
  ///
  /// # 参数
  /// - `code`: 退出码
  ///
  /// # 返回值
  /// `退出代码 {code}`，或后面接上详情
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let notes = FfmpegNotes::default();
  /// assert_eq!(notes.failure(1), "退出代码 1");
  /// ```
  pub fn failure(&self, code: i32) -> String {
    let detail = self.detail();
    if detail.is_empty() { format!("退出代码 {code}") } else { format!("退出代码 {code}：{detail}") }
  }

  /// 挑出最多四行原因，没有原因时用最后四行
  ///
  /// # 返回值
  /// 用空格拼起来的文本
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let notes = FfmpegNotes::default();
  /// assert!(notes.detail().is_empty());
  /// ```
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

/// 判断这一行是进度或片头信息
///
/// # 参数
/// - `line`: 一行输出
///
/// # 返回值
/// 含 `time=`，或以 `frame=`、`size=`、`Exiting with exit code`、`Input #`、`Output #`、`Stream #`、`Metadata:`、`Duration:`、`Side data:`、`major_brand`、`minor_version`、`compatible_brands`、`creation_time`、`handler_name` 开头时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(is_progress("frame=  12 fps=3.0"));
/// assert!(!is_progress("Error opening output files"));
/// ```
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

/// 判断这一行像失败原因
///
/// # 参数
/// - `line`: 一行输出
///
/// # 返回值
/// 小写后含 `error`、`invalid`、`not found`、`no decoder` 或 `failed` 时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(is_reason("Error opening output files"));
/// assert!(!is_reason("Stream #0:0"));
/// ```
fn is_reason(line: &str) -> bool {
  let lower = line.to_ascii_lowercase();
  lower.contains("error")
    || lower.contains("invalid")
    || lower.contains("not found")
    || lower.contains("no decoder")
    || lower.contains("failed")
}

/// 从 ffmpeg 输出里的 `time=` 解析当前秒数
///
/// 没有 `time=`，或后面的时间文本解析失败时返回 `None`。
///
/// # 参数
/// - `line`: 一行输出的字节
///
/// # 返回值
/// 解析成功时的秒数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let line = b"frame=1 time=00:01:30.00 bitrate=1.0kbits/s".to_vec();
/// assert_eq!(parse_time_from_ffmpeg_output(&line), Some(90.0));
/// ```
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

/// 取最大宽高，帧率取排序后 `len / 2` 那一档
///
/// 没有视频时宽高用 1920 和 1080，帧率用 30。
///
/// # 参数
/// - `videos_info`: 各段视频信息
///
/// # 返回值
/// `(宽度, 高度, 帧率)`
///
/// # Panics
/// 帧率是 NaN，`partial_cmp` 得到 `None` 时，`unwrap` 会 panic
///
/// # 示例
/// ```
/// let (width, height, fps) = calculate_target_params(&[]);
/// assert_eq!((width, height, fps), (1920, 1080, 30.0));
/// ```
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
