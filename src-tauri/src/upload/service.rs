//! 上传文件的记录。所属记录必须已经存在。

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use super::dto::{Upload, UploadImage, UploadInput};
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

/// 按标识读出属于这条记录的未加密图片
///
/// 所属或记录对不上时按不存在处理。类型不是图片时拒绝。已加密时不读字节。
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 文件标识
/// - `owner`: 所属类型
/// - `owner_id`: 所属记录标识
///
/// # 返回值
/// MIME 和 base64 字节
///
/// # 错误
/// 记录不存在、所属不匹配或文件缺失时返回不存在。已加密时返回文件已经加密。类型不是图片时返回类型不正确
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let image = read_image(db, id, owner, owner_id)?;
/// ```
pub fn read_image(db: &DbState, id: &str, owner: &str, owner_id: &str) -> Result<UploadImage, UploadError> {
  let row = db.with_conn(|conn| repository::get(conn, id.trim()))?;
  if row.owner != owner.trim() || row.owner_id != owner_id.trim() {
    return Err(UploadError::NotFound);
  }
  if row.kind != super::constants::KIND_IMAGE {
    return Err(UploadError::KindInvalid);
  }
  if row.encrypted {
    return Err(UploadError::Encrypted);
  }
  let bytes = storage::read_image_file(db, &row.rel_path)?;
  Ok(UploadImage { mime: row.mime, data: BASE64.encode(bytes) })
}

/// 把粘贴的图片字节暂存到导入临时目录
///
/// # 参数
/// - `db`: 数据库状态
/// - `ext`: 不带点的扩展名
/// - `data`: 标准 base64
///
/// # 返回值
/// 临时文件绝对路径
///
/// # 错误
/// base64 无效或写盘失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let path = stage(db, "png", data)?;
/// ```
pub fn stage(db: &DbState, ext: &str, data: &str) -> Result<String, UploadError> {
  let bytes = BASE64.decode(data.trim()).map_err(|_| UploadError::KindInvalid)?;
  let path = storage::stage_bytes(db, ext, &bytes)?;
  Ok(path.to_string_lossy().into_owned())
}

/// 分配一个转码输出用的临时路径
///
/// # 参数
/// - `db`: 数据库状态
/// - `ext`: 不带点的扩展名
///
/// # 返回值
/// 临时文件绝对路径
///
/// # 错误
/// 扩展名不支持或目录创建失败时返回上传文件错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let path = tmp(db, "avif")?;
/// ```
pub fn tmp(db: &DbState, ext: &str) -> Result<String, UploadError> {
  let path = storage::alloc_tmp(db, ext)?;
  Ok(path.to_string_lossy().into_owned())
}

/// 删掉导入临时目录里的文件
///
/// # 参数
/// - `db`: 数据库状态
/// - `path`: 临时文件绝对路径
///
/// # 返回值
/// 删除完成时返回空成功
///
/// # 错误
/// 路径不在临时目录里时返回不存在
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// discard(db, path)?;
/// ```
pub fn discard(db: &DbState, path: &str) -> Result<(), UploadError> {
  storage::discard_tmp(db, path)
}

#[cfg(test)]
mod tests {
  use rusqlite::params;

  use base64::Engine;

  use super::{BASE64, *};
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

  #[test]
  fn read_image_checks_owner_and_encryption() {
    let (_dir, db) = setup();
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
        rel_path: "upload/image/photo.jpg".into(),
        mime: "image/jpeg".into(),
        name: "photo.jpg".into(),
        size: 3,
        sort: 0,
        locked: false,
      },
    )
    .expect("upload");
    let stored = storage::resolve(&db, &upload.rel_path);
    std::fs::create_dir_all(stored.parent().expect("parent")).expect("dir");
    std::fs::write(&stored, b"jpg").expect("file");

    let image = read_image(&db, &upload.id, Owner::Plan.as_str(), &plan_id).expect("read");
    assert_eq!(image.mime, "image/jpeg");
    assert_eq!(BASE64.decode(image.data).expect("decode"), b"jpg");
    assert!(matches!(
      read_image(&db, &upload.id, Owner::Plan.as_str(), "other"),
      Err(UploadError::NotFound)
    ));

    db.with_conn(|conn| crate::upload::set_encrypted(conn, &upload.id, true)).expect("encrypt");
    assert!(matches!(
      read_image(&db, &upload.id, Owner::Plan.as_str(), &plan_id),
      Err(UploadError::Encrypted)
    ));
  }

  #[test]
  fn stage_and_discard_stay_in_tmp() {
    let (_dir, db) = setup();
    let path = stage(&db, "png", &BASE64.encode(b"png")).expect("stage");
    assert!(std::path::Path::new(&path).exists());
    assert!(matches!(discard(&db, "/tmp/not-ours.png"), Err(UploadError::NotFound)));
    discard(&db, &path).expect("discard");
    assert!(!std::path::Path::new(&path).exists());
    let pending = tmp(&db, "avif").expect("tmp");
    assert!(!std::path::Path::new(&pending).exists());
    assert!(pending.ends_with(".avif"));
  }
}
