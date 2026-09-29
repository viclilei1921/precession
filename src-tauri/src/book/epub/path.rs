//! EPUB 路径与 URI 工具

use regex::Regex;

/// 判断外部 URI（排除 blob）
pub fn is_external_uri(uri: &str) -> bool {
  let lower = uri.to_ascii_lowercase();
  if lower.starts_with("blob:") {
    return false;
  }
  let re = Regex::new(r"^[a-z][a-z0-9+\-.]*:").expect("valid regex");
  re.is_match(&lower)
}

/// 类似 Node.js path.relative
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

/// 类似 path.dirname，保留尾部 /
pub fn path_dirname(path: &str) -> String {
  match path.rfind('/') {
    Some(i) => path[..=i].to_string(),
    None => String::new(),
  }
}

/// 解析 SMIL/metadata duration 时间（hh:mm:ss, mm:ss, 12.3min/ms/h/s）
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

/// splitTOCHref: `a.xhtml#id` -> `(a.xhtml, Some(id))`
pub fn split_toc_href(href: &str) -> (String, Option<String>) {
  let mut it = href.splitn(2, '#');
  let path = it.next().unwrap_or_default().to_string();
  let frag = it.next().map(|x| x.to_string());
  (path, frag)
}

/// resolveURL 的 Rust 版本（主要用于 EPUB 内相对路径）
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
