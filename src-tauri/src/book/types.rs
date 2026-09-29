//! 电子书统一类型：格式、元数据、章节与错误

use std::fmt;

/// 已支持的本地电子书格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
  Epub,
  Mobi,
  Txt,
}

impl Format {
  /// 和文件扩展名一致的格式名
  pub fn as_str(self) -> &'static str {
    match self {
      Format::Epub => "epub",
      Format::Mobi => "mobi",
      Format::Txt => "txt",
    }
  }

  /// 从探测结果字符串解析
  pub fn parse(s: &str) -> Option<Self> {
    match s {
      "epub" => Some(Format::Epub),
      "mobi" => Some(Format::Mobi),
      "txt" => Some(Format::Txt),
      _ => None,
    }
  }
}

impl fmt::Display for Format {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

/// 解析后的书籍元数据
#[derive(Debug, Clone)]
pub struct BookMeta {
  /// 书名
  pub title: String,
  /// 作者
  pub author: String,
  /// 格式
  pub format: Format,
}

/// 目录项（无正文）
#[derive(Debug, Clone)]
pub struct ChapterRef {
  /// 章节索引
  pub index: usize,
  /// 章节标题
  pub title: String,
}

/// 单章正文
#[derive(Debug, Clone)]
pub struct Chapter {
  /// 章节索引
  pub index: usize,
  /// 章节标题
  pub title: String,
  /// 章节内容
  pub content: String,
}

/// 电子书解析错误。给调用方的只有固定文案，解析细节写进日志。
#[derive(Debug, thiserror::Error)]
pub enum BookError {
  /// 扩展名和文件头都认不出
  #[error("不支持的电子书格式")]
  Unsupported,
  /// 路径上没有这个文件
  #[error("书籍文件不存在")]
  NotFound,
  /// 章节下标超出目录
  #[error("章节索引越界")]
  ChapterOutOfRange,
  /// 文件能打开，但内容解不开
  #[error("无法解析该电子书")]
  Invalid,
}

impl BookError {
  /// 记下解析细节，对外只返回「无法解析该电子书」
  pub(crate) fn invalid(err: impl std::fmt::Display) -> Self {
    tauri_plugin_log::log::error!("book: {err}");
    Self::Invalid
  }
}
