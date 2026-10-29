//! 成长记录。里程碑和精彩瞬间都必须指定成员。

use super::dto::{GrowthEntry, GrowthEntryInput, GrowthMeasure, GrowthMeasureInput};
use super::error::GrowthError;
use super::repository;
use crate::db::state::DbState;

/// 按成员、类型和时间范围列出成长记录
///
/// # 参数
/// - `db`: 数据库状态
/// - `member_id`: 成员标识
/// - `kind`: 记录类型，未指定时跳过类型校验
/// - `from`: 起始发生时间
/// - `to`: 结束发生时间
///
/// # 返回值
/// 返回符合条件的成长记录
///
/// # 错误
/// 指定的类型不受支持时返回类型不正确；查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list(db, member_id, kind, from, to)?;
/// ```
pub fn list(
  db: &DbState,
  member_id: Option<&str>,
  kind: Option<&str>,
  from: Option<i64>,
  to: Option<i64>,
) -> Result<Vec<GrowthEntry>, GrowthError> {
  if let Some(kind) = kind
    && !repository::allowed_kind(kind)
  {
    return Err(GrowthError::KindInvalid);
  }
  db.with_conn(move |conn| repository::list(conn, member_id, kind, from, to))
}

/// 按标识读取成长记录
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 成长记录标识
///
/// # 返回值
/// 返回对应的成长记录
///
/// # 错误
/// 读取失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = get(db, id)?;
/// ```
pub fn get(db: &DbState, id: &str) -> Result<GrowthEntry, GrowthError> {
  db.with_conn(|conn| repository::get(conn, id))
}

/// 校验输入后创建成长记录
///
/// # 参数
/// - `db`: 数据库状态
/// - `input`: 成长记录输入
///
/// # 返回值
/// 返回新建的成长记录
///
/// # 错误
/// 输入不合法或写入失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = create(db, input)?;
/// ```
pub fn create(db: &DbState, input: GrowthEntryInput) -> Result<GrowthEntry, GrowthError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::create(conn, &input))
}

/// 校验输入后更新成长记录
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 成长记录标识
/// - `input`: 成长记录输入
///
/// # 返回值
/// 返回更新后的成长记录
///
/// # 错误
/// 输入不合法或更新失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = update(db, id, input)?;
/// ```
pub fn update(db: &DbState, id: &str, input: GrowthEntryInput) -> Result<GrowthEntry, GrowthError> {
  let input = prepare(input)?;
  db.with_conn(|conn| repository::update(conn, id, &input))
}

/// 按标识删除成长记录，再在后台清理它留下的上传文件
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 成长记录标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete(db, id)?;
/// ```
pub fn delete(db: &DbState, id: &str) -> Result<(), GrowthError> {
  db.with_conn(|conn| repository::delete(conn, id))?;
  crate::upload::sweep_later(db);
  Ok(())
}

/// 整理并校验成长记录输入
///
/// # 参数
/// - `input`: 待写入的成长记录
///
/// # 返回值
/// 返回去掉类型、成员标识、标题和正文首尾空白后的输入
///
/// # 错误
/// 标题为空时返回标题不能为空；类型不受支持时返回类型不正确；成员标识为空时返回需要指定成员
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let input = prepare(raw);
/// ```
/// 按成员列出身高体重，发生时间晚的在前
///
/// # 参数
/// - `db`: 数据库状态
/// - `member_id`: 成员标识
///
/// # 返回值
/// 返回该成员的测量
///
/// # 错误
/// 成员标识为空时返回需要指定成员；查询失败时返回对应错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let rows = list_measures(db, member_id)?;
/// ```
pub fn list_measures(db: &DbState, member_id: &str) -> Result<Vec<GrowthMeasure>, GrowthError> {
  let member_id = member_id.trim().to_string();
  if member_id.is_empty() {
    return Err(GrowthError::MemberRequired);
  }
  db.with_conn(move |conn| repository::list_measures(conn, &member_id))
}

/// 校验输入后创建身高体重
///
/// # 参数
/// - `db`: 数据库状态
/// - `input`: 测量输入
///
/// # 返回值
/// 返回新建的测量
///
/// # 错误
/// 输入不合法或写入失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = create_measure(db, input)?;
/// ```
pub fn create_measure(db: &DbState, input: GrowthMeasureInput) -> Result<GrowthMeasure, GrowthError> {
  let input = prepare_measure(input)?;
  db.with_conn(|conn| repository::create_measure(conn, &input))
}

/// 校验输入后更新身高体重
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 测量标识
/// - `input`: 测量输入
///
/// # 返回值
/// 返回更新后的测量
///
/// # 错误
/// 输入不合法或更新失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let row = update_measure(db, id, input)?;
/// ```
pub fn update_measure(db: &DbState, id: &str, input: GrowthMeasureInput) -> Result<GrowthMeasure, GrowthError> {
  let input = prepare_measure(input)?;
  db.with_conn(|conn| repository::update_measure(conn, id, &input))
}

