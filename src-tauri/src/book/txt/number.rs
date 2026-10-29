//! 中文 / 阿拉伯

/// 解析阿拉伯数字或中文数字
///
/// 先去掉首尾空白。空串返回 `None`。能按十进制 `u32` 解析时直接返回，否则交给中文数字解析。
///
/// # 参数
/// - `s`: 数字文本
///
/// # 返回值
/// 解析成功返回整数，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(parse_chinese_or_arabic("12"), Some(12));
/// assert_eq!(parse_chinese_or_arabic("十二"), Some(12));
/// assert_eq!(parse_chinese_or_arabic(""), None);
/// ```
pub(super) fn parse_chinese_or_arabic(s: &str) -> Option<u32> {
  let s = s.trim();
  if s.is_empty() {
    return None;
  }
  if let Ok(n) = s.parse::<u32>() {
    return Some(n);
  }
  parse_chinese_num(s)
}

/// 把中文数字字符串解析成整数
///
/// 纯阿拉伯数字直接按 `u32` 解析。中文逐字累计：零到九写入当前数，十、百、千写入节，万、亿进位到总计。没有任何有效数字字符时返回 `None`。末尾余数在上一单位大于 10 且不超过 1000、并且原文不含「零」时，按上一单位的十分之一补位。
///
/// # 参数
/// - `s`: 中文或阿拉伯数字文本
///
/// # 返回值
/// 解析成功返回整数，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(parse_chinese_num("十"), Some(10));
/// assert_eq!(parse_chinese_num("二十三"), Some(23));
/// assert_eq!(parse_chinese_num("一百零一"), Some(101));
/// assert_eq!(parse_chinese_num("abc"), None);
/// ```
pub(super) fn parse_chinese_num(s: &str) -> Option<u32> {
  // 如果已经是纯阿拉伯数字，直接解析并返回 Some
  if let Ok(num) = s.parse::<u32>() {
    return Some(num);
  }

  let mut total = 0;
  let mut section = 0;
  let mut number = 0;
  let mut last_unit = 1;
  let mut has_valid_char = false; // 记录是否匹配到了有效的中文数字字符

  let chars: Vec<char> = s.chars().collect();

  for &c in &chars {
    match c {
      '零' => {
        has_valid_char = true;
      }
      '一' => {
        number = 1;
        has_valid_char = true;
      }
      '二' | '两' => {
        number = 2;
        has_valid_char = true;
      }
      '三' => {
        number = 3;
        has_valid_char = true;
      }
      '四' => {
        number = 4;
        has_valid_char = true;
      }
      '五' => {
        number = 5;
        has_valid_char = true;
      }
      '六' => {
        number = 6;
        has_valid_char = true;
      }
      '七' => {
        number = 7;
        has_valid_char = true;
      }
      '八' => {
        number = 8;
        has_valid_char = true;
      }
      '九' => {
        number = 9;
        has_valid_char = true;
      }
      '十' => {
        if number == 0 {
          number = 1;
        }
        section += number * 10;
        number = 0;
        last_unit = 10;
        has_valid_char = true;
      }
      '百' => {
        section += number * 100;
        number = 0;
        last_unit = 100;
        has_valid_char = true;
      }
      '千' => {
        section += number * 1000;
        number = 0;
        last_unit = 1000;
        has_valid_char = true;
      }
      '万' => {
        section += number;
        total += section * 10_000;
        section = 0;
        number = 0;
        last_unit = 10_000;
        has_valid_char = true;
      }
      '亿' => {
        section += number;
        total += section * 100_000_000;
        section = 0;
        number = 0;
        last_unit = 100_000_000;
        has_valid_char = true;
      }
      _ => {}
    }
  }

  // 如果遍历完发现一个有效数字都没遇到，返回 None
  if !has_valid_char {
    return None;
  }

  if number > 0 {
    if last_unit > 10 && last_unit <= 1000 && !chars.contains(&'零') {
      section += number * (last_unit / 10);
    } else {
      section += number;
    }
  }

  Some(total + section)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_chinese_numbers() {
    assert_eq!(parse_chinese_num("一"), Some(1));
    assert_eq!(parse_chinese_num("十"), Some(10));
    assert_eq!(parse_chinese_num("十二"), Some(12));
    assert_eq!(parse_chinese_num("二十"), Some(20));
    assert_eq!(parse_chinese_num("二十三"), Some(23));
    assert_eq!(parse_chinese_num("一百零一"), Some(101));
  }
}
