use serde::{Deserialize, Serialize};

/// 一条成长记录：里程碑或精彩瞬间
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthEntry {
  /// 成长记录ID
  pub id: String,
  /// 成长记录类型
  pub kind: String,
  /// 会员ID
  pub member_id: String,
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
  /// 成员ID列表
  pub member_ids: Vec<String>,
  /// 标签ID列表
  pub tag_ids: Vec<String>,
  /// 地点ID列表
  pub place_ids: Vec<String>,
  /// 创建时间
  pub created_at: i64,
  /// 更新时间
  pub updated_at: i64,
}

/// 新建或修改成长记录
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthEntryInput {
  /// 成长记录类型
  pub kind: String,
  /// 会员ID
  pub member_id: String,
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
  /// 成员ID列表
  pub member_ids: Vec<String>,
  /// 标签ID列表
  pub tag_ids: Vec<String>,
  /// 地点ID列表
  pub place_ids: Vec<String>,
}
