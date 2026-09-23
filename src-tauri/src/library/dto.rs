use serde::{Deserialize, Serialize};

/// 书架上的一本书
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
  pub id: String,
  pub title: String,
  pub author: String,
  pub cover_path: String,
  pub status: String,
  pub progress: f64,
  pub rating: Option<i64>,
  pub started_at: Option<i64>,
  pub finished_at: Option<i64>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 新建或修改书
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookWrite {
  pub title: String,
  pub author: String,
  pub cover_path: String,
  pub status: String,
  pub progress: f64,
  pub rating: Option<i64>,
  pub started_at: Option<i64>,
  pub finished_at: Option<i64>,
}

/// 书摘或读书笔记，以及它在书里的位置
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteItem {
  pub id: String,
  pub book_id: String,
  #[serde(rename = "type")]
  pub r#type: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub chapter: String,
  pub location: String,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 新建或修改书摘、读书笔记
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteWrite {
  pub book_id: String,
  #[serde(rename = "type")]
  pub r#type: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub chapter: String,
  pub location: String,
}
