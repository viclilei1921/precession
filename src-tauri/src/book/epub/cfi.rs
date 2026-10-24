//! EPUB CFI 解析与序列化（参考 foliate-js/epubcfi.js）
//!
//! 前端阅读进度暂未接线；保留供高亮/定位扩展使用。

#![allow(dead_code)]

use super::error::EpubError;

/// CFI step
///
/// # 字段
/// - `index`: 步号
/// - `id`: 紧跟步号的非空方括号文本，没有则为空
/// - `offset`: 字符偏移。步号为奇数时才写进路径
/// - `temporal`: 时间偏移，没有则为空
/// - `spatial`: 空间坐标
/// - `text`: 其余方括号文本
/// - `side`: 方括号参数 `s`，没有则为空
///
/// # 示例
/// 见 [`part_to_string`]
#[derive(Debug, Clone, PartialEq)]
pub struct CfiPart {
  /// 步号
  pub index: u32,
  /// 紧跟步号的非空方括号文本，没有则为空
  pub id: Option<String>,
  /// 字符偏移。步号为奇数时才写进路径
  pub offset: Option<u32>,
  /// 时间偏移，没有则为空
  pub temporal: Option<f64>,
  /// 空间坐标
  pub spatial: Vec<f64>,
  /// 其余方括号文本
  pub text: Vec<String>,
  /// 方括号参数 `s`，没有则为空
  pub side: Option<String>,
}

impl CfiPart {
  /// 构造只带步号的 CFI 片段
  ///
  /// # 参数
  /// - `index`: 步号
  ///
  /// # 返回值
  /// 其余字段为空的 `CfiPart`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let part = CfiPart::new(6);
  /// assert_eq!(part.index, 6);
  /// assert!(part.id.is_none());
  /// assert!(part.offset.is_none());
  /// ```
  pub(crate) fn new(index: u32) -> Self {
    Self { index, id: None, offset: None, temporal: None, spatial: Vec::new(), text: Vec::new(), side: None }
  }
}

/// 解析后的 CFI，可为普通 CFI 或 range CFI
///
/// # 变体
/// - `Path`: 没有逗号的路径，按 `!` 分成多层 [`CfiPart`]
/// - `Range`: 逗号必须正好分成三段：父路径、起点和终点
///
/// # 示例
/// 见 [`parse_cfi`]
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedCfi {
  /// 没有逗号的路径，按 `!` 分成多层 [`CfiPart`]
  Path(Vec<Vec<CfiPart>>),
  /// 逗号必须正好分成三段：父路径、起点和终点
  Range {
    /// 父路径
    parent: Vec<Vec<CfiPart>>,
    /// 起点
    start: Vec<Vec<CfiPart>>,
    /// 终点
    end: Vec<Vec<CfiPart>>,
  },
}

/// Calibre 高亮结构（对应 fromCalibreHighlight 输入）
///
/// 起点和终点若以 `/4` 开头会去掉这段，再接到 spine 前缀后面。
///
/// # 字段
/// - `spine_index`: spine 下标，用来生成 CFI 前缀
/// - `start_cfi`: 起点 CFI
/// - `end_cfi`: 终点 CFI
///
/// # 示例
/// 见 [`from_calibre_highlight`]
pub struct CalibreHighlight {
  /// spine 下标，用来生成 CFI 前缀
  pub spine_index: usize,
  /// 起点 CFI
  pub start_cfi: String,
  /// 终点 CFI
  pub end_cfi: String,
}

/// CFI 分词结果
///
/// # 变体
/// - `Step`: `/` 后的步号
/// - `Offset`: `:` 后的字符偏移
/// - `Temporal`: `~` 后的时间
/// - `Spatial`: `@` 后的一个空间坐标，后续 `:` 再产生一个
/// - `BracketText`: 方括号里按逗号拆出的文本
/// - `Side`: 方括号参数 `s`
/// - `Indirection`: `!`
/// - `RangeSep`: `,`
///
/// # 示例
/// 见 [`tokenize`]
#[derive(Debug, Clone, PartialEq)]
enum CfiToken {
  /// `/` 后的步号
  Step(u32),
  /// `:` 后的字符偏移
  Offset(u32),
  /// `~` 后的时间
  Temporal(f64),
  /// `@` 后的一个空间坐标，后续 `:` 再产生一个
  Spatial(f64),
  /// 方括号里按逗号拆出的文本
  BracketText(String),
  /// 方括号参数 `s`
  Side(String),
  /// `!`
  Indirection,
  /// `,`
  RangeSep,
}

