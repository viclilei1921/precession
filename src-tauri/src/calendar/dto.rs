use serde::{Deserialize, Serialize};

use super::lunar::{ComprehensiveAlmanac, HourYiJi, LunarCell, WuXing};

/// 区间内的一天（官方、个人覆盖和周末合成）。
///
/// 有个人覆盖时是否上班跟覆盖走；否则有官方记录时取放假的反面；都没有时周末不上班。
///
/// # 字段
/// - `date`: 公历日期
/// - `weekday`: ISO 星期：1=周一 … 7=周日
/// - `is_weekend`: 是否周末，`weekday` 大于等于 6
/// - `is_workday`: 这一天是否上班
/// - `holiday_name`: 官方节日名，没有官方记录时为空
/// - `official_is_off_day`: 官方是否放假，没有官方记录时为空
/// - `user_overridden`: 是否有个人覆盖
/// - `lunar`: 月历农历，年份不支持时为空
///
/// # 示例
/// ```
/// let saturday = CalendarDay {
///   date: "2026-09-26".into(),
///   weekday: 6,
///   is_weekend: true,
///   is_workday: false,
///   holiday_name: None,
///   official_is_off_day: None,
///   user_overridden: false,
///   lunar: None,
/// };
/// assert!(saturday.is_weekend && !saturday.is_workday);
/// ```
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDay {
  /// 公历日期
  pub date: String,
  /// ISO 星期：1=周一 … 7=周日
  pub weekday: i32,
  /// 是否周末，`weekday` 大于等于 6
  pub is_weekend: bool,
  /// 这一天是否上班
  pub is_workday: bool,
  /// 官方节日名，没有官方记录时为空
  pub holiday_name: Option<String>,
  /// 官方是否放假，没有官方记录时为空
  pub official_is_off_day: Option<bool>,
  /// 是否有个人覆盖
  pub user_overridden: bool,
  /// 月历农历，年份不支持时为空
  pub lunar: Option<Lunar>,
}

/// 月历格子用的农历，不含宜忌。
///
/// # 字段
/// - `year`: 农历年
/// - `month`: 农历月
/// - `day`: 农历日
/// - `is_leap`: 是否闰月
/// - `text`: 中文月日
/// - `gan_zhi`: 日干支
/// - `animal`: 生肖
/// - `solar_term`: 当天节气，不是节气当天则为空
///
/// # 示例
/// 见 [`Lunar::from`]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Lunar {
  /// 农历年
  pub year: i32,
  /// 农历月
  pub month: u32,
  /// 农历日
  pub day: u32,
  /// 是否闰月
  pub is_leap: bool,
  /// 中文月日
  pub text: String,
  /// 日干支
  pub gan_zhi: String,
  /// 生肖
  pub animal: String,
  /// 当天节气，不是节气当天则为空
  pub solar_term: Option<String>,
}

impl From<LunarCell> for Lunar {
  /// 把轻量农历抄成月历农历
  ///
  /// # 参数
  /// - `cell`: 轻量农历
  ///
  /// # 返回值
  /// 返回月历用的农历字段
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let lunar = Lunar::from(cell);
  /// ```
  fn from(cell: LunarCell) -> Self {
    Lunar {
      year: cell.year,
      month: cell.month,
      day: cell.day,
      is_leap: cell.is_leap,
      text: cell.text,
      gan_zhi: cell.gan_zhi,
      animal: cell.animal,
      solar_term: cell.solar_term,
    }
  }
}

/// 一次区间查询。
///
/// `list` 在结束早于开始，或闭区间超过 366 天时不会返回它。
///
/// # 字段
/// - `from_date`: 开始日期
/// - `to_date`: 结束日期
/// - `days`: 闭区间内的每一天
///
/// # 示例
/// 见 [`super::service::list`]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarRange {
  /// 开始日期
  pub from_date: String,
  /// 结束日期
  pub to_date: String,
  /// 闭区间内的每一天
  pub days: Vec<CalendarDay>,
}

/// 保存某一天是否上班。
///
/// 保存时会去掉备注空白，空则不存。
///
/// # 字段
/// - `is_workday`: 是否上班
/// - `note`: 备注
///
/// # 示例
/// ```
/// let input = CalendarDayInput { is_workday: true, note: Some("  ".into()) };
/// assert!(input.note.as_deref().is_some_and(|text| text.trim().is_empty()));
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDayInput {
  /// 是否上班
  pub is_workday: bool,
  /// 备注
  pub note: Option<String>,
}

