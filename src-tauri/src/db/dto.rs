use serde::Serialize;

/// 数据库状态
///
/// 用于前端展示数据库状态
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
