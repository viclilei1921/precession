//! 片头书名 / 作者提取

use std::sync::LazyLock;

use regex::Regex;

/// 从一行里提取书名
///
/// 去掉首尾空白后，空行或超过 64 个字符返回 `None`。优先取第一对书名号里的非空内容，其次识别 `书名：`、`书名:`、`书名` 前缀。都不匹配时返回整行去空白后的文本。
///
/// # 参数
/// - `line`: 待检查的一行
///
/// # 返回值
/// 提取到书名时返回 `Some`，空行或超长时返回 `None`
///
/// # Panics
/// 书名号正则编译失败时 `unwrap` 会 panic
///
/// # 示例
/// ```
/// assert_eq!(extract_title("《红楼梦》"), Some("红楼梦".to_string()));
/// assert_eq!(extract_title("书名：红楼梦"), Some("红楼梦".to_string()));
/// assert_eq!(extract_title(""), None);
/// ```
pub(super) fn extract_title(line: &str) -> Option<String> {
  let t = line.trim();
  if t.is_empty() || t.chars().count() > 64 {
    return None;
  }

  // 优先取《书名》
  static RE_BOOK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"《([^》]+)》").unwrap());
  if let Some(c) = RE_BOOK.captures(t) {
    let inner = c.get(1)?.as_str().trim();
    if !inner.is_empty() {
      return Some(inner.to_string());
    }
  }

  // 书名：xxx / 书名:xxx / 书名xxx
  for prefix in ["书名：", "书名:", "书名"] {
    if let Some(rest) = t.strip_prefix(prefix) {
      let rest = rest.trim().trim_start_matches(['：', ':']).trim();
      if !rest.is_empty() {
        return Some(rest.to_string());
      }
    }
  }

  Some(t.to_string())
}

/// 从一行里提取作者
///
/// 去掉首尾空白后，空行或超过 64 个字符返回 `None`。按 `作者：`、`作者:`、`作者 `、`作者`、`著：`、`著:`、`著 `、`著` 的顺序去掉前缀，再去掉多余冒号和空白。前缀后还有内容才返回。
///
/// # 参数
/// - `line`: 待检查的一行
///
/// # 返回值
/// 提取到作者时返回 `Some`，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(extract_author("作者：曹雪芹"), Some("曹雪芹".to_string()));
/// assert_eq!(extract_author("著曹雪芹"), Some("曹雪芹".to_string()));
/// assert_eq!(extract_author("正文"), None);
/// ```
pub(super) fn extract_author(line: &str) -> Option<String> {
  let t = line.trim();
  if t.is_empty() || t.chars().count() > 64 {
    return None;
  }
  for prefix in ["作者：", "作者:", "作者 ", "作者", "著：", "著:", "著 "] {
    if let Some(rest) = t.strip_prefix(prefix) {
      let rest = rest.trim().trim_start_matches(['：', ':']).trim();
      if !rest.is_empty() {
        return Some(rest.to_string());
      }
    }
  }
  // 「著曹雪芹」无冒号空格
  if let Some(rest) = t.strip_prefix('著') {
    let rest = rest.trim().trim_start_matches(['：', ':']).trim();
    if !rest.is_empty() {
      return Some(rest.to_string());
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extracts_title_and_author() {
    assert_eq!(extract_title("《红楼梦》"), Some("红楼梦".to_string()));
    assert_eq!(extract_title("《红楼梦》作者：曹雪芹"), Some("红楼梦".to_string()));
    assert_eq!(extract_title("你好《红楼梦》"), Some("红楼梦".to_string()));
    assert_eq!(extract_title("书名：红楼梦"), Some("红楼梦".to_string()));
    assert_eq!(extract_title("书名:红楼梦"), Some("红楼梦".to_string()));
    assert_eq!(extract_title("书名红楼梦"), Some("红楼梦".to_string()));
    assert_eq!(extract_author("作者：曹雪芹"), Some("曹雪芹".to_string()));
    assert_eq!(extract_author("作者:曹雪芹"), Some("曹雪芹".to_string()));
    assert_eq!(extract_author("作者曹雪芹"), Some("曹雪芹".to_string()));
    assert_eq!(extract_author("著：曹雪芹"), Some("曹雪芹".to_string()));
    assert_eq!(extract_author("著:曹雪芹"), Some("曹雪芹".to_string()));
    assert_eq!(extract_author("著曹雪芹"), Some("曹雪芹".to_string()));
  }
}
