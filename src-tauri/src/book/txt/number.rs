//! 中文 / 阿拉伯

/// 解析常见中文数字 / 阿拉伯数字（约 1–9999）
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
