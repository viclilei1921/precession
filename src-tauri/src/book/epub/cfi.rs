//! EPUB CFI 解析与序列化（参考 foliate-js/epubcfi.js）
//!
//! 前端阅读进度暂未接线；保留供高亮/定位扩展使用。

use super::error::EpubError;

/// CFI step
#[derive(Debug, Clone, PartialEq)]
pub struct CfiPart {
  pub index: u32,
  pub id: Option<String>,
  pub offset: Option<u32>,
  pub temporal: Option<f64>,
  pub spatial: Vec<f64>,
  pub text: Vec<String>,
  pub side: Option<String>,
}

impl CfiPart {
  pub(crate) fn new(index: u32) -> Self {
    Self { index, id: None, offset: None, temporal: None, spatial: Vec::new(), text: Vec::new(), side: None }
  }
}

/// 解析后的 CFI，可为普通 CFI 或 range CFI
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedCfi {
  Path(Vec<Vec<CfiPart>>),
  Range { parent: Vec<Vec<CfiPart>>, start: Vec<Vec<CfiPart>>, end: Vec<Vec<CfiPart>> },
}

/// Calibre 高亮结构（对应 fromCalibreHighlight 输入）
pub struct CalibreHighlight {
  pub spine_index: usize,
  pub start_cfi: String,
  pub end_cfi: String,
}

#[derive(Debug, Clone, PartialEq)]
enum CfiToken {
  Step(u32),
  Offset(u32),
  Temporal(f64),
  Spatial(f64),
  BracketText(String),
  Side(String),
  Indirection,
  RangeSep,
}

/// 判断是否为 epubcfi(...) 格式
pub fn is_cfi(s: &str) -> bool {
  let s = s.trim();
  s.starts_with("epubcfi(") && s.ends_with(')')
}

/// joinIndir: 将多个 CFI 内部路径用 ! 连接
pub fn join_indir(parts: &[&str]) -> String {
  let joined = parts.iter().map(|x| unwrap_cfi(x)).collect::<Vec<_>>().join("!");
  wrap_cfi(&joined)
}

/// parse: 解析 CFI 字符串
pub fn parse_cfi(cfi: &str) -> Result<ParsedCfi, EpubError> {
  let tokens = tokenize(&unwrap_cfi(cfi))?;
  let commas = find_token_indices(&tokens, |x| matches!(x, CfiToken::RangeSep));

  if commas.is_empty() {
    let path = parse_indirections(&tokens)?;
    return Ok(ParsedCfi::Path(path));
  }

  let split = split_at(&tokens, &commas);
  if split.len() != 3 {
    return Err(EpubError("无效 range CFI：逗号分段数量错误".to_string()));
  }
  Ok(ParsedCfi::Range {
    parent: parse_indirections(&split[0])?,
    start: parse_indirections(&split[1])?,
    end: parse_indirections(&split[2])?,
  })
}

/// toString: 将 ParsedCfi 序列化回字符串
pub fn cfi_to_string(parsed: &ParsedCfi) -> String {
  let inner = match parsed {
    ParsedCfi::Path(path) => path_to_inner(path),
    ParsedCfi::Range { parent, start, end } => {
      format!("{},{},{}", path_to_inner(parent), path_to_inner(start), path_to_inner(end))
    }
  };
  wrap_cfi(&inner)
}

/// collapse: range CFI 折叠为起点或终点路径
pub fn collapse_cfi(parsed: &ParsedCfi, to_end: bool) -> Vec<Vec<CfiPart>> {
  match parsed {
    ParsedCfi::Path(path) => path.clone(),
    ParsedCfi::Range { parent, start, end } => {
      let picked = if to_end { end } else { start };
      concat_paths(parent, picked)
    }
  }
}