/// 判断文本是否包在 `epubcfi()` 里
///
/// 去掉首尾空白后，检查是否以 `epubcfi(` 开头并以 `)` 结尾。
///
/// # 参数
/// - `s`: 待判断的文本
///
/// # 返回值
/// 符合该外形时返回 `true`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert!(is_cfi("epubcfi(/6/2)"));
/// assert!(!is_cfi("/6/2"));
/// ```
pub fn is_cfi(s: &str) -> bool {
  let s = s.trim();
  s.starts_with("epubcfi(") && s.ends_with(')')
}

/// 用 `!` 连接多段 CFI 内部路径
///
/// 每段先去掉 `epubcfi()` 外壳，再用 `!` 拼接，最后重新包上外壳。
///
/// # 参数
/// - `parts`: 要连接的 CFI 片段
///
/// # 返回值
/// 包在 `epubcfi()` 里的连接结果
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(join_indir(&["/6", "/2"]), "epubcfi(/6!/2)");
/// ```
pub fn join_indir(parts: &[&str]) -> String {
  let joined = parts.iter().map(|x| unwrap_cfi(x)).collect::<Vec<_>>().join("!");
  wrap_cfi(&joined)
}

/// 解析 CFI 字符串
///
/// 先去掉 `epubcfi()` 外壳再分词。没有逗号时得到普通路径。有逗号时必须正好分成三段，分别作为 range 的父路径、起点和终点。
///
/// # 参数
/// - `cfi`: CFI 文本，可以带或不带 `epubcfi()` 外壳
///
/// # 返回值
/// 成功返回路径或 range
///
/// # 错误
/// 数字、方括号解析失败，或逗号分段不是三段时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let parsed = parse_cfi("epubcfi(/6/2)").unwrap();
/// assert!(matches!(parsed, ParsedCfi::Path(_)));
/// assert!(parse_cfi("epubcfi(/6,/2)").is_err());
/// ```
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

/// 把解析后的 CFI 序列化成字符串
///
/// 普通路径直接序列化。range 把父路径、起点、终点用逗号连起来。结果包上 `epubcfi()`。
///
/// # 参数
/// - `parsed`: 已解析的 CFI
///
/// # 返回值
/// `epubcfi(...)` 字符串
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let parsed = ParsedCfi::Path(vec![vec![CfiPart::new(6), CfiPart::new(2)]]);
/// assert_eq!(cfi_to_string(&parsed), "epubcfi(/6/2)");
/// ```
pub fn cfi_to_string(parsed: &ParsedCfi) -> String {
  let inner = match parsed {
    ParsedCfi::Path(path) => path_to_inner(path),
    ParsedCfi::Range { parent, start, end } => {
      format!("{},{},{}", path_to_inner(parent), path_to_inner(start), path_to_inner(end))
    }
  };
  wrap_cfi(&inner)
}

/// 把 CFI 折叠成一条路径
///
/// 普通路径原样克隆。range 在 `to_end` 为真时取终点，否则取起点，再接到父路径后面。
///
/// # 参数
/// - `parsed`: 已解析的 CFI
/// - `to_end`: 为真时取 range 终点
///
/// # 返回值
/// 按间接层分组的路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let parsed = parse_cfi("epubcfi(/6,/2,/4)").unwrap();
/// let start = collapse_cfi(&parsed, false);
/// assert_eq!(cfi_to_string(&ParsedCfi::Path(start)), "epubcfi(/6/2)");
/// ```
pub fn collapse_cfi(parsed: &ParsedCfi, to_end: bool) -> Vec<Vec<CfiPart>> {
  match parsed {
    ParsedCfi::Path(path) => path.clone(),
    ParsedCfi::Range { parent, start, end } => {
      let picked = if to_end { end } else { start };
      concat_paths(parent, picked)
    }
  }
}

