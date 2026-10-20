//! 一条记录属于哪个业务表。只有这里和 `lib.rs` 知道全部业务模块。

use rusqlite::Connection;

/// 记录属于哪个业务表
///
/// 只有这里和 `lib.rs` 知道全部业务模块。
///
/// # 变体
/// - `Plan`: 计划，字符串为 `plan`
/// - `JournalEntry`: 日记，字符串为 `journal_entry`
/// - `GrowthEntry`: 成长，字符串为 `growth_entry`
/// - `Book`: 书籍，字符串为 `book`
/// - `BookNote`: 书籍笔记，字符串为 `book_note`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
  /// 计划
  Plan,
  /// 日记
  JournalEntry,
  /// 成长
  GrowthEntry,
  /// 书籍
  Book,
  /// 书籍笔记
  BookNote,
}

impl Owner {
  /// 返回业务表对应的字符串
  ///
  /// # 返回值
  /// `plan`、`journal_entry`、`growth_entry`、`book` 或 `book_note`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(Owner::Plan.as_str(), "plan");
  /// assert_eq!(Owner::BookNote.as_str(), "book_note");
  /// ```
  pub(crate) fn as_str(self) -> &'static str {
    match self {
      Self::Plan => "plan",
      Self::JournalEntry => "journal_entry",
      Self::GrowthEntry => "growth_entry",
      Self::Book => "book",
      Self::BookNote => "book_note",
    }
  }

  /// 把字符串解析成业务表
  ///
  /// # 参数
  /// - `value`: 表名字符串
  ///
  /// # 返回值
  /// 识别出的变体；无法识别时为 `None`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(Owner::parse("journal_entry"), Some(Owner::JournalEntry));
  /// assert_eq!(Owner::parse("missing"), None);
  /// ```
  pub(crate) fn parse(value: &str) -> Option<Self> {
    Some(match value {
      "plan" => Self::Plan,
      "journal_entry" => Self::JournalEntry,
      "growth_entry" => Self::GrowthEntry,
      "book" => Self::Book,
      "book_note" => Self::BookNote,
      _ => return None,
    })
  }
}

/// 查询指定业务记录是否存在
///
/// # 参数
/// - `conn`: 已打开的数据库连接
/// - `owner`: 业务表
/// - `id`: 记录主键
///
/// # 返回值
/// 记录存在时为 `true`
///
/// # 错误
/// 查询失败时返回 `rusqlite::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let found = exists(&conn, Owner::Plan, id)?;
/// ```
pub(crate) fn exists(conn: &Connection, owner: Owner, id: &str) -> Result<bool, rusqlite::Error> {
  match owner {
    Owner::Plan => crate::plan::exists(conn, id),
    Owner::JournalEntry => crate::journal::exists(conn, id),
    Owner::GrowthEntry => crate::growth::exists(conn, id),
    Owner::Book => crate::library::book_exists(conn, id),
    Owner::BookNote => crate::library::note_exists(conn, id),
  }
}
