use std::{
  fs,
  path::PathBuf,
  sync::{Arc, Mutex},
};

use rusqlite::Connection;
use zeroize::Zeroizing;

use super::constants::{DATA_DIR, DB_DEVICE_WRAP, DB_DEVICE_WRAP_TMP, DB_DIR, DB_HEADER, DB_SQLITE, DB_USER_LOCK};
use super::dto::DbStatus;
use super::error::DbError;
use super::storage::{create_db, open_db, read_header, remove_create_artifacts, write_device_wrap_atomic};
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
  /// 根据应用数据目录组装数据库路径和空会话
  ///
  /// 目录是 `app_data_dir/data/db`。设备槽用当前平台槽，迁移列表为空。
  ///
  /// # 参数
  /// - `app_data_dir`: 应用本地数据目录
  ///
  /// # 返回值
  /// 尚未解锁的 `DbState`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let state = DbState::new(app_data_dir);
  /// ```
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

  /// 换上给定的设备槽
  ///
  /// # 参数
  /// - `device`: 系统密钥库或测试用槽
  ///
  /// # 返回值
  /// 换槽后的句柄
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let state = state.with_device(device);
  /// ```
  #[cfg_attr(not(any(test, target_os = "android")), allow(dead_code))]
  pub fn with_device(mut self, device: Arc<dyn DeviceSlot>) -> Self {
    self.device = device;
    self
  }

  /// 换上给定的业务表迁移列表
  ///
  /// # 参数
  /// - `migrators`: 建库或解锁后要执行的迁移
  ///
  /// # 返回值
  /// 换上迁移列表后的句柄
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let state = state.with_migrators(migrators);
  /// ```
  pub fn with_migrators(mut self, migrators: Vec<SchemaMigrator>) -> Self {
    self.migrators = migrators;
    self
  }

  /// 按注册顺序执行业务表迁移
  ///
  /// # 参数
  /// - `conn`: 已就绪的数据库连接
  ///
  /// # 返回值
  /// 全部迁移成功时不返回数据
  ///
  /// # 错误
  /// 任一迁移返回错误时传出该 `DbError`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// state.apply_migrators(&conn)?;
  /// ```
  fn apply_migrators(&self, conn: &Connection) -> Result<(), DbError> {
    for migrate in &self.migrators {
      migrate(conn)?;
    }
    Ok(())
  }

  /// 查看库文件、会话、设备槽和用户锁定标记是否存在
  ///
  /// # 返回值
  /// 四项标志组成的 `DbStatus`
  ///
  /// # Panics
  /// `session` 互斥锁中毒时，`lock().unwrap()` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// let status = state.status();
  /// ```
  pub fn status(&self) -> DbStatus {
    let exists = self.sqlite.exists() || self.header.exists();

    let unlocked = self.session.lock().unwrap().is_some();
    let device_unlock = self.device_wrap.exists();
    let user_locked = self.user_lock_path().exists();

    DbStatus { exists, unlocked, device_unlock, user_locked }
  }

  /// 返回用户锁定标记的路径
  ///
  /// # 返回值
  /// 数据库目录下的 `user.lock` 路径
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let path = state.user_lock_path();
  /// ```
  fn user_lock_path(&self) -> PathBuf {
    self.dir.join(DB_USER_LOCK)
  }

  /// 删除用户锁定标记文件
  ///
  /// 删除失败时忽略错误。
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// state.clear_user_lock();
  /// ```
  fn clear_user_lock(&self) {
    let _ = fs::remove_file(self.user_lock_path());
  }

  /// 校验密码后创建加密库并留下解锁会话
  ///
  /// 库或头文件已存在，或会话已解锁时拒绝。创建或迁移失败时删掉本次留下的库文件和头文件。
  ///
  /// # 参数
  /// - `password`: 档案密码
  /// - `password_confirm`: 再输入一次的密码
  ///
  /// # 返回值
  /// 成功时不返回数据，并把连接放进会话
  ///
  /// # 错误
  /// - 密码为空返回 `PasswordEmpty`，短于 8 个字符返回 `PasswordTooShort`，两次不一致返回 `PasswordMismatch`
  /// - 库文件或头文件已存在，或会话已解锁时返回 `AlreadyExists`
  /// - 生成或封装 DEK 失败时，把 `CryptoError` 转成 `DbError` 返回
  /// - 建库失败时删掉残留并返回该 `DbError`
  /// - 迁移失败时删掉残留并返回迁移错误
  ///
  /// # Panics
  /// `session` 互斥锁中毒时，`lock().unwrap()` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// state.create(password, password_confirm)?;
  /// ```
  pub fn create(&self, password: &str, password_confirm: &str) -> Result<(), DbError> {
    check_new_password(password, password_confirm)?;

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
    let conn = create_db(self, &dek, &header).map_err(|e| {
      tauri_plugin_log::log::error!("db create: {e}");
      remove_create_artifacts(self);
      e
    })?;

    if let Err(e) = self.apply_migrators(&conn) {
      drop(conn);
      remove_create_artifacts(self);
      return Err(e);
    }

    *self.session.lock().unwrap() = Some(Session { dek, conn });

    tauri_plugin_log::log::info!("db created");
    Ok(())
  }

  /// 用密码解开密钥头并打开数据库
  ///
  /// 成功后清掉用户锁定标记。
  ///
  /// # 参数
  /// - `password`: 档案密码
  ///
  /// # 返回值
  /// 成功时不返回数据，并把连接放进会话
  ///
  /// # 错误
  /// - 库文件和头文件都不存在时返回 `NotFound`
  /// - 会话已经解锁时返回 `AlreadyUnlocked`
  /// - 读头、解 DEK、打开库或迁移失败时传出对应的 `DbError`
  ///
  /// # Panics
  /// `session` 互斥锁中毒时，`lock().unwrap()` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// state.unlock(password)?;
  /// ```
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

  /// 核对密码后把会话里的 DEK 登记进设备槽
  ///
  /// 写入 `device.wrap` 失败时会调用 `forget`。
  ///
  /// # 参数
  /// - `password`: 档案密码
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// - 密码为空返回 `PasswordEmpty`，短于 8 个字符返回 `PasswordTooShort`
  /// - 没有会话时返回 `Locked`
  /// - 读头或解 DEK 失败时传出对应的 `DbError`
  /// - 解开的 DEK 与会话里的不一致时返回 `Corrupt`
  /// - 登记或原子写入失败时返回对应的 `DbError`
  ///
  /// # Panics
  /// `session` 互斥锁中毒时，`lock().unwrap()` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// state.enable_device(password)?;
  /// ```
  pub fn enable_device(&self, password: &str) -> Result<(), DbError> {
    check_password(password)?;

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

  /// 用设备槽解开数据库
  ///
  /// 有用户锁定标记时先调用 `confirm`。成功后清掉该标记。
  ///
  /// # 返回值
  /// 成功时不返回数据，并把连接放进会话
  ///
  /// # 错误
  /// - 库文件和头文件都不存在时返回 `NotFound`
  /// - 会话已经解锁时返回 `AlreadyUnlocked`
  /// - 没有 `device.wrap` 时返回 `DeviceInvalid`
  /// - 系统验证、读取密文、打开设备槽、打开库或迁移失败时传出对应的 `DbError`
  /// - 读 `device.wrap` 失败时返回 `Io`
  ///
  /// # Panics
  /// `session` 互斥锁中毒时，`lock().unwrap()` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// state.unlock_device()?;
  /// ```
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

  /// 删掉系统里的设备密钥和设备槽文件
  ///
  /// 正式文件和临时文件删除失败时忽略错误。
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// `forget` 失败时把 `DeviceError` 转成 `DbError` 返回
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// state.disable_device()?;
  /// ```
  pub fn disable_device(&self) -> Result<(), DbError> {
    self.device.forget()?;
    let _ = fs::remove_file(&self.device_wrap);
    let _ = fs::remove_file(self.dir.join(DB_DEVICE_WRAP_TMP));
    tauri_plugin_log::log::info!("device unlock disabled");
    Ok(())
  }

  /// 清掉内存中的数据库会话
  ///
  /// # 返回值
  /// 不返回数据
  ///
  /// # Panics
  /// `session` 互斥锁中毒时，`lock().unwrap()` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// state.release_session();
  /// ```
  pub fn release_session(&self) {
    *self.session.lock().unwrap() = None;
  }

  /// 清掉会话并写下用户锁定标记
  ///
  /// # 返回值
  /// 成功时不返回数据
  ///
  /// # 错误
  /// 创建数据库目录或写入 `user.lock` 失败时返回 `Io`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// state.lock()?;
  /// ```
  pub fn lock(&self) -> Result<(), DbError> {
    self.release_session();
    fs::create_dir_all(&self.dir).map_err(|_| DbError::Io)?;
    fs::write(self.user_lock_path(), b"1").map_err(|_| DbError::Io)?;
    Ok(())
  }

  /// 在已解锁的连接上执行一次操作
  ///
  /// `E` 必须能从 `DbError` 转来，未解锁时才能把 `Locked` 交回调用方。
  ///
  /// # 参数
  /// - `work`: 拿到连接后要执行的操作
  ///
  /// # 返回值
  /// 成功时返回 `work` 的 `T`
  ///
  /// # 错误
  /// 未解锁时返回 `E::from(DbError::Locked)`；`work` 返回错误时原样传出
  ///
  /// # Panics
  /// `session` 互斥锁中毒时，`lock().unwrap()` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// let value = state.with_conn(|conn| read_row(conn))?;
  /// ```
  pub fn with_conn<T, E>(&self, work: impl FnOnce(&Connection) -> Result<T, E>) -> Result<T, E>
  where
    // 未解锁时我们只有 DbError::Locked，必须能转成调用方的 E（例如 DemoError）。
    E: From<DbError>,
  {
    let session = self.session.lock().unwrap();

    match &*session {
      Some(session) => work(&session.conn),
      None => Err(E::from(DbError::Locked)),
    }
  }
}

