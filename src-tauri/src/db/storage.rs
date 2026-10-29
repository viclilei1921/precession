//! 这个文件不是「业务表怎么增删改」，而是 数据库的看门人：
//! 负责把加密库建起来、用 DEK 打开、读写密钥头。
//! 真正的业务逻辑之类以后会走 DbState::with_conn，不会直接堆在这里
//!
//! 可以把它想成仓库管理员的几项固定动作：建库、开库、贴门牌、验货、失败时扫地。
//!
//! 目录大致是：应用数据目录 / db/
//! data.sqlite - SQLCipher 加密数据库。没有正确 DEK，里面全是乱码
//! key.header.json - 密钥头：盐、Argon2 参数、被 KEK 包住的 DEK
//! key.header.json.tmp - 写头文件时的临时稿，写完立刻改名成正式文件
//!
//! 不管怎么派生密钥，只负责「文件和 SQLite 连接」这一层
//! 创建 create_db
//! 建目录 → 打开 sqlite → PRAGMA key(DEK)
//! → 建 db_meta + 插入探针 → 读探针
//! → 原子写 key.header.json
//! 失败则 remove_create_artifacts
//!
//! 解锁 open_db
//! 打开 sqlite → PRAGMA key(DEK) → 读探针
//! 辅助
//! read_header          读门牌
//! write_header_atomic  安全写门牌
//! apply_raw_key        把 DEK 喂给 SQLCipher
//! init_schema          第一张表
//! verify_probe         验货

use std::fs;

use rusqlite::Connection;
use zeroize::Zeroizing;

use super::constants::{DB_DEVICE_WRAP_TMP, DB_HEADER_TMP, PROBE_KEY, PROBE_VALUE};
use super::error::DbError;
use super::state::DbState;
use crate::crypto::header::KeyHeader;

/// 建目录、用 DEK 打开库、写入探针并原子保存密钥头
///
/// # 参数
/// - `state`: 数据库路径
/// - `dek`: 32 字节页加密密钥
/// - `header`: 要写入的密钥头
///
/// # 返回值
/// 已设置密钥并通过对探针校验的连接
///
/// # 错误
/// - 创建目录失败时返回 `Io`
/// - 打开 sqlite、设置密钥或建表失败时返回 `Internal`
/// - 探针读不到或值不对时返回 `Corrupt`
/// - 写密钥头失败时返回对应的 `DbError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let conn = create_db(state, dek, header)?;
/// ```
pub fn create_db(state: &DbState, dek: &[u8; 32], header: &KeyHeader) -> Result<Connection, DbError> {
  // 确保目录存在
  fs::create_dir_all(&state.dir).map_err(|_| DbError::Io)?;

  // 打开（或新建）data.sqlite
  let conn = Connection::open(&state.sqlite).map_err(|e| {
    tauri_plugin_log::log::error!("db create sqlite: {e}");
    DbError::Internal
  })?;

  // 把 32 字节 DEK 交给 SQLCipher，之后每一页都用它加密
  apply_raw_key(&conn, dek)?;

  // 建一张 db_meta 表，插入探针行 ok = precession.db.ready
  init_schema(&conn)?;

  // 立刻读回探针，确认「密钥生效 + 表结构正常」
  verify_probe(&conn)?;

  // 原子写入密钥头（先写 .tmp，再 rename 成 key.header.json）
  write_header_atomic(state, header)?;

  Ok(conn)
}

/// 删除建库留下的 sqlite 文件和密钥头
///
/// 删除失败时忽略错误。
///
/// # 参数
/// - `state`: 数据库路径
///
/// # 返回值
/// 不返回数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// remove_create_artifacts(state);
/// ```
pub fn remove_create_artifacts(state: &DbState) {
  let _ = fs::remove_file(&state.sqlite);
  let _ = fs::remove_file(&state.header);
}

/// 用 DEK 打开已有库并核对探针
///
/// # 参数
/// - `path`: sqlite 文件路径
/// - `dek`: 32 字节页加密密钥
///
/// # 返回值
/// 已设置密钥并通过对探针校验的连接
///
/// # 错误
/// - 打开 sqlite 或设置密钥失败时返回 `Internal`
/// - 探针读不到或值不对时返回 `Corrupt`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let conn = open_db(path, dek)?;
/// ```
pub fn open_db(path: &std::path::Path, dek: &[u8; 32]) -> Result<Connection, DbError> {
  // 打开 data.sqlite
  let conn = Connection::open(path).map_err(|e| {
    tauri_plugin_log::log::error!("db open sqlite: {e}");
    DbError::Internal
  })?;

  // 把 32 字节 DEK 交给 SQLCipher，之后每一页都用它加密
  apply_raw_key(&conn, dek)?;

  // 立刻读回探针，确认「密钥生效 + 表结构正常」
  verify_probe(&conn)?;

  Ok(conn)
}

