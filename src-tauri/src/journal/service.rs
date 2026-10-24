//! 手记。校验放在进库之前。

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

use super::dto::{JournalCitation, JournalEntry, JournalEntryInput, JournalLink};
use super::error::JournalError;
use super::repository;
use crate::db::state::DbState;

/// 类型合法时按类型和时间列出手记
///
/// # 参数
/// - `db`: 数据库状态
/// - `kind`: 手记类型，空则不限
/// - `from`: 起始发生时间，空则不限
/// - `to`: 结束发生时间，空则不限
///
/// # 返回值
/// 返回手记列表
///
/// # 错误
/// 类型非法时返回 `KindInvalid`，连接或查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entries = list(db, kind, from, to)?;
/// ```
pub fn list(
  db: &DbState,
  kind: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<JournalEntry>, JournalError> {
  if let Some(kind) = kind
    && !repository::allowed_kind(kind)
  {
    return Err(JournalError::KindInvalid);
  }
  db.with_conn(move |conn| repository::list(conn, kind, from, to))
}

/// 按主键读取手记
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 手记主键
///
/// # 返回值
/// 返回手记
///
/// # 错误
/// 连接或查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entry = get(db, id)?;
/// ```
pub fn get(db: &DbState, id: &str) -> Result<JournalEntry, JournalError> {
  db.with_conn(|conn| repository::get(conn, id))
}

/// 整理输入后创建手记
///
/// # 参数
/// - `db`: 数据库状态
/// - `input`: 手记内容
///
/// # 返回值
/// 返回新建的手记
///
/// # 错误
/// 输入不合法或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entry = create(db, input)?;
/// ```
pub fn create(db: &DbState, input: JournalEntryInput) -> Result<JournalEntry, JournalError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::create(conn, &input))
}

/// 整理输入后更新手记
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 手记主键
/// - `input`: 手记内容
///
/// # 返回值
/// 返回更新后的手记
///
/// # 错误
/// 输入不合法或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let entry = update(db, id, input)?;
/// ```
pub fn update(db: &DbState, id: &str, input: JournalEntryInput) -> Result<JournalEntry, JournalError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::update(conn, id, &input))
}

/// 删除手记，再在后台清理它留下的上传文件
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 手记主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 连接或删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete(db, id)?;
/// ```
pub fn delete(db: &DbState, id: &str) -> Result<(), JournalError> {
  db.with_conn(|conn| repository::delete(conn, id))?;
  crate::upload::sweep_later(db);
  Ok(())
}

/// 列出与手记相关的流转
///
/// # 参数
/// - `db`: 数据库状态
/// - `entry_id`: 手记主键
///
/// # 返回值
/// 返回流转列表
///
/// # 错误
/// 连接或查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let links = link_list(db, entry_id)?;
/// ```
pub fn link_list(db: &DbState, entry_id: &str) -> Result<Vec<JournalLink>, JournalError> {
  db.with_conn(|conn| repository::link_list(conn, entry_id))
}

/// 创建手记流转
///
/// # 参数
/// - `db`: 数据库状态
/// - `from_id`: 起点手记主键
/// - `to_id`: 终点手记主键
/// - `kind`: 流转类型
///
/// # 返回值
/// 返回新建的流转
///
/// # 错误
/// 连接或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let link = link_create(db, from_id, to_id, kind)?;
/// ```
pub fn link_create(db: &DbState, from_id: &str, to_id: &str, kind: &str) -> Result<JournalLink, JournalError> {
  let kind = kind.to_string();
  db.with_conn(move |conn| repository::link_create(conn, from_id, to_id, &kind))
}

/// 删除流转
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 流转主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 连接或删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// link_delete(db, id)?;
/// ```
pub fn link_delete(db: &DbState, id: &str) -> Result<(), JournalError> {
  db.with_conn(|conn| repository::link_delete(conn, id))
}

/// 列出手记的书摘引用
///
/// # 参数
/// - `db`: 数据库状态
/// - `entry_id`: 手记主键
///
/// # 返回值
/// 返回书摘引用列表
///
/// # 错误
/// 连接或查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let citations = citation_list(db, entry_id)?;
/// ```
pub fn citation_list(db: &DbState, entry_id: &str) -> Result<Vec<JournalCitation>, JournalError> {
  db.with_conn(|conn| repository::citation_list(conn, entry_id))
}

/// 创建书摘引用
///
/// # 参数
/// - `db`: 数据库状态
/// - `entry_id`: 手记主键
/// - `book_note_id`: 书摘主键
///
/// # 返回值
/// 返回新建的引用
///
/// # 错误
/// 连接或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let citation = citation_create(db, entry_id, book_note_id)?;
/// ```
pub fn citation_create(db: &DbState, entry_id: &str, book_note_id: &str) -> Result<JournalCitation, JournalError> {
  db.with_conn(|conn| repository::citation_create(conn, entry_id, book_note_id))
}