/// 个人对某一天的覆盖。
///
/// # 字段
/// - `id`: 主键
/// - `date`: 日期
/// - `is_workday`: 是否上班
/// - `note`: 备注
/// - `created_at`: 创建时间，Unix 毫秒
/// - `updated_at`: 更新时间，Unix 毫秒
///
/// # 示例
/// 见 [`super::service::upsert_day`]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarOverride {
  /// 主键
  pub id: String,
  /// 日期
  pub date: String,
  /// 是否上班
  pub is_workday: bool,
  /// 备注
  pub note: Option<String>,
  /// 创建时间，Unix 毫秒
  pub created_at: i64,
  /// 更新时间，Unix 毫秒
  pub updated_at: i64,
}

/// holiday-cn 单日。
///
/// `isOffDay` 为 false 表示调休上班。
///
/// # 字段
/// - `name`: 节日名
/// - `date`: 日期
/// - `is_off_day`: 是否放假，false 表示调休上班
///
/// # 示例
/// ```
/// let makeup = OfficialDayInput {
///   name: "国庆节".into(),
///   date: "2026-09-26".into(),
///   is_off_day: false,
/// };
/// assert!(!makeup.is_off_day);
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialDayInput {
  /// 节日名
  pub name: String,
  /// 日期
  pub date: String,
  /// 是否放假，false 表示调休上班
  #[serde(alias = "is_off_day")]
  pub is_off_day: bool,
}

/// 按年替换官方节假日。
///
/// 校验时年份要在 1970 到 2100，日期不能空、不能重复，而且必须属于该年。
///
/// # 字段
/// - `year`: 年份
/// - `days`: 该年的官方日期，缺省为空
///
/// # 示例
/// ```
/// let input = OfficialImportInput {
///   year: 2026,
///   days: vec![OfficialDayInput {
///     name: "国庆节".into(),
///     date: "2026-10-01".into(),
///     is_off_day: true,
///   }],
/// };
/// assert!(input.days[0].date.starts_with(&input.year.to_string()));
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialImportInput {
  /// 年份
  pub year: i32,
  /// 该年的官方日期，缺省为空
  #[serde(default)]
  pub days: Vec<OfficialDayInput>,
}

/// 官方导入结果。
///
/// # 字段
/// - `year`: 年份
/// - `count`: 写入条数
///
/// # 示例
/// 见 [`super::service::import_official`]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialImport {
  /// 年份
  pub year: i32,
  /// 写入条数
  pub count: i64,
}

/// 单日黄历里的农历段。
///
/// # 字段
/// - `l_year`: 农历年
/// - `l_month`: 农历月
/// - `l_day`: 农历日
/// - `is_leap`: 是否闰月
/// - `text`: 中文月日
///
/// # 示例
/// 见 [`Almanac::from`]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlmanacLunar {
  /// 农历年
  pub l_year: i32,
  /// 农历月
  pub l_month: u32,
  /// 农历日
  pub l_day: u32,
  /// 是否闰月
  pub is_leap: bool,
  /// 中文月日
  pub text: String,
}

/// 五行。
///
/// # 字段
/// - `nayin`: 纳音
/// - `gan_wx`: 天干五行
/// - `zhi_wx`: 地支五行
///
/// # 示例
/// 见 [`AlmanacWuXing::from`]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlmanacWuXing {
  /// 纳音
  pub nayin: String,
  /// 天干五行
  pub gan_wx: String,
  /// 地支五行
  pub zhi_wx: String,
}

impl From<WuXing> for AlmanacWuXing {
  /// 把五行抄成黄历五行
  ///
  /// # 参数
  /// - `value`: 五行
  ///
  /// # 返回值
  /// 返回纳音、天干五行和地支五行
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let wu_xing = AlmanacWuXing::from(value);
  /// ```
  fn from(value: WuXing) -> Self {
    AlmanacWuXing { nayin: value.nayin, gan_wx: value.gan_wx, zhi_wx: value.zhi_wx }
  }
}

/// 时辰宜忌。
///
/// # 字段
/// - `hour_index`: 时辰序号
/// - `zhi`: 地支
/// - `yi`: 宜
/// - `ji`: 忌
/// - `time_range`: 时间范围
/// - `chong_sha`: 冲煞
/// - `cai_shen`: 财神方位
/// - `xi_shen`: 喜神方位
/// - `fu_shen`: 福神方位
///
/// # 示例
/// 见 [`AlmanacHour::from`]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlmanacHour {
  /// 时辰序号
  pub hour_index: u8,
  /// 地支
  pub zhi: String,
  /// 宜
  pub yi: String,
  /// 忌
  pub ji: String,
  /// 时间范围
  pub time_range: String,
  /// 冲煞
  pub chong_sha: String,
  /// 财神方位
  pub cai_shen: String,
  /// 喜神方位
  pub xi_shen: String,
  /// 福神方位
  pub fu_shen: String,
}

