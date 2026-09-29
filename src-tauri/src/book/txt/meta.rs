//! 片头书名 / 作者提取

use std::sync::LazyLock;

use regex::Regex;

/// 从一行提取书名（支持《书名》、`书名：` 前缀）
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

/// 从一行提取作者（`作者：` / `著：` 等）
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
