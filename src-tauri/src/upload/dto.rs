use serde::{Deserialize, Serialize};

/// 上传到应用里、挂在某条记录上的文件
///
/// `kind` 只能是 `image`、`video`、`book` 或 `file`。文件本体在数据目录的 `upload/{kind}/` 下。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 文件 ID
/// - `owner`: 所属记录类型
/// - `owner_id`: 所属记录 ID
/// - `kind`: 文件类型，`image`、`video`、`book` 或 `file`
/// - `rel_path`: 相对数据根目录的路径
/// - `mime`: MIME 类型
/// - `name`: 原始文件名
/// - `size`: 导入时的源文件字节数
/// - `sort`: 排序
/// - `locked`: 是否锁定
/// - `encrypted`: 是否已加密
/// - `created_at`: 创建时间
///
/// # 示例
/// 见 [`upload_create`](super::commands::upload_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Upload {
  /// 文件 ID
  pub id: String,
  /// 所属记录类型
  pub owner: String,
  /// 所属记录 ID
  pub owner_id: String,
  /// 文件类型，`image`、`video`、`book` 或 `file`
  pub kind: String,
  /// 相对数据根目录的路径
  pub rel_path: String,
  /// MIME 类型
  pub mime: String,
  /// 原始文件名
  pub name: String,
  /// 导入时的源文件字节数
  pub size: i64,
  /// 排序
  pub sort: i64,
  /// 是否锁定
  pub locked: bool,
  /// 是否已加密
  pub encrypted: bool,
  /// 创建时间
  pub created_at: i64,
}

/// 写入文件元数据
///
/// 类型只能是 `image`、`video`、`book` 或 `file`。`name`、`size` 缺省时为空和 0。
///
/// # 字段
/// - `owner`: 所属记录类型
/// - `owner_id`: 所属记录 ID
/// - `kind`: 文件类型，`image`、`video`、`book` 或 `file`
/// - `rel_path`: 相对数据根目录的路径
/// - `mime`: MIME 类型
/// - `name`: 原始文件名
/// - `size`: 源文件字节数
/// - `sort`: 排序
/// - `locked`: 是否锁定
///
/// # 示例
/// 见 [`upload_create`](super::commands::upload_create)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadInput {
  /// 所属记录类型
  pub owner: String,
  /// 所属记录 ID
  pub owner_id: String,
  /// 文件类型，`image`、`video`、`book` 或 `file`
  pub kind: String,
  /// 相对数据根目录的路径
  pub rel_path: String,
  /// MIME 类型
  pub mime: String,
  /// 原始文件名
  #[serde(default)]
  pub name: String,
  /// 源文件字节数
  #[serde(default)]
  pub size: i64,
  /// 排序
  pub sort: i64,
  /// 是否锁定
  pub locked: bool,
}
