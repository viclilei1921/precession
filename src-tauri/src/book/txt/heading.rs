//! 章节标题分类（强 / 特殊 / 软）与行内标题探测

use std::sync::OnceLock;

use regex::Regex;

use super::number::parse_chinese_or_arabic;

/// 定义解析后的结果类型
///
/// [`format_line`] 把章节写成 `第{num}章: {title}`，卷写成 `卷{num}: {title}`。特殊章节在没有数字也没有标题时只留下类型名。
///
/// # 变体
/// - `Chapter`: 章节号和标题
/// - `Volume`: 卷号和标题
/// - `Special`: 特殊章节。`kind` 为番外、附录、后记、序言、序章、序、楔子、前言、终章、引子之一
///
/// # 示例
/// 见 [`format_line`]
#[derive(Debug, PartialEq)]
pub enum NovelLine {
  /// 章节号和标题
  Chapter(u32, String),
  /// 卷号和标题
  Volume(u32, String),
  /// 特殊章节。`kind` 为番外、附录、后记、序言、序章、序、楔子、前言、终章、引子之一
  Special {
    /// 类型名
    kind: String,
    /// 可选编号
    num: Option<u32>,
    /// 类型名前后拼出的标题
    title: String,
  },
}

/// 把解析后的行格式化成标题文本
///
/// 章节写成 `第{num}章: {title}`，卷写成 `卷{num}: {title}`。特殊章节在没有数字也没有标题时只返回类型名，否则拼成 `{kind}{num}: {title}`，标题为空时不写冒号。
///
/// # 参数
/// - `line`: 已分类的章节、卷或特殊行
///
/// # 返回值
/// 规范化后的标题
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(format_line(&NovelLine::Chapter(1, "开头".to_string())), "第1章: 开头");
/// assert_eq!(format_line(&NovelLine::Volume(2, "上".to_string())), "卷2: 上");
/// assert_eq!(
///   format_line(&NovelLine::Special { kind: "后记".to_string(), num: None, title: String::new() }),
///   "后记"
/// );
/// ```
pub fn format_line(line: &NovelLine) -> String {
  match line {
    NovelLine::Chapter(num, title) => format!("第{}章: {}", num, title),
    NovelLine::Volume(num, title) => format!("卷{}: {}", num, title),
    NovelLine::Special { kind, num, title } => {
      // 组装特殊章节的输出格式
      let num_str = match num {
        Some(n) => n.to_string(),
        None => String::new(),
      };

      // 如果既没有数字也没有标题（比如只写了“后记”两个字），直接返回类型名
      if num_str.is_empty() && title.is_empty() {
        kind.clone()
      } else {
        let sep = if title.is_empty() { "" } else { ": " };
        format!("{}{}{}{}", kind, num_str, sep, title)
      }
    }
  }
}

