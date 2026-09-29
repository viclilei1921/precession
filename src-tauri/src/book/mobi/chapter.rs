//! MOBI 分章启发式

pub(crate) fn looks_like_chapter_title(title: &str) -> bool {
  if title.is_empty() || title.chars().count() > 20 {
    return false;
  }
  // 排除含标点或引号的正文行（章节标题通常不含这些字符）
  !title.contains(['，', '。', '？', '！', '、', '；', '：', '"', '"', '"', '\'', '?', '!', ',', '.', ';', ':'])
}
