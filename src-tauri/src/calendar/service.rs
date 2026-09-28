//! 日历。不持有连接，一律走 `DbState::with_conn`。黄历不读库。

use std::collections::{HashMap, HashSet};

use chrono::{Datelike, Days, NaiveDate};

use super::constants::MAX_RANGE_DAYS;
use super::dto::{
  Almanac, CalendarDay, CalendarDayInput, CalendarOverride, CalendarRange, Lunar, OfficialImport, OfficialImportInput,
};
use super::error::CalendarError;
use super::lunar::LunarCalendar;
use super::repository::{self, OfficialInsert, OfficialRow};
use crate::db::state::DbState;

pub fn list(db: &DbState, from: &str, to: &str) -> Result<CalendarRange, CalendarError> {
  let from_date = parse_date(from)?;
  let to_date = parse_date(to)?;
  validate_range(from_date, to_date)?;
  let from_s = from_date.to_string();
  let to_s = to_date.to_string();
  let (official, overrides) = db.with_conn(|conn| -> Result<_, CalendarError> {
    let official = repository::list_official(conn, &from_s, &to_s)?;
    let overrides = repository::list_overrides(conn, &from_s, &to_s)?;
    Ok((official, overrides))
  })?;
  let official_map: HashMap<String, OfficialRow> =
    official.into_iter().map(|row| (row.cal_date.clone(), row)).collect();
  let override_map: HashMap<String, CalendarOverride> =
    overrides.into_iter().map(|row| (row.date.clone(), row)).collect();
  let days = dates_inclusive(from_date, to_date)
    .into_iter()
    .map(|date| compose_day(date, official_map.get(&date.to_string()), override_map.get(&date.to_string())))
    .collect();
  Ok(CalendarRange { from_date: from_s, to_date: to_s, days })
}

pub fn upsert_day(db: &DbState, date: &str, input: CalendarDayInput) -> Result<CalendarOverride, CalendarError> {
  let parsed = parse_date(date)?;
  let date = parsed.to_string();
  let note = normalize_note(input.note.as_deref());
  db.with_conn(|conn| repository::upsert_day(conn, &date, input.is_workday, note))
}

pub fn delete_day(db: &DbState, date: &str) -> Result<(), CalendarError> {
  let date = parse_date(date)?.to_string();
  db.with_conn(|conn| repository::delete_day(conn, &date))
}

pub fn import_official(db: &DbState, input: OfficialImportInput) -> Result<OfficialImport, CalendarError> {
  let (year, days) = validate_official(&input)?;
  let count = db.with_conn(|conn| repository::replace_official_year(conn, year, &days))?;
  Ok(OfficialImport { year, count })
}

pub fn almanac(date: &str) -> Result<Almanac, CalendarError> {
  let date = parse_date(date)?;
  if !LunarCalendar::supports_date(&date) {
    return Err(CalendarError::LunarYear);
  }
  Ok(Almanac::from(LunarCalendar::get_comprehensive_almanac(&date)))
}

fn compose_day(date: NaiveDate, official: Option<&OfficialRow>, user: Option<&CalendarOverride>) -> CalendarDay {
  let weekday = date.weekday().number_from_monday() as i32;
  let is_weekend = weekday >= 6;
  let is_workday = if let Some(day) = user {
    day.is_workday
  } else if let Some(day) = official {
    !day.is_off_day
  } else {
    !is_weekend
  };
  CalendarDay {
    date: date.to_string(),
    weekday,
    is_weekend,
    is_workday,
    holiday_name: official.map(|day| day.name.clone()),
    official_is_off_day: official.map(|day| day.is_off_day),
    user_overridden: user.is_some(),
    lunar: LunarCalendar::lunar_cell(&date).map(Lunar::from),
  }
}

fn parse_date(raw: &str) -> Result<NaiveDate, CalendarError> {
  NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d").map_err(|_| CalendarError::DateInvalid)
}

fn validate_range(from: NaiveDate, to: NaiveDate) -> Result<(), CalendarError> {
  if to < from {
    return Err(CalendarError::RangeOrder);
  }
  let days = to.signed_duration_since(from).num_days() + 1;
  if days > MAX_RANGE_DAYS {
    return Err(CalendarError::RangeTooLong);
  }
  Ok(())
}

fn dates_inclusive(from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
  let count = to.signed_duration_since(from).num_days();
  (0..=count).map(|offset| from + Days::new(offset as u64)).collect()
}

fn normalize_note(note: Option<&str>) -> Option<&str> {
  note.map(str::trim).filter(|text| !text.is_empty())
}

