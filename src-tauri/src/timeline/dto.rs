use serde::Serialize;

/// 今天和回顾看到的一条记录
///
/// `kind` 来自计划、手记、成长记录或书摘：`plan`、`diary`、`spark`、`writing`、`milestone`、`moment`、`excerpt`、`note`。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 记录 ID
/// - `kind`: 记录类型，上面列出的八种之一
/// - `occurred_at`: 发生时间
/// - `title`: 标题
/// - `body`: 正文
/// - `locked`: 是否锁定
/// - `highlight`: 是否高亮
///
/// # 示例
/// 见 [`timeline_list`](super::commands::timeline_list)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
  /// 记录 ID
  pub id: String,
  /// 记录类型，`plan`、`diary`、`spark`、`writing`、`milestone`、`moment`、`excerpt` 或 `note`
  pub kind: String,
  /// 发生时间
  pub occurred_at: i64,
  /// 标题
  pub title: String,
  /// 正文
  pub body: String,
  /// 是否锁定
  pub locked: bool,
  /// 是否高亮
  pub highlight: bool,
}
