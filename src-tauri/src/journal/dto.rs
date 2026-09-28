use serde::{Deserialize, Serialize};

/// 一条手记：日记、灵感或写作
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
  pub id: String,
  pub kind: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub locked: bool,
  pub highlight: bool,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 新建或修改手记
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryInput {
  pub kind: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub locked: bool,
  pub highlight: bool,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
}

/// 手记之间的流转：灵感到写作，写作到日记
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalLink {
  pub id: String,
  pub from_id: String,
  pub to_id: String,
  pub kind: String,
  pub created_at: i64,
}

/// 手记引用的一条书摘或读书笔记
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalCitation {
  pub id: String,
  pub entry_id: String,
  pub book_note_id: String,
  pub created_at: i64,
}
