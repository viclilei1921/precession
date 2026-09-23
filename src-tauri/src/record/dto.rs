use serde::{Deserialize, Serialize};

/// 今天和回顾看到的记录卡片
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordCard {
  pub id: String,
  #[serde(rename = "type")]
  pub r#type: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub subject_member_id: Option<String>,
  pub locked: bool,
  pub highlight: bool,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 新建或修改手记、里程碑、精彩瞬间
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWrite {
  #[serde(rename = "type")]
  pub r#type: String,
  pub occurred_at: i64,
  pub title: String,
  pub body: String,
  pub subject_member_id: Option<String>,
  pub locked: bool,
  pub highlight: bool,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
}

/// 挂在记录上的图片或视频元数据
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
  pub id: String,
  pub record_id: String,
  pub media_kind: String,
  pub rel_path: String,
  pub mime: String,
  pub sort: i64,
  pub locked: bool,
  pub created_at: i64,
}

/// 记录之间的引用
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordLink {
  pub id: String,
  pub from_id: String,
  pub to_id: String,
  pub kind: String,
  pub created_at: i64,
}

/// 一条记录及其成员、标签、地点、媒体
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDetail {
  pub record: RecordCard,
  pub member_ids: Vec<String>,
  pub tag_ids: Vec<String>,
  pub place_ids: Vec<String>,
  pub media: Vec<MediaItem>,
}
