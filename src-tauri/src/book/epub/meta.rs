//! EPUB 元数据与章节 DTO

use std::collections::HashMap;

/// EPUB 轻量元数据（标题 + 作者）
///
/// 包解析时标题或作者为空，会分别写成「未知标题」「未知作者」。
///
/// # 字段
/// - `title`: 书名
/// - `author`: 作者
///
/// # 示例
/// ```
/// let meta = EpubMeta { title: "书".to_string(), author: String::new() };
/// assert!(meta.author.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct EpubMeta {
  /// 书名
  pub title: String,
  /// 作者
  pub author: String,
}

/// 更完整的 EPUB 元数据
///
/// 对齐 epub.js getMetadata 的常用字段。
///
/// # 字段
/// - `identifier`: 主标识。优先取 `unique-identifier` 指向的那条，否则取第一条
/// - `title`: 书名，缺失时为「未知标题」
/// - `author`: 作者，取第一条 `creator`，缺失时为「未知作者」
/// - `language`: 全部语言
/// - `description`: 简介
/// - `publisher`: 全部出版者
/// - `published`: 出版日期。优先 `event="publication"`，否则用遇到的日期文本
/// - `modified`: 修改时间。来自 `dcterms:modified`，或 `event="modification"` 的日期
/// - `subject`: 全部主题
/// - `rights`: 权利声明
/// - `series_name`: 系列名。先取 `belongs-to-collection`，空缺时用 `calibre:series`
/// - `series_position`: 组内序号，按 `f64` 解析。先取 `group-position`，空缺时用 `calibre:series_index`
/// - `alt_identifier`: 其余标识，不含主标识
/// - `source`: 全部来源
/// - `rendition`: rendition 属性，键转成驼峰
/// - `media`: media overlay 属性，键转成驼峰
/// - `media_duration`: `media` 里 `duration` 解析出的秒数
///
/// # 示例
/// ```
/// let meta = EpubMetaFull {
///   title: "书".to_string(),
///   author: "人".to_string(),
///   series_position: Some(1.5),
///   ..EpubMetaFull::default()
/// };
/// assert_eq!(meta.series_position, Some(1.5));
/// assert!(meta.identifier.is_none());
/// ```
#[derive(Debug, Clone, Default)]
pub struct EpubMetaFull {
  /// 主标识。优先取 `unique-identifier` 指向的那条，否则取第一条
  pub identifier: Option<String>,
  /// 书名，缺失时为「未知标题」
  pub title: String,
  /// 作者，取第一条 `creator`，缺失时为「未知作者」
  pub author: String,
  /// 全部语言
  pub language: Vec<String>,
  /// 简介
  pub description: Option<String>,
  /// 全部出版者
  pub publisher: Vec<String>,
  /// 出版日期。优先 `event="publication"`，否则用遇到的日期文本
  pub published: Option<String>,
  /// 修改时间。来自 `dcterms:modified`，或 `event="modification"` 的日期
  pub modified: Option<String>,
  /// 全部主题
  pub subject: Vec<String>,
  /// 权利声明
  pub rights: Option<String>,
  /// 系列名。先取 `belongs-to-collection`，空缺时用 `calibre:series`
  pub series_name: Option<String>,
  /// 组内序号，按 `f64` 解析。先取 `group-position`，空缺时用 `calibre:series_index`
  pub series_position: Option<f64>,
  /// 其余标识，不含主标识
  pub alt_identifier: Vec<String>,
  /// 全部来源
  pub source: Vec<String>,
  /// rendition 属性，键转成驼峰
  pub rendition: HashMap<String, String>,
  /// media overlay 属性，键转成驼峰
  pub media: HashMap<String, String>,
  /// `media` 里 `duration` 解析出的秒数
  pub media_duration: Option<f64>,
}

/// EPUB 单章（标题 + HTML 正文）
///
/// 组装时标题为空会写成 `第 {n} 章`，序号从 1 开始。
///
/// # 字段
/// - `title`: 章标题
/// - `content`: HTML 正文
///
/// # 示例
/// ```
/// let chapter = EpubChapter { title: "第 1 章".to_string(), content: "<p>正文</p>".to_string() };
/// assert!(chapter.content.starts_with("<p>"));
/// ```
#[derive(Debug, Clone)]
pub struct EpubChapter {
  /// 章标题
  pub title: String,
  /// HTML 正文
  pub content: String,
}
