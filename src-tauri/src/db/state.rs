use std::{
  fs,
  path::PathBuf,
  sync::{Arc, Mutex},
};

use rusqlite::Connection;
use zeroize::Zeroizing;

use super::constants::{DB_DEVICE_WRAP, DB_DEVICE_WRAP_TMP, DB_DIR, DB_HEADER, DB_SQLITE, DB_USER_LOCK};
use super::dto::DbStatus;
use super::error::DbError;
use super::repository::{create_db, open_db, read_header, remove_create_artifacts, write_device_wrap_atomic};
use crate::constants::path::DATA_DIR;
use crate::crypto::dek::{open_dek, platform_kdf_params, random_dek, seal_dek};
use crate::crypto::device::{self, DeviceSlot};

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
  /// 设备槽密文
  pub device_wrap: PathBuf,
  /// 数据库会话
  session: Arc<Mutex<Option<Session>>>,
  /// 系统密钥库。测试可以换成内存假槽。
  device: Arc<dyn DeviceSlot>,
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
      device_wrap: dir.join(DB_DEVICE_WRAP),
      dir,
      session: Arc::new(Mutex::new(None)),
      device: device::platform_slot(),
      migrators: Vec::new(),
    }
  }

  /// 换成测试用的设备槽，或在 Android 上接系统密钥库插件。
  #[cfg_attr(not(any(test, target_os = "android")), allow(dead_code))]
  pub fn with_device(mut self, device: Arc<dyn DeviceSlot>) -> Self {
    self.device = device;
    self
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
    let device_unlock = self.device_wrap.exists();
    let user_locked = self.user_lock_path().exists();

    DbStatus { exists, unlocked, device_unlock, user_locked }
  }

  fn user_lock_path(&self) -> PathBuf {
    self.dir.join(DB_USER_LOCK)
  }

  fn clear_user_lock(&self) {
    let _ = fs::remove_file(self.user_lock_path());
  }

  pub fn create(&self, password: &str) -> Result<(), DbError> {
    let DbStatus { exists, unlocked, .. } = self.status();
    if exists || unlocked {
      return Err(DbError::AlreadyExists);
    }

    let _ = fs::remove_file(&self.device_wrap);
    let _ = fs::remove_file(self.dir.join(DB_DEVICE_WRAP_TMP));
    self.clear_user_lock();

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
    let DbStatus { exists, unlocked, .. } = self.status();
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
    self.clear_user_lock();

    Ok(())
  }

  /// 已解锁，并且密码能解开头文件时，把内存里的 DEK 登记进设备槽。
  pub fn enable_device(&self, password: &str) -> Result<(), DbError> {
    let session_dek = {
      let guard = self.session.lock().unwrap();
      match &*guard {
        Some(session) => session.dek.clone(),
        None => return Err(DbError::Locked),
      }
    };

    let header = read_header(&self.header)?;
    let opened = open_dek(password, &header)?;
    if opened.as_ref() != session_dek.as_ref() {
      return Err(DbError::Corrupt);
    }

    let blob = self.device.enroll(&session_dek)?;
    if let Err(err) = write_device_wrap_atomic(self, &blob) {
      let _ = self.device.forget();
      return Err(err);
    }
    tauri_plugin_log::log::info!("device unlock enabled");
    Ok(())
  }

  /// 用设备槽解开 DEK。没有用户锁定标记时静默打开；有标记时先走系统验证。
  pub fn unlock_device(&self) -> Result<(), DbError> {
    let DbStatus { exists, unlocked, user_locked, .. } = self.status();
    if !exists {
      return Err(DbError::NotFound);
    }
    if unlocked {
      return Err(DbError::AlreadyUnlocked);
    }
    if !self.device_wrap.exists() {
      return Err(DbError::DeviceInvalid);
    }
    if user_locked {
      self.device.confirm()?;
    }

    let blob = fs::read(&self.device_wrap).map_err(|_| DbError::Io)?;
    let dek = self.device.open(&blob)?;
    let conn = open_db(&self.sqlite, &dek)?;
    self.apply_migrators(&conn)?;
    *self.session.lock().unwrap() = Some(Session { dek, conn });
    self.clear_user_lock();
    tauri_plugin_log::log::info!("db unlocked with device slot");
    Ok(())
  }

  /// 删除系统里的设备密钥和 `device.wrap`。不改数据库。
  pub fn disable_device(&self) -> Result<(), DbError> {
    self.device.forget()?;
    let _ = fs::remove_file(&self.device_wrap);
    let _ = fs::remove_file(self.dir.join(DB_DEVICE_WRAP_TMP));
    tauri_plugin_log::log::info!("device unlock disabled");
    Ok(())
  }

  /// 丢掉内存里的会话。不留下锁定标记，下次启动仍可静默打开。
  pub fn release_session(&self) {
    *self.session.lock().unwrap() = None;
  }

  pub fn lock(&self) -> Result<(), DbError> {
    self.release_session();
    fs::create_dir_all(&self.dir).map_err(|_| DbError::Io)?;
    fs::write(self.user_lock_path(), b"1").map_err(|_| DbError::Io)?;
    Ok(())
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::crypto::device::MemorySlot;

  fn setup() -> (tempfile::TempDir, DbState, Arc<MemorySlot>) {
    let dir = tempfile::tempdir().expect("tempdir");
    let slot = Arc::new(MemorySlot::new());
    let db = DbState::new(dir.path().to_path_buf()).with_device(slot.clone());
    db.create("test-password-123").expect("create db");
    (dir, db, slot)
  }

  #[test]
  fn device_slot_requires_password_and_roundtrips() {
    let (dir, db, slot) = setup();
    assert!(!db.status().device_unlock);

    assert!(matches!(db.enable_device("wrong-password"), Err(DbError::WrongPassword)));
    assert!(!db.status().device_unlock);

    db.enable_device("test-password-123").expect("enable");
    assert!(db.status().device_unlock);

    let blob = fs::read(&db.device_wrap).expect("read wrap");
    assert!(matches!(
      MemorySlot::new().open(&blob),
      Err(crate::crypto::device::DeviceError::Invalid)
    ));

    db.lock().expect("lock");
    assert!(!db.status().unlocked);
    assert!(db.status().user_locked);
    assert!(matches!(db.enable_device("test-password-123"), Err(DbError::Locked)));

    let restarted = DbState::new(dir.path().to_path_buf()).with_device(slot.clone());
    assert!(restarted.status().user_locked);
    assert!(!restarted.status().unlocked);

    db.unlock_device().expect("unlock device");
    assert!(db.status().unlocked);
    assert!(!db.status().user_locked);

    db.disable_device().expect("disable");
    assert!(!db.status().device_unlock);
    assert!(slot.open(&blob).is_err());

    db.lock().expect("lock");
    assert!(matches!(db.unlock_device(), Err(DbError::DeviceInvalid)));
    assert!(db.status().user_locked);
    db.unlock("test-password-123").expect("password still works");
    assert!(db.status().unlocked);
    assert!(!db.status().user_locked);
  }
}
