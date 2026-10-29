//! 成员。不持有连接，一律走 `DbState::with_conn`。

use super::dto::{Member, MemberInput};
use super::error::MemberError;
use super::repository;
use crate::db::state::DbState;
use crate::owner::Owner;
use crate::upload::KIND_IMAGE;
use crate::upload::error::UploadError;

/// 列出成员
///
/// # 参数
/// - `db`: 数据库状态
///
/// # 返回值
/// 返回成员列表
///
/// # 错误
/// 连接或查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let members = list(db)?;
/// ```
pub fn list(db: &DbState) -> Result<Vec<Member>, MemberError> {
  db.with_conn(repository::list)
}

/// 按主键读取成员
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 成员主键
///
/// # 返回值
/// 返回成员
///
/// # 错误
/// 连接或查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let member = get(db, id)?;
/// ```
pub fn get(db: &DbState, id: &str) -> Result<Member, MemberError> {
  db.with_conn(|conn| repository::get(conn, id))
}

/// 整理输入后创建成员
///
/// 头像若是未挂载图片，会改挂到这位家人。标识对不上或不是图片时，不留下成员。
///
/// # 参数
/// - `db`: 数据库状态
/// - `input`: 成员内容
///
/// # 返回值
/// 返回新建的成员
///
/// # 错误
/// 输入不合法、头像无法使用或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let member = create(db, input)?;
/// ```
pub fn create(db: &DbState, input: MemberInput) -> Result<Member, MemberError> {
  let input = prepare(input)?;
  check_avatar(db, "", &input.avatar)?;
  let member = db.with_conn(|conn| repository::create(conn, &input))?;
  if let Err(err) = retain_avatar(db, &member.id, &member.avatar) {
    let _ = delete(db, &member.id);
    return Err(err);
  }
  Ok(member)
}

/// 整理输入后更新成员
///
/// 换上头像时，未挂载的图片会改挂过来，原来的头像文件会删掉。清空头像也会删掉文件。
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 成员主键
/// - `input`: 成员内容
///
/// # 返回值
/// 返回更新后的成员
///
/// # 错误
/// 输入不合法、头像无法使用或写入失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let member = update(db, id, input)?;
/// ```
pub fn update(db: &DbState, id: &str, input: MemberInput) -> Result<Member, MemberError> {
  let input = prepare(input)?;
  check_avatar(db, id, &input.avatar)?;
  let member = db.with_conn(|conn| repository::update(conn, id, &input))?;
  retain_avatar(db, id, &member.avatar)?;
  Ok(member)
}

/// 删除成员，并清掉这位家人的头像文件
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 成员主键
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
pub fn delete(db: &DbState, id: &str) -> Result<(), MemberError> {
  db.with_conn(|conn| repository::delete(conn, id))?;
  crate::upload::sweep_later(db);
  Ok(())
}

/// 去掉名称、关系、性别和头像标识的空白，并拒绝空名称
///
/// # 参数
/// - `input`: 成员内容
///
/// # 返回值
/// 返回整理后的输入
///
/// # 错误
/// 名称去空白后为空时返回 `MemberError::NameEmpty`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let input = prepare(input)?;
/// ```
fn prepare(mut input: MemberInput) -> Result<MemberInput, MemberError> {
  input.name = input.name.trim().to_string();
  if input.name.is_empty() {
    return Err(MemberError::NameEmpty);
  }
  input.relation = input.relation.trim().to_string();
  input.gender = input.gender.trim().to_string();
  input.avatar = input.avatar.trim().to_string();
  Ok(input)
}

/// 确认头像为空、已挂在这位家人上，或是一张未挂载的图片
///
/// # 参数
/// - `db`: 数据库状态
/// - `member_id`: 成员主键。新建时为空
/// - `avatar`: 头像文件标识
///
/// # 返回值
/// 可以使用时返回空
///
/// # 错误
/// 文件不存在、不是图片，或不属于这位家人且不是未挂载时返回 `ReferencedMissing`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// check_avatar(db, member_id, avatar)?;
/// ```
fn check_avatar(db: &DbState, member_id: &str, avatar: &str) -> Result<(), MemberError> {
  if avatar.is_empty() {
    return Ok(());
  }
  let row = load_upload(db, avatar)?;
  if avatar_allowed(&row, member_id) {
    return Ok(());
  }
  Err(MemberError::ReferencedMissing)
}

/// 把头像改挂到这位家人，并删掉不再使用的头像文件
///
/// # 参数
/// - `db`: 数据库状态
/// - `member_id`: 成员主键
/// - `avatar`: 要留下的头像文件标识，清空时为空
///
/// # 返回值
/// 成功时返回空
///
/// # 错误
/// 头像无法使用或文件操作失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// retain_avatar(db, member_id, avatar)?;
/// ```
fn retain_avatar(db: &DbState, member_id: &str, avatar: &str) -> Result<(), MemberError> {
  if !avatar.is_empty() {
    let row = load_upload(db, avatar)?;
    if !avatar_allowed(&row, member_id) {
      return Err(MemberError::ReferencedMissing);
    }
    if row.owner == Owner::Loose.as_str() {
      crate::upload::attach(db, &[avatar.to_string()], Owner::Member.as_str(), member_id).map_err(upload_err)?;
    }
  }
  let rows = crate::upload::list(db, Owner::Member.as_str(), member_id).map_err(upload_err)?;
  for row in rows {
    if row.id != avatar {
      crate::upload::delete(db, &row.id).map_err(upload_err)?;
    }
  }
  Ok(())
}

