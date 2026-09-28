//! 成员档案，以及记录上的成员引用。只收已解锁的 `&Connection`。

use rusqlite::{Connection, OptionalExtension, params};

use super::constants::{MEMBER_REF_TABLE, MEMBER_TABLE, SCHEMA_VERSION, SCHEMA_VERSION_KEY};
use super::dto::{Member, MemberInput};
use super::error::MemberError;
use crate::utils::id::new_uuid_v4;
use crate::utils::time::now_unix_ms;

fn db_fail(context: &str, err: rusqlite::Error) -> MemberError {
  tauri_plugin_log::log::error!("{context}: {err}");
  MemberError::Internal
}

fn map_member(row: &rusqlite::Row<'_>) -> rusqlite::Result<Member> {
  Ok(Member {
    id: row.get(0)?,
    name: row.get(1)?,
    relation: row.get(2)?,
    gender: row.get(3)?,
    birthday: row.get(4)?,
    created_at: row.get(5)?,
    updated_at: row.get(6)?,
  })
}

pub(super) fn ensure_schema(conn: &Connection) -> Result<(), MemberError> {
  let current = crate::db::schema::read_version(conn, SCHEMA_VERSION_KEY).map_err(|e| db_fail("member schema", e))?;
  if current < 1 {
    conn
      .execute_batch(
        "CREATE TABLE IF NOT EXISTS member (
           id TEXT PRIMARY KEY NOT NULL,
           name TEXT NOT NULL,
           relation TEXT NOT NULL DEFAULT '',
           gender TEXT NOT NULL DEFAULT '',
           birthday INTEGER,
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL,
           deleted_at INTEGER
         );
         CREATE INDEX IF NOT EXISTS member_updated_at ON member(updated_at);
         CREATE TABLE IF NOT EXISTS member_ref (
           member_id TEXT NOT NULL,
           owner TEXT NOT NULL,
           owner_id TEXT NOT NULL,
           PRIMARY KEY (member_id, owner, owner_id)
         );
         CREATE INDEX IF NOT EXISTS member_ref_owner ON member_ref(owner, owner_id);",
      )
      .map_err(|e| db_fail("member migrate v1", e))?;
  }
  if current < SCHEMA_VERSION {
    crate::db::schema::write_version(conn, SCHEMA_VERSION_KEY, SCHEMA_VERSION)
      .map_err(|e| db_fail("member schema", e))?;
  }
  Ok(())
}

pub(super) fn exists(conn: &Connection, id: &str) -> Result<bool, rusqlite::Error> {
  let sql = format!("SELECT 1 FROM {MEMBER_TABLE} WHERE id = ?1 AND deleted_at IS NULL");
  let found: Option<i64> = conn.query_row(&sql, [id], |row| row.get(0)).optional()?;
  Ok(found.is_some())
}

pub(super) fn list(conn: &Connection) -> Result<Vec<Member>, MemberError> {
  let sql = format!(
    "SELECT id, name, relation, gender, birthday, created_at, updated_at
     FROM {MEMBER_TABLE} WHERE deleted_at IS NULL ORDER BY updated_at DESC"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("member list prepare", e))?;
  let rows = stmt.query_map([], map_member).map_err(|e| db_fail("member list query", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("member list collect", e))
}

pub(super) fn get(conn: &Connection, id: &str) -> Result<Member, MemberError> {
  let sql = format!(
    "SELECT id, name, relation, gender, birthday, created_at, updated_at
     FROM {MEMBER_TABLE} WHERE id = ?1 AND deleted_at IS NULL"
  );
  conn.query_row(&sql, [id], map_member).map_err(|e| match e {
    rusqlite::Error::QueryReturnedNoRows => MemberError::NotFound,
    other => db_fail("member get", other),
  })
}

pub(super) fn create(conn: &Connection, input: &MemberInput) -> Result<Member, MemberError> {
  let id = new_uuid_v4();
  let now = now_unix_ms();
  let sql = format!(
    "INSERT INTO {MEMBER_TABLE} (id, name, relation, gender, birthday, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)"
  );
  conn
    .execute(&sql, params![id, input.name, input.relation, input.gender, input.birthday, now])
    .map_err(|e| db_fail("member create", e))?;
  get(conn, &id)
}

pub(super) fn update(conn: &Connection, id: &str, input: &MemberInput) -> Result<Member, MemberError> {
  let now = now_unix_ms();
  let sql = format!(
    "UPDATE {MEMBER_TABLE}
     SET name = ?1, relation = ?2, gender = ?3, birthday = ?4, updated_at = ?5
     WHERE id = ?6 AND deleted_at IS NULL"
  );
  let n = conn
    .execute(&sql, params![input.name, input.relation, input.gender, input.birthday, now, id])
    .map_err(|e| db_fail("member update", e))?;
  if n == 0 {
    return Err(MemberError::NotFound);
  }
  get(conn, id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> Result<(), MemberError> {
  let now = now_unix_ms();
  let sql = format!("UPDATE {MEMBER_TABLE} SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL");
  let n = conn.execute(&sql, params![now, id]).map_err(|e| db_fail("member delete", e))?;
  if n == 0 {
    return Err(MemberError::NotFound);
  }
  Ok(())
}

pub(super) fn replace(conn: &Connection, owner: &str, owner_id: &str, ids: &[String]) -> Result<(), MemberError> {
  for id in ids {
    let alive = exists(conn, id).map_err(|e| db_fail("member ref", e))?;
    if !alive {
      return Err(MemberError::ReferencedMissing);
    }
  }
  let sql = format!("DELETE FROM {MEMBER_REF_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("member ref clear", e))?;
  let sql = format!("INSERT INTO {MEMBER_REF_TABLE} (member_id, owner, owner_id) VALUES (?1, ?2, ?3)");
  for id in ids {
    conn.execute(&sql, params![id, owner, owner_id]).map_err(|e| db_fail("member ref insert", e))?;
  }
  Ok(())
}

pub(super) fn list_ids(conn: &Connection, owner: &str, owner_id: &str) -> Result<Vec<String>, MemberError> {
  let sql = format!(
    "SELECT r.member_id FROM {MEMBER_REF_TABLE} r
     JOIN {MEMBER_TABLE} m ON m.id = r.member_id AND m.deleted_at IS NULL
     WHERE r.owner = ?1 AND r.owner_id = ?2
     ORDER BY r.member_id"
  );
  let mut stmt = conn.prepare(&sql).map_err(|e| db_fail("member ref list", e))?;
  let rows = stmt.query_map(params![owner, owner_id], |row| row.get(0)).map_err(|e| db_fail("member ref list", e))?;
  rows.collect::<Result<Vec<_>, _>>().map_err(|e| db_fail("member ref list", e))
}

pub(super) fn clear(conn: &Connection, owner: &str, owner_id: &str) -> Result<(), MemberError> {
  let sql = format!("DELETE FROM {MEMBER_REF_TABLE} WHERE owner = ?1 AND owner_id = ?2");
  conn.execute(&sql, params![owner, owner_id]).map_err(|e| db_fail("member ref clear", e))?;
  Ok(())
}
