//! 农历/黄历计算逻辑（自 Tauri 移植，保留原判定写法）。

#![allow(clippy::manual_range_contains, clippy::unnecessary_cast)]

use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use super::data::*;

/// 农历日期
#[derive(Debug, Clone, Serialize)]
pub struct LunarDate {
  pub l_year: i32,
  pub l_month: u32,
  pub l_day: u32,
  pub is_leap: bool,
}

/// 列表格子用的轻量农历（不含宜忌）。
#[derive(Debug, Clone, Serialize)]
pub struct LunarCell {
  pub year: i32,
  pub month: u32,
  pub day: u32,
  pub is_leap: bool,
  pub text: String,
  pub gan_zhi: String,
  pub animal: String,
  pub solar_term: Option<String>,
}

/// 时辰宜忌项
#[derive(Debug, Clone, Serialize)]
pub struct HourYiJi {
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

/// 综合黄历信息
#[derive(Debug, Clone, Serialize)]
pub struct ComprehensiveAlmanac {
  pub solar_date: String,
  pub lunar: LunarAlmanacLunar,
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
  pub wu_xing: WuXing,
  pub chong_sha: String,
  pub zhi_shen: String,
  pub hour_yi_ji: Vec<HourYiJi>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LunarAlmanacLunar {
  pub l_year: i32,
  pub l_month: u32,
  pub l_day: u32,
  pub is_leap: bool,
  pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WuXing {
  pub nayin: String,
  pub gan_wx: String,
  pub zhi_wx: String,
}

pub struct LunarCalendar;

impl LunarCalendar {
  pub const YEAR_MIN: i32 = 1900;
  pub const YEAR_MAX: i32 = 2100;

  pub fn supports_date(date: &NaiveDate) -> bool {
    (Self::YEAR_MIN..=Self::YEAR_MAX).contains(&date.year())
  }

  /// 绘制用轻量农历；超出 1900–2100 返回 None。
  pub fn lunar_cell(date: &NaiveDate) -> Option<LunarCell> {
    if !Self::supports_date(date) {
      return None;
    }
    let lunar = Self::get_lunar_date(date);
    if !(1..=12).contains(&lunar.l_month) {
      return None;
    }
    let solar_term = Self::twenty_four_term_days_of(date).and_then(|(ti, is_today)| {
      if is_today == 1 { Some(SOLAR_TERM[((ti + 24) % 24) as usize].to_string()) } else { None }
    });
    Some(LunarCell {
      year: lunar.l_year,
      month: lunar.l_month,
      day: lunar.l_day,
      is_leap: lunar.is_leap,
      text: Self::get_lunar_date_string(date),
      gan_zhi: Self::cyclical(Self::get_stems_branch_day(date)),
      animal: ANIMALS[((lunar.l_year - 1900) % 12 + 12) as usize % 12].to_string(),
      solar_term,
    })
  }

  /// 公历 → 农历
  pub fn get_lunar_date(date: &NaiveDate) -> LunarDate {
    let year = date.year();
    if year < 1900 || year > 2100 {
      return LunarDate { l_year: year, l_month: 1, l_day: 1, is_leap: false };
    }

    let base = NaiveDate::from_ymd_opt(1900, 1, 31).unwrap();
    let offset = date.signed_duration_since(base).num_days();
    if offset < 0 {
      return LunarDate { l_year: 1899, l_month: 12, l_day: 1, is_leap: false };
    }

    let mut offset = offset as i64;
    let mut temp: i64 = 0;
    let mut i: i32 = 1900;

    while i < 2101 && offset > 0 {
      temp = Self::l_year_days(i) as i64;
      offset -= temp;
      i += 1;
    }
    if offset < 0 {
      offset += temp;
      i -= 1;
    }

    let l_year = i;
    let leap = Self::leap_month(i);
    let mut is_leap = false;
    let mut l_month: i32 = 1;

    while l_month <= 12 && offset > 0 {
      if leap > 0 && l_month == leap + 1 && !is_leap {
        l_month -= 1;
        is_leap = true;
        temp = Self::leap_days(l_year) as i64;
      } else {
        temp = Self::month_days(l_year, l_month) as i64;
      }
      if is_leap && l_month == leap + 1 {
        is_leap = false;
      }
      offset -= temp;
      l_month += 1;
    }

    if offset == 0 && leap > 0 && l_month == leap + 1 {
      if is_leap {
        is_leap = false;
      } else {
        is_leap = true;
        l_month -= 1;
      }
    }
    if offset < 0 {
      offset += temp;
      l_month -= 1;
    }

    LunarDate { l_year, l_month: l_month as u32, l_day: (offset + 1) as u32, is_leap }
  }

  fn l_year_days(y: i32) -> u32 {
    let mut sum: u32 = 348;
    let idx = (y - 1900) as usize;
    if idx >= LUNAR_INFO.len() {
      return 348;
    }
    let info = LUNAR_INFO[idx];
    let mut i: u32 = 32768;
    while i > 8 {
      if info & i != 0 {
        sum += 1;
      }
      i >>= 1;
    }
    sum + Self::leap_days(y)
  }

  fn leap_days(y: i32) -> u32 {
    if Self::leap_month(y) > 0 {
      let idx = (y - 1900) as usize;
      if idx < LUNAR_INFO.len() && (LUNAR_INFO[idx] & 65536) != 0 { 30 } else { 29 }
    } else {
      0
    }
  }

  fn leap_month(y: i32) -> i32 {
    let idx = (y - 1900) as usize;
    if idx >= LUNAR_INFO.len() {
      return 0;
    }
    (LUNAR_INFO[idx] & 15) as i32
  }

  fn month_days(y: i32, m: i32) -> u32 {
    if m < 1 || m > 12 {
      return 0;
    }
    let idx = (y - 1900) as usize;
    if idx >= LUNAR_INFO.len() {
      return 29;
    }
    if (LUNAR_INFO[idx] & (65536 >> m)) != 0 { 30 } else { 29 }
  }

  fn day_of_year(date: &NaiveDate) -> u32 {
    let mut sum: u32 = 0;
    let month = date.month();
    let year = date.year();
    for i in 0..(month - 1) {
      sum += SOLAR_MONTH[i as usize];
    }
    sum += date.day();
    if month > 2 {
      let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
      if is_leap {
        sum += 1;
      }
    }
    sum
  }

  fn get_interval_days(d1: &NaiveDate, d2: &NaiveDate) -> i64 {
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    d2.signed_duration_since(epoch).num_days() - d1.signed_duration_since(epoch).num_days()
  }

  fn get_stems_branch_day(date: &NaiveDate) -> i32 {
    let base = NaiveDate::from_ymd_opt(1899, 2, 4).unwrap();
    let days = Self::get_interval_days(&base, date);
    if days < 0 {
      return -1;
    }
    let days = days as i32;
    let v = (6 * ((days + 9) % 10) - 5 * ((days + 3) % 12) + 60) % 60;
    if v < 0 { v + 60 } else { v }
  }

  fn cyclical(num: i32) -> String {
    if num < 0 || num >= 60 {
      return String::new();
    }
    format!("{}{}", GAN[(num % 10) as usize], ZHI[(num % 12) as usize])
  }

  fn get_branch_day(date: &NaiveDate) -> i32 {
    let base = NaiveDate::from_ymd_opt(1899, 2, 4).unwrap();
    let days = Self::get_interval_days(&base, date);
    if days < 0 {
      return 0;
    }
    ((days + 3) % 12) as i32
  }

  fn get_lunar_hour_index(hour: u32) -> u32 {
    ((hour / 2) + (hour % 2)) % 12
  }

  fn twenty_four_term_days_of(date: &NaiveDate) -> Option<(i32, i32)> {
    let y = date.year() - 1900;
    if y < 0 || y >= 200 {
      return None;
    }
    let d = Self::day_of_year(date) as i32 - 1;
    let terms = terms_offset();
    let base = (y as usize) * 24;
    if base + 24 > terms.len() {
      return None;
    }
    for i in 0..24 {
      let offset = terms[base + i] as i32;
      if offset > d {
        return Some((((y - 1) * 24 + i as i32), 0));
      }
      if offset == d {
        return Some((((y - 1) * 24 + i as i32), 1));
      }
    }
    // 冬至之后仍属该节气时段，但不是节气当天。
    Some((((y - 1) * 24 + 23), 0))
  }

  fn find_pre_term(year: i32, day_of_year_minus_1: i32) -> i32 {
    let base = (year - 1900) as usize;
    let terms = terms_offset();
    if base * 24 + 24 > terms.len() {
      return -1;
    }
    for i in 0..24 {
      let o = terms[base * 24 + i] as i32;
      if day_of_year_minus_1 == o {
        return i as i32;
      }
      if day_of_year_minus_1 < o {
        return i as i32 - 1;
      }
    }
    23
  }

  fn get_stems_branch_month(year: i32, day_of_year_minus_1: i32) -> i32 {
    let n = Self::find_pre_term(year, day_of_year_minus_1);
    if n < 0 {
      return -1;
    }
    let r = 12 * (year - 1899) + (n + 2) / 2 - 2;
    let v = (6 * ((r + 2) % 10) - 5 * ((r + 2) % 12) + 60) % 60;
    if v < 0 { v + 60 } else { v }
  }

  fn get_stems_branch_month_as_string(date: &NaiveDate) -> &'static str {
    let idx = Self::get_stems_branch_month(date.year(), Self::day_of_year(date) as i32 - 1);
    if idx < 0 { "" } else { ZHI[(idx % 12) as usize] }
  }

  fn get_advanced_h_data(date: &NaiveDate) -> (String, String) {
    let base = NaiveDate::from_ymd_opt(1901, 1, 1).unwrap();
    let r = Self::twenty_four_term_days_of(date);
    let mut y: i32 = -1;
    if let Some((a, b)) = r {
      let j = if a % 2 == 0 { a / 2 } else { a / 2 + 1 };
      let j = if b > 0 && a % 2 == 0 { j + 1 } else { j };
      let o = Self::get_interval_days(&base, date).abs();
      y = (((5 + o - j as i64) % 12) + 12) as i32 % 12;
    }
    let idx = if y >= 2 { y - 2 } else { y + 10 };
    let jian_chu = JIAN_CHU_NAMES.get(idx as usize).copied().unwrap_or("建日");
    let dao_day = if ["除", "定", "执", "危", "成", "开"].iter().any(|s| jian_chu.starts_with(s)) {
      "黄道"
    } else {
      "黑道"
    };
    (jian_chu.to_string(), dao_day.to_string())
  }

  fn get_chong_sha(date: &NaiveDate, hour_or_index: i32) -> String {
    let branch_index = if hour_or_index < 0 {
      Self::get_branch_day(date)
    } else {
      let hour_index =
        if hour_or_index <= 11 { hour_or_index } else { Self::get_lunar_hour_index(hour_or_index as u32) as i32 };
      let sb = Self::get_stem_branch_hour(date, hour_index);
      sb % 12
    };
    const CHONG_MAP: [usize; 12] = [6, 7, 8, 9, 10, 11, 0, 1, 2, 3, 4, 5];
    const SHA_DIR_MAP: [usize; 12] = [4, 2, 0, 6, 4, 2, 0, 6, 4, 2, 0, 6];
    let bi = (branch_index as usize) % 12;
    let chong_idx = CHONG_MAP.get(bi).copied().unwrap_or(0);
    let sha_idx = SHA_DIR_MAP.get(bi).copied().unwrap_or(4);
    let chong = ANIMALS.get(chong_idx).copied().unwrap_or("");
    let dir = COMPASS_NAMES.get(sha_idx).map(|s| s.strip_prefix('正').unwrap_or(s)).unwrap_or("南");
    format!("冲{}煞{}", chong, dir)
  }

  fn get_stem_branch_hour(date: &NaiveDate, hour_index: i32) -> i32 {
    let base = NaiveDate::from_ymd_opt(1899, 2, 4).unwrap();
    let days = Self::get_interval_days(&base, date) as i32;
    let dt = (days + 9) % 10;
    let dt_adj = if dt > 4 { dt - 5 } else { dt };
    let v = (6 * ((hour_index + 2 * dt_adj) % 10) - 5 * hour_index + 60) % 60;
    if v < 0 { v + 60 } else { v }
  }

  fn get_god_directions(date: &NaiveDate) -> (String, String, String) {
    let stem_index = Self::get_stems_branch_day(date) % 10;
    let cai = match stem_index {
      0 => 1,
      1 => 5,
      2 | 3 => 6,
      4 | 5 => 0,
      6 | 7 => 2,
      8 | 9 => 4,
      _ => 0,
    };
    let xi = match stem_index {
      0 | 5 => 1,
      1 | 6 => 7,
      2 | 7 => 5,
      3 | 8 => 4,
      4 | 9 => 3,
      _ => 0,
    };
    let fu = match stem_index {
      0 | 1 => 3,
      2 | 3 => 2,
      4 => 0,
      5 => 4,
      6 | 7 => 5,
      8 => 7,
      9 => 6,
      _ => 0,
    };
    (
      COMPASS_NAMES[cai as usize].to_string(),
      COMPASS_NAMES[xi as usize].to_string(),
      COMPASS_NAMES[fu as usize].to_string(),
    )
  }

  fn get_zhi_shen(date: &NaiveDate) -> String {
    let month_zhi = Self::get_stems_branch_month_as_string(date);
    let day_zhi_idx = Self::get_branch_day(date);
    let mut month_zhi_idx = 0;
    for (i, (z, _)) in MONTH_ZHI_TO_CODE.iter().enumerate() {
      if *z == month_zhi {
        month_zhi_idx = i;
        break;
      }
    }
    let offset: i32 = match month_zhi_idx {
      0 | 6 => 8,
      1 | 7 => 10,
      2 | 8 => 0,
      3 | 9 => 2,
      4 | 10 => 4,
      _ => 6,
    };
    let r = (day_zhi_idx - offset + 12) % 12;
    ZHI_SHEN_NAMES.get(r as usize).copied().unwrap_or("").to_string()
  }

  fn get_daily_yi_ji(date: &NaiveDate) -> Option<(String, String)> {
    let (a, b) = Self::twenty_four_term_days_of(date)?;
    let mut r = if a % 2 == 0 { a / 2 } else { a / 2 + 1 };
    if b > 0 && a % 2 == 0 {
      r += 1;
    }
    let base = NaiveDate::from_ymd_opt(1901, 1, 1).unwrap();
    let interval = Self::get_interval_days(&base, date).unsigned_abs();
    let v = 5i64 + interval as i64 - r as i64;
    let key1 = ((v % 12) + 12) % 12;
    let key2 = (15 + interval) % 60;
    let key = format!("{}-{}", key1, key2);
    let dict = super::data::daily_suit_avoid_dict();
    dict.get(&key).map(|v| (v.yi.clone(), v.ji.clone()))
  }

  fn get_hour_yi_ji(date: &NaiveDate, hour_index: u8) -> (String, String) {
    let day_idx = Self::get_stems_branch_day(date);
    if day_idx < 0 || day_idx >= 60 {
      return (String::new(), String::new());
    }
    let dict = super::data::almanac_core_dict();
    let rec = match dict.get(day_idx as usize) {
      Some(r) => r,
      None => return (String::new(), String::new()),
    };
    let idx = (hour_index as usize).min(11);
    let (yi, ji) = match idx {
      0 => (rec.yi0.clone(), rec.ji0.clone()),
      1 => (rec.yi1.clone(), rec.ji1.clone()),
      2 => (rec.yi2.clone(), rec.ji2.clone()),
      3 => (rec.yi3.clone(), rec.ji3.clone()),
      4 => (rec.yi4.clone(), rec.ji4.clone()),
      5 => (rec.yi5.clone(), rec.ji5.clone()),
      6 => (rec.yi6.clone(), rec.ji6.clone()),
      7 => (rec.yi7.clone(), rec.ji7.clone()),
      8 => (rec.yi8.clone(), rec.ji8.clone()),
      9 => (rec.yi9.clone(), rec.ji9.clone()),
      10 => (rec.yi10.clone(), rec.ji10.clone()),
      _ => (rec.yi11.clone(), rec.ji11.clone()),
    };
    (yi, ji)
  }

  /// 吉神宜趋、凶神宜忌 (神煞)，对应 TS getShenSha
  fn get_shen_sha(date: &NaiveDate) -> (String, String) {
    let day_of_year = Self::day_of_year(date);
    let j = Self::get_stems_branch_month(date.year(), day_of_year as i32 - 1);
    if j < 0 {
      return ("[无神煞]".to_string(), "[无神煞]".to_string());
    }
    let f = ((j + 10) % 12) + 1;
    let day_gz = Self::cyclical(Self::get_stems_branch_day(date));
    let key = format!("{}-{}", f, day_gz);
    let dict = super::data::auspicious_gods_dict();
    match dict.get(&key) {
      Some(rec) => (rec.jsyq.clone(), rec.xsyj.clone()),
      None => ("[无神煞]".to_string(), "[无神煞]".to_string()),
    }
  }

  /// 胎神方位 (宜避)，对应 TS getTaiShen
  fn get_tai_shen(date: &NaiveDate) -> String {
    let month_zhi = Self::get_stems_branch_month_as_string(date);
    if month_zhi.is_empty() {
      return "暂无".to_string();
    }
    let code = super::data::MONTH_ZHI_TO_CODE.iter().find(|(z, _)| *z == month_zhi).map(|(_, c)| *c);
    let code = match code {
      Some(c) => c,
      None => return "暂无".to_string(),
    };
    let day_gz = Self::cyclical(Self::get_stems_branch_day(date));
    let key = format!("{}-{}", code, day_gz);
    super::data::inuspicious_gods_dict().get(&key).cloned().unwrap_or_else(|| "暂无".to_string())
  }

  fn get_peng_zu_bai_ji(date: &NaiveDate) -> String {
    let index = Self::get_stems_branch_day(date);
    if index < 0 {
      return String::new();
    }
    let stem = (index % 10) as usize;
    let branch = (index % 12) as usize;
    format!(
      "{} {}",
      M_PZ_STEM.get(stem).copied().unwrap_or(""),
      M_PZ_BRANCH.get(branch).copied().unwrap_or("")
    )
  }

  fn get_wu_xing(date: &NaiveDate) -> WuXing {
    let gz = Self::cyclical(Self::get_stems_branch_day(date));
    let nayin = super::data::get_nayin(&gz);
    let mut chars = gz.chars();
    let (gan_wx, zhi_wx) = match (chars.next(), chars.next()) {
      (Some(g), Some(z)) => (
        super::data::gan_to_wuxing(&g.to_string()),
        super::data::zhi_to_wuxing(&z.to_string()),
      ),
      _ => (String::new(), String::new()),
    };
    WuXing { nayin, gan_wx, zhi_wx }
  }

  fn get_lunar_date_string(date: &NaiveDate) -> String {
    let lunar = Self::get_lunar_date(date);
    let month_idx = lunar.l_month.saturating_sub(1) as usize;
    if month_idx >= N_STR_3.len() {
      return String::new();
    }
    let month_str = format!("{}{}月", if lunar.is_leap { "闰" } else { "" }, N_STR_3[month_idx]);
    let day_str = match lunar.l_day {
      10 => "初十".to_string(),
      20 => "二十".to_string(),
      30 => "三十".to_string(),
      _ => format!(
        "{}{}",
        N_STR_2[(lunar.l_day / 10) as usize],
        N_STR_1[(lunar.l_day % 10) as usize]
      ),
    };
    format!("{}{}", month_str, day_str)
  }

  /// 时辰吉凶
  #[allow(dead_code)]
  pub fn get_hour_ji_xiong(date: &NaiveDate, hour: u32) -> String {
    let hour_index = ((hour / 2) + (hour % 2)) % 12;
    let day_branch_index = Self::get_stems_branch_day(date) % 12;
    if day_branch_index < 0 || day_branch_index >= 60 {
      return JX_NAMES[1].to_string();
    }
    let offset: i32 = match day_branch_index {
      0 | 6 => 8,
      1 | 7 => 10,
      2 | 8 => 0,
      3 | 9 => 2,
      4 | 10 => 4,
      _ => 6,
    };
    let mut diff = hour_index as i32 - offset;
    if diff < 0 {
      diff += 12;
    }
    let n = match diff {
      0 | 1 | 4 | 5 | 7 | 10 => 0,
      _ => 1,
    };
    JX_NAMES[n].to_string()
  }

  /// 综合黄历
  pub fn get_comprehensive_almanac(date: &NaiveDate) -> ComprehensiveAlmanac {
    let lunar = Self::get_lunar_date(date);
    let stem_branch = Self::cyclical(Self::get_stems_branch_day(date));
    let (jian_chu, dao_day) = Self::get_advanced_h_data(date);
    let (cai, xi, fu) = Self::get_god_directions(date);
    let lunar_str = Self::get_lunar_date_string(date);
    let animal = ANIMALS[((lunar.l_year - 1900) % 12 + 12) as usize % 12].to_string();
    let solar_term = Self::twenty_four_term_days_of(date)
      .map(|(ti, _)| SOLAR_TERM[((ti + 24) % 24) as usize].to_string())
      .unwrap_or_default();
    let weekday = date.weekday().num_days_from_sunday() as usize;
    let nc_week = format!("星期{}", N_STR_1.get(weekday).copied().unwrap_or(""));
    let peng_zu = Self::get_peng_zu_bai_ji(date);
    let chong_sha = Self::get_chong_sha(date, -1);
    let zhi_shen = Self::get_zhi_shen(date);
    let wu_xing = Self::get_wu_xing(date);

    let (yi, ji) = Self::get_daily_yi_ji(date).unwrap_or_else(|| ("暂无".to_string(), "暂无".to_string()));
    let (ji_shen, xiong_shen) = Self::get_shen_sha(date);
    let tai_shen = Self::get_tai_shen(date);

    let hour_yi_ji: Vec<HourYiJi> = [23, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21]
      .iter()
      .enumerate()
      .map(|(i, &h)| {
        let (yi, ji) = Self::get_hour_yi_ji(date, i as u8);
        let chong = Self::get_chong_sha(date, i as i32);
        let (c, x, f) = Self::get_god_directions_by_hour(date, i as i32);
        let time_range = get_fetal_gods(h);
        HourYiJi {
          hour_index: i as u8,
          zhi: ZHI[i].to_string(),
          yi,
          ji,
          time_range: time_range.unwrap_or_default(),
          chong_sha: chong,
          cai_shen: c,
          xi_shen: x,
          fu_shen: f,
        }
      })
      .collect();

    ComprehensiveAlmanac {
      solar_date: format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day()),
      lunar: LunarAlmanacLunar {
        l_year: lunar.l_year,
        l_month: lunar.l_month,
        l_day: lunar.l_day,
        is_leap: lunar.is_leap,
        text: lunar_str,
      },
      gan_zhi: stem_branch,
      animal,
      nc_week,
      solar_term,
      peng_zu,
      ji_shen,
      xiong_shen,
      jian_chu,
      dao_day,
      tai_shen,
      yi,
      ji,
      cai_shen: cai,
      xi_shen: xi,
      fu_shen: fu,
      wu_xing,
      chong_sha,
      zhi_shen,
      hour_yi_ji,
    }
  }

