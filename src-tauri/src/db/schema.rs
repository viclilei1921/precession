//! `db_meta` 里的 schema 版本。表归 `db` 所有，业务模块只读写自己的版本键。

use rusqlite::{Connection, OptionalExtension, params};

/// 读取 `db_meta` 里的 schema 版本
///
/// 没有这一行，或值不能解析成整数时返回 0。
///
/// # 参数
/// - `conn`: 已打开的数据库连接
/// - `key`: 版本键
///
/// # 返回值
/// 解析出的版本号；缺失或无法解析时为 0
///
/// # 错误
/// 查询失败时返回 `rusqlite::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let version = read_version(&conn, "member")?;
/// ```
pub(crate) fn read_version(conn: &Connection, key: &str) -> Result<i64, rusqlite::Error> {
  let current: Option<String> =
    conn.query_row("SELECT value FROM db_meta WHERE key = ?1", [key], |row| row.get(0)).optional()?;
  Ok(current.and_then(|value| value.parse().ok()).unwrap_or(0))
}

/// 写入或覆盖 `db_meta` 里的 schema 版本
///
/// # 参数
/// - `conn`: 已打开的数据库连接
/// - `key`: 版本键
/// - `version`: 要保存的版本号，写入前转成字符串
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 执行插入或更新失败时返回 `rusqlite::Error`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// write_version(&conn, "member", 1)?;
/// ```
pub(crate) fn write_version(conn: &Connection, key: &str, version: i64) -> Result<(), rusqlite::Error> {
  let version = version.to_string();
  conn.execute(
    "INSERT INTO db_meta (key, value) VALUES (?1, ?2)
     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    params![key, version],
  )?;
  Ok(())
}
