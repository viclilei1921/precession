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

/// 读出的一张正文图片
///
/// `data` 是标准 base64，不含 `data:` 前缀。
///
/// # 字段
/// - `mime`: MIME 类型
/// - `data`: 图片字节的 base64
///
/// # 示例
/// 见 [`upload_image`](super::commands::upload_image)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadImage {
  /// MIME 类型
  pub mime: String,
  /// 图片字节的 base64
  pub data: String,
}

/// 文件目录里的一条记录
///
/// 在 [`Upload`] 上补所属记录的标题和封存状态。所属记录已删除时标题为空。书没有封存。
/// 书摘的 `parent_id` 和 `parent_title` 指向那本书，其余为空。
///
/// # 字段
/// - `upload`: 文件记录
/// - `owner_title`: 所属记录标题。记录已不在时为空
/// - `owner_kind`: 手记、成长、书摘的种类，或书的书架状态。计划为空
/// - `owner_sealed`: 所属记录是否封存。书恒为 `false`
/// - `owner_at`: 手记、成长、书摘的发生时间，书和计划的创建时间。记录已不在时为 0
/// - `parent_id`: 书摘所属的书。其余为空
/// - `parent_title`: 书摘所属书的标题。书已不在时为空
///
/// # 示例
/// 见 [`upload_catalog`](super::commands::upload_catalog)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadCatalog {
  /// 文件记录
  #[serde(flatten)]
  pub upload: Upload,
  /// 所属记录标题。记录已不在时为空
  pub owner_title: String,
  /// 手记、成长、书摘的种类，或书的书架状态。计划为空
  pub owner_kind: String,
  /// 所属记录是否封存。书恒为 `false`
  pub owner_sealed: bool,
  /// 手记、成长、书摘的发生时间，书和计划的创建时间。记录已不在时为 0
  pub owner_at: i64,
  /// 书摘所属的书。其余为空
  pub parent_id: String,
  /// 书摘所属书的标题。书已不在时为空
  pub parent_title: String,
}