/// 判断这张图片能不能当作这位家人的头像
///
/// 未挂载的图片可以。已经挂在这位家人身上的也可以。
///
/// # 参数
/// - `row`: 文件记录
/// - `member_id`: 成员主键。新建时为空
///
/// # 返回值
/// 可以当作头像时为 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let ok = avatar_allowed(&row, member_id);
/// ```
fn avatar_allowed(row: &crate::upload::dto::Upload, member_id: &str) -> bool {
  if row.kind != KIND_IMAGE {
    return false;
  }
  if row.owner == Owner::Loose.as_str() {
    return true;
  }
  row.owner == Owner::Member.as_str() && row.owner_id == member_id
}

/// 按标识读取上传文件
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 文件标识
///
/// # 返回值
/// 返回文件记录
///
/// # 错误
/// 读取失败时返回对应的成员错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = load_upload(db, id)?;
/// ```
fn load_upload(db: &DbState, id: &str) -> Result<crate::upload::dto::Upload, MemberError> {
  db.with_conn(|conn| crate::upload::get(conn, id).map_err(upload_err))
}

/// 把上传文件错误转成成员错误
///
/// # 参数
/// - `err`: 上传文件错误
///
/// # 返回值
/// 未解锁时返回 `Locked`，找不到或类型不对时返回 `ReferencedMissing`，其余返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let err = upload_err(upload_error);
/// ```
fn upload_err(err: UploadError) -> MemberError {
  match err {
    UploadError::Locked => MemberError::Locked,
    UploadError::NotFound | UploadError::ReferencedMissing | UploadError::KindInvalid => MemberError::ReferencedMissing,
    _ => MemberError::Internal,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db =
      DbState::new(dir.path().to_path_buf()).with_migrators(vec![crate::member::migrate, crate::upload::migrate]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn crud_and_lock() {
    let (_dir, db) = setup();
    assert!(matches!(
      create(
        &db,
        MemberInput {
          name: "  ".into(),
          relation: String::new(),
          gender: String::new(),
          birthday: None,
          avatar: String::new(),
        }
      ),
      Err(MemberError::NameEmpty)
    ));

    let member = create(
      &db,
      MemberInput {
        name: " 宝宝 ".into(),
        relation: "child".into(),
        gender: "男".into(),
        birthday: Some(1),
        avatar: String::new(),
      },
    )
    .expect("create");
    assert_eq!(member.name, "宝宝");
    assert_eq!(list(&db).expect("list").len(), 1);

    delete(&db, &member.id).expect("delete");
    assert!(list(&db).expect("empty").is_empty());

    db.lock().expect("lock");
    assert!(matches!(list(&db), Err(MemberError::Locked)));
  }

  fn seed_image(db: &DbState, id: &str, owner: &str) {
    db.with_conn(|conn| {
      conn
        .execute(
          "INSERT INTO upload (id, owner, owner_id, kind, rel_path, mime, name, size, sort, locked, encrypted, created_at)
           VALUES (?1, ?2, 'loose', 'image', 'note', 'image/avif', 'a.avif', 1, 0, 0, 0, 1)",
          rusqlite::params![id, owner],
        )
        .map_err(|_| MemberError::Internal)
    })
    .expect("seed");
  }

  fn person(name: &str, avatar: &str) -> MemberInput {
    MemberInput {
      name: name.into(),
      relation: String::new(),
      gender: String::new(),
      birthday: None,
      avatar: avatar.into(),
    }
  }

  #[test]
  fn avatar_mounts_replaces_and_leaves_with_the_member() {
    let (_dir, db) = setup();
    assert!(matches!(
      create(&db, person("宝宝", "missing")),
      Err(MemberError::ReferencedMissing)
    ));
    assert!(list(&db).expect("none").is_empty());

    seed_image(&db, "pic", "loose");
    seed_image(&db, "next", "loose");
    seed_image(&db, "note", "journal_entry");
    assert!(matches!(
      create(&db, person("宝宝", "note")),
      Err(MemberError::ReferencedMissing)
    ));

    let member = create(&db, person("宝宝", "pic")).expect("create");
    assert_eq!(member.avatar, "pic");
    let rows = crate::upload::list(&db, "member", &member.id).expect("files");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "pic");

    let replaced = update(&db, &member.id, person("宝宝", "next")).expect("replace");
    assert_eq!(replaced.avatar, "next");
    let rows = crate::upload::list(&db, "member", &member.id).expect("kept");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "next");
    assert!(matches!(
      db.with_conn(|conn| crate::upload::get(conn, "pic")),
      Err(UploadError::NotFound)
    ));

    delete(&db, &member.id).expect("delete");
    assert!(crate::upload::list(&db, "member", &member.id).expect("gone").is_empty());
  }
}
