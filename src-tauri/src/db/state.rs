use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};

use rusqlite::Connection;
use zeroize::Zeroizing;

use super::constants::{DB_DIR, DB_HEADER, DB_SQLITE};
use super::dto::DbStatus;
use super::error::DbError;
use super::repository::{create_db, open_db, read_header, remove_create_artifacts};
use crate::constants::path::DATA_DIR;
use crate::crypto::dek::{open_dek, platform_kdf_params, random_dek, seal_dek};

/// 业务模块 schema 迁移。建库/解锁且连接已就绪后调用，不让 `db` 依赖具体功能表。
pub type SchemaMigrator = fn(&Connection) -> Result<(), DbError>;

struct Session {
  /// SQLCipher 页加密密钥。连接里已经有一份；
  /// 这里再留一份，给以后改密或再开连接，避免用户重复输密码。
  #[allow(dead_code)]
  dek: Zeroizing<[u8; 32]>,
  /// 已 `PRAGMA key` 的连接。业务侧通过 `with_conn` 使用。
  conn: Connection,
}

/// 进程级数据库句柄。
///
/// Tauri 每个 command 都是一次独立调用，解锁后的连接必须放在 State 里才能跨 IPC 复用。
/// `Arc`：command、退出钩子、`spawn_blocking` 共享同一份。
/// `Mutex`：`rusqlite::Connection` 不能并行用；create / unlock / lock 也必须互斥。
#[derive(Clone)]
pub struct DbState {
  /// 数据库目录
  pub dir: PathBuf,
  /// 数据库文件
  pub sqlite: PathBuf,
  /// 数据库头文件
  pub header: PathBuf,
  /// 数据库会话
  session: Arc<Mutex<Option<Session>>>,
  /// 各业务模块的建表 / 迁移。`Clone` 只复制函数指针。
  migrators: Vec<SchemaMigrator>,
}

impl DbState {
  /// 用数据库目录初始化句柄。路径由调用方解析（应用数据目录 + `DB_DIR`）。
  pub fn new(app_data_dir: PathBuf) -> Self {
    let dir = app_data_dir.join(DATA_DIR).join(DB_DIR);

    Self {
      sqlite: dir.join(DB_SQLITE),
      header: dir.join(DB_HEADER),
      dir,
      session: Arc::new(Mutex::new(None)),
      migrators: Vec::new(),
    }
  }

  /// 注册业务表迁移。后续功能在 `lib.rs` 往这个列表里追加即可。
  pub fn with_migrators(mut self, migrators: Vec<SchemaMigrator>) -> Self {
    self.migrators = migrators;
    self
  }

  fn apply_migrators(&self, conn: &Connection) -> Result<(), DbError> {
    for migrate in &self.migrators {
      migrate(conn)?;
    }
    Ok(())
  }

  /// 获取数据库状态。
  pub fn status(&self) -> DbStatus {
    let exists = self.sqlite.exists() || self.header.exists();

    let unlocked = self.session.lock().unwrap().is_some();

    DbStatus { exists, unlocked }
  }

  pub fn create(&self, password: &str) -> Result<(), DbError> {
    let DbStatus { exists, unlocked } = self.status();
    if exists || unlocked {
      return Err(DbError::AlreadyExists);
    }

    let dek = random_dek()?;
    let kdf = platform_kdf_params();
    let header = seal_dek(password, &dek, kdf)?;

    // 创建数据库, 失败时删除临时文件
    let conn = create_db(&self, &dek, &header).map_err(|e| {
      tauri_plugin_log::log::error!("db create: {e}");
      remove_create_artifacts(&self);
      e
    })?;

    if let Err(e) = self.apply_migrators(&conn) {
      drop(conn);
      remove_create_artifacts(&self);
      return Err(e);
    }

    *self.session.lock().unwrap() = Some(Session { dek, conn });

    tauri_plugin_log::log::info!("db created");
    Ok(())
  }

  pub fn unlock(&self, password: &str) -> Result<(), DbError> {
    let DbStatus { exists, unlocked } = self.status();
    if !exists {
      return Err(DbError::NotFound);
    }

    if unlocked {
      return Err(DbError::AlreadyUnlocked);
    }

    let header = read_header(&self.header)?;
    let dek = open_dek(password, &header)?;
    let conn = open_db(&self.sqlite, &dek)?;
    self.apply_migrators(&conn)?;

    *self.session.lock().unwrap() = Some(Session { dek, conn });

    Ok(())
  }

  pub fn lock(&self) {
    *self.session.lock().unwrap() = None;
  }

  /// 业务模块读写库走这里，不要自己 `Connection::open`。
  ///
  /// `work`：拿到连接后要执行的操作。
  /// `T`：操作成功时的返回值；`E`：操作失败时的错误（可以是 `DbError`，也可以是业务自己的错误）。
  pub fn with_conn<T, E>(&self, work: impl FnOnce(&Connection) -> Result<T, E>) -> Result<T, E>
  where
    // 未解锁时我们只有 DbError::Locked，必须能转成调用方的 E（例如 DemoError）
    E: From<DbError>,
  {
    let session = self.session.lock().unwrap();

    match &*session {
      Some(session) => work(&session.conn),
      None => Err(E::from(DbError::Locked)),
    }
  }
}
