use serde::{Deserialize, Serialize};

/// 成员档案
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
  pub id: String,
  pub name: String,
  pub relation: String,
  pub gender: String,
  pub birthday: Option<i64>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 新建或修改成员
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberWrite {
  pub name: String,
  pub relation: String,
  pub gender: String,
  pub birthday: Option<i64>,
}

/// 标签
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
  pub id: String,
  pub name: String,
  pub created_at: i64,
  pub updated_at: i64,
}

/// 地点
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
  pub id: String,
  pub name: String,
  pub created_at: i64,
  pub updated_at: i64,
}
