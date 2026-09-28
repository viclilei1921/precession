use serde::{Deserialize, Serialize};

use super::lunar::{ComprehensiveAlmanac, HourYiJi, LunarCell, WuXing};

/// 区间内的一天（官方、个人覆盖和周末合成）。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDay {
  pub date: String,
  /// ISO 星期：1=周一 … 7=周日
  pub weekday: i32,
  pub is_weekend: bool,
  pub is_workday: bool,
  pub holiday_name: Option<String>,
  pub official_is_off_day: Option<bool>,
  pub user_overridden: bool,
  pub lunar: Option<Lunar>,
}

/// 月历格子用的农历，不含宜忌。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Lunar {
  pub year: i32,
  pub month: u32,
  pub day: u32,
  pub is_leap: bool,
  pub text: String,
  pub gan_zhi: String,
  pub animal: String,
  pub solar_term: Option<String>,
}

impl From<LunarCell> for Lunar {
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
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarRange {
  pub from_date: String,
  pub to_date: String,
  pub days: Vec<CalendarDay>,
}

/// 保存某一天是否上班。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDayInput {
  pub is_workday: bool,
  pub note: Option<String>,
}

/// 个人对某一天的覆盖。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarOverride {
  pub id: String,
  pub date: String,
  pub is_workday: bool,
  pub note: Option<String>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// holiday-cn 单日。`isOffDay` 为 false 表示调休上班。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialDayInput {
  pub name: String,
  pub date: String,
  #[serde(alias = "is_off_day")]
  pub is_off_day: bool,
}

/// 按年替换官方节假日。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialImportInput {
  pub year: i32,
  #[serde(default)]
  pub days: Vec<OfficialDayInput>,
}

/// 官方导入结果。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialImport {
  pub year: i32,
  pub count: i64,
}

/// 单日黄历里的农历段。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlmanacLunar {
  pub l_year: i32,
  pub l_month: u32,
  pub l_day: u32,
  pub is_leap: bool,
  pub text: String,
}

/// 五行。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlmanacWuXing {
  pub nayin: String,
  pub gan_wx: String,
  pub zhi_wx: String,
}

impl From<WuXing> for AlmanacWuXing {
  fn from(value: WuXing) -> Self {
    AlmanacWuXing { nayin: value.nayin, gan_wx: value.gan_wx, zhi_wx: value.zhi_wx }
  }
}

/// 时辰宜忌。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlmanacHour {
  pub hour_index: u8,
  pub zhi: String,
  pub yi: String,
  pub ji: String,
  pub time_range: String,
  pub chong_sha: String,
  pub cai_shen: String,
  pub xi_shen: String,
  pub fu_shen: String,
}

impl From<HourYiJi> for AlmanacHour {
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
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Almanac {
  pub solar_date: String,
  pub lunar: AlmanacLunar,
  pub gan_zhi: String,
  pub animal: String,
  pub nc_week: String,
  pub solar_term: String,
  pub peng_zu: String,
  pub ji_shen: String,
  pub xiong_shen: String,
  pub jian_chu: String,
  pub dao_day: String,
  pub tai_shen: String,
  pub yi: String,
  pub ji: String,
  pub cai_shen: String,
  pub xi_shen: String,
  pub fu_shen: String,
  pub wu_xing: AlmanacWuXing,
  pub chong_sha: String,
  pub zhi_shen: String,
  pub hour_yi_ji: Vec<AlmanacHour>,
}

impl From<ComprehensiveAlmanac> for Almanac {
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
