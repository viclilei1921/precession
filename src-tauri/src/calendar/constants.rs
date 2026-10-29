pub(super) const OFFICIAL_TABLE: &str = "calendar_official";
pub(super) const DAY_TABLE: &str = "calendar_day";

pub(super) const SCHEMA_VERSION_KEY: &str = "calendar.schema_version";
pub(super) const SCHEMA_VERSION: i64 = 1;

/// 区间查询允许的最大闭区间天数（含起止）。
pub(super) const MAX_RANGE_DAYS: i64 = 366;
