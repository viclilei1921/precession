use serde::Serialize;

use crate::db::error::DbError;

/// 日历操作失败
///
/// 序列化时只写出错误文本。数据库锁定变成 `Locked`，其余数据库错误变成 `Internal`。
///
/// # 变体
/// - `DateInvalid`: 日期不是 `YYYY-MM-DD`
/// - `RangeOrder`: 结束早于开始
/// - `RangeTooLong`: 闭区间超过 366 天
/// - `LunarYear`: 黄历只算 1900–2100
/// - `YearRange`: 官方导入年份
/// - `DaysEmpty`: 官方导入没有日期
/// - `NameEmpty`: 节日名为空
/// - `DateYear`: 日期不在导入年份里
/// - `DuplicateDate`: 同一年里日期重复
/// - `OverrideMissing`: 这一天没有个人覆盖
/// - `Locked`: 数据库未解锁
/// - `Internal`: 操作失败
#[derive(Debug, thiserror::Error)]
pub enum CalendarError {
  /// 日期不是 `YYYY-MM-DD`
  #[error("日期格式不正确")]
  DateInvalid,
  /// 结束早于开始
  #[error("结束日期不能早于开始日期")]
  RangeOrder,
  /// 闭区间超过 366 天
  #[error("日期区间不能超过 366 天")]
  RangeTooLong,
  /// 黄历只算 1900–2100
  #[error("日期年份必须在 1900 到 2100 之间")]
  LunarYear,
  /// 官方导入年份
  #[error("年份必须在 1970 到 2100 之间")]
  YearRange,
  /// 官方导入没有日期
  #[error("官方日期不能为空")]
  DaysEmpty,
  /// 节日名为空
  #[error("节日名不能为空")]
  NameEmpty,
  /// 日期不在导入年份里
  #[error("日期必须属于该年")]
  DateYear,
  /// 同一年里日期重复
  #[error("日期重复")]
  DuplicateDate,
  /// 这一天没有个人覆盖
  #[error("这一天没有个人安排")]
  OverrideMissing,
  /// 数据库未解锁
  #[error("数据库未解锁")]
  Locked,
  /// 操作失败
  #[error("操作失败")]
  Internal,
}

impl From<DbError> for CalendarError {
  /// 把数据库错误转成日历错误
  ///
  /// # 参数
  /// - `err`: 数据库错误
  ///
  /// # 返回值
  /// 锁定时返回 `Locked`，其余返回 `Internal`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let err = CalendarError::from(DbError::Locked);
  /// ```
  fn from(err: DbError) -> Self {
    match err {
      DbError::Locked => CalendarError::Locked,
      _ => CalendarError::Internal,
    }
  }
}

impl Serialize for CalendarError {
  /// 把日历错误序列化成错误文本
  ///
  /// # 参数
  /// - `serializer`: 序列化器
  ///
  /// # 返回值
  /// 成功时返回序列化器的完成值
  ///
  /// # 错误
  /// 序列化器拒绝写入时返回其错误
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let text = serde_json::to_string(&CalendarError::DateInvalid);
  /// ```
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_string())
  }
}
