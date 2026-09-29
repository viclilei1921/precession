//! EPUB 元数据与章节 DTO

use std::collections::HashMap;

/// EPUB 轻量元数据（标题 + 作者）
#[derive(Debug, Clone)]
pub struct EpubMeta {
  pub title: String,
  pub author: String,
}

/// 更完整的 EPUB 元数据（对齐 epub.js getMetadata 的常用字段）
#[derive(Debug, Clone, Default)]
pub struct EpubMetaFull {
  pub identifier: Option<String>,
  pub title: String,
  pub author: String,
  pub language: Vec<String>,
  pub description: Option<String>,
  pub publisher: Vec<String>,
  pub published: Option<String>,
  pub modified: Option<String>,
  pub subject: Vec<String>,
  pub rights: Option<String>,
  pub series_name: Option<String>,
  pub series_position: Option<f64>,
  pub alt_identifier: Vec<String>,
  pub source: Vec<String>,
  pub rendition: HashMap<String, String>,
  pub media: HashMap<String, String>,
  pub media_duration: Option<f64>,
}

/// EPUB 单章（标题 + HTML 正文）
#[derive(Debug, Clone)]
pub struct EpubChapter {
  pub title: String,
  pub content: String,
}
