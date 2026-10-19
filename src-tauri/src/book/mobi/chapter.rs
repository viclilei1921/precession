//! MOBI 分章启发式

/// 判断字符串是否像章节标题
///
/// 空字符串或超过 20 个字符返回 `false`。含中英文标点或引号也返回 `false`。
///
/// # 参数
/// - `title`: 待判断的标题文本
///
/// # 返回值
/// 像章节标题时返回 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(looks_like_chapter_title("第一章"));
/// assert!(!looks_like_chapter_title("你好，世界"));
/// assert!(!looks_like_chapter_title(""));
/// ```
pub(crate) fn looks_like_chapter_title(title: &str) -> bool {
  if title.is_empty() || title.chars().count() > 20 {
    return false;
  }
  // 排除含标点或引号的正文行（章节标题通常不含这些字符）
  !title.contains(['，', '。', '？', '！', '、', '；', '：', '"', '"', '"', '\'', '?', '!', ',', '.', ';', ':'])
}
