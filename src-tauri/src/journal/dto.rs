use serde::{Deserialize, Serialize};

/// 一条手记：日记、灵感或写作
///
/// `kind` 只能是 `diary`、`spark` 或 `writing`。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 手记 ID
/// - `kind`: 手记类型，`diary`、`spark` 或 `writing`
/// - `occurred_at`: 发生时间
/// - `title`: 标题
/// - `body`: 内容
/// - `locked`: 是否锁定
/// - `highlight`: 是否高亮
/// - `member_ids`: 成员 ID 列表
/// - `tag_ids`: 标签 ID 列表
/// - `place_ids`: 地点 ID 列表
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
///
/// # 示例
/// 见 [`journal_entry_create`](super::commands::journal_entry_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
  /// 手记 ID
  pub id: String,
  /// 手记类型，`diary`、`spark` 或 `writing`
  pub kind: String,
  /// 发生时间
  pub occurred_at: i64,
  /// 标题
  pub title: String,
  /// 内容
  pub body: String,
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

/// 新建或修改手记
///
/// 标题不能为空。类型只能是 `diary`、`spark` 或 `writing`。
///
/// # 字段
/// - `kind`: 手记类型，`diary`、`spark` 或 `writing`
/// - `occurred_at`: 发生时间
/// - `title`: 标题，不能为空
/// - `body`: 内容
/// - `locked`: 是否锁定
/// - `highlight`: 是否高亮
/// - `member_ids`: 成员 ID 列表
/// - `tag_ids`: 标签 ID 列表
/// - `place_ids`: 地点 ID 列表
///
/// # 示例
/// 见 [`journal_entry_create`](super::commands::journal_entry_create)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryInput {
  /// 手记类型，`diary`、`spark` 或 `writing`
  pub kind: String,
  /// 发生时间
  pub occurred_at: i64,
  /// 标题，不能为空
  pub title: String,
  /// 内容
  pub body: String,
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

/// 手记之间的流转：灵感到写作，写作到日记
///
/// `kind` 只能是 `spark_to_writing` 或 `writing_to_diary`。
///
/// # 字段
/// - `id`: 流转 ID
/// - `from_id`: 来源手记
/// - `to_id`: 目标手记
/// - `kind`: 流转类型，`spark_to_writing` 或 `writing_to_diary`
/// - `created_at`: 创建时间
///
/// # 示例
/// 见 [`journal_link_create`](super::commands::journal_link_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalLink {
  /// 流转 ID
  pub id: String,
  /// 来源手记
  pub from_id: String,
  /// 目标手记
  pub to_id: String,
  /// 流转类型，`spark_to_writing` 或 `writing_to_diary`
  pub kind: String,
  /// 创建时间
  pub created_at: i64,
}

/// 手记引用的一条书摘或读书笔记
///
/// # 字段
/// - `id`: 引用 ID
/// - `entry_id`: 手记 ID
/// - `book_note_id`: 书摘或读书笔记 ID
/// - `created_at`: 创建时间
///
/// # 示例
/// 见 [`journal_citation_create`](super::commands::journal_citation_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalCitation {
  /// 引用 ID
  pub id: String,
  /// 手记 ID
  pub entry_id: String,
  /// 书摘或读书笔记 ID
  pub book_note_id: String,
  /// 创建时间
  pub created_at: i64,
}
