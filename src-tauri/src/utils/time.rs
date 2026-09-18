use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_utc8() -> (i32, u8, u8, u8, u8, u8) {
  const OFFSET_SECS: i64 = 8 * 3600;
  let unix = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0) + OFFSET_SECS;

  let days = unix.div_euclid(86_400);
  let tod = unix.rem_euclid(86_400) as u32;
  let hour = (tod / 3_600) as u8;
  let minute = ((tod % 3_600) / 60) as u8;
  let second = (tod % 60) as u8;

  let z = days + 719_468;
  let era = z.div_euclid(146_097);
  let doe = z.rem_euclid(146_097);
  let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
  let mut year = yoe + era * 400;
  let doy = doe - (365 * yoe + yoe / 4 - yoe / 36_524);
  let mp = (5 * doy + 2) / 153;
  let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
  let month = if mp < 10 { mp + 3 } else { mp - 9 };
  if month <= 2 {
    year += 1;
  }

  (year as i32, month as u8, day, hour, minute, second)
}
