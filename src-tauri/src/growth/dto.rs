use serde::{Deserialize, Serialize};

/// 一条成长记录：里程碑或精彩瞬间
///
/// `kind` 只能是 `milestone` 或 `moment`。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 成长记录 ID
/// - `kind`: 成长记录类型，`milestone` 或 `moment`
/// - `member_id`: 会员 ID
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
/// 见 [`growth_entry_create`](super::commands::growth_entry_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthEntry {
  /// 成长记录 ID
  pub id: String,
  /// 成长记录类型，`milestone` 或 `moment`
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
///
/// 标题不能为空，必须指定成员，类型只能是 `milestone` 或 `moment`。
///
/// # 字段
/// - `kind`: 成长记录类型，`milestone` 或 `moment`
/// - `member_id`: 会员 ID，不能为空
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
/// 见 [`growth_entry_create`](super::commands::growth_entry_create)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthEntryInput {
  /// 成长记录类型，`milestone` 或 `moment`
  pub kind: String,
  /// 会员 ID，不能为空
  pub member_id: String,
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
  /// 成员ID列表
  pub member_ids: Vec<String>,
  /// 标签ID列表
  pub tag_ids: Vec<String>,
  /// 地点ID列表
  pub place_ids: Vec<String>,
}

/// 一次身高体重
///
/// 身高用毫米，体重用克。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 测量 ID
/// - `member_id`: 成员 ID，不能为空
/// - `occurred_at`: 测量时间
/// - `height_mm`: 身高，毫米，必须大于 0
/// - `weight_g`: 体重，克，必须大于 0
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
///
/// # 示例
/// 见 [`growth_measure_create`](super::commands::growth_measure_create)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthMeasure {
  /// 测量 ID
  pub id: String,
  /// 成员 ID，不能为空
  pub member_id: String,
  /// 测量时间
  pub occurred_at: i64,
  /// 身高，毫米，必须大于 0
  pub height_mm: i64,
  /// 体重，克，必须大于 0
  pub weight_g: i64,
  /// 创建时间
  pub created_at: i64,
  /// 更新时间
  pub updated_at: i64,
}

/// 新建或修改一次身高体重
///
/// 必须指定成员，身高和体重都要大于 0。
///
/// # 字段
/// - `member_id`: 成员 ID，不能为空
/// - `occurred_at`: 测量时间
/// - `height_mm`: 身高，毫米，必须大于 0
/// - `weight_g`: 体重，克，必须大于 0
///
/// # 示例
/// 见 [`growth_measure_create`](super::commands::growth_measure_create)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrowthMeasureInput {
  /// 成员 ID，不能为空
  pub member_id: String,
  /// 测量时间
  pub occurred_at: i64,
  /// 身高，毫米，必须大于 0
  pub height_mm: i64,
  /// 体重，克，必须大于 0
  pub weight_g: i64,
}