/// buildRange: 从两个 CFI 生成 range CFI
pub fn build_range(from: &str, to: &str) -> Result<String, EpubError> {
  let from_parsed = parse_cfi(from)?;
  let to_parsed = parse_cfi(to)?;
  let from_path = collapse_cfi(&from_parsed, false);
  let to_path = collapse_cfi(&to_parsed, true);
  let range = build_range_from_paths(&from_path, &to_path)?;
  Ok(cfi_to_string(&range))
}

/// compare: 比较两个 CFI，返回 -1/0/1
pub fn compare_cfi(a: &str, b: &str) -> Result<i32, EpubError> {
  let mut pa = parse_cfi(a)?;
  let mut pb = parse_cfi(b)?;
  if matches!(pa, ParsedCfi::Range { .. }) || matches!(pb, ParsedCfi::Range { .. }) {
    let c1 = compare_paths(&collapse_cfi(&pa, false), &collapse_cfi(&pb, false));
    if c1 != 0 {
      return Ok(c1);
    }
    pa = ParsedCfi::Path(collapse_cfi(&pa, true));
    pb = ParsedCfi::Path(collapse_cfi(&pb, true));
  }
  Ok(compare_paths(
    match &pa {
      ParsedCfi::Path(p) => p,
      _ => unreachable!(),
    },
    match &pb {
      ParsedCfi::Path(p) => p,
      _ => unreachable!(),
    },
  ))
}

/// fake.fromIndex: 由 spine index 生成标准 CFI
pub fn cfi_from_spine_index(index: usize) -> String {
  wrap_cfi(&format!("/6/{}", (index + 1) * 2))
}

/// fake.toIndex: 从 CFI 解析 spine index
pub fn cfi_to_spine_index(cfi: &str) -> Option<usize> {
  let parsed = parse_cfi(cfi).ok()?;
  let collapsed = collapse_cfi(&parsed, false);
  let part = collapsed.last()?.last()?;
  Some(part.index as usize / 2 - 1)
}

/// fromCalibrePos
pub fn from_calibre_pos(pos: &str) -> Result<String, EpubError> {
  let parsed = parse_cfi(pos)?;
  let mut path = match parsed {
    ParsedCfi::Path(p) => p,
    ParsedCfi::Range { .. } => {
      return Err(EpubError("Calibre 位置 CFI 不应为 range".to_string()));
    }
  };
  let Some(parts) = path.get_mut(0) else {
    return Err(EpubError("Calibre 位置 CFI 为空".to_string()));
  };
  if parts.len() < 2 {
    return Err(EpubError("Calibre 位置 CFI 结构非法".to_string()));
  }
  let item = parts.remove(0);
  let _ = parts.remove(0);
  let rebuilt = ParsedCfi::Path(vec![vec![CfiPart::new(6), item], parts.clone()]);
  Ok(cfi_to_string(&rebuilt))
}

/// fromCalibreHighlight
pub fn from_calibre_highlight(input: &CalibreHighlight) -> Result<String, EpubError> {
  let pre = format!("{}!", cfi_from_spine_index(input.spine_index));
  let start = format!("{}{}", pre, input.start_cfi.strip_prefix("/4").unwrap_or(&input.start_cfi));
  let end = format!("{}{}", pre, input.end_cfi.strip_prefix("/4").unwrap_or(&input.end_cfi));
  build_range(&start, &end)
}

// ── CFI 内部实现 ──
fn wrap_cfi(inner: &str) -> String {
  if is_cfi(inner) { inner.to_string() } else { format!("epubcfi({})", inner) }
}

fn unwrap_cfi(input: &str) -> String {
  let s = input.trim();
  if is_cfi(s) { s[8..s.len() - 1].to_string() } else { s.to_string() }
}

fn escape_cfi(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  for ch in s.chars() {
    if matches!(ch, '^' | '[' | ']' | '(' | ')' | ',' | ';' | '=') {
      out.push('^');
    }
    out.push(ch);
  }
  out
}

fn unescape_cfi(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let mut esc = false;
  for ch in s.chars() {
    if esc {
      out.push(ch);
      esc = false;
    } else if ch == '^' {
      esc = true;
    } else {
      out.push(ch);
    }
  }
  out
}