const MIN_PASSWORD_LEN: usize = 8;

/// 检查密码非空且不少于 8 个字符
///
/// # 参数
/// - `password`: 档案密码
///
/// # 返回值
/// 通过时不返回数据
///
/// # 错误
/// 空字符串返回 `PasswordEmpty`；不足 8 个字符返回 `PasswordTooShort`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(check_password("").is_err());
/// assert!(check_password("short").is_err());
/// assert!(check_password("12345678").is_ok());
/// ```
fn check_password(password: &str) -> Result<(), DbError> {
  if password.is_empty() {
    return Err(DbError::PasswordEmpty);
  }
  if password.chars().count() < MIN_PASSWORD_LEN {
    return Err(DbError::PasswordTooShort);
  }
  Ok(())
}

/// 检查新密码长度，并要求两次输入相同
///
/// # 参数
/// - `password`: 档案密码
/// - `password_confirm`: 再输入一次的密码
///
/// # 返回值
/// 通过时不返回数据
///
/// # 错误
/// 空字符串返回 `PasswordEmpty`；不足 8 个字符返回 `PasswordTooShort`；两次不同返回 `PasswordMismatch`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(check_new_password("12345678", "12345678").is_ok());
/// assert!(check_new_password("12345678", "87654321").is_err());
/// ```
fn check_new_password(password: &str, password_confirm: &str) -> Result<(), DbError> {
  check_password(password)?;
  if password != password_confirm {
    return Err(DbError::PasswordMismatch);
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::crypto::device::MemorySlot;

  fn setup() -> (tempfile::TempDir, DbState, Arc<MemorySlot>) {
    let dir = tempfile::tempdir().expect("tempdir");
    let slot = Arc::new(MemorySlot::new());
    let db = DbState::new(dir.path().to_path_buf()).with_device(slot.clone());
    db.create("test-password-123", "test-password-123").expect("create db");
    (dir, db, slot)
  }

  #[test]
  fn device_slot_requires_password_and_round_trips() {
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
