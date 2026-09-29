//! MOBI HTML 清理与正文提取

use regex::Regex;

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

pub(crate) fn contains_renderable_media(html: &str) -> bool {
  Regex::new(r"(?i)<(img|svg|video|audio|source|image)\b").unwrap().is_match(html)
}

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