/// 用两个 CFI 生成 range CFI
///
/// 起点按折叠后的起始路径，终点按折叠后的结束路径，再序列化。
///
/// # 参数
/// - `from`: 起点 CFI
/// - `to`: 终点 CFI
///
/// # 返回值
/// 成功返回 range CFI 字符串
///
/// # 错误
/// 任一 CFI 解析失败，或路径为空时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(build_range("epubcfi(/6/2)", "epubcfi(/6/4)").unwrap(), "epubcfi(/6,/2,/4)");
/// ```
pub fn build_range(from: &str, to: &str) -> Result<String, EpubError> {
  let from_parsed = parse_cfi(from)?;
  let to_parsed = parse_cfi(to)?;
  let from_path = collapse_cfi(&from_parsed, false);
  let to_path = collapse_cfi(&to_parsed, true);
  let range = build_range_from_paths(&from_path, &to_path)?;
  Ok(cfi_to_string(&range))
}

/// 比较两个 CFI 的先后
///
/// 任一边是 range 时，先比较折叠后的起点；起点相同再把两边都折叠成终点后比较。最后按路径比较步号，并在每层最后一步比较偏移。
///
/// # 参数
/// - `a`: 左边的 CFI
/// - `b`: 右边的 CFI
///
/// # 返回值
/// `a` 在前返回 `-1`，相同返回 `0`，`a` 在后返回 `1`
///
/// # 错误
/// 任一 CFI 解析失败时返回 `Err`
///
/// # Panics
/// 折叠后仍不是路径时 `unreachable!` 会 panic
///
/// # 示例
/// ```
/// assert_eq!(compare_cfi("epubcfi(/6/2)", "epubcfi(/6/4)").unwrap(), -1);
/// assert_eq!(compare_cfi("epubcfi(/6/2)", "epubcfi(/6/2)").unwrap(), 0);
/// ```
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

/// 生成 spine 下标对应的 CFI
///
/// 生成 `/6/` 加上 `(index + 1) * 2`，再包上 `epubcfi()`。
///
/// # 参数
/// - `index`: 从 0 开始的 spine 下标
///
/// # 返回值
/// 对应的 CFI 字符串
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(cfi_from_spine_index(0), "epubcfi(/6/2)");
/// assert_eq!(cfi_from_spine_index(1), "epubcfi(/6/4)");
/// ```
pub fn cfi_from_spine_index(index: usize) -> String {
  wrap_cfi(&format!("/6/{}", (index + 1) * 2))
}

/// 从 CFI 反推 spine 下标
///
/// 解析并折叠到起点后，取最后一层最后一个步号，计算 `步号 / 2 - 1`。
///
/// # 参数
/// - `cfi`: CFI 文本
///
/// # 返回值
/// 解析成功且路径非空时返回下标，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(cfi_to_spine_index("epubcfi(/6/2)"), Some(0));
/// assert_eq!(cfi_to_spine_index("epubcfi(/6/4)"), Some(1));
/// assert_eq!(cfi_to_spine_index("not-cfi"), None);
/// ```
pub fn cfi_to_spine_index(cfi: &str) -> Option<usize> {
  let parsed = parse_cfi(cfi).ok()?;
  let collapsed = collapse_cfi(&parsed, false);
  let part = collapsed.last()?.last()?;
  Some(part.index as usize / 2 - 1)
}

/// 把 Calibre 位置 CFI 改写成带 `/6` 的路径
///
/// 只接受普通路径。取第一层，至少要有两个步。丢掉前两个步，把第一个步接到新建的步号 6 后面，其余步作为下一层。
///
/// # 参数
/// - `pos`: Calibre 位置 CFI
///
/// # 返回值
/// 成功返回改写后的 CFI
///
/// # 错误
/// 是 range、路径为空或第一层不足两步时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(from_calibre_pos("epubcfi(/2/4)").unwrap(), "epubcfi(/6/2!)");
/// ```
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