/// 识别一行是否为章节标题并返回规范化文本
///
/// 去掉首尾空白后，空行或超过 32 个字符返回 `None`。依次尝试章节、卷、特殊章节三类正则；数字能解析时用 `format_line` 输出。特殊章节会把关键字前后的文字合成标题。
///
/// # 参数
/// - `line`: 待检查的一行
///
/// # 返回值
/// 是标题时返回规范化文本，否则返回 `None`
///
/// # Panics
/// 正则编译失败或捕获组缺失时 `unwrap` 会 panic
///
/// # 示例
/// ```
/// assert_eq!(extract_chapter_title("第1章: 开头").as_deref(), Some("第1章: 开头"));
/// assert_eq!(extract_chapter_title("后记").as_deref(), Some("后记"));
/// assert_eq!(extract_chapter_title("普通正文"), None);
/// ```
pub(super) fn extract_chapter_title(line: &str) -> Option<String> {
  let t = line.trim();

  if t.is_empty() || t.chars().count() > 32 {
    return None;
  }

  // 1. 匹配常规章节 (升级版)
  // 兼容: 第xxx章 标题 | xxx、标题 | xxx：标题 | xxx: 标题
  static CHAPTER_RE: OnceLock<Regex> = OnceLock::new();
  let chapter_re = CHAPTER_RE.get_or_init(|| {
    // (?:第\s*)? 允许省略“第”
    // (?:章|[、：:]) 允许分隔符为“章”、“、”、“：”或半角“:”
    // 章后可再跟冒号（如「第1章: 开头」）；也可用顿号/冒号作分隔（如「二、农家」）
    Regex::new(r"^\s*(?:第\s*)?([零一二两三四五六七八九十百千万亿0-9]+)\s*(?:章\s*[：:]?|[、：:])\s*(.*)$").unwrap()
  });

  // 2. 匹配卷 (保持不变)
  static VOLUME_RE: OnceLock<Regex> = OnceLock::new();
  let volume_re = VOLUME_RE.get_or_init(|| {
      Regex::new(r"^\s*(?:第\s*)?([零一二两三四五六七八九十百千万亿0-9]+)\s*卷\s*(.*)$|^\s*卷\s*([零一二两三四五六七八九十百千万亿0-9]+)\s*(.*)$").unwrap()
  });

  // 3. 匹配特殊章节 (升级版)
  // 增加了对前缀的捕获: ([^，。！？]{0,20}?) 允许前面有角色名/卷名，但排除了常见句子标点防误判
  static SPECIAL_RE: OnceLock<Regex> = OnceLock::new();
  let special_re = SPECIAL_RE.get_or_init(|| {
      Regex::new(r"^\s*([^，。！？]{0,20}?)(番外|附录|后记|序言|序章|序|楔子|前言|终章|引子)(?:\s*第?\s*([零一二两三四五六七八九十百千万亿0-9]+)\s*[章篇]?)?\s*(.*)$").unwrap()
  });

  // 优先匹配常规章节
  if let Some(caps) = chapter_re.captures(line) {
    if let Some(num) = parse_chinese_or_arabic(caps.get(1).unwrap().as_str()) {
      let title = caps.get(2).unwrap().as_str().trim().to_string();
      return Some(format_line(&NovelLine::Chapter(num, title)));
    }
  }

  // 匹配卷
  if let Some(caps) = volume_re.captures(line) {
    let num_str = caps.get(1).or_else(|| caps.get(3)).unwrap().as_str();
    if let Some(num) = parse_chinese_or_arabic(num_str) {
      let title = caps.get(2).or_else(|| caps.get(4)).unwrap().as_str().trim().to_string();
      return Some(format_line(&NovelLine::Volume(num, title)));
    }
  }

  // 匹配特殊章节
  if let Some(caps) = special_re.captures(line) {
    let prefix = caps.get(1).unwrap().as_str().trim();
    let kind = caps.get(2).unwrap().as_str().to_string();
    let num = caps.get(3).and_then(|m| parse_chinese_or_arabic(m.as_str()));
    let suffix = caps.get(4).unwrap().as_str().trim();

    // 【拼接标题】处理像“仁康皇帝番外一”的情况
    // 如果前后都有文字，拼在一起；如果只有一边有，取有的那边。
    let title = if prefix.is_empty() {
      suffix.to_string()
    } else if suffix.is_empty() {
      prefix.to_string()
    } else {
      format!("{} {}", prefix, suffix)
    };

    return Some(format_line(&NovelLine::Special { kind, num, title }));
  }

  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extracts_chapter_title() {
    let cases = [
      ("第1章: 开头", Some("第1章: 开头")),
      ("第一百八章 决战之巅", Some("第180章: 决战之巅")),
      ("卷一百零八 归去来兮", Some("卷108: 归去来兮")),
      ("  楔子 陨落的神明  ", Some("楔子: 陨落的神明")),
      ("番外一 剑神之子", Some("番外1: 剑神之子")),
      ("番外 第13篇 往事", Some("番外13: 往事")),
      ("后记", Some("后记")),
      ("二、农家生活", Some("第2章: 农家生活")),
      ("一百八、虚线", Some("第180章: 虚线")),
      ("仁康皇帝番外一", Some("番外1: 仁康皇帝")),
      ("两百二十三：暴怒", Some("第223章: 暴怒")),
    ];
    for (line, expected) in cases {
      assert_eq!(extract_chapter_title(line).as_deref(), expected, "input: {line:?}");
    }
  }
}
