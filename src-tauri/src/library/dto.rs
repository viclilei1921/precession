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
pub struct BookInput {
  pub title: String,
  pub author: String,
  pub cover_path: String,
  pub status: String,
  pub progress: f64,
  pub rating: Option<i64>,
  pub started_at: Option<i64>,
  pub finished_at: Option<i64>,
}

/// 书摘或读书笔记
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookNote {
  pub id: String,
  pub book_id: String,
  pub kind: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub chapter: String,
  pub location: String,
  pub locked: bool,
  pub highlight: bool,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 新建或修改书摘、读书笔记
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookNoteInput {
  pub book_id: String,
  pub kind: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub chapter: String,
  pub location: String,
  pub locked: bool,
  pub highlight: bool,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
}
