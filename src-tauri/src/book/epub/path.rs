//! EPUB 路径与 URI 工具

use regex::Regex;

/// 判断字符串是否为外部 URI
///
/// 先转成小写。以 `blob:` 开头返回 `false`。其余用正则检查是否以 `字母开头的 scheme:` 开头。
///
/// # 参数
/// - `uri`: 待判断的地址
///
/// # 返回值
/// 像外部 scheme URI 时返回 `true`
///
/// # Panics
/// 正则编译失败时 `expect` 会 panic
///
/// # 示例
/// ```
/// assert!(is_external_uri("https://example.com/a"));
/// assert!(!is_external_uri("blob:abc"));
/// assert!(!is_external_uri("chapter.xhtml"));
/// ```
pub fn is_external_uri(uri: &str) -> bool {
  let lower = uri.to_ascii_lowercase();
  if lower.starts_with("blob:") {
    return false;
  }
  let re = Regex::new(r"^[a-z][a-z0-9+\-.]*:").expect("valid regex");
  re.is_match(&lower)
}

/// 计算从一个路径到另一个路径的相对路径
///
/// `from` 为空时直接返回 `to`。两边去掉末尾 `/` 后按 `/` 分段，找到第一处不同的下标，前面补对应层数的 `..`，再接上目标剩余段。完全相同则返回空字符串。
///
/// # 参数
/// - `from`: 起点路径
/// - `to`: 目标路径
///
/// # 返回值
/// 用 `/` 连接的相对路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(path_relative("OPS/chapter", "OPS/images/a.png"), "../images/a.png");
/// assert_eq!(path_relative("a/b", "a/b"), "");
/// ```
pub fn path_relative(from: &str, to: &str) -> String {
  if from.is_empty() {
    return to.to_string();
  }
  let as_vec = from.trim_end_matches('/').split('/').collect::<Vec<_>>();
  let bs_vec = to.trim_end_matches('/').split('/').collect::<Vec<_>>();
  let max = as_vec.len().max(bs_vec.len());
  let mut diff_at = None;
  for i in 0..max {
    if as_vec.get(i) != bs_vec.get(i) {
      diff_at = Some(i);
      break;
    }
  }
  let Some(i) = diff_at else {
    return String::new();
  };
  let mut out = Vec::new();
  out.extend(vec![".."; as_vec.len().saturating_sub(i)]);
  out.extend(bs_vec[i..].iter().copied());
  out.join("/")
}

/// 取路径中最后一个 `/` 之前的目录
///
/// 保留这个 `/`。没有 `/` 时返回空字符串。
///
/// # 参数
/// - `path`: 源路径
///
/// # 返回值
/// 带尾部 `/` 的目录，或空字符串
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(path_dirname("a/b/c.xhtml"), "a/b/");
/// assert_eq!(path_dirname("c.xhtml"), "");
/// ```
pub fn path_dirname(path: &str) -> String {
  match path.rfind('/') {
    Some(i) => path[..=i].to_string(),
    None => String::new(),
  }
}

/// 把时长文本解析成秒
///
/// 去掉首尾空白。空串返回 `None`。三段 `时:分:秒` 或两段 `分:秒` 按数字相加。否则匹配数值加可选单位：`h` 乘 3600，`min` 乘 60，`ms` 乘 0.001，缺省或其他单位按秒。
///
/// # 参数
/// - `input`: 时长文本
///
/// # 返回值
/// 解析成功返回秒数，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(parse_clock("1:02:03"), Some(3723.0));
/// assert_eq!(parse_clock("1:30"), Some(90.0));
/// assert_eq!(parse_clock("2min"), Some(120.0));
/// assert_eq!(parse_clock(""), None);
/// ```
pub fn parse_clock(input: &str) -> Option<f64> {
  let s = input.trim();
  if s.is_empty() {
    return None;
  }
  let parts = s.split(':').collect::<Vec<_>>();
  if parts.len() == 3 {
    let h = parts[0].parse::<f64>().ok()?;
    let m = parts[1].parse::<f64>().ok()?;
    let sec = parts[2].parse::<f64>().ok()?;
    return Some(h * 3600.0 + m * 60.0 + sec);
  }
  if parts.len() == 2 {
    let m = parts[0].parse::<f64>().ok()?;
    let sec = parts[1].parse::<f64>().ok()?;
    return Some(m * 60.0 + sec);
  }
  let re = Regex::new(r"^([0-9]*\.?[0-9]+)\s*([a-zA-Z]+)?$").ok()?;
  let caps = re.captures(s)?;
  let n = caps.get(1)?.as_str().parse::<f64>().ok()?;
  let unit = caps.get(2).map(|x| x.as_str()).unwrap_or("s");
  let factor = match unit {
    "h" => 3600.0,
    "min" => 60.0,
    "ms" => 0.001,
    _ => 1.0,
  };
  Some(n * factor)
}