fn validate_official(input: &OfficialImportInput) -> Result<(i32, Vec<OfficialInsert>), CalendarError> {
  if !(1970..=2100).contains(&input.year) {
    return Err(CalendarError::YearRange);
  }
  if input.days.is_empty() {
    return Err(CalendarError::DaysEmpty);
  }
  let mut seen = HashSet::new();
  let mut days = Vec::with_capacity(input.days.len());
  for day in &input.days {
    let name = day.name.trim();
    if name.is_empty() {
      return Err(CalendarError::NameEmpty);
    }
    let date = parse_date(&day.date)?;
    if date.year() != input.year {
      return Err(CalendarError::DateYear);
    }
    let cal_date = date.to_string();
    if !seen.insert(cal_date.clone()) {
      return Err(CalendarError::DuplicateDate);
    }
    days.push(OfficialInsert { cal_date, name: name.to_string(), is_off_day: day.is_off_day });
  }
  Ok((input.year, days))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::calendar::dto::OfficialDayInput;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::calendar::migrate]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  fn day_named<'a>(range: &'a CalendarRange, date: &str) -> &'a CalendarDay {
    range.days.iter().find(|day| day.date == date).expect("day")
  }

  fn official(name: &str, date: &str, is_off_day: bool) -> OfficialDayInput {
    OfficialDayInput { name: name.to_string(), date: date.to_string(), is_off_day }
  }

  #[test]
  fn compose_override_and_import() {
    let (_dir, db) = setup();
    assert!(matches!(list(&db, "09-26", "2026-09-28"), Err(CalendarError::DateInvalid)));
    assert!(matches!(list(&db, "2026-09-28", "2026-09-26"), Err(CalendarError::RangeOrder)));
    assert!(matches!(
      list(&db, "2026-01-01", "2027-01-02"),
      Err(CalendarError::RangeTooLong)
    ));

    let weekend = list(&db, "2026-09-26", "2026-09-28").expect("weekend");
    let saturday = day_named(&weekend, "2026-09-26");
    assert!(saturday.is_weekend && !saturday.is_workday && !saturday.user_overridden);
    assert!(saturday.lunar.as_ref().is_some_and(|lunar| !lunar.text.is_empty()));
    let monday = day_named(&weekend, "2026-09-28");
    assert!(!monday.is_weekend && monday.is_workday);

    assert!(matches!(
      import_official(
        &db,
        OfficialImportInput { year: 1969, days: vec![official("元旦", "1969-01-01", true)] }
      ),
      Err(CalendarError::YearRange)
    ));
    assert!(matches!(
      import_official(&db, OfficialImportInput { year: 2026, days: vec![] }),
      Err(CalendarError::DaysEmpty)
    ));
    assert!(matches!(
      import_official(
        &db,
        OfficialImportInput { year: 2026, days: vec![official("  ", "2026-01-01", true)] }
      ),
      Err(CalendarError::NameEmpty)
    ));
    assert!(matches!(
      import_official(
        &db,
        OfficialImportInput { year: 2026, days: vec![official("元旦", "2025-01-01", true)] }
      ),
      Err(CalendarError::DateYear)
    ));
    assert!(matches!(
      import_official(
        &db,
        OfficialImportInput {
          year: 2026,
          days: vec![official("元旦", "2026-01-01", true), official("又一天", "2026-01-01", false)],
        }
      ),
      Err(CalendarError::DuplicateDate)
    ));

    let imported = import_official(
      &db,
      OfficialImportInput {
        year: 2026,
        days: vec![official("国庆节", "2026-10-01", true), official("调休", "2026-09-27", false)],
      },
    )
    .expect("import");
    assert_eq!(imported.count, 2);

    let mixed = list(&db, "2026-09-27", "2026-10-01").expect("mixed");
    let holiday = day_named(&mixed, "2026-10-01");
    assert_eq!(holiday.holiday_name.as_deref(), Some("国庆节"));
    assert_eq!(holiday.official_is_off_day, Some(true));
    assert!(!holiday.is_workday);
    let makeup = day_named(&mixed, "2026-09-27");
    assert!(makeup.is_weekend && makeup.is_workday);
    assert_eq!(makeup.official_is_off_day, Some(false));

    let saved = upsert_day(
      &db,
      "2026-09-26",
      CalendarDayInput { is_workday: true, note: Some("  ".into()) },
    )
    .expect("upsert");
    assert!(saved.is_workday && saved.note.is_none());
    let overridden = list(&db, "2026-09-26", "2026-09-26").expect("override");
    let saturday = day_named(&overridden, "2026-09-26");
    assert!(saturday.user_overridden && saturday.is_workday);

    delete_day(&db, "2026-09-26").expect("delete");
    let restored_range = list(&db, "2026-09-26", "2026-09-26").expect("restored");
    let restored = day_named(&restored_range, "2026-09-26");
    assert!(!restored.user_overridden && !restored.is_workday);
    assert!(matches!(delete_day(&db, "2026-09-26"), Err(CalendarError::OverrideMissing)));

    assert!(matches!(almanac("1899-01-01"), Err(CalendarError::LunarYear)));
    assert!(matches!(almanac("2101-01-01"), Err(CalendarError::LunarYear)));
    let book = almanac("2026-09-28").expect("almanac");
    assert_eq!(book.solar_date, "2026-09-28");
    assert!(!book.lunar.text.is_empty());
    assert!(!book.hour_yi_ji.is_empty());
  }

  #[test]
  fn locked() {
    let (_dir, db) = setup();
    db.lock().expect("lock");
    assert!(matches!(list(&db, "2026-09-28", "2026-09-28"), Err(CalendarError::Locked)));
  }
}
