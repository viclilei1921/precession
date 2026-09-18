use serde::Serialize;

/// 数据库状态
///
/// 用于前端展示数据库状态
#[derive(Clone, Debug, Serialize)]
pub struct DbStatus {
  /// 数据库文件是否存在
  pub exists: bool,
  /// 数据库是否已解锁
  pub unlocked: bool,
}
