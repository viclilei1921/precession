use serde::Serialize;

/// 标签
///
/// 名称不能为空，且不能与已有标签重名。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 标签 ID
/// - `name`: 名称
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
///
/// # 示例
/// 见 [`tag_create`](super::commands::tag_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
  /// 标签 ID
  pub id: String,
  /// 名称
  pub name: String,
  /// 创建时间
  pub created_at: i64,
  /// 更新时间
  pub updated_at: i64,
}
