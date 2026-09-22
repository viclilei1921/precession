use serde::Serialize;

/// 示例表一行
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoItem {
  pub id: String,
  pub title: String,
  pub body: String,
  pub created_at: i64,
  pub updated_at: i64,
}