impl From<HourYiJi> for AlmanacHour {
  /// 把时辰宜忌抄成黄历时辰
  ///
  /// # 参数
  /// - `value`: 时辰宜忌
  ///
  /// # 返回值
  /// 返回时辰字段
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let hour = AlmanacHour::from(value);
  /// ```
  fn from(value: HourYiJi) -> Self {
    AlmanacHour {
      hour_index: value.hour_index,
      zhi: value.zhi,
      yi: value.yi,
      ji: value.ji,
      time_range: value.time_range,
      chong_sha: value.chong_sha,
      cai_shen: value.cai_shen,
      xi_shen: value.xi_shen,
      fu_shen: value.fu_shen,
    }
  }
}

/// 单日完整黄历。
///
/// # 字段
/// - `solar_date`: 公历日期
/// - `lunar`: 农历段
/// - `gan_zhi`: 日干支
/// - `animal`: 生肖
/// - `nc_week`: 星期
/// - `solar_term`: 节气，没有时为空
/// - `peng_zu`: 彭祖百忌
/// - `ji_shen`: 吉神
/// - `xiong_shen`: 凶神
/// - `jian_chu`: 建除
/// - `dao_day`: 黄道或黑道
/// - `tai_shen`: 胎神方位
/// - `yi`: 宜，没有时为「暂无」
/// - `ji`: 忌，没有时为「暂无」
/// - `cai_shen`: 财神方位
/// - `xi_shen`: 喜神方位
/// - `fu_shen`: 福神方位
/// - `wu_xing`: 五行
/// - `chong_sha`: 冲煞
/// - `zhi_shen`: 值神
/// - `hour_yi_ji`: 十二时辰宜忌
///
/// # 示例
/// 见 [`Almanac::from`]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Almanac {
  /// 公历日期
  pub solar_date: String,
  /// 农历段
  pub lunar: AlmanacLunar,
  /// 日干支
  pub gan_zhi: String,
  /// 生肖
  pub animal: String,
  /// 星期
  pub nc_week: String,
  /// 节气，没有时为空
  pub solar_term: String,
  /// 彭祖百忌
  pub peng_zu: String,
  /// 吉神
  pub ji_shen: String,
  /// 凶神
  pub xiong_shen: String,
  /// 建除
  pub jian_chu: String,
  /// 黄道或黑道
  pub dao_day: String,
  /// 胎神方位
  pub tai_shen: String,
  /// 宜，没有时为「暂无」
  pub yi: String,
  /// 忌，没有时为「暂无」
  pub ji: String,
  /// 财神方位
  pub cai_shen: String,
  /// 喜神方位
  pub xi_shen: String,
  /// 福神方位
  pub fu_shen: String,
  /// 五行
  pub wu_xing: AlmanacWuXing,
  /// 冲煞
  pub chong_sha: String,
  /// 值神
  pub zhi_shen: String,
  /// 十二时辰宜忌
  pub hour_yi_ji: Vec<AlmanacHour>,
}

impl From<ComprehensiveAlmanac> for Almanac {
  /// 把综合黄历抄成对外黄历
  ///
  /// # 参数
  /// - `value`: 综合黄历
  ///
  /// # 返回值
  /// 返回对外黄历，时辰逐项转换
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let book = Almanac::from(value);
  /// ```
  fn from(value: ComprehensiveAlmanac) -> Self {
    Almanac {
      solar_date: value.solar_date,
      lunar: AlmanacLunar {
        l_year: value.lunar.l_year,
        l_month: value.lunar.l_month,
        l_day: value.lunar.l_day,
        is_leap: value.lunar.is_leap,
        text: value.lunar.text,
      },
      gan_zhi: value.gan_zhi,
      animal: value.animal,
      nc_week: value.nc_week,
      solar_term: value.solar_term,
      peng_zu: value.peng_zu,
      ji_shen: value.ji_shen,
      xiong_shen: value.xiong_shen,
      jian_chu: value.jian_chu,
      dao_day: value.dao_day,
      tai_shen: value.tai_shen,
      yi: value.yi,
      ji: value.ji,
      cai_shen: value.cai_shen,
      xi_shen: value.xi_shen,
      fu_shen: value.fu_shen,
      wu_xing: AlmanacWuXing::from(value.wu_xing),
      chong_sha: value.chong_sha,
      zhi_shen: value.zhi_shen,
      hour_yi_ji: value.hour_yi_ji.into_iter().map(AlmanacHour::from).collect(),
    }
  }
}
