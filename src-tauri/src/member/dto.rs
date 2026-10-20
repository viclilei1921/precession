use serde::{Deserialize, Serialize};

/// 成员档案
///
/// 线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 成员 ID
/// - `name`: 名称
/// - `relation`: 关系
/// - `gender`: 性别
/// - `birthday`: 生日，可以没有
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
///
/// # 示例
/// 见 [`member_create`](super::commands::member_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
  /// 成员 ID
  pub id: String,
  /// 名称
  pub name: String,
  /// 关系
  pub relation: String,
  /// 性别
  pub gender: String,
  /// 生日，可以没有
  pub birthday: Option<i64>,
  /// 创建时间
  pub created_at: i64,
  /// 更新时间
  pub updated_at: i64,
}

/// 新建或修改成员
///
/// 名称不能为空。
///
/// # 字段
/// - `name`: 名称，不能为空
/// - `relation`: 关系
/// - `gender`: 性别
/// - `birthday`: 生日，可以没有
///
/// # 示例
/// 见 [`member_create`](super::commands::member_create)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberInput {
  /// 名称，不能为空
  pub name: String,
  /// 关系
  pub relation: String,
  /// 性别
  pub gender: String,
  /// 生日，可以没有
  pub birthday: Option<i64>,
}
