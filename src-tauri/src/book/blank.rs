//! 判断一章正文是不是空白。TXT 是纯文本，EPUB 和 MOBI 是 HTML。

use std::sync::OnceLock;

use regex::Regex;

struct Cleaners {
  comment: Regex,
  script: Regex,
  style: Regex,
  tag: Regex,
  space_entity: Regex,
}

fn cleaners() -> &'static Cleaners {
  static CLEAN: OnceLock<Cleaners> = OnceLock::new();
  CLEAN.get_or_init(|| Cleaners {
    comment: Regex::new(r"(?s)<!--.*?-->").expect("valid regex"),
    script: Regex::new(r"(?is)<script\b[^>]*>.*?</script>").expect("valid regex"),
    style: Regex::new(r"(?is)<style\b[^>]*>.*?</style>").expect("valid regex"),
    tag: Regex::new(r"(?is)<[^>]+>").expect("valid regex"),
    space_entity: Regex::new(r"(?i)&(?:nbsp|ensp|emsp|thinsp|#160|#x0*a0);").expect("valid regex"),
  })
}

fn is_space(ch: char) -> bool {
  ch.is_whitespace() || matches!(ch, '\u{00a0}' | '\u{feff}' | '\u{200b}' | '\u{200c}' | '\u{200d}')
}

fn contains_media(content: &str) -> bool {
  let lower = content.to_ascii_lowercase();
  ["<img", "<svg", "<video", "<audio", "<object", "<iframe"].iter().any(|tag| lower.contains(tag))
}

/// 判断章节正文去掉标签和空白后是否什么都没有
///
/// 纯文本只看空白。HTML 先去掉注释、脚本和样式，再去掉标签。图片、音视频、SVG、对象和框架算有内容。
///
/// # 参数
/// - `content`: 章节正文。TXT 为纯文本，EPUB 和 MOBI 为 HTML
///
/// # 返回值
/// 没有可见文字、也没有媒体时返回 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(chapter_is_blank(" \n"));
/// assert!(!chapter_is_blank("有字"));
/// assert!(chapter_is_blank("<p>&nbsp;</p>"));
/// assert!(!chapter_is_blank("<p><img src=\"a.png\"></p>"));
/// ```
pub(crate) fn chapter_is_blank(content: &str) -> bool {
  if contains_media(content) {
    return false;
  }
  if !content.contains('<') && !content.contains('&') {
    return content.chars().all(is_space);
  }
  let clean = cleaners();
  let without_comment = clean.comment.replace_all(content, "");
  let without_script = clean.script.replace_all(&without_comment, "");
  let without_style = clean.style.replace_all(&without_script, "");
  let without_tag = clean.tag.replace_all(&without_style, "");
  let text = clean.space_entity.replace_all(&without_tag, " ");
  text.chars().all(is_space)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn blank_text_and_empty_html() {
    assert!(chapter_is_blank(""));
    assert!(chapter_is_blank(" \n\t"));
    assert!(chapter_is_blank("<p></p>"));
    assert!(chapter_is_blank("<body><br/></body>"));
    assert!(chapter_is_blank("<p>&nbsp;</p>"));
    assert!(chapter_is_blank("<style>body{color:red}</style>"));
  }

  #[test]
  fn keeps_text_and_media() {
    assert!(!chapter_is_blank("有字"));
    assert!(!chapter_is_blank("<p>有字</p>"));
    assert!(!chapter_is_blank("<p><img src=\"a.png\"></p>"));
    assert!(!chapter_is_blank("<svg><path d=\"M0 0\"/></svg>"));
  }
}
