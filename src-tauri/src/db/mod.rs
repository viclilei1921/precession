pub mod commands;
pub mod dto;
pub mod error;
pub mod state;

pub(crate) mod schema;

mod constants;
mod storage;

use std::path::PathBuf;

/// 用应用数据目录创建挂上全部业务迁移的数据库句柄
///
/// # 参数
/// - `app_data_dir`: 应用本地数据目录
///
/// # 返回值
/// 已登记成员、标签、地点、媒体、计划、日记、成长、书架和日历迁移的 `DbState`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let state = app_state(app_data_dir);
/// ```
pub fn app_state(app_data_dir: PathBuf) -> state::DbState {
  state::DbState::new(app_data_dir).with_migrators(vec![
    crate::member::migrate,
    crate::tag::migrate,
    crate::place::migrate,
    crate::media::migrate,
    crate::plan::migrate,
    crate::journal::migrate,
    crate::growth::migrate,
    crate::library::migrate,
    crate::calendar::migrate,
  ])
}
