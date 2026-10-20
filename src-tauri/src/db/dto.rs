use serde::Serialize;

/// 数据库状态
///
/// 用于前端展示数据库状态。
///
/// # 字段
/// - `exists`: 数据库文件是否存在
/// - `unlocked`: 数据库是否已解锁
/// - `device_unlock`: `device.wrap` 是否存在。查询状态时不访问系统密钥库。
/// - `user_locked`: 用户点过锁定。重启后仍然为真，直到密码或系统验证解锁成功。
///
/// # 示例
/// ```
/// let status = DbStatus {
///   exists: true,
///   unlocked: false,
///   device_unlock: true,
///   user_locked: true,
/// };
/// assert!(status.exists && status.user_locked && !status.unlocked);
/// ```
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStatus {
  /// 数据库文件是否存在
  pub exists: bool,
  /// 数据库是否已解锁
  pub unlocked: bool,
  /// `device.wrap` 是否存在。查询状态时不访问系统密钥库。
  pub device_unlock: bool,
  /// 用户点过锁定。重启后仍然为真，直到密码或系统验证解锁成功。
  pub user_locked: bool,
}