/// 把 Calibre 高亮转成 range CFI
///
/// 用 spine 下标生成前缀并加上 `!`。起点和终点若以 `/4` 开头则去掉这段，再接上前缀，最后生成 range。
///
/// # 参数
/// - `input`: spine 下标以及起止 CFI
///
/// # 返回值
/// 成功返回 range CFI
///
/// # 错误
/// 生成 range 失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let input = CalibreHighlight {
///   spine_index: 0,
///   start_cfi: "/4/2".to_string(),
///   end_cfi: "/4/4".to_string(),
/// };
/// assert_eq!(from_calibre_highlight(&input).unwrap(), "epubcfi(/6/2!,/2,/4)");
/// ```
pub fn from_calibre_highlight(input: &CalibreHighlight) -> Result<String, EpubError> {
  let pre = format!("{}!", cfi_from_spine_index(input.spine_index));
  let start = format!("{}{}", pre, input.start_cfi.strip_prefix("/4").unwrap_or(&input.start_cfi));
  let end = format!("{}{}", pre, input.end_cfi.strip_prefix("/4").unwrap_or(&input.end_cfi));
  build_range(&start, &end)
}

// ── CFI 内部实现 ──
/// 给内部路径补上 `epubcfi()` 外壳
///
/// 已经是 CFI 时原样返回。
///
/// # 参数
/// - `inner`: 内部路径或完整 CFI
///
/// # 返回值
/// 带外壳的 CFI
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(wrap_cfi("/6/2"), "epubcfi(/6/2)");
/// assert_eq!(wrap_cfi("epubcfi(/6/2)"), "epubcfi(/6/2)");
/// ```
fn wrap_cfi(inner: &str) -> String {
  if is_cfi(inner) { inner.to_string() } else { format!("epubcfi({})", inner) }
}

/// 去掉 CFI 外壳并保留内部路径
///
/// 去掉首尾空白后，若是 `epubcfi(...)` 则去掉前 8 个字符和最后一个字符，否则返回去空白后的原文。
///
/// # 参数
/// - `input`: 可能带外壳的 CFI
///
/// # 返回值
/// 内部路径或原文
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(unwrap_cfi(" epubcfi(/6/2) "), "/6/2");
/// assert_eq!(unwrap_cfi("/6/2"), "/6/2");
/// ```
fn unwrap_cfi(input: &str) -> String {
  let s = input.trim();
  if is_cfi(s) { s[8..s.len() - 1].to_string() } else { s.to_string() }
}

/// 转义 CFI 特殊字符
///
/// 在 `^`、`[`、`]`、`(`、`)`、`,`、`;`、`=` 前面插入 `^`。
///
/// # 参数
/// - `s`: 原始文本
///
/// # 返回值
/// 转义后的文本
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(escape_cfi("a,b"), "a^,b");
/// assert_eq!(escape_cfi("x^y"), "x^^y");
/// ```
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

/// 还原 CFI 转义
///
/// `^` 使下一个字符按字面写入。末尾单独的 `^` 被丢弃。
///
/// # 参数
/// - `s`: 可能含 `^` 转义的文本
///
/// # 返回值
/// 还原后的文本
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(unescape_cfi("a^,b"), "a,b");
/// assert_eq!(unescape_cfi("a^"), "a");
/// ```
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

/// 把 CFI 内部路径切成 token
///
/// `!` 为间接层，`,` 为 range 分隔。`/`、`:` 后读整数，`~`、`@` 后读浮点，`@` 还可用 `:` 继续读空间坐标。`[` 读取方括号断言。`^` 跳过自身和下一个字符。其他字符跳过。
///
/// # 参数
/// - `input`: 去掉外壳后的 CFI
///
/// # 返回值
/// 成功返回 token 列表
///
/// # 错误
/// 整数、浮点或方括号解析失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let tokens = tokenize("/6:1").unwrap();
/// assert_eq!(tokens, vec![CfiToken::Step(6), CfiToken::Offset(1)]);
/// assert!(tokenize("/").is_err());
/// ```
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

/// 从字符切片读取一个无符号整数 token
///
/// 从 `i` 起连续读取 ASCII 数字。没有数字时失败。数字超出 `u32` 时失败。
///
/// # 参数
/// - `chars`: 源字符
/// - `i`: 起始下标
///
/// # 返回值
/// 成功返回数值和新的下标
///
/// # 错误
/// 没有数字或超出 `u32` 时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let chars: Vec<char> = "12a".chars().collect();
/// assert_eq!(parse_u32_token(&chars, 0).unwrap(), (12, 2));
/// ```
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