fn tokenize(input: &str) -> Result<Vec<CfiToken>, EpubError> {
  let chars = input.chars().collect::<Vec<_>>();
  let mut i = 0usize;
  let mut tokens = Vec::new();

  while i < chars.len() {
    let ch = chars[i];
    match ch {
      '!' => {
        tokens.push(CfiToken::Indirection);
        i += 1;
      }
      ',' => {
        tokens.push(CfiToken::RangeSep);
        i += 1;
      }
      '/' => {
        let (num, next) = parse_u32_token(&chars, i + 1)?;
        tokens.push(CfiToken::Step(num));
        i = next;
      }
      ':' => {
        let (num, next) = parse_u32_token(&chars, i + 1)?;
        tokens.push(CfiToken::Offset(num));
        i = next;
      }
      '~' => {
        let (num, next) = parse_f64_token(&chars, i + 1)?;
        tokens.push(CfiToken::Temporal(num));
        i = next;
      }
      '@' => {
        let (first, mut next) = parse_f64_token(&chars, i + 1)?;
        tokens.push(CfiToken::Spatial(first));
        while next < chars.len() && chars[next] == ':' {
          let (n, n2) = parse_f64_token(&chars, next + 1)?;
          tokens.push(CfiToken::Spatial(n));
          next = n2;
        }
        i = next;
      }
      '[' => {
        let (raw, next) = parse_bracket(&chars, i + 1)?;
        let (values, side) = parse_bracket_value(&raw);
        for v in values {
          tokens.push(CfiToken::BracketText(v));
        }
        if let Some(s) = side {
          tokens.push(CfiToken::Side(s));
        }
        i = next;
      }
      '^' => {
        i += 2;
      }
      _ => {
        i += 1;
      }
    }
  }
  Ok(tokens)
}

fn parse_u32_token(chars: &[char], mut i: usize) -> Result<(u32, usize), EpubError> {
  let start = i;
  while i < chars.len() && chars[i].is_ascii_digit() {
    i += 1;
  }
  if i == start {
    return Err(EpubError("CFI 数字 token 解析失败".to_string()));
  }
  let s = chars[start..i].iter().collect::<String>();
  let n = s.parse::<u32>().map_err(|_| EpubError("CFI 数字超出范围".to_string()))?;
  Ok((n, i))
}

fn parse_f64_token(chars: &[char], mut i: usize) -> Result<(f64, usize), EpubError> {
  let start = i;
  while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
    i += 1;
  }
  if i == start {
    return Err(EpubError("CFI 浮点 token 解析失败".to_string()));
  }
  let s = chars[start..i].iter().collect::<String>();
  let n = s.parse::<f64>().map_err(|_| EpubError("CFI 浮点数解析失败".to_string()))?;
  Ok((n, i))
}

fn parse_bracket(chars: &[char], mut i: usize) -> Result<(String, usize), EpubError> {
  let mut raw = String::new();
  let mut esc = false;
  while i < chars.len() {
    let ch = chars[i];
    if esc {
      raw.push(ch);
      esc = false;
      i += 1;
      continue;
    }
    if ch == '^' {
      raw.push(ch);
      esc = true;
      i += 1;
      continue;
    }
    if ch == ']' {
      return Ok((raw, i + 1));
    }
    raw.push(ch);
    i += 1;
  }
  Err(EpubError("CFI 方括号未闭合".to_string()))
}

fn parse_bracket_value(raw: &str) -> (Vec<String>, Option<String>) {
  let mut values = Vec::new();
  let mut side = None;

  for segment in split_unescaped(raw, ',') {
    let mut text = None;
    for (idx, chunk) in split_unescaped(segment, ';').iter().enumerate() {
      if idx == 0 {
        let v = unescape_cfi(chunk);
        if !v.is_empty() {
          text = Some(v);
        }
        continue;
      }
      let kv = split_unescaped(chunk, '=');
      if kv.len() == 2 && kv[0] == "s" {
        side = Some(unescape_cfi(kv[1]));
      }
    }
    if let Some(t) = text {
      values.push(t);
    }
  }
  (values, side)
}

