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

use super::constants::{DB_HEADER, DB_HEADER_TMP, DB_SQLITE, PROBE_KEY, PROBE_VALUE};
use super::error::DbError;
use super::state::DbState;
use crate::crypto::header::KeyHeader;

/// 从零建一座加密库
/// 1. 随机生成 32 字节 DEK
/// 2. 密码 → KEK → 把 DEK 包进 key.header.json
/// 3. 打开空的 data.sqlite
/// 4. PRAGMA key = "x'<DEK hex>'";     ← DEK 唯一进入数据库的方式
/// 5. CREATE TABLE db_meta ...
/// 6. INSERT 探针
/// 7. SELECT 探针  （确认加密读写成功）
/// 8. 连接留在内存里；之后 INSERT 日记、账本都自动按页加密
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

/// 删除创建过程中产生的临时文件
pub fn remove_create_artifacts(state: &DbState) {
  let _ = fs::remove_file(state.dir.join(DB_SQLITE));
  let _ = fs::remove_file(state.dir.join(DB_HEADER));
  let _ = fs::remove_file(&state.sqlite);
  let _ = fs::remove_file(&state.header);
}

/// 用已有 DEK 打开库
/// 1. 读 key.header.json
/// 2. 密码 → KEK → 解开 DEK
/// 3. 打开已有的 data.sqlite
/// 4. 同样的 PRAGMA key = "x'<DEK hex>'";
/// 5. SELECT 探针               ← 错钥/坏库在这里暴露
/// 6. 业务 SQL 正常使用，不再出现 DEK
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

/// 读取头文件: 把门牌读进来
pub fn read_header(path: &std::path::Path) -> Result<KeyHeader, DbError> {
  let raw = fs::read_to_string(path).map_err(|_| DbError::Io)?;
  Ok(KeyHeader::from_json(&raw)?)
}

/// 先写草稿再改名
fn write_header_atomic(state: &DbState, header: &KeyHeader) -> Result<(), DbError> {
  let json = header.to_pretty_json()?;
  fs::write(state.dir.join(DB_HEADER_TMP), json).map_err(|_| DbError::Io)?;
  fs::rename(state.dir.join(DB_HEADER_TMP), &state.header).map_err(|_| DbError::Io)?;
  Ok(())
}

/// 把 DEK 交给 SQLCipher
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

/// 初始化 schema: 放下第一张桌子和一枚探针
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

/// 验证探针
fn verify_probe(conn: &Connection) -> Result<(), DbError> {
  let value: String = conn
    .query_row("SELECT value FROM db_meta WHERE key = ?1", [PROBE_KEY], |row| row.get(0))
    .map_err(|_| DbError::Corrupt)?;
  if value != PROBE_VALUE {
    return Err(DbError::Corrupt);
  }
  Ok(())
}