/// 从文件读取并解析密钥头
///
/// # 参数
/// - `path`: 密钥头路径
///
/// # 返回值
/// 通过校验的 `KeyHeader`
///
/// # 错误
/// 读文件失败时返回 `Io`；JSON 或字段校验失败时把 `CryptoError` 转成 `DbError` 返回
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let header = read_header(path)?;
/// ```
pub fn read_header(path: &std::path::Path) -> Result<KeyHeader, DbError> {
  let raw = fs::read_to_string(path).map_err(|_| DbError::Io)?;
  Ok(KeyHeader::from_json(&raw)?)
}

/// 先写临时文件再改名，替换设备槽密文
///
/// # 参数
/// - `state`: 数据库路径
/// - `bytes`: `device.wrap` 的全部字节
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 写临时文件或改名失败时返回 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// write_device_wrap_atomic(state, bytes)?;
/// ```
pub fn write_device_wrap_atomic(state: &DbState, bytes: &[u8]) -> Result<(), DbError> {
  let tmp = state.dir.join(DB_DEVICE_WRAP_TMP);
  fs::write(&tmp, bytes).map_err(|_| DbError::Io)?;
  fs::rename(&tmp, &state.device_wrap).map_err(|_| DbError::Io)?;
  Ok(())
}

/// 先写临时文件再改名，替换密钥头
///
/// # 参数
/// - `state`: 数据库路径
/// - `header`: 要写入的密钥头
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 序列化失败时把 `CryptoError` 转成 `DbError`；写临时文件或改名失败时返回 `Io`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// write_header_atomic(state, header)?;
/// ```
fn write_header_atomic(state: &DbState, header: &KeyHeader) -> Result<(), DbError> {
  let json = header.to_pretty_json()?;
  fs::write(state.dir.join(DB_HEADER_TMP), json).map_err(|_| DbError::Io)?;
  fs::rename(state.dir.join(DB_HEADER_TMP), &state.header).map_err(|_| DbError::Io)?;
  Ok(())
}

/// 用原始密钥形式执行 `PRAGMA key`
///
/// # 参数
/// - `conn`: sqlite 连接
/// - `dek`: 32 字节页加密密钥
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 执行 `PRAGMA` 失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// apply_raw_key(&conn, dek)?;
/// ```
fn apply_raw_key(conn: &Connection, dek: &[u8; 32]) -> Result<(), DbError> {
  // DEK 是 32 个原始字节，里面什么都可能有（0、引号、换行），不能直接拼进 SQL 字符串。先变成 64 个 0-9a-f 字符，才能安全写进语句里。
  let hex_key = Zeroizing::new(hex::encode(dek));

  // SQLCipher 认两种喂钥匙方式：
  // -- 方式 A：通行短语（不要用）
  // PRAGMA key = '我的密码';
  // -- SQLCipher 会再用 PBKDF2 把这串字变成密钥
  // -- 方式 B：原始密钥（项目用这个）
  // PRAGMA key = "x'<64 hex>'";
  // -- 告诉引擎：这 32 字节就是最终密钥，别再派生了

  // 所以这里用的是方式 B
  let sql = Zeroizing::new(format!("PRAGMA key = \"x'{}'\";", hex_key.as_str()));

  // 使用日志输出 DEK 进行调试
  // 可以使用DB Browser for SQLite 查看数据库文件内容
  // tauri_plugin_log::log::warn!("DEBUG sqlcipher key: x'{}'", hex_key.as_str());

  conn.execute_batch(sql.as_str()).map_err(|_| DbError::Internal)
}

/// 创建 `db_meta` 并插入探针行
///
/// # 参数
/// - `conn`: 已设置密钥的连接
///
/// # 返回值
/// 成功时不返回数据
///
/// # 错误
/// 建表或插入失败时返回 `Internal`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// init_schema(&conn)?;
/// ```
fn init_schema(conn: &Connection) -> Result<(), DbError> {
  // 建一张 db_meta 表，插入探针行 ok = precession.db.ready
  conn
    .execute(
      "CREATE TABLE db_meta (
         key TEXT PRIMARY KEY NOT NULL,
         value TEXT NOT NULL
       )",
      [],
    )
    .map_err(|e| {
      tauri_plugin_log::log::error!("db create schema: {e}");
      DbError::Internal
    })?;

  // 插入探针行 ok = precession.db.ready
  conn.execute("INSERT INTO db_meta (key, value) VALUES (?1, ?2)", [PROBE_KEY, PROBE_VALUE]).map_err(|e| {
    tauri_plugin_log::log::error!("db insert probe: {e}");
    DbError::Internal
  })?;
  Ok(())
}

/// 读回探针并核对固定值
///
/// # 参数
/// - `conn`: 已设置密钥的连接
///
/// # 返回值
/// 探针值等于 `PROBE_VALUE` 时不返回数据
///
/// # 错误
/// 查不到探针，或值不等于 `PROBE_VALUE` 时返回 `Corrupt`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// verify_probe(&conn)?;
/// ```
fn verify_probe(conn: &Connection) -> Result<(), DbError> {
  let value: String = conn
    .query_row("SELECT value FROM db_meta WHERE key = ?1", [PROBE_KEY], |row| row.get(0))
    .map_err(|_| DbError::Corrupt)?;
  if value != PROBE_VALUE {
    return Err(DbError::Corrupt);
  }
  Ok(())
}
