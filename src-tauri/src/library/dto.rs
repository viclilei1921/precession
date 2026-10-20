use serde::{Deserialize, Serialize};

/// 书架上的一本书
///
/// `status` 只能是 `want`、`reading` 或 `finished`。`progress` 在 0 到 1 之间。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 书 ID
/// - `title`: 书名
/// - `author`: 作者
/// - `cover_path`: 封面路径
/// - `status`: 阅读状态，`want`、`reading` 或 `finished`
/// - `progress`: 进度，0 到 1
/// - `rating`: 评分，可以没有
/// - `started_at`: 开始阅读时间，可以没有
/// - `finished_at`: 读完时间，可以没有
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
///
/// # 示例
/// 见 [`book_create`](super::commands::book_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
  /// 书 ID
  pub id: String,
  /// 书名
  pub title: String,
  /// 作者
  pub author: String,
  /// 封面路径
  pub cover_path: String,
  /// 阅读状态，`want`、`reading` 或 `finished`
  pub status: String,
  /// 进度，0 到 1
  pub progress: f64,
  /// 评分，可以没有
  pub rating: Option<i64>,
  /// 开始阅读时间，可以没有
  pub started_at: Option<i64>,
  /// 读完时间，可以没有
  pub finished_at: Option<i64>,
  /// 创建时间
  pub created_at: i64,
  /// 更新时间
  pub updated_at: i64,
}

/// 新建或修改书
///
/// 标题不能为空。状态只能是 `want`、`reading` 或 `finished`，进度必须在 0 到 1。
///
/// # 字段
/// - `title`: 书名，不能为空
/// - `author`: 作者
/// - `cover_path`: 封面路径
/// - `status`: 阅读状态，`want`、`reading` 或 `finished`
/// - `progress`: 进度，0 到 1
/// - `rating`: 评分，可以没有
/// - `started_at`: 开始阅读时间，可以没有
/// - `finished_at`: 读完时间，可以没有
///
/// # 示例
/// 见 [`book_create`](super::commands::book_create)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookInput {
  /// 书名，不能为空
  pub title: String,
  /// 作者
  pub author: String,
  /// 封面路径
  pub cover_path: String,
  /// 阅读状态，`want`、`reading` 或 `finished`
  pub status: String,
  /// 进度，0 到 1
  pub progress: f64,
  /// 评分，可以没有
  pub rating: Option<i64>,
  /// 开始阅读时间，可以没有
  pub started_at: Option<i64>,
  /// 读完时间，可以没有
  pub finished_at: Option<i64>,
}

/// 书摘或读书笔记
///
/// `kind` 只能是 `excerpt` 或 `note`。
///
/// # 字段
/// - `id`: 笔记 ID
/// - `book_id`: 所属书
/// - `kind`: 类型，`excerpt` 或 `note`
/// - `occurred_at`: 发生时间
/// - `title`: 标题
/// - `body`: 正文
/// - `chapter`: 章节
/// - `location`: 位置
/// - `locked`: 是否锁定
/// - `highlight`: 是否高亮
/// - `member_ids`: 成员 ID 列表
/// - `tag_ids`: 标签 ID 列表
/// - `place_ids`: 地点 ID 列表
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
///
/// # 示例
/// 见 [`book_note_create`](super::commands::book_note_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookNote {
  /// 笔记 ID
  pub id: String,
  /// 所属书
  pub book_id: String,
  /// 类型，`excerpt` 或 `note`
  pub kind: String,
  /// 发生时间
  pub occurred_at: i64,
  /// 标题
  pub title: String,
  /// 正文
  pub body: String,
  /// 章节
  pub chapter: String,
  /// 位置
  pub location: String,
  /// 是否锁定
  pub locked: bool,
  /// 是否高亮
  pub highlight: bool,
  /// 成员 ID 列表
  pub member_ids: Vec<String>,
  /// 标签 ID 列表
  pub tag_ids: Vec<String>,
  /// 地点 ID 列表
  pub place_ids: Vec<String>,
  /// 创建时间
  pub created_at: i64,
  /// 更新时间
  pub updated_at: i64,
}

/// 新建或修改书摘、读书笔记
///
/// 标题不能为空。类型只能是 `excerpt` 或 `note`。
///
/// # 字段
/// - `book_id`: 所属书
/// - `kind`: 类型，`excerpt` 或 `note`
/// - `occurred_at`: 发生时间
/// - `title`: 标题，不能为空
/// - `body`: 正文
/// - `chapter`: 章节
/// - `location`: 位置
/// - `locked`: 是否锁定
/// - `highlight`: 是否高亮
/// - `member_ids`: 成员 ID 列表
/// - `tag_ids`: 标签 ID 列表
/// - `place_ids`: 地点 ID 列表
///
/// # 示例
/// 见 [`book_note_create`](super::commands::book_note_create)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookNoteInput {
  /// 所属书
  pub book_id: String,
  /// 类型，`excerpt` 或 `note`
  pub kind: String,
  /// 发生时间
  pub occurred_at: i64,
  /// 标题，不能为空
  pub title: String,
  /// 正文
  pub body: String,
  /// 章节
  pub chapter: String,
  /// 位置
  pub location: String,
  /// 是否锁定
  pub locked: bool,
  /// 是否高亮
  pub highlight: bool,
  /// 成员 ID 列表
  pub member_ids: Vec<String>,
  /// 标签 ID 列表
  pub tag_ids: Vec<String>,
  /// 地点 ID 列表
  pub place_ids: Vec<String>,
}