/// 删除书摘引用
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 引用主键
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 连接或删除失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// citation_delete(db, id)?;
/// ```
pub fn citation_delete(db: &DbState, id: &str) -> Result<(), JournalError> {
  db.with_conn(|conn| repository::citation_delete(conn, id))
}

/// 用密码加密正文并编码成 Base64
///
/// # 参数
/// - `body`: 正文
/// - `password`: 密码
///
/// # 返回值
/// 返回编码后的密文
///
/// # 错误
/// 密码为空时返回 `PasswordEmpty`，加密失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let sealed = seal_body(body, password)?;
/// ```
pub fn seal_body(body: &str, password: &str) -> Result<String, JournalError> {
  if password.is_empty() {
    return Err(JournalError::PasswordEmpty);
  }
  let bytes = crate::chacha::chacha::encrypt_bytes(body.as_bytes(), password).map_err(cipher_err)?;
  Ok(BASE64.encode(bytes))
}

/// 解码并解密正文
///
/// # 参数
/// - `body`: Base64 密文
/// - `password`: 密码
///
/// # 返回值
/// 返回正文文本
///
/// # 错误
/// 密码为空时返回 `PasswordEmpty`，解码、解密或文本转换失败时返回 `BodyUnreadable` 或其他对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let plain = open_body(body, password)?;
/// ```
pub fn open_body(body: &str, password: &str) -> Result<String, JournalError> {
  if password.is_empty() {
    return Err(JournalError::PasswordEmpty);
  }
  let raw = BASE64.decode(body.trim()).map_err(|_| JournalError::BodyUnreadable)?;
  let plain = crate::chacha::chacha::decrypt_bytes(&raw, password).map_err(cipher_err)?;
  String::from_utf8(plain).map_err(|_| JournalError::BodyUnreadable)
}

/// 把加解密错误映射成手记错误
///
/// # 参数
/// - `err`: 加解密错误
///
/// # 返回值
/// 空密码返回 `PasswordEmpty`；魔数、版本、截断或取消返回 `BodyUnreadable`；同路径或内部错误返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let err = cipher_err(source);
/// ```
fn cipher_err(err: crate::chacha::chacha::FileError) -> JournalError {
  match err {
    crate::chacha::chacha::FileError::PasswordEmpty => JournalError::PasswordEmpty,
    crate::chacha::chacha::FileError::BadMagic
    | crate::chacha::chacha::FileError::BadVersion
    | crate::chacha::chacha::FileError::Truncated
    | crate::chacha::chacha::FileError::Canceled => JournalError::BodyUnreadable,
    crate::chacha::chacha::FileError::SamePath | crate::chacha::chacha::FileError::Internal => JournalError::Internal,
  }
}

/// 去掉类型、标题和正文空白，并校验标题与类型
///
/// # 参数
/// - `input`: 手记内容
///
/// # 返回值
/// 返回整理后的输入
///
/// # 错误
/// 标题为空时返回 `TitleEmpty`，类型非法时返回 `KindInvalid`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let input = prepare(input)?;
/// ```
fn prepare(mut input: JournalEntryInput) -> Result<JournalEntryInput, JournalError> {
  input.kind = input.kind.trim().to_string();
  input.title = input.title.trim().to_string();
  input.body = input.body.trim().to_string();
  if input.title.is_empty() {
    return Err(JournalError::TitleEmpty);
  }
  if !repository::allowed_kind(&input.kind) {
    return Err(JournalError::KindInvalid);
  }
  Ok(input)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::journal::constants::KIND_DIARY;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![
      crate::member::migrate,
      crate::tag::migrate,
      crate::place::migrate,
      crate::upload::migrate,
      crate::journal::migrate,
    ]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn diary_and_lock() {
    let (_dir, db) = setup();
    let mut empty = sample();
    empty.title = " ".into();
    assert!(matches!(create(&db, empty), Err(JournalError::TitleEmpty)));

    let entry = create(&db, sample()).expect("create");
    let day = list(&db, Some(KIND_DIARY), Some(20), Some(21)).expect("list");
    assert_eq!(day.len(), 1);
    assert_eq!(day[0].id, entry.id);

    db.lock().expect("lock");
    assert!(matches!(list(&db, None, None, None), Err(JournalError::Locked)));
  }

  #[test]
  fn seal_and_open_body() {
    assert!(matches!(seal_body("今天下雨", ""), Err(JournalError::PasswordEmpty)));
    let sealed = seal_body("今天下雨", "diary-password").expect("seal");
    assert_eq!(open_body(&sealed, "diary-password").expect("open"), "今天下雨");
    assert!(matches!(
      open_body("not-base64", "diary-password"),
      Err(JournalError::BodyUnreadable)
    ));
  }

  fn sample() -> JournalEntryInput {
    JournalEntryInput {
      kind: KIND_DIARY.into(),
      occurred_at: 20,
      title: "公园".into(),
      body: String::new(),
      locked: false,
      highlight: false,
      member_ids: Vec::new(),
      tag_ids: Vec::new(),
      place_ids: Vec::new(),
    }
  }
}