/// 把目录链接拆成路径和片段
///
/// 只在第一个 `#` 处切开。没有片段时第二部分为 `None`。
///
/// # 参数
/// - `href`: 目录链接
///
/// # 返回值
/// 路径，以及可选的片段
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(split_toc_href("a.xhtml#id"), ("a.xhtml".to_string(), Some("id".to_string())));
/// assert_eq!(split_toc_href("a.xhtml"), ("a.xhtml".to_string(), None));
/// ```
pub fn split_toc_href(href: &str) -> (String, Option<String>) {
  let mut it = href.splitn(2, '#');
  let path = it.next().unwrap_or_default().to_string();
  let frag = it.next().map(|x| x.to_string());
  (path, frag)
}

/// 按基准路径解析相对地址
///
/// 外部 URI 或空地址原样返回。以 `#` 开头时接到基准路径去掉片段后的部分。以 `/` 开头时去掉开头的 `/`，否则接到基准路径的目录后。然后截掉 `?` 及其后的内容，再做路径规范化。
///
/// # 参数
/// - `url`: 待解析的地址
/// - `relative_to`: 基准路径
///
/// # 返回值
/// 解析后的路径或原外部地址
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(resolve_url("#frag", "dir/ch.xhtml#old"), "dir/ch.xhtml#frag");
/// assert_eq!(resolve_url("img/a.png", "dir/ch.xhtml"), "dir/img/a.png");
/// assert_eq!(resolve_url("https://example.com", "dir/ch.xhtml"), "https://example.com");
/// ```
pub fn resolve_url(url: &str, relative_to: &str) -> String {
  if is_external_uri(url) || url.is_empty() {
    return url.to_string();
  }

  // 片段链接：沿用同一文档路径
  if url.starts_with('#') {
    let base = relative_to.split('#').next().unwrap_or(relative_to);
    return format!("{}{}", base, url);
  }

  let mut path = if url.starts_with('/') {
    url.trim_start_matches('/').to_string()
  } else {
    format!("{}{}", path_dirname(relative_to), url)
  };

  // 去掉 query（对齐 epub.js 在本地路径场景会清空 search）
  if let Some(i) = path.find('?') {
    path.truncate(i);
  }

  normalize_path(&path)
}

/// 规范化路径中的 `.` 和 `..`
///
/// 跳过空段和 `.`。遇到 `..` 时弹出上一段，没有上一段则忽略。原来以 `/` 开头时结果也以 `/` 开头。
///
/// # 参数
/// - `path`: 源路径
///
/// # 返回值
/// 折叠后的路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(normalize_path("a/./b/../c"), "a/c");
/// assert_eq!(normalize_path("/a/../b"), "/b");
/// ```
pub(crate) fn normalize_path(path: &str) -> String {
  let mut out = Vec::<&str>::new();
  let has_leading = path.starts_with('/');
  for seg in path.split('/') {
    if seg.is_empty() || seg == "." {
      continue;
    }
    if seg == ".." {
      if !out.is_empty() {
        out.pop();
      }
      continue;
    }
    out.push(seg);
  }
  let joined = out.join("/");
  if has_leading { format!("/{}", joined) } else { joined }
}
