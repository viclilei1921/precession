//! MOBI HTML 清理与正文提取

use regex::Regex;

/// 取出 HTML 正文
///
/// 能匹配到 `body` 时返回标签内部。否则去掉 `head` 以及 `html` 标签，再去掉首尾空白。
///
/// # 参数
/// - `html`: 原始 HTML
///
/// # 返回值
/// 正文或清理后的剩余文本
///
/// # Panics
/// 正则编译失败时 `unwrap` 会 panic
///
/// # 示例
/// ```
/// assert_eq!(extract_body("<html><body>hi</body></html>"), "hi");
/// assert_eq!(extract_body("<p>hi</p>"), "<p>hi</p>");
/// ```
pub(crate) fn extract_body(html: &str) -> String {
  let body_re = Regex::new(r"(?is)<body[^>]*>(.*)</body>").unwrap();
  if let Some(caps) = body_re.captures(html) {
    return caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
  }
  let head_re = Regex::new(r"(?is)<head[^>]*>.*?</head>").unwrap();
  let html_tag_re = Regex::new(r"(?i)</?html[^>]*>").unwrap();
  let result = head_re.replace_all(html, "");
  let result = html_tag_re.replace_all(&result, "");
  result.trim().to_string()
}

/// 判断 HTML 是否含可渲染媒体标签
///
/// # 参数
/// - `html`: 待检查的 HTML
///
/// # 返回值
/// 出现 `img`、`svg`、`video`、`audio`、`source` 或 `image` 起始标签时返回 `true`
///
/// # Panics
/// 正则编译失败时 `unwrap` 会 panic
///
/// # 示例
/// ```
/// assert!(contains_renderable_media("<img src=\"a\">"));
/// assert!(!contains_renderable_media("<p>hi</p>"));
/// ```
pub(crate) fn contains_renderable_media(html: &str) -> bool {
  Regex::new(r"(?i)<(img|svg|video|audio|source|image)\b").unwrap().is_match(html)
}

/// 去掉没有 src 的 img 标签
///
/// 有 `src=` 的 img 原样保留，其余 img 从结果中删除。
///
/// # 参数
/// - `html`: 原始 HTML
///
/// # 返回值
/// 删除坏图片标签后的 HTML
///
/// # Panics
/// 正则编译失败时 `unwrap` 会 panic
///
/// # 示例
/// ```
/// assert_eq!(strip_broken_img_tags("<img><img src=\"a\">"), "<img src=\"a\">");
/// ```
pub(crate) fn strip_broken_img_tags(html: &str) -> String {
  let img_re = Regex::new(r#"(?is)<img\b[^>]*>\s*(?:</img>)?"#).unwrap();
  let src_re = Regex::new(r#"(?i)\bsrc\s*="#).unwrap();

  let mut result = String::with_capacity(html.len());
  let mut last_end = 0usize;
  for m in img_re.find_iter(html) {
    result.push_str(&html[last_end..m.start()]);
    let tag = m.as_str();
    if src_re.is_match(tag) {
      result.push_str(tag);
    }
    last_end = m.end();
  }
  result.push_str(&html[last_end..]);
  result
}

/// 清理 MOBI HTML 中的非正文标记
///
/// 依次去掉 `mbp:` 标签、`guide`、`reference`、`filepos`、`recindex`、`mediarecindex`、没有 src 的 img、空的 `a`、`script`、`iframe`、`object`、`embed`、`on*` 事件属性和 `javascript:` 链接，最后去掉首尾空白。
///
/// # 参数
/// - `html`: 原始 HTML
///
/// # 返回值
/// 清理后的 HTML
///
/// # Panics
/// 正则编译失败时 `unwrap` 会 panic
///
/// # 示例
/// ```
/// assert_eq!(clean_mobi_html("<p filepos=\"1\">hi</p>"), "<p>hi</p>");
/// assert_eq!(clean_mobi_html("<mbp:pagebreak/>x"), "x");
/// ```
pub(crate) fn clean_mobi_html(html: &str) -> String {
  let mobi_tag_re = Regex::new(r"(?i)</?mbp:[^>]*>").unwrap();
  let result = mobi_tag_re.replace_all(html, "");

  let guide_re = Regex::new(r"(?is)<guide[^>]*>.*?</guide>").unwrap();
  let result = guide_re.replace_all(&result, "");

  let ref_re = Regex::new(r"(?i)<reference[^>]*>").unwrap();
  let result = ref_re.replace_all(&result, "");

  let filepos_re = Regex::new(r#"(?i)\s+filepos\s*=\s*["']?\d+["']?"#).unwrap();
  let result = filepos_re.replace_all(&result, "");

  let recindex_re = Regex::new(r#"(?i)\s+(?:recindex|mediarecindex)\s*=\s*["']?\d+["']?"#).unwrap();
  let result = recindex_re.replace_all(&result, "");
  let result = strip_broken_img_tags(&result);

  let empty_a_re = Regex::new(r#"(?i)<a\s*>\s*</a>"#).unwrap();
  let result = empty_a_re.replace_all(&result, "");

  let script_re = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
  let result = script_re.replace_all(&result, "");

  let iframe_re = Regex::new(r"(?is)<iframe[^>]*>.*?</iframe>").unwrap();
  let result = iframe_re.replace_all(&result, "");

  let object_re = Regex::new(r"(?is)<object[^>]*>.*?</object>").unwrap();
  let result = object_re.replace_all(&result, "");

  let embed_re = Regex::new(r"(?i)<embed[^>]*>").unwrap();
  let result = embed_re.replace_all(&result, "");

  let event_re = Regex::new(r#"(?i)\s+on\w+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)"#).unwrap();
  let result = event_re.replace_all(&result, "");

  let js_href_re = Regex::new(r#"(?i)(href|src)\s*=\s*["']?\s*javascript:[^"'>]*["']?"#).unwrap();
  let result = js_href_re.replace_all(&result, "");

  result.trim().to_string()
}

/// 还原常见 HTML 实体
///
/// 依次替换 `&lt;`、`&gt;`、`&quot;`、`&#39;`、`&apos;`、`&#x27;`、`&nbsp;`，最后替换 `&amp;`。
///
/// # 参数
/// - `s`: 含实体的文本
///
/// # 返回值
/// 替换后的字符串
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(unescape_html("&lt;a&amp;b&gt;"), "<a&b>");
/// assert_eq!(unescape_html("&nbsp;"), "\u{00A0}");
/// ```
pub(crate) fn unescape_html(s: &str) -> String {
  s.replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&quot;", "\"")
    .replace("&#39;", "'")
    .replace("&apos;", "'")
    .replace("&#x27;", "'")
    .replace("&nbsp;", "\u{00A0}")
    .replace("&amp;", "&")
}
