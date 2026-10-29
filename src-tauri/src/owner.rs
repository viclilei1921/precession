//! 一条记录属于哪个业务表。只有这里和 `lib.rs` 知道全部业务模块。

use rusqlite::{Connection, OptionalExtension};

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
/// - `Member`: 家人，字符串为 `member`
/// - `Loose`: 不挂在记录上的文件，字符串为 `loose`
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
  /// 家人
  Member,
  /// 不挂在记录上的文件
  Loose,
}

impl Owner {
  /// 返回业务表对应的字符串
  ///
  /// # 返回值
  /// `plan`、`journal_entry`、`growth_entry`、`book`、`book_note`、`member` 或 `loose`
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
      Self::Member => "member",
      Self::Loose => "loose",
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
      "member" => Self::Member,
      "loose" => Self::Loose,
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
    Owner::Member => crate::member::exists(conn, id),
    Owner::Loose => Ok(true),
  }
}

/// 所属记录是否已封存
///
/// 书、家人和未挂载的文件没有封存。所属类型无法识别、记录不存在或已删除时为 `false`，调用方可以删掉残留文件。
///
/// # 参数
/// - `conn`: 已打开的数据库连接
/// - `owner`: 业务表字符串
/// - `id`: 记录主键
///
/// # 返回值
/// 未删除的计划、手记、成长或书摘处于锁定时为 `true`
///
/// # 错误
/// 查询失败时返回 `rusqlite::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let locked = sealed(&conn, "journal_entry", id)?;
/// ```
pub(crate) fn sealed(conn: &Connection, owner: &str, id: &str) -> Result<bool, rusqlite::Error> {
  let Some(owner) = Owner::parse(owner) else {
    return Ok(false);
  };
  let sql = match owner {
    Owner::Plan => "SELECT locked FROM plan WHERE id = ?1 AND deleted_at IS NULL",
    Owner::JournalEntry => "SELECT locked FROM journal_entry WHERE id = ?1 AND deleted_at IS NULL",
    Owner::GrowthEntry => "SELECT locked FROM growth_entry WHERE id = ?1 AND deleted_at IS NULL",
    Owner::Book | Owner::Member | Owner::Loose => return Ok(false),
    Owner::BookNote => "SELECT locked FROM book_note WHERE id = ?1 AND deleted_at IS NULL",
  };
  let locked: Option<i64> = conn.query_row(sql, [id], |row| row.get(0)).optional()?;
  Ok(locked.unwrap_or(0) != 0)
}