/// 从字符切片读取一个浮点 token
///
/// 从 `i` 起读取 ASCII 数字和 `.`。没有读到字符时失败。不能解析成 `f64` 时失败。
///
/// # 参数
/// - `chars`: 源字符
/// - `i`: 起始下标
///
/// # 返回值
/// 成功返回数值和新的下标
///
/// # 错误
/// 没有数字或浮点解析失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let chars: Vec<char> = "1.5,".chars().collect();
/// assert_eq!(parse_f64_token(&chars, 0).unwrap(), (1.5, 3));
/// ```
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

/// 读取方括号中的原始内容
///
/// 从 `i` 开始直到未转义的 `]`。`^` 使下一个字符按字面进入结果，这个 `^` 本身也会保留。没有闭合时失败。
///
/// # 参数
/// - `chars`: 源字符
/// - `i`: 左括号后的起始下标
///
/// # 返回值
/// 成功返回括号内原文和 `]` 之后的下标
///
/// # 错误
/// 扫描到结尾仍未遇到 `]` 时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let chars: Vec<char> = "id],".chars().collect();
/// assert_eq!(parse_bracket(&chars, 0).unwrap(), ("id".to_string(), 3));
/// ```
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

/// 解析方括号断言里的文本和 side
///
/// 先按未转义的逗号分段。每段再按未转义的分号切开：第一段还原转义后作为文本，后面形如 `s=` 的键写入 side。
///
/// # 参数
/// - `raw`: 方括号内部的原文
///
/// # 返回值
/// 文本列表，以及可选的 side
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(parse_bracket_value("id;s=left"), (vec!["id".to_string()], Some("left".to_string())));
/// ```
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

/// 按分隔符切分，忽略被 `^` 转义的分隔符
///
/// `^` 使下一个字符不参与判断。分隔符本身不进入结果。
///
/// # 参数
/// - `input`: 源文本
/// - `sep`: 分隔字符
///
/// # 返回值
/// 按出现顺序切出的子串，保留转义符
///
/// # Panics
/// 用字符序号对字符串切片时，非 ASCII 字符会在非字符边界 panic
///
/// # 示例
/// ```
/// assert_eq!(split_unescaped("a,b", ','), vec!["a", "b"]);
/// assert_eq!(split_unescaped("a^,b", ','), vec!["a^,b"]);
/// ```
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

/// 按 `!` token 把路径拆成多层再解析
///
/// # 参数
/// - `tokens`: 一层或多层的 token
///
/// # 返回值
/// 成功返回每一层的 CFI 片段
///
/// # 错误
/// 任一层解析失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let tokens = vec![CfiToken::Step(6), CfiToken::Indirection, CfiToken::Step(2)];
/// let path = parse_indirections(&tokens).unwrap();
/// assert_eq!(path.len(), 2);
/// assert_eq!(path[1][0].index, 2);
/// ```
fn parse_indirections(tokens: &[CfiToken]) -> Result<Vec<Vec<CfiPart>>, EpubError> {
  let splits = split_at(tokens, &find_token_indices(tokens, |x| matches!(x, CfiToken::Indirection)));
  splits.into_iter().map(|x| parse_parts(&x)).collect()
}

/// 把一层 token 解析成 CFI 片段
///
/// 步号新建片段。偏移、时间、空间、side 写到上一个片段；前面没有步时失败。紧跟步号的非空方括号文本写入 `id`，其余写入 `text`。`!` 和逗号 token 被忽略。
///
/// # 参数
/// - `tokens`: 同一层的 token
///
/// # 返回值
/// 成功返回该层片段
///
/// # 错误
/// 偏移、时间、空间、方括号或 side 前面没有步时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let parts = parse_parts(&[CfiToken::Step(6), CfiToken::Offset(3)]).unwrap();
/// assert_eq!(parts.len(), 1);
/// assert_eq!(parts[0].index, 6);
/// assert_eq!(parts[0].offset, Some(3));
/// ```
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

/// 把一个 CFI 片段序列化成内部路径
///
/// 先写 `/步号`。有 id 时写成 `[id]`，有 side 时在同一括号里追加 `;s=`。步号为奇数且有偏移时写 `:偏移`。有时间写 `~`，有空间坐标写 `@` 并用 `:` 连接。有文本，或没有 id 但有 side 时，再写一对方括号。
///
/// # 参数
/// - `part`: 要序列化的片段
///
/// # 返回值
/// 该片段的路径文本
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(part_to_string(&CfiPart::new(6)), "/6");
/// let mut part = CfiPart::new(1);
/// part.offset = Some(4);
/// assert_eq!(part_to_string(&part), "/1:4");
/// ```
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

