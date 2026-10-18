use serde::{Deserialize, Serialize};

/// 裁剪片段。`start` / `duration` 为 `HH:MM:SS` 或秒数。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSegment {
  pub start: String,
  pub duration: String,
}

/// 一条音轨。`index` 是 `Stream #0:N` 的 N。
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
  pub index: u32,
  pub codec: String,
  pub sample_rate: u32,
  pub channels: u32,
  pub channel_layout: String,
  pub bitrate_kbps: u32,
  pub language: String,
  pub title: String,
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
  pub pix_fmt: String,
  pub bitrate_kbps: u32,
  pub audio_tracks: Vec<AudioTrack>,
}

/// 写出容器。侧车只打开了这三种封装。
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VideoContainer {
  Mp4,
  Mov,
  MpegTs,
}

impl VideoContainer {
  pub fn muxer(self) -> &'static str {
    match self {
      Self::Mp4 => "mp4",
      Self::Mov => "mov",
      Self::MpegTs => "mpegts",
    }
  }
}

impl Default for VideoContainer {
  fn default() -> Self {
    Self::Mp4
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BitDepth {
  #[default]
  Eight,
  Ten,
}

impl BitDepth {
  pub fn pix_fmt(self) -> &'static str {
    match self {
      Self::Eight => "yuv420p",
      Self::Ten => "yuv420p10le",
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HevcPreset {
  VeryFast,
  Fast,
  #[default]
  Medium,
  Slow,
  Slower,
}

impl HevcPreset {
  fn ffmpeg(self) -> &'static str {
    match self {
      Self::VeryFast => "veryfast",
      Self::Fast => "fast",
      Self::Medium => "medium",
      Self::Slow => "slow",
      Self::Slower => "slower",
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum HevcTune {
  #[default]
  None,
  Animation,
  Grain,
}

impl HevcTune {
  fn ffmpeg(self) -> Option<&'static str> {
    match self {
      Self::None => None,
      Self::Animation => Some("animation"),
      Self::Grain => Some("grain"),
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Vp9Content {
  #[default]
  General,
  Screen,
  Film,
}

impl Vp9Content {
  fn ffmpeg(self) -> Option<&'static str> {
    match self {
      Self::General => None,
      Self::Screen => Some("screen"),
      Self::Film => Some("film"),
    }
  }
}

fn av1_preset() -> u8 {
  6
}

fn av1_crf() -> u8 {
  35
}

fn hevc_crf() -> u8 {
  26
}

fn vp9_cpu_used() -> u8 {
  2
}

fn vp9_crf() -> u8 {
  33
}

fn audio_bitrate() -> u32 {
  128
}

fn default_true() -> bool {
  true
}

/// 当前这次用哪种编码。字段跟 `codec` 写在一起，缺的用该编码的默认值。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "codec", rename_all = "camelCase")]
pub enum VideoParams {
  Av1 {
    #[serde(default = "av1_preset")]
    preset: u8,
    #[serde(default = "av1_crf")]
    crf: u8,
    #[serde(default)]
    bit_depth: BitDepth,
  },
  Hevc {
    #[serde(default)]
    preset: HevcPreset,
    #[serde(default = "hevc_crf")]
    crf: u8,
    #[serde(default)]
    tune: HevcTune,
    #[serde(default)]
    bit_depth: BitDepth,
  },
  Vp9 {
    #[serde(default = "vp9_cpu_used")]
    cpu_used: u8,
    #[serde(default = "vp9_crf")]
    crf: u8,
    #[serde(default)]
    content: Vp9Content,
  },
}

impl Default for VideoParams {
  fn default() -> Self {
    Self::Av1 { preset: av1_preset(), crf: av1_crf(), bit_depth: BitDepth::Eight }
  }
}

impl VideoParams {
  pub fn pix_fmt(&self) -> &'static str {
    match self {
      Self::Av1 { bit_depth, .. } | Self::Hevc { bit_depth, .. } => bit_depth.pix_fmt(),
      Self::Vp9 { .. } => "yuv420p",
    }
  }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AudioMode {
  First,
  Index { index: u32 },
  Drop,
}

impl Default for AudioMode {
  fn default() -> Self {
    Self::First
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct AudioSelection {
  pub mode: AudioMode,
  pub channels: Option<u8>,
  pub sample_rate: Option<u32>,
  #[serde(default = "audio_bitrate")]
  pub bitrate_kbps: u32,
}

impl Default for AudioSelection {
  fn default() -> Self {
    Self { mode: AudioMode::First, channels: None, sample_rate: None, bitrate_kbps: audio_bitrate() }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Fit {
  #[default]
  Contain,
  Fill,
  Cover,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VideoSize {
  pub width: u32,
  pub height: u32,
  #[serde(default)]
  pub fit: Fit,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TitleSource {
  #[default]
  Text,
  Filename,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TitleAnchor {
  #[default]
  TopStart,
  TopCenter,
  TopEnd,
  CenterStart,
  Center,
  CenterEnd,
  BottomStart,
  BottomCenter,
  BottomEnd,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct TitleOverlay {
  pub source: TitleSource,
  pub text: String,
  pub anchor: TitleAnchor,
  pub font_size: u32,
  pub font_file: Option<String>,
  pub start: Option<f64>,
  pub end: Option<f64>,
}

impl Default for TitleOverlay {
  fn default() -> Self {
    Self {
      source: TitleSource::Text,
      text: String::new(),
      anchor: TitleAnchor::TopStart,
      font_size: 0,
      font_file: None,
      start: None,
      end: None,
    }
  }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SubtitleSource {
  #[default]
  ExternalFile,
  EmbeddedTrack,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct SubtitleOverlay {
  pub source: SubtitleSource,
  pub file: Option<String>,
  pub track_index: Option<u32>,
  pub fonts_dir: Option<String>,
  pub force_style: Option<String>,
}

impl Default for SubtitleOverlay {
  fn default() -> Self {
    Self { source: SubtitleSource::ExternalFile, file: None, track_index: None, fonts_dir: None, force_style: None }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct VideoEncode {
  pub container: VideoContainer,
  #[serde(default)]
  pub video: VideoParams,
  pub fps: Option<u32>,
  pub size: Option<VideoSize>,
  pub audio: AudioSelection,
  #[serde(default = "default_true")]
  pub faststart: bool,
  pub title: Option<TitleOverlay>,
  pub subtitle: Option<SubtitleOverlay>,
}

impl Default for VideoEncode {
  fn default() -> Self {
    Self {
      container: VideoContainer::Mp4,
      video: VideoParams::default(),
      fps: None,
      size: None,
      audio: AudioSelection::default(),
      faststart: true,
      title: None,
      subtitle: None,
    }
  }
}

/// 入队前收紧参数。越界的速度和质量夹进范围内，不合法的组合直接失败。
pub fn normalize_encode(encode: &mut VideoEncode) -> Result<(), String> {
  match &mut encode.video {
    VideoParams::Av1 { preset, crf, .. } => {
      *preset = (*preset).min(13);
      *crf = (*crf).min(63);
    }
    VideoParams::Hevc { crf, .. } => {
      *crf = (*crf).min(51);
    }
    VideoParams::Vp9 { cpu_used, crf, .. } => {
      *cpu_used = (*cpu_used).min(5);
      *crf = (*crf).min(63);
    }
  }

  if encode.fps.is_some_and(|fps| fps < 1 || fps > 120) {
    encode.fps = None;
  }

  if encode.audio.channels.is_some_and(|channels| channels != 1 && channels != 2) {
    return Err("声道只支持 1 或 2".to_string());
  }
  if encode.audio.sample_rate.is_some_and(|rate| rate != 44100 && rate != 48000) {
    return Err("采样率只支持 44100 或 48000".to_string());
  }
  if !(64..=320).contains(&encode.audio.bitrate_kbps) {
    return Err("音频码率要在 64 到 320 之间".to_string());
  }

  if let Some(size) = encode.size {
    if size.width == 0 || size.height == 0 {
      encode.size = None;
    } else if size.fit == Fit::Cover {
      return Err("当前版本不支持裁剪铺满".to_string());
    }
  }

  if let Some(title) = &encode.title {
    if title.source == TitleSource::Text && title.text.trim().is_empty() {
      return Err("标题文字不能为空".to_string());
    }
    if let (Some(start), Some(end)) = (title.start, title.end) {
      if end <= start {
        return Err("标题结束时间要晚于开始".to_string());
      }
    }
  }

  if let Some(subtitle) = &encode.subtitle {
    match subtitle.source {
      SubtitleSource::ExternalFile => {
        let file = subtitle.file.as_deref().unwrap_or("").trim();
        if file.is_empty() {
          return Err("字幕文件不能为空".to_string());
        }
        if !std::path::Path::new(file).is_file() {
          return Err("字幕文件不存在".to_string());
        }
      }
      SubtitleSource::EmbeddedTrack => {
        if subtitle.track_index.is_none() {
          return Err("未指定字幕轨".to_string());
        }
      }
    }
  }

  if let Some(title) = &mut encode.title {
    if title.font_file.as_deref().is_some_and(|path| !std::path::Path::new(path).is_file()) {
      tauri_plugin_log::log::warn!("字体不存在，改用系统字体");
      title.font_file = None;
    }
  }
  if let Some(subtitle) = &mut encode.subtitle {
    if subtitle.fonts_dir.as_deref().is_some_and(|path| !std::path::Path::new(path).is_dir()) {
      tauri_plugin_log::log::warn!("字体目录不存在，改用系统字体目录");
      subtitle.fonts_dir = None;
    }
  }

  if matches!(encode.video, VideoParams::Vp9 { .. }) && encode.container == VideoContainer::MpegTs {
    return Err("VP9 不能写入 MPEG-TS".to_string());
  }
  if encode.container == VideoContainer::MpegTs {
    encode.faststart = false;
  }
  Ok(())
}

pub fn even_dim(value: u32) -> u32 {
  let value = value.max(2);
  if value % 2 == 0 { value } else { value - 1 }
}

/// 采样率怎么定。合并固定 48000；转码和裁剪能留 44100 或 48000。
#[derive(Clone, Copy, Debug)]
pub enum RatePolicy {
  Source,
  Force(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioPlan {
  /// 绝对流序号。空表示不要音频。
  pub stream_index: Option<u32>,
  pub channels: u8,
  pub sample_rate: u32,
  pub bitrate_kbps: u32,
  pub fallback: bool,
}

pub fn plan_audio(selection: &AudioSelection, tracks: &[AudioTrack], rate: RatePolicy) -> AudioPlan {
  if matches!(selection.mode, AudioMode::Drop) || tracks.is_empty() {
    return AudioPlan {
      stream_index: None,
      channels: selection.channels.unwrap_or(2),
      sample_rate: 0,
      bitrate_kbps: selection.bitrate_kbps,
      fallback: false,
    };
  }

  let (track, fallback) = match selection.mode {
    AudioMode::Drop => unreachable!(),
    AudioMode::First => (tracks.first().expect("tracks 非空"), false),
    AudioMode::Index { index } => match tracks.iter().find(|track| track.index == index) {
      Some(track) => (track, false),
      None => (tracks.first().expect("tracks 非空"), true),
    },
  };

  let channels = selection.channels.unwrap_or(if track.channels <= 1 { 1 } else { 2 });
  let sample_rate = match rate {
    RatePolicy::Force(rate) => rate,
    RatePolicy::Source => selection
      .sample_rate
      .unwrap_or(if track.sample_rate == 44100 || track.sample_rate == 48000 { track.sample_rate } else { 48000 }),
  };

  AudioPlan { stream_index: Some(track.index), channels, sample_rate, bitrate_kbps: selection.bitrate_kbps, fallback }
}

/// 用户要保留音频，但这条片子没有可选音轨。
pub fn ensure_audio(mode: AudioMode, plan: &AudioPlan, path: &str) -> Result<(), String> {
  if !matches!(mode, AudioMode::Drop) && plan.stream_index.is_none() {
    Err(format!("没有音轨: {path}"))
  } else {
    Ok(())
  }
}

fn push_pair(args: &mut Vec<String>, key: &str, value: impl Into<String>) {
  args.push(key.to_string());
  args.push(value.into());
}

fn tile_columns(width: u32) -> u8 {
  if width > 0 && width < 1280 {
    1
  } else if width > 1920 {
    3
  } else {
    2
  }
}

/// 视频和音频的编码参数。流选择由调用方 `-map`。
pub fn encode_args(encode: &VideoEncode, audio: &AudioPlan, width: u32) -> Vec<String> {
  let mut args = Vec::new();

  match &encode.video {
    VideoParams::Av1 { preset, crf, .. } => {
      push_pair(&mut args, "-c:v", "libsvtav1");
      push_pair(&mut args, "-crf", crf.to_string());
      push_pair(&mut args, "-preset", preset.to_string());
    }
    VideoParams::Hevc { preset, crf, tune, bit_depth } => {
      push_pair(&mut args, "-c:v", "libx265");
      push_pair(&mut args, "-crf", crf.to_string());
      push_pair(&mut args, "-preset", preset.ffmpeg());
      if let Some(tune) = tune.ffmpeg() {
        push_pair(&mut args, "-tune", tune);
      }
      if *bit_depth == BitDepth::Ten {
        push_pair(&mut args, "-profile:v", "main10");
      }
    }
    VideoParams::Vp9 { cpu_used, crf, content } => {
      push_pair(&mut args, "-c:v", "libvpx-vp9");
      push_pair(&mut args, "-crf", crf.to_string());
      push_pair(&mut args, "-b:v", "0");
      push_pair(&mut args, "-deadline", "good");
      push_pair(&mut args, "-cpu-used", cpu_used.to_string());
      push_pair(&mut args, "-row-mt", "1");
      push_pair(&mut args, "-auto-alt-ref", "1");
      push_pair(&mut args, "-lag-in-frames", "25");
      push_pair(&mut args, "-frame-parallel", "0");
      push_pair(&mut args, "-tile-columns", tile_columns(width).to_string());
      if let Some(content) = content.ffmpeg() {
        push_pair(&mut args, "-tune-content", content);
      }
    }
  }

  push_pair(&mut args, "-pix_fmt", encode.video.pix_fmt());

  if audio.stream_index.is_some() {
    push_pair(&mut args, "-c:a", "aac");
    push_pair(&mut args, "-ac", audio.channels.to_string());
    push_pair(&mut args, "-b:a", format!("{}k", audio.bitrate_kbps));
    if audio.sample_rate > 0 {
      push_pair(&mut args, "-ar", audio.sample_rate.to_string());
    }
  } else {
    args.push("-an".to_string());
  }

  if matches!(encode.video, VideoParams::Hevc { .. })
    && matches!(encode.container, VideoContainer::Mp4 | VideoContainer::Mov)
  {
    push_pair(&mut args, "-tag:v", "hvc1");
  }
  if encode.faststart && matches!(encode.container, VideoContainer::Mp4 | VideoContainer::Mov) {
    push_pair(&mut args, "-movflags", "+faststart");
  }

  args
}

pub fn push_output(args: &mut Vec<String>, encode: &VideoEncode, output: &str) {
  push_pair(args, "-f", encode.container.muxer());
  args.push("-y".to_string());
  args.push(output.to_string());
  args.push("-hide_banner".to_string());
}

/// 这段上的标题时间。`None` 表示整段都画。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TitleSpan {
  All,
  Range(f64, f64),
  Skip,
}

pub fn title_span(title: &TitleOverlay, offset: f64, duration: f64) -> TitleSpan {
  if title.start.is_none() && title.end.is_none() {
    return TitleSpan::All;
  }
  let start = title.start.unwrap_or(0.0);
  let end = title.end.unwrap_or(f64::MAX);
  let local_start = (start - offset).max(0.0);
  let local_end = if end.is_finite() { (end - offset).min(duration) } else { duration };
  if local_start + 0.001 >= local_end { TitleSpan::Skip } else { TitleSpan::Range(local_start, local_end) }
}

pub fn source_codec_matches(codec: &str, video: &VideoParams) -> bool {
  let codec = codec.to_ascii_lowercase();
  match video {
    VideoParams::Av1 { .. } => codec.contains("av1"),
    VideoParams::Hevc { .. } => codec.contains("hevc") || codec.contains("h265"),
    VideoParams::Vp9 { .. } => codec.contains("vp9"),
  }
}

fn quote_filter(value: &str) -> String {
  let mut out = String::from("'");
  for ch in value.chars() {
    if ch == '\\' {
      out.push_str("\\\\");
    } else if ch == '\'' {
      out.push_str("'\\''");
    } else {
      out.push(ch);
    }
  }
  out.push('\'');
  out
}

fn scaled(base: f64, width: u32, min: u32) -> u32 {
  let width = if width == 0 { 1920 } else { width };
  ((base * width as f64 / 1920.0).round() as u32).max(min)
}

fn anchor_xy(anchor: TitleAnchor, margin: u32) -> (String, String) {
  let margin = margin.to_string();
  let x = match anchor {
    TitleAnchor::TopStart | TitleAnchor::CenterStart | TitleAnchor::BottomStart => margin.clone(),
    TitleAnchor::TopCenter | TitleAnchor::Center | TitleAnchor::BottomCenter => "(w-text_w)/2".to_string(),
    TitleAnchor::TopEnd | TitleAnchor::CenterEnd | TitleAnchor::BottomEnd => format!("w-text_w-{margin}"),
  };
  let y = match anchor {
    TitleAnchor::TopStart | TitleAnchor::TopCenter | TitleAnchor::TopEnd => margin,
    TitleAnchor::CenterStart | TitleAnchor::Center | TitleAnchor::CenterEnd => "(h-text_h)/2".to_string(),
    TitleAnchor::BottomStart | TitleAnchor::BottomCenter | TitleAnchor::BottomEnd => format!("h-text_h-{margin}"),
  };
  (x, y)
}

pub struct TitleDraw<'a> {
  pub text: &'a str,
  pub anchor: TitleAnchor,
  pub font_file: &'a str,
  pub font_size: u32,
  pub width: u32,
  pub window: Option<(f64, f64)>,
}

pub struct SubtitleDraw<'a> {
  pub file: &'a str,
  pub fonts_dir: Option<&'a str>,
  pub track_index: Option<u32>,
  pub force_style: Option<&'a str>,
  /// 裁剪时把时间轴拨回源视频，字幕时间才能对上。
  pub source_start: Option<f64>,
}

pub struct VideoFilter<'a> {
  pub fps: Option<f64>,
  pub size: Option<VideoSize>,
  pub title: Option<TitleDraw<'a>>,
  pub subtitle: Option<SubtitleDraw<'a>>,
  pub pix_fmt: &'a str,
}

pub fn video_filter(spec: &VideoFilter<'_>) -> Option<String> {
  let mut filters = Vec::new();
  if let Some(fps) = spec.fps.filter(|fps| *fps > 0.0) {
    filters.push(format!("fps={fps}"));
  }
  if let Some(size) = spec.size {
    let width = even_dim(size.width);
    let height = even_dim(size.height);
    filters.push("scale='trunc(iw*sar/2)*2':'trunc(ih/2)*2'".to_string());
    filters.push("setsar=1".to_string());
    match size.fit {
      Fit::Fill => filters.push(format!("scale={width}:{height}")),
      Fit::Contain | Fit::Cover => {
        filters.push(format!("scale={width}:{height}:force_original_aspect_ratio=decrease"));
        filters.push(format!("pad={width}:{height}:(ow-iw)/2:(oh-ih)/2"));
      }
    }
    filters.push("setsar=1".to_string());
  }
  if let Some(title) = &spec.title {
    let font_size = if title.font_size == 0 { 24 } else { title.font_size };
    let size = scaled(font_size as f64, title.width, 16);
    let margin = scaled(10.0, title.width, 8);
    let (x, y) = anchor_xy(title.anchor, margin);
    let mut draw = format!(
      "drawtext=fontfile={}:text={}:expansion=none:fontcolor=white:fontsize={size}:x={}:y={}",
      quote_filter(title.font_file),
      quote_filter(title.text),
      quote_filter(&x),
      quote_filter(&y),
    );
    if let Some((start, end)) = title.window {
      draw.push_str(&format!(":enable='between(t,{start:.3},{end:.3})'"));
    }
    filters.push(draw);
  }
  if let Some(subtitle) = &spec.subtitle {
    if subtitle.source_start.is_some_and(|start| start > 0.0) {
      filters.push(format!("setpts=PTS+{}/TB", subtitle.source_start.unwrap_or(0.0)));
    }
    let mut sub = format!("subtitles={}", quote_filter(subtitle.file));
    if let Some(index) = subtitle.track_index {
      sub.push_str(&format!(":si={index}"));
    }
    if let Some(dir) = subtitle.fonts_dir.filter(|dir| !dir.is_empty()) {
      sub.push_str(&format!(":fontsdir={}", quote_filter(dir)));
    }
    if let Some(style) = subtitle.force_style.filter(|style| !style.is_empty()) {
      sub.push_str(&format!(":force_style={}", quote_filter(style)));
    }
    filters.push(sub);
    if subtitle.source_start.is_some_and(|start| start > 0.0) {
      filters.push("setpts=PTS-STARTPTS".to_string());
    }
  }
  if filters.is_empty() {
    return None;
  }
  filters.push(format!("format={}", spec.pix_fmt));
  Some(filters.join(","))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn stereo() -> AudioPlan {
    AudioPlan { stream_index: Some(1), channels: 2, sample_rate: 48000, bitrate_kbps: 128, fallback: false }
  }

  #[test]
  fn default_json_is_av1_mp4() {
    let encode: VideoEncode = serde_json::from_str("{}").expect("默认对象");
    let joined = encode_args(&encode, &stereo(), 1920).join(" ");
    assert!(joined.contains("libsvtav1"), "{joined}");
    assert!(joined.contains("-preset 6"), "{joined}");
    assert!(joined.contains("-crf 35"), "{joined}");
    assert!(joined.contains("-pix_fmt yuv420p"), "{joined}");
    assert!(joined.contains("-c:a aac"), "{joined}");
    assert!(joined.contains("-b:a 128k"), "{joined}");
    assert!(joined.contains("-movflags +faststart"), "{joined}");
    assert!(!joined.contains("hvc1"), "{joined}");
  }

  #[test]
  fn hevc_shortcut_uses_its_own_defaults() {
    let encode: VideoEncode = serde_json::from_str(r#"{"video":{"codec":"hevc"}}"#).expect("hevc");
    match encode.video {
      VideoParams::Hevc { preset, crf, tune, bit_depth } => {
        assert_eq!(preset, HevcPreset::Medium);
        assert_eq!(crf, 26);
        assert_eq!(tune, HevcTune::None);
        assert_eq!(bit_depth, BitDepth::Eight);
      }
      other => panic!("应是 hevc: {other:?}"),
    }
  }

  #[test]
  fn hevc_ten_bit_mp4_tags_hvc1_and_mpegts_does_not() {
    let mut encode = VideoEncode {
      video: VideoParams::Hevc {
        preset: HevcPreset::Fast,
        crf: 22,
        tune: HevcTune::Animation,
        bit_depth: BitDepth::Ten,
      },
      ..VideoEncode::default()
    };
    let joined = encode_args(&encode, &AudioPlan { stream_index: None, ..stereo() }, 1280).join(" ");
    assert!(joined.contains("libx265"), "{joined}");
    assert!(joined.contains("-preset fast"), "{joined}");
    assert!(joined.contains("-tune animation"), "{joined}");
    assert!(joined.contains("-profile:v main10"), "{joined}");
    assert!(joined.contains("-pix_fmt yuv420p10le"), "{joined}");
    assert!(joined.contains("-tag:v hvc1"), "{joined}");
    assert!(joined.contains("-an"), "{joined}");

    encode.container = VideoContainer::MpegTs;
    normalize_encode(&mut encode).expect("mpegts");
    let joined = encode_args(&encode, &stereo(), 1280).join(" ");
    assert!(!joined.contains("hvc1"), "{joined}");
    assert!(!joined.contains("faststart"), "{joined}");
  }

  #[test]
  fn vp9_locks_internal_speed_flags() {
    let encode = VideoEncode {
      container: VideoContainer::Mp4,
      video: VideoParams::Vp9 { cpu_used: 2, crf: 33, content: Vp9Content::Screen },
      faststart: true,
      ..VideoEncode::default()
    };
    let joined = encode_args(&encode, &stereo(), 3840).join(" ");
    assert!(joined.contains("libvpx-vp9"), "{joined}");
    assert!(joined.contains("-b:v 0"), "{joined}");
    assert!(joined.contains("-deadline good"), "{joined}");
    assert!(joined.contains("-cpu-used 2"), "{joined}");
    assert!(joined.contains("-row-mt 1"), "{joined}");
    assert!(joined.contains("-tile-columns 3"), "{joined}");
    assert!(joined.contains("-tune-content screen"), "{joined}");
  }

  #[test]
  fn normalize_clamps_and_rejects() {
    let mut encode = VideoEncode {
      video: VideoParams::Av1 { preset: 99, crf: 80, bit_depth: BitDepth::Eight },
      fps: Some(400),
      ..VideoEncode::default()
    };
    normalize_encode(&mut encode).expect("夹取");
    match encode.video {
      VideoParams::Av1 { preset, crf, .. } => {
        assert_eq!(preset, 13);
        assert_eq!(crf, 63);
      }
      other => panic!("{other:?}"),
    }
    assert_eq!(encode.fps, None);

    encode.video = VideoParams::Vp9 { cpu_used: 2, crf: 33, content: Vp9Content::General };
    encode.container = VideoContainer::MpegTs;
    assert!(normalize_encode(&mut encode).unwrap_err().contains("VP9"));

    encode.container = VideoContainer::Mp4;
    encode.size = Some(VideoSize { width: 100, height: 100, fit: Fit::Cover });
    assert!(normalize_encode(&mut encode).unwrap_err().contains("铺满"));
  }

  #[test]
  fn title_window_maps_onto_a_segment() {
    let title = TitleOverlay { start: Some(8.0), end: Some(12.0), ..TitleOverlay::default() };
    assert_eq!(title_span(&title, 0.0, 5.0), TitleSpan::Skip);
    assert_eq!(title_span(&title, 10.0, 5.0), TitleSpan::Range(0.0, 2.0));
    assert_eq!(title_span(&TitleOverlay::default(), 3.0, 4.0), TitleSpan::All);
  }

  #[test]
  fn filter_puts_drawtext_before_subtitles() {
    let filter = video_filter(&VideoFilter {
      fps: Some(30.0),
      size: Some(VideoSize { width: 1919, height: 1080, fit: Fit::Contain }),
      title: Some(TitleDraw {
        text: "片名",
        anchor: TitleAnchor::TopStart,
        font_file: "/System/Library/Fonts/PingFang.ttc",
        font_size: 0,
        width: 1920,
        window: None,
      }),
      subtitle: Some(SubtitleDraw {
        file: "/tmp/a.srt",
        fonts_dir: Some("/Library/Fonts"),
        track_index: None,
        force_style: None,
        source_start: Some(5.0),
      }),
      pix_fmt: "yuv420p",
    })
    .expect("有滤镜");
    let title_at = filter.find("drawtext").expect("标题");
    let subtitle_at = filter.find("subtitles").expect("字幕");
    assert!(title_at < subtitle_at, "{filter}");
    assert!(filter.contains("scale=1918:1080"), "{filter}");
    assert!(filter.contains("setpts=PTS+5"), "{filter}");
    assert!(filter.ends_with("format=yuv420p"), "{filter}");
  }

  #[test]
  fn missing_audio_track_falls_back_to_the_first() {
    let tracks = vec![AudioTrack { index: 1, channels: 1, sample_rate: 44100, ..AudioTrack::default() }];
    let selection = AudioSelection { mode: AudioMode::Index { index: 9 }, ..AudioSelection::default() };
    let plan = plan_audio(&selection, &tracks, RatePolicy::Source);
    assert_eq!(plan.stream_index, Some(1));
    assert!(plan.fallback);
    assert_eq!(plan.channels, 1);
    assert_eq!(plan.sample_rate, 44100);
  }
}
