use serde::{Deserialize, Serialize};

/// 挂在记录上的图片或视频
///
/// `kind` 只能是 `image` 或 `video`。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 媒体 ID
/// - `owner`: 所属记录类型
/// - `owner_id`: 所属记录 ID
/// - `kind`: 媒体类型，`image` 或 `video`
/// - `rel_path`: 相对路径
/// - `mime`: MIME 类型
/// - `sort`: 排序
/// - `locked`: 是否锁定
/// - `encrypted`: 是否已加密
/// - `created_at`: 创建时间
///
/// # 示例
/// 见 [`media_create`](super::commands::media_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
  /// 媒体 ID
  pub id: String,
  /// 所属记录类型
  pub owner: String,
  /// 所属记录 ID
  pub owner_id: String,
  /// 媒体类型，`image` 或 `video`
  pub kind: String,
  /// 相对路径
  pub rel_path: String,
  /// MIME 类型
  pub mime: String,
  /// 排序
  pub sort: i64,
  /// 是否锁定
  pub locked: bool,
  /// 是否已加密
  pub encrypted: bool,
  /// 创建时间
  pub created_at: i64,
}

/// 写入媒体元数据
///
/// 类型只能是 `image` 或 `video`。
///
/// # 字段
/// - `owner`: 所属记录类型
/// - `owner_id`: 所属记录 ID
/// - `kind`: 媒体类型，`image` 或 `video`
/// - `rel_path`: 相对路径
/// - `mime`: MIME 类型
/// - `sort`: 排序
/// - `locked`: 是否锁定
///
/// # 示例
/// 见 [`media_create`](super::commands::media_create)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInput {
  /// 所属记录类型
  pub owner: String,
  /// 所属记录 ID
  pub owner_id: String,
  /// 媒体类型，`image` 或 `video`
  pub kind: String,
  /// 相对路径
  pub rel_path: String,
  /// MIME 类型
  pub mime: String,
  /// 排序
  pub sort: i64,
  /// 是否锁定
  pub locked: bool,
}
