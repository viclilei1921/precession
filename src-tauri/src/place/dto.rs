use serde::Serialize;

/// 地点
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
  pub id: String,
  pub name: String,
  pub created_at: i64,
  pub updated_at: i64,
}