fn split_unescaped(input: &str, sep: char) -> Vec<&str> {
  let mut out = Vec::new();
  let mut start = 0usize;
  let chars = input.chars().collect::<Vec<_>>();
  let mut esc = false;
  for (i, ch) in chars.iter().enumerate() {
    if esc {
      esc = false;
      continue;
    }
    if *ch == '^' {
      esc = true;
      continue;
    }
    if *ch == sep {
      out.push(&input[start..i]);
      start = i + 1;
    }
  }
  out.push(&input[start..]);
  out
}

fn parse_indirections(tokens: &[CfiToken]) -> Result<Vec<Vec<CfiPart>>, EpubError> {
  let splits = split_at(tokens, &find_token_indices(tokens, |x| matches!(x, CfiToken::Indirection)));
  splits.into_iter().map(|x| parse_parts(&x)).collect()
}

fn parse_parts(tokens: &[CfiToken]) -> Result<Vec<CfiPart>, EpubError> {
  let mut parts = Vec::new();
  let mut prev_was_step = false;

  for token in tokens {
    match token {
      CfiToken::Step(index) => {
        parts.push(CfiPart::new(*index));
        prev_was_step = true;
      }
      CfiToken::Offset(offset) => {
        let Some(last) = parts.last_mut() else {
          return Err(EpubError("CFI offset 缺少 step".to_string()));
        };
        last.offset = Some(*offset);
        prev_was_step = false;
      }
      CfiToken::Temporal(t) => {
        let Some(last) = parts.last_mut() else {
          return Err(EpubError("CFI temporal 缺少 step".to_string()));
        };
        last.temporal = Some(*t);
        prev_was_step = false;
      }
      CfiToken::Spatial(v) => {
        let Some(last) = parts.last_mut() else {
          return Err(EpubError("CFI spatial 缺少 step".to_string()));
        };
        last.spatial.push(*v);
        prev_was_step = false;
      }
      CfiToken::BracketText(text) => {
        let Some(last) = parts.last_mut() else {
          return Err(EpubError("CFI bracket 缺少 step".to_string()));
        };
        if prev_was_step && !text.is_empty() && last.id.is_none() {
          last.id = Some(text.clone());
        } else {
          last.text.push(text.clone());
        }
        prev_was_step = false;
      }
      CfiToken::Side(side) => {
        let Some(last) = parts.last_mut() else {
          return Err(EpubError("CFI side 缺少 step".to_string()));
        };
        last.side = Some(side.clone());
        prev_was_step = false;
      }
      CfiToken::Indirection | CfiToken::RangeSep => {}
    }
  }
  Ok(parts)
}

fn part_to_string(part: &CfiPart) -> String {
  let param = part.side.as_ref().map(|x| format!(";s={}", escape_cfi(x))).unwrap_or_default();
  let mut out = format!("/{}", part.index);
  if let Some(id) = &part.id {
    out.push_str(&format!("[{}{}]", escape_cfi(id), param));
  }
  if let Some(offset) = part.offset
    && part.index % 2 == 1
  {
    out.push_str(&format!(":{}", offset));
  }
  if let Some(t) = part.temporal {
    out.push_str(&format!("~{}", t));
  }
  if !part.spatial.is_empty() {
    out.push('@');
    out.push_str(&part.spatial.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(":"));
  }
  if !part.text.is_empty() || (part.id.is_none() && part.side.is_some()) {
    let text = part.text.iter().map(|x| escape_cfi(x)).collect::<Vec<_>>().join(",");
    out.push_str(&format!("[{}{}]", text, param));
  }
  out
}

