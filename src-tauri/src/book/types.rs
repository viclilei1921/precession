//! 电子书统一类型：格式、元数据、章节与错误

use std::fmt;

/// 已支持的本地电子书格式
///
/// # 变体
/// - `Epub`: EPUB，字符串为 `epub`
/// - `Mobi`: MOBI，字符串为 `mobi`
/// - `Txt`: 纯文本，字符串为 `txt`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
  /// EPUB，字符串为 `epub`
  Epub,
  /// MOBI，字符串为 `mobi`
  Mobi,
  /// 纯文本，字符串为 `txt`
  Txt,
}

impl Format {
  /// 返回与扩展名一致的格式名
  ///
  /// # 返回值
  /// `epub`、`mobi` 或 `txt`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(Format::Epub.as_str(), "epub");
  /// assert_eq!(Format::Mobi.as_str(), "mobi");
  /// assert_eq!(Format::Txt.as_str(), "txt");
  /// ```
  pub fn as_str(self) -> &'static str {
    match self {
      Format::Epub => "epub",
      Format::Mobi => "mobi",
      Format::Txt => "txt",
    }
  }

  /// 把格式名解析为枚举
  ///
  /// # 参数
  /// - `s`: 格式名，只接受 `epub`、`mobi`、`txt`
  ///
  /// # 返回值
  /// 匹配时返回对应格式，其余返回 `None`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(Format::parse("epub"), Some(Format::Epub));
  /// assert_eq!(Format::parse("pdf"), None);
  /// ```
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
  /// 把格式名写入格式化器
  ///
  /// # 参数
  /// - `f`: 输出格式化器
  ///
  /// # 返回值
  /// 写入成功时返回 `Ok(())`
  ///
  /// # 错误
  /// 写入失败时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(Format::Txt.to_string(), "txt");
  /// ```
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

/// 解析后的书籍元数据
///
/// # 字段
/// - `title`: 书名
/// - `author`: 作者
/// - `format`: 格式，只有 epub、mobi、txt
///
/// # 示例
/// ```
/// let meta = BookMeta {
///   title: "书".to_string(),
///   author: "人".to_string(),
///   format: Format::Txt,
/// };
/// assert_eq!(meta.format.as_str(), "txt");
/// ```
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
///
/// # 字段
/// - `index`: 章节索引
/// - `title`: 章节标题
///
/// # 示例
/// ```
/// let outline = [
///   ChapterRef { index: 0, title: "序".to_string() },
///   ChapterRef { index: 1, title: "正文".to_string() },
/// ];
/// assert_eq!(outline[1].index, 1);
/// ```
#[derive(Debug, Clone)]
pub struct ChapterRef {
  /// 章节索引
  pub index: usize,
  /// 章节标题
  pub title: String,
}

/// 单章正文
///
/// # 字段
/// - `index`: 章节索引
/// - `title`: 章节标题
/// - `content`: 章节内容
///
/// # 示例
/// ```
/// let chapter = Chapter { index: 0, title: "正文".to_string(), content: "段落".to_string() };
/// assert!(chapter.content.contains("段落"));
/// ```
#[derive(Debug, Clone)]
pub struct Chapter {
  /// 章节索引
  pub index: usize,
  /// 章节标题
  pub title: String,
  /// 章节内容
  pub content: String,
}

/// 电子书解析错误
///
/// 给调用方的只有固定文案，解析细节写进日志。
///
/// # 变体
/// - `Unsupported`: 扩展名和文件头都认不出
/// - `NotFound`: 路径上没有这个文件
/// - `ChapterOutOfRange`: 章节下标超出目录
/// - `Invalid`: 文件能打开，但内容解不开
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
  /// 记录解析细节并返回统一的无效错误
  ///
  /// 把 `err` 的显示文本写入错误日志，对外只返回 `BookError::Invalid`。
  ///
  /// # 参数
  /// - `err`: 需要记入日志的解析细节
  ///
  /// # 返回值
  /// 始终返回 `BookError::Invalid`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let err = BookError::invalid("opf 缺少 spine");
  /// assert!(matches!(err, BookError::Invalid));
  /// ```
  pub(crate) fn invalid(err: impl std::fmt::Display) -> Self {
    tauri_plugin_log::log::error!("book: {err}");
    Self::Invalid
  }
}
