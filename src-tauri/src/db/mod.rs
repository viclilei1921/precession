pub mod commands;
pub mod dto;
pub mod error;
pub mod state;

pub(crate) mod schema;

mod constants;
mod storage;

pub(crate) use constants::{DB_DEVICE_WRAP_TMP, DB_DIR, DB_HEADER, DB_SQLITE, DB_USER_LOCK};
pub(crate) use storage::read_header;

use std::path::PathBuf;

/// 用应用数据目录创建挂上全部业务迁移的数据库句柄
///
/// # 参数
/// - `app_data_dir`: 应用本地数据目录
///
/// # 返回值
/// 已登记成员、标签、地点、上传文件、计划、日记、成长、书架和日历迁移，以及上传文件清理钩子的 `DbState`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let state = app_state(app_data_dir);
/// ```
pub fn app_state(app_data_dir: PathBuf) -> state::DbState {
  state::DbState::new(app_data_dir)
    .with_migrators(vec![
      crate::member::migrate,
      crate::tag::migrate,
      crate::place::migrate,
      crate::upload::migrate,
      crate::plan::migrate,
      crate::journal::migrate,
      crate::growth::migrate,
      crate::library::migrate,
      crate::calendar::migrate,
    ])
    .with_open_hooks(vec![crate::upload::on_open])
}
