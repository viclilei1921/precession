//! 一条记录属于哪个业务表。只有这里和 `lib.rs` 知道全部业务模块。

use rusqlite::Connection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
  Plan,
  JournalEntry,
  GrowthEntry,
  Book,
  BookNote,
}

impl Owner {
  pub(crate) fn as_str(self) -> &'static str {
    match self {
      Self::Plan => "plan",
      Self::JournalEntry => "journal_entry",
      Self::GrowthEntry => "growth_entry",
      Self::Book => "book",
      Self::BookNote => "book_note",
    }
  }

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

pub(crate) fn exists(conn: &Connection, owner: Owner, id: &str) -> Result<bool, rusqlite::Error> {
  match owner {
    Owner::Plan => crate::plan::exists(conn, id),
    Owner::JournalEntry => crate::journal::exists(conn, id),
    Owner::GrowthEntry => crate::growth::exists(conn, id),
    Owner::Book => crate::library::book_exists(conn, id),
    Owner::BookNote => crate::library::note_exists(conn, id),
  }
}
