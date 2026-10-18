//! 把编码配置收成一条视频滤镜。没有标题、字幕、缩放和帧率时返回空。

use std::path::Path;

use super::models::{
  Fit, SubtitleDraw, SubtitleSource, TitleDraw, TitleSource, TitleSpan, VideoEncode, VideoFilter, VideoInfo, VideoSize,
  even_dim, title_span, video_filter,
};
use crate::utils::font::{default_fonts_dir, get_default_font_path};

pub struct GraphRequest<'a> {
  pub encode: &'a VideoEncode,
  pub path: &'a str,
  pub width: u32,
  /// 合并、追加传入对齐后的帧率。转码和裁剪留空，改用 `encode.fps`。
  pub fps: Option<f64>,
  /// 空则跟源。有值时按适配方式缩放。
  pub size: Option<VideoSize>,
  /// 这一段在成片时间轴上的起点。
  pub offset: f64,
  pub duration: f64,
  /// 裁剪片段在源视频上的起点，用来对齐字幕。
  pub source_start: Option<f64>,
}

pub fn build_video_filter(req: &GraphRequest<'_>) -> Option<String> {
  let fps = req.fps.or_else(|| req.encode.fps.map(|fps| fps as f64));
  let output_width = req.size.map(|size| size.width).filter(|width| *width > 0).unwrap_or(req.width);
  let font_file = font_file(req);
  let fonts_dir = fonts_dir(req);
  let text = req.encode.title.as_ref().and_then(|title| overlay_text(title, req));
  let window = req.encode.title.as_ref().and_then(|title| match title_span(title, req.offset, req.duration) {
    TitleSpan::Skip => None,
    TitleSpan::All => Some(None),
    TitleSpan::Range(start, end) => Some(Some((start, end))),
  });
  let title = match (req.encode.title.as_ref(), text, window) {
    (Some(overlay), Some(text), Some(window)) => Some(TitleDraw {
      text,
      anchor: overlay.anchor,
      font_file: &font_file,
      font_size: overlay.font_size,
      width: output_width,
      window,
    }),
    _ => None,
  };
  let subtitle_file = req.encode.subtitle.as_ref().map(|subtitle| match subtitle.source {
    SubtitleSource::ExternalFile => subtitle.file.clone().unwrap_or_default(),
    SubtitleSource::EmbeddedTrack => req.path.to_string(),
  });
  let subtitle = req.encode.subtitle.as_ref().zip(subtitle_file.as_deref()).map(|(subtitle, file)| SubtitleDraw {
    file,
    fonts_dir: Some(fonts_dir.as_str()),
    track_index: match subtitle.source {
      SubtitleSource::EmbeddedTrack => subtitle.track_index,
      SubtitleSource::ExternalFile => None,
    },
    force_style: subtitle.force_style.as_deref(),
    source_start: req.source_start,
  });

  video_filter(&VideoFilter { fps, size: req.size, title, subtitle, pix_fmt: req.encode.video.pix_fmt() })
}

fn overlay_text<'a>(title: &'a super::models::TitleOverlay, req: &'a GraphRequest<'_>) -> Option<&'a str> {
  let text = match title.source {
    TitleSource::Text => title.text.trim(),
    TitleSource::Filename => file_name(req.path),
  };
  if text.is_empty() { None } else { Some(text) }
}

fn font_file(req: &GraphRequest<'_>) -> String {
  req
    .encode
    .title
    .as_ref()
    .and_then(|title| title.font_file.clone())
    .filter(|path| Path::new(path).is_file())
    .map(|path| path.replace('\\', "/"))
    .unwrap_or_else(get_default_font_path)
}

fn fonts_dir(req: &GraphRequest<'_>) -> String {
  req
    .encode
    .subtitle
    .as_ref()
    .and_then(|subtitle| subtitle.fonts_dir.clone())
    .filter(|path| Path::new(path).is_dir())
    .map(|path| path.replace('\\', "/"))
    .unwrap_or_else(default_fonts_dir)
}

/// 目标画面。空配置表示跟源；宽高收成偶数。
pub fn explicit_size(encode: &VideoEncode) -> Option<VideoSize> {
  encode.size.map(|size| VideoSize {
    width: even_dim(size.width),
    height: even_dim(size.height),
    fit: if size.fit == Fit::Cover { Fit::Contain } else { size.fit },
  })
}

/// 合并时没有指定尺寸，就用最大宽高并补边。
pub fn canvas_size(width: u32, height: u32) -> VideoSize {
  VideoSize { width: even_dim(width), height: even_dim(height), fit: Fit::Contain }
}

pub fn file_label(path: &str) -> String {
  Path::new(path).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string())
}

fn file_name(path: &str) -> &str {
  Path::new(path).file_name().and_then(|name| name.to_str()).unwrap_or(path)
}

pub fn output_size(encode: &VideoEncode, info: &VideoInfo) -> (u32, u32) {
  if let Some(size) = explicit_size(encode) {
    (size.width, size.height)
  } else {
    (even_dim(info.width), even_dim(info.height))
  }
}