/// 把多层路径序列化并用 `!` 连接
///
/// # 参数
/// - `path`: 按间接层分组的片段
///
/// # 返回值
/// 内部路径，不含 `epubcfi()` 外壳
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let path = vec![vec![CfiPart::new(6)], vec![CfiPart::new(2)]];
/// assert_eq!(path_to_inner(&path), "/6!/2");
/// ```
fn path_to_inner(path: &[Vec<CfiPart>]) -> String {
  path.iter().map(|parts| parts.iter().map(part_to_string).collect::<String>()).collect::<Vec<_>>().join("!")
}

/// 把两段路径接成一条
///
/// 任一边为空时返回另一边的副本。否则保留 `a` 除最后一层外的前缀，把 `a` 的最后一层和 `b` 的第一层拼在一起，再接上 `b` 的其余层。
///
/// # 参数
/// - `a`: 前一段路径
/// - `b`: 后一段路径
///
/// # 返回值
/// 拼接后的路径
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let merged = concat_paths(&[vec![CfiPart::new(6)]], &[vec![CfiPart::new(2)]]);
/// assert_eq!(path_to_inner(&merged), "/6/2");
/// ```
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

/// 从两条局部路径构造 range
///
/// 只使用两边的最后一层。步号相同且两边都没有偏移时放入父路径，一旦不再满足就把后续片段分别放入起点和终点。父路径再接上 `from` 去掉最后一层后的前缀。
///
/// # 参数
/// - `from`: 起点路径
/// - `to`: 终点路径
///
/// # 返回值
/// 成功返回 range
///
/// # 错误
/// 任一边没有最后一层时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let from = vec![vec![CfiPart::new(6), CfiPart::new(2)]];
/// let to = vec![vec![CfiPart::new(6), CfiPart::new(4)]];
/// let range = build_range_from_paths(&from, &to).unwrap();
/// assert_eq!(cfi_to_string(&range), "epubcfi(/6,/2,/4)");
/// ```
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

/// 按层比较两条 CFI 路径
///
/// 某一侧缺少对应层或步时，缺少的一侧更小。步号不同按步号决定。一层的最后一步再比较偏移，缺省偏移按 0。
///
/// # 参数
/// - `a`: 左边的路径
/// - `b`: 右边的路径
///
/// # 返回值
/// `a` 在前返回 `-1`，相同返回 `0`，`a` 在后返回 `1`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let a = vec![vec![CfiPart::new(6), CfiPart::new(2)]];
/// let b = vec![vec![CfiPart::new(6), CfiPart::new(4)]];
/// assert_eq!(compare_paths(&a, &b), -1);
/// assert_eq!(compare_paths(&a, &a), 0);
/// ```
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

/// 收集谓词命中的 token 下标
///
/// # 参数
/// - `tokens`: token 列表
/// - `f`: 判断某个 token 是否命中
///
/// # 返回值
/// 命中位置的下标
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let tokens = [CfiToken::Step(6), CfiToken::RangeSep];
/// let idx = find_token_indices(&tokens, |token| matches!(token, CfiToken::RangeSep));
/// assert_eq!(idx, vec![1]);
/// ```
fn find_token_indices<F>(tokens: &[CfiToken], f: F) -> Vec<usize>
where
  F: Fn(&CfiToken) -> bool,
{
  tokens.iter().enumerate().filter_map(|(i, t)| if f(t) { Some(i) } else { None }).collect()
}

/// 按下标把切片切成多段
///
/// 下标处的元素本身不进入任何一段。下标大于当前起点时放入中间那段，否则放入空段。循环结束后若起点仍在切片内，再放入剩余元素。
///
/// # 参数
/// - `items`: 待切分的切片
/// - `indices`: 作为分隔位置的下标
///
/// # 返回值
/// 按顺序切出的各段
///
/// # Panics
/// `indices` 中的下标大于 `items` 长度时，切片会 panic
///
/// # 示例
/// ```
/// assert_eq!(split_at(&[10, 20, 30], &[1]), vec![vec![10], vec![30]]);
/// ```
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
