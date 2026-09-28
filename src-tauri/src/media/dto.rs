use serde::{Deserialize, Serialize};

/// 挂在记录上的图片或视频
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
  pub id: String,
  pub owner: String,
  pub owner_id: String,
  pub kind: String,
  pub rel_path: String,
  pub mime: String,
  pub sort: i64,
  pub locked: bool,
  pub created_at: i64,
}

/// 写入媒体元数据
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInput {
  pub owner: String,
  pub owner_id: String,
  pub kind: String,
  pub rel_path: String,
  pub mime: String,
  pub sort: i64,
  pub locked: bool,
}
