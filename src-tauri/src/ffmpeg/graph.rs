//! 把编码配置收成一条视频滤镜。没有标题、字幕、缩放和帧率时返回空，像素格式只靠 `-pix_fmt`。

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

/// 按请求拼出一条视频滤镜
///
/// 帧率优先用请求里的值，没有再用 `encode.fps`。标题要同时有文字和时间窗才画。外挂字幕用给定文件，内嵌字幕用这一段的源路径。字体文件或字体目录不存在时改用默认值。
///
/// # 参数
/// - `req`: 滤镜请求
///
/// # 返回值
/// 有帧率、尺寸、标题或字幕时返回滤镜字符串，否则 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let filter = build_video_filter(&req);
/// ```
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

/// 取出要画的标题文字
///
/// 自定义文字会去掉两端空白。文件名取这一段路径的文件名。空字符串返回 `None`。
///
/// # 参数
/// - `title`: 标题配置
/// - `req`: 滤镜请求
///
/// # 返回值
/// 非空文字
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let text = overlay_text(title, req);
/// ```
fn overlay_text<'a>(title: &'a super::models::TitleOverlay, req: &'a GraphRequest<'_>) -> Option<&'a str> {
  let text = match title.source {
    TitleSource::Text => title.text.trim(),
    TitleSource::Filename => file_name(req.path),
  };
  if text.is_empty() { None } else { Some(text) }
}

/// 选用标题字体文件
///
/// 配置了路径且该路径是文件时用它，并把反斜杠换成斜杠。否则用默认字体路径。
///
/// # 参数
/// - `req`: 滤镜请求
///
/// # 返回值
/// 字体文件路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let path = font_file(req);
/// ```
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

/// 选用字幕字体目录
///
/// 配置了路径且该路径是目录时用它，并把反斜杠换成斜杠。否则用默认字体目录。
///
/// # 参数
/// - `req`: 滤镜请求
///
/// # 返回值
/// 字体目录
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let dir = fonts_dir(req);
/// ```
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

/// 取出目标画面，并把宽高收成偶数
///
/// 没有配置时返回 `None`。`Cover` 在这里改成 `Contain`。
///
/// # 参数
/// - `encode`: 编码配置
///
/// # 返回值
/// 有尺寸时返回收成偶数后的画面
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let size = explicit_size(&VideoEncode::default());
/// assert!(size.is_none());
/// ```
pub fn explicit_size(encode: &VideoEncode) -> Option<VideoSize> {
  encode.size.map(|size| VideoSize {
    width: even_dim(size.width),
    height: even_dim(size.height),
    fit: if size.fit == Fit::Cover { Fit::Contain } else { size.fit },
  })
}

/// 用给定宽高做出补边用的画面
///
/// 宽高先收成偶数，适配方式固定为 `Contain`。
///
/// # 参数
/// - `width`: 宽度
/// - `height`: 高度
///
/// # 返回值
/// 目标画面
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let size = canvas_size(1919, 1080);
/// assert_eq!(size.width, 1918);
/// assert_eq!(size.fit, Fit::Contain);
/// ```
pub fn canvas_size(width: u32, height: u32) -> VideoSize {
  VideoSize { width: even_dim(width), height: even_dim(height), fit: Fit::Contain }
}

/// 取文件名，没有文件名时用整段路径
///
/// # 参数
/// - `path`: 文件路径
///
/// # 返回值
/// 用来显示的名字
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(file_label("a/b.mp4"), "b.mp4");
/// ```
pub fn file_label(path: &str) -> String {
  Path::new(path).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string())
}

/// 取 UTF-8 文件名，没有时用整段路径
///
/// # 参数
/// - `path`: 文件路径
///
/// # 返回值
/// 文件名或原路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(file_name("a/b.mp4"), "b.mp4");
/// ```
fn file_name(path: &str) -> &str {
  Path::new(path).file_name().and_then(|name| name.to_str()).unwrap_or(path)
}

/// 取输出宽高
///
/// 配置了尺寸时用收成偶数后的目标宽高，否则用源宽高的偶数。
///
/// # 参数
/// - `encode`: 编码配置
/// - `info`: 源视频信息
///
/// # 返回值
/// `(宽度, 高度)`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let (width, height) = output_size(&VideoEncode::default(), &info);
/// let _ = (width, height);
/// ```
pub fn output_size(encode: &VideoEncode, info: &VideoInfo) -> (u32, u32) {
  if let Some(size) = explicit_size(encode) {
    (size.width, size.height)
  } else {
    (even_dim(info.width), even_dim(info.height))
  }
}