/// 按标识删除身高体重
///
/// # 参数
/// - `db`: 数据库状态
/// - `id`: 测量标识
///
/// # 返回值
/// 删除成功时返回空成功
///
/// # 错误
/// 删除失败时返回成长记录错误
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// delete_measure(db, id)?;
/// ```
pub fn delete_measure(db: &DbState, id: &str) -> Result<(), GrowthError> {
  db.with_conn(|conn| repository::delete_measure(conn, id))
}

/// 整理并校验身高体重输入
///
/// # 参数
/// - `input`: 待写入的测量
///
/// # 返回值
/// 返回去掉成员标识首尾空白后的输入
///
/// # 错误
/// 成员标识为空时返回需要指定成员；身高或体重不大于 0 时返回测量不正确
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let input = prepare_measure(raw);
/// ```
fn prepare_measure(mut input: GrowthMeasureInput) -> Result<GrowthMeasureInput, GrowthError> {
  input.member_id = input.member_id.trim().to_string();
  if input.member_id.is_empty() {
    return Err(GrowthError::MemberRequired);
  }
  if input.height_mm <= 0 || input.weight_g <= 0 {
    return Err(GrowthError::MeasureInvalid);
  }
  Ok(input)
}

fn prepare(mut input: GrowthEntryInput) -> Result<GrowthEntryInput, GrowthError> {
  input.kind = input.kind.trim().to_string();
  input.member_id = input.member_id.trim().to_string();
  input.title = input.title.trim().to_string();
  input.body = input.body.trim().to_string();
  if input.title.is_empty() {
    return Err(GrowthError::TitleEmpty);
  }
  if !repository::allowed_kind(&input.kind) {
    return Err(GrowthError::KindInvalid);
  }
  if input.member_id.is_empty() {
    return Err(GrowthError::MemberRequired);
  }
  Ok(input)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::growth::constants::KIND_MILESTONE;

  fn setup() -> (tempfile::TempDir, DbState) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = DbState::new(dir.path().to_path_buf()).with_migrators(vec![
      crate::member::migrate,
      crate::tag::migrate,
      crate::place::migrate,
      crate::upload::migrate,
      crate::growth::migrate,
    ]);
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db)
  }

  #[test]
  fn milestone_requires_member_and_lock() {
    let (_dir, db) = setup();
    let mut input = sample("missing");
    input.member_id.clear();
    assert!(matches!(create(&db, input), Err(GrowthError::MemberRequired)));

    let member_id = crate::utils::id::new_uuid_v4();
    db.with_conn(|conn| {
      conn
        .execute(
          "INSERT INTO member (id, name, relation, gender, created_at, updated_at) VALUES (?1, '宝宝', 'child', '', ?2, ?2)",
          rusqlite::params![member_id, 1_i64],
        )
        .map_err(|_| GrowthError::Internal)
    })
    .expect("member");

    let created = create(&db, sample(&member_id)).expect("milestone");
    assert!(created.highlight);
    assert_eq!(created.member_id, member_id);

    db.lock().expect("lock");
    assert!(matches!(list(&db, None, None, None, None), Err(GrowthError::Locked)));
  }

  #[test]
  fn measure_requires_positive_size() {
    let (_dir, db) = setup();
    let member_id = crate::utils::id::new_uuid_v4();
    db.with_conn(|conn| {
      conn
        .execute(
          "INSERT INTO member (id, name, relation, gender, created_at, updated_at) VALUES (?1, '宝宝', 'child', '', ?2, ?2)",
          rusqlite::params![member_id, 1_i64],
        )
        .map_err(|_| GrowthError::Internal)
    })
    .expect("member");

    let mut input =
      GrowthMeasureInput { member_id: member_id.clone(), occurred_at: 20, height_mm: 0, weight_g: 13_200 };
    assert!(matches!(create_measure(&db, input), Err(GrowthError::MeasureInvalid)));

    input = GrowthMeasureInput { member_id: member_id.clone(), occurred_at: 20, height_mm: 920, weight_g: 13_200 };
    let created = create_measure(&db, input).expect("measure");
    assert_eq!(created.height_mm, 920);
    let rows = list_measures(&db, &member_id).expect("list");
    assert_eq!(rows.len(), 1);
    delete_measure(&db, &created.id).expect("delete");
    assert!(list_measures(&db, &member_id).expect("empty").is_empty());
  }

  fn sample(member_id: &str) -> GrowthEntryInput {
    GrowthEntryInput {
      kind: KIND_MILESTONE.into(),
      member_id: member_id.into(),
      occurred_at: 10,
      title: "学会站".into(),
      body: String::new(),
      locked: false,
      highlight: true,
      member_ids: Vec::new(),
      tag_ids: Vec::new(),
      place_ids: Vec::new(),
    }
  }
}
