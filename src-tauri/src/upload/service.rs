//! 上传文件的记录。所属记录必须已经存在。

use super::dto::{Upload, UploadInput};
use super::error::UploadError;
use super::{repository, storage};
use crate::db::state::DbState;

/// 列出所属记录上的文件
///
/// # 参数
/// - `db`: 数据库状态
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// 返回文件记录列表
///
/// # 错误
/// 查询失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list(db, owner, owner_id)?;
/// ```
pub fn list(db: &DbState, owner: &str, owner_id: &str) -> Result<Vec<Upload>, UploadError> {
  db.with_conn(|conn| repository::list(conn, owner, owner_id))
}

/// 校验输入后创建文件记录
///
/// # 参数
/// - `db`: 数据库状态
/// - `input`: 文件输入
///
/// # 返回值
/// 返回新建的文件记录
///
/// # 错误
/// 输入不合法或写入失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = create(db, input)?;
/// ```
pub fn create(db: &DbState, input: UploadInput) -> Result<Upload, UploadError> {
  let (owner, input) = repository::prepare(input)?;
  db.with_conn(|conn| repository::create(conn, owner, &input))
}

/// 按标识删除文件记录，再删掉上传目录里的文件
///
/// 文件删除失败时忽略，留给孤儿清理。
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 文件标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除记录失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete(db, id)?;
/// ```
pub fn delete(db: &DbState, id: &str) -> Result<(), UploadError> {
  let rel = db.with_conn(|conn| repository::delete(conn, id))?;
  storage::remove(db, &rel);
  Ok(())
}

#[cfg(test)]
mod tests {
  use rusqlite::params;

  use super::*;
  use crate::db::error::DbError;
  use crate::owner::Owner;
  use crate::upload::constants::KIND_IMAGE;
  use crate::utils::id::new_uuid_v4;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![
      crate::member::migrate,
      crate::tag::migrate,
      crate::place::migrate,
      crate::upload::migrate,
      crate::plan::migrate,
    ]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn attach_delete_and_lock() {
    let (_dir, db) = setup();
    assert!(matches!(
      create(
        &db,
        UploadInput {
          owner: Owner::Plan.as_str().into(),
          owner_id: "missing".into(),
          kind: KIND_IMAGE.into(),
          rel_path: "a.jpg".into(),
          mime: "image/jpeg".into(),
          name: "a.jpg".into(),
          size: 1,
          sort: 0,
          locked: false,
        },
      ),
      Err(UploadError::ReferencedMissing)
    ));

    let plan_id = new_uuid_v4();
    db.with_conn(|conn| {
      conn
        .execute(
          "INSERT INTO plan (id, title, body, status, priority, result, locked, highlight, created_at, updated_at)
           VALUES (?1, '出门', '', 'inbox', 0, '', 0, 0, 1, 1)",
          params![plan_id],
        )
        .expect("plan");
      Ok::<(), DbError>(())
    })
    .expect("seed");

    let upload = create(
      &db,
      UploadInput {
        owner: Owner::Plan.as_str().into(),
        owner_id: plan_id.clone(),
        kind: KIND_IMAGE.into(),
        rel_path: "upload/image/a.jpg".into(),
        mime: "image/jpeg".into(),
        name: " 照片.jpg ".into(),
        size: 3,
        sort: 0,
        locked: false,
      },
    )
    .expect("upload");
    assert!(!upload.encrypted);
    assert_eq!(upload.name, "照片.jpg");
    assert_eq!(upload.size, 3);
    assert_eq!(list(&db, Owner::Plan.as_str(), &plan_id).expect("list").len(), 1);

    let stored = storage::resolve(&db, &upload.rel_path);
    std::fs::create_dir_all(stored.parent().expect("parent")).expect("dir");
    std::fs::write(&stored, b"jpg").expect("file");
    delete(&db, &upload.id).expect("delete");
    assert!(!stored.exists());
    assert!(list(&db, Owner::Plan.as_str(), &plan_id).expect("empty").is_empty());

    db.lock().expect("lock");
    assert!(matches!(list(&db, Owner::Plan.as_str(), &plan_id), Err(UploadError::Locked)));
  }
}
