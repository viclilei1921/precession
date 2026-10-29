//! 按路径缓存已打开的书，翻章时不再重新解析。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use super::OpenedBook;
use super::types::BookError;

static OPEN: LazyLock<Mutex<HashMap<PathBuf, OpenedBook>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// 打开路径上的书，已缓存时直接用
///
/// # 参数
/// - `path`: 书籍文件路径
/// - `work`: 拿到已打开书籍后执行的操作
///
/// # 返回值
/// 返回 `work` 的结果
///
/// # 错误
/// 打开失败时返回解析错误；`work` 的错误原样传出
///
/// # Panics
/// 缓存锁中毒时取回内部数据，不会 panic
///
/// # 示例
/// ```ignore
/// let outline = with_book(path, |book| Ok(book.outline()))?;
/// ```
pub(crate) fn with_book<T>(
  path: &Path,
  work: impl FnOnce(&OpenedBook) -> Result<T, BookError>,
) -> Result<T, BookError> {
  let mut guard = OPEN.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
  if !guard.contains_key(path) {
    let book = OpenedBook::open(path)?;
    guard.insert(path.to_path_buf(), book);
  }
  let book = guard.get(path).expect("刚放入缓存");
  work(book)
}
