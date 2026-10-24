//! 阅读前清掉章节 HTML 里的脚本和可执行链接。

use std::sync::LazyLock;

use regex::Regex;

static SCRIPT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)<script\b[^>]*>.*?</script>").unwrap());
static SCRIPT_OPEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)<script\b[^>]*>.*").unwrap());
static IFRAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)<iframe\b[^>]*>.*?</iframe>").unwrap());
static OBJECT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)<object\b[^>]*>.*?</object>").unwrap());
static EMBED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<embed\b[^>]*>").unwrap());
static EVENT: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r#"(?i)\s+on\w+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)"#).unwrap());
static JAVASCRIPT: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r#"(?i)(href|src)\s*=\s*["']?\s*javascript:[^"'>]*["']?"#).unwrap());

/// 去掉 `script`、`iframe`、`object`、`embed`、事件属性和 `javascript:` 链接
///
/// # 参数
/// - `html`: 章节 HTML 片段
///
/// # 返回值
/// 清理后的 HTML
///
/// # Panics
/// 正则在首次使用时编译失败会 panic。这些模式是固定的。
///
/// # 示例
/// ```ignore
/// let clean = sanitize_html("<p onclick=\"x\">hi</p><script>bad()</script>");
/// assert!(clean.contains("hi"));
/// ```
pub(crate) fn sanitize_html(html: &str) -> String {
  let result = SCRIPT.replace_all(html, "");
  let result = SCRIPT_OPEN.replace_all(&result, "");
  let result = IFRAME.replace_all(&result, "");
  let result = OBJECT.replace_all(&result, "");
  let result = EMBED.replace_all(&result, "");
  let result = EVENT.replace_all(&result, "");
  let result = JAVASCRIPT.replace_all(&result, "");
  result.trim().to_string()
}

#[cfg(test)]
mod tests {
  use super::sanitize_html;

  #[test]
  fn strips_script_iframe_and_handlers() {
    let raw = r#"<p onclick="go()">句子</p><script>alert(1)</script><iframe src="x"></iframe><a href="javascript:alert(1)">链</a>"#;
    let clean = sanitize_html(raw);
    assert!(clean.contains("句子"));
    assert!(!clean.contains("script"));
    assert!(!clean.contains("iframe"));
    assert!(!clean.contains("onclick"));
    assert!(!clean.contains("javascript:"));
  }
}
