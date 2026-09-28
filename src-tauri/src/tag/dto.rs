use serde::Serialize;

/// 标签
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
  pub id: String,
  pub name: String,
  pub created_at: i64,
  pub updated_at: i64,
}