fn path_to_inner(path: &[Vec<CfiPart>]) -> String {
  path.iter().map(|parts| parts.iter().map(part_to_string).collect::<String>()).collect::<Vec<_>>().join("!")
}

fn concat_paths(a: &[Vec<CfiPart>], b: &[Vec<CfiPart>]) -> Vec<Vec<CfiPart>> {
  if a.is_empty() {
    return b.to_vec();
  }
  if b.is_empty() {
    return a.to_vec();
  }

  let mut out = Vec::new();
  if a.len() > 1 {
    out.extend_from_slice(&a[..a.len() - 1]);
  }
  let mut merged_last = a[a.len() - 1].clone();
  merged_last.extend_from_slice(&b[0]);
  out.push(merged_last);
  if b.len() > 1 {
    out.extend_from_slice(&b[1..]);
  }
  out
}

fn build_range_from_paths(from: &[Vec<CfiPart>], to: &[Vec<CfiPart>]) -> Result<ParsedCfi, EpubError> {
  let local_from = from.last().ok_or_else(|| EpubError("from CFI 为空".to_string()))?;
  let local_to = to.last().ok_or_else(|| EpubError("to CFI 为空".to_string()))?;

  let mut local_parent = Vec::new();
  let mut local_start = Vec::new();
  let mut local_end = Vec::new();
  let mut push_to_parent = true;
  let len = local_from.len().max(local_to.len());

  for i in 0..len {
    let a = local_from.get(i);
    let b = local_to.get(i);
    if push_to_parent {
      push_to_parent = match (a, b) {
        (Some(x), Some(y)) => x.index == y.index && x.offset.is_none() && y.offset.is_none(),
        _ => false,
      };
    }
    if push_to_parent {
      if let Some(x) = a {
        local_parent.push(x.clone());
      }
    } else {
      if let Some(x) = a {
        local_start.push(x.clone());
      }
      if let Some(y) = b {
        local_end.push(y.clone());
      }
    }
  }

  let mut parent = if from.len() > 1 { from[..from.len() - 1].to_vec() } else { Vec::new() };
  parent.push(local_parent);

  Ok(ParsedCfi::Range { parent, start: vec![local_start], end: vec![local_end] })
}

fn compare_paths(a: &[Vec<CfiPart>], b: &[Vec<CfiPart>]) -> i32 {
  let max_outer = a.len().max(b.len());
  for i in 0..max_outer {
    let pa = a.get(i).cloned().unwrap_or_default();
    let pb = b.get(i).cloned().unwrap_or_default();
    let max_inner = pa.len().max(pb.len());
    for j in 0..max_inner {
      let x = pa.get(j);
      let y = pb.get(j);
      match (x, y) {
        (None, Some(_)) => return -1,
        (Some(_), None) => return 1,
        (Some(xv), Some(yv)) => {
          if xv.index > yv.index {
            return 1;
          }
          if xv.index < yv.index {
            return -1;
          }
          if j == max_inner.saturating_sub(1) {
            let xo = xv.offset.unwrap_or(0);
            let yo = yv.offset.unwrap_or(0);
            if xo > yo {
              return 1;
            }
            if xo < yo {
              return -1;
            }
          }
        }
        (None, None) => {}
      }
    }
  }
  0
}

fn find_token_indices<F>(tokens: &[CfiToken], f: F) -> Vec<usize>
where
  F: Fn(&CfiToken) -> bool,
{
  tokens.iter().enumerate().filter_map(|(i, t)| if f(t) { Some(i) } else { None }).collect()
}

fn split_at<T: Clone>(items: &[T], indices: &[usize]) -> Vec<Vec<T>> {
  let mut out = Vec::new();
  let mut start = 0usize;
  for idx in indices {
    if *idx > start {
      out.push(items[start..*idx].to_vec());
    } else {
      out.push(Vec::new());
    }
    start = *idx + 1;
  }
  if start <= items.len() {
    out.push(items[start..].to_vec());
  }
  out
}