  fn get_god_directions_by_hour(date: &NaiveDate, hour: i32) -> (String, String, String) {
    let hour_index = if hour <= 11 { hour } else { Self::get_lunar_hour_index(hour as u32) as i32 };
    let sb = Self::get_stem_branch_hour(date, hour_index);
    let stem_index = sb % 10;
    let cai = match stem_index {
      0 => 1,
      1 => 5,
      2 | 3 => 6,
      4 | 5 => 0,
      6 | 7 => 2,
      8 | 9 => 4,
      _ => 0,
    };
    let xi = match stem_index {
      0 | 5 => 1,
      1 | 6 => 7,
      2 | 7 => 5,
      3 | 8 => 4,
      4 | 9 => 3,
      _ => 0,
    };
    let fu = match stem_index {
      0 | 1 => 3,
      2 | 3 => 2,
      4 => 0,
      5 => 4,
      6 | 7 => 5,
      8 => 7,
      9 => 6,
      _ => 0,
    };
    (
      COMPASS_NAMES[cai as usize].to_string(),
      COMPASS_NAMES[xi as usize].to_string(),
      COMPASS_NAMES[fu as usize].to_string(),
    )
  }
}

fn get_fetal_gods(hour: u32) -> Option<String> {
  let h = hour.min(23);
  let map: [(u32, &str); 12] = [
    (23, "23:00 - 00:59"),
    (0, "23:00 - 00:59"),
    (1, "01:00 - 02:59"),
    (3, "03:00 - 04:59"),
    (5, "05:00 - 06:59"),
    (7, "07:00 - 08:59"),
    (9, "09:00 - 10:59"),
    (11, "11:00 - 12:59"),
    (13, "13:00 - 14:59"),
    (15, "15:00 - 16:59"),
    (17, "17:00 - 18:59"),
    (19, "19:00 - 20:59"),
  ];
  let idx = h / 2;
  map.get(idx as usize).map(|(_, s)| s.to_string())
}

#[cfg(test)]
mod unit_tests {
  use super::*;

  fn solar_term(year: i32, month: u32, day: u32) -> Option<String> {
    let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
    LunarCalendar::lunar_cell(&date).and_then(|c| c.solar_term)
  }

  #[test]
  fn winter_solstice_only_on_term_day() {
    assert_eq!(solar_term(2026, 12, 21), None);
    assert_eq!(solar_term(2026, 12, 22).as_deref(), Some("冬至"));
    assert_eq!(solar_term(2026, 12, 23), None);
    assert_eq!(solar_term(2026, 12, 31), None);
    assert_eq!(solar_term(2026, 1, 5).as_deref(), Some("小寒"));
    assert_eq!(solar_term(2026, 1, 6), None);
  }
}
