//! 按行提取元数据并分章

use crate::book::types::{BookMeta, Chapter, Format};

use super::heading::extract_chapter_title;
use super::meta::{extract_author, extract_title};

/// 按章节标题切分；无明确章节时整篇为「正文」一章
pub(super) fn split_txt_book(text: &str, stem: &str) -> (BookMeta, Vec<Chapter>) {
  let lines: Vec<&str> = text.lines().collect();
  let mut meta = BookMeta { title: String::new(), author: String::new(), format: Format::Txt };

  for (i, line) in lines.iter().enumerate() {
    if extract_chapter_title(line).is_some() {
      // 遇到章节标题，跳出循环
      break;
    }
    if meta.title.is_empty() {
      // 提取书名
      if let Some(title) = extract_title(line) {
        meta.title = title;
      }
    }
    if meta.author.is_empty() {
      // 提取作者
      if let Some(author) = extract_author(line) {
        meta.author = author;
      }
    }
    if i > 12 {
      // 超过12行，跳出循环
      break;
    }
  }

  if meta.title.is_empty() {
    meta.title = stem.to_string();
  }

  let mut chapters: Vec<Chapter> = Vec::new();
  let mut chapter_index = 0;
  let mut chapter_title = String::new();
  let mut chapter_content = String::new();

  for line in &lines {
    if let Some(title) = extract_chapter_title(line) {
      // 已有章节标题：先落盘上一章，再开新章
      // 尚无标题时（前言正文已在 content 里）：直接把当前行设为第一章标题
      if !chapter_title.is_empty() {
        chapters.push(Chapter {
          index: chapter_index,
          title: std::mem::take(&mut chapter_title),
          content: std::mem::take(&mut chapter_content),
        });
        chapter_index += 1;
      }
      chapter_title = title;
      continue;
    }

    if !chapter_content.is_empty() {
      chapter_content.push('\n');
    }
    chapter_content.push_str(line);
  }

  if chapter_title.is_empty() {
    chapter_title = "正文".to_string();
  }
  chapters.push(Chapter { index: chapter_index, title: chapter_title, content: chapter_content });

  (meta, chapters)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn splits_wen_wan_style_sample() {
    let text = r#"A32《重生之温婉》全集

作者：六月浩雪

声明:本书由奇书网自网络收集整理制作,仅供交流学习使用。

楔子

“温婉，对不起，请不要怪我”最好的朋友刘倩愧疚地说着。

温婉闭上了眼睛。

一、重生

大齐三十五年的冬天，特别冷。

温婉艰难地睁开眼睛。

二、农家生活

“姐儿，要不是公主去得早，她们哪里敢这么欺负。”
"#;
    let (meta, chapters) = split_txt_book(text, "fallback");
    assert_eq!(meta.title, "重生之温婉");
    assert_eq!(meta.author, "六月浩雪");
    assert_eq!(chapters.len(), 3);
    // 标题经 extract_chapter_title 规范化，见 heading::format_line
    assert_eq!(chapters[0].title, "楔子");
    assert_eq!(chapters[1].title, "第1章: 重生");
    assert_eq!(chapters[2].title, "第2章: 农家生活");
    assert!(chapters[0].content.contains("声明"));
    assert!(chapters[1].content.contains("大齐三十五年"));
  }

  #[test]
  fn splits_di_zhang_chapters() {
    let text = "书名\n\n第一章 开始\n\n正文甲\n\n第二章 继续\n\n正文乙\n";
    let (_, chapters) = split_txt_book(text, "x");
    assert_eq!(chapters.len(), 2);
    assert_eq!(chapters[0].title, "第1章: 开始");
    assert_eq!(chapters[1].title, "第2章: 继续");
  }

  #[test]
  fn volume_and_chapter_mix() {
    let text = "书\n\n第一卷 少年\n\n正文卷首\n\n第一章 出门\n\n出门了\n\n第二章 遇雨\n\n下雨了\n\n第二卷 江湖\n\n第三章 远行\n\n走了\n";
    let (_, chapters) = split_txt_book(text, "x");
    assert!(chapters.len() >= 4);
    assert_eq!(chapters[0].title, "卷1: 少年");
    assert!(chapters.iter().any(|c| c.title.starts_with("卷2")));
  }

  #[test]
  fn fallback_single_body_when_no_headings() {
    let text = "只有一段没有章节标记的文字。\n第二行。\n";
    let (meta, chapters) = split_txt_book(text, "stem");
    assert_eq!(chapters.len(), 1);
    assert_eq!(chapters[0].title, "正文");
    assert!(chapters[0].content.contains("没有章节标记"));
    // 短首行会被当作书名
    assert_eq!(meta.title, "只有一段没有章节标记的文字。");
  }

  #[test]
  fn uses_stem_when_first_line_is_heading() {
    let text = "第一章 开篇\n\n正文内容\n";
    let (meta, chapters) = split_txt_book(text, "文件名");
    assert_eq!(meta.title, "文件名");
    assert_eq!(chapters[0].title, "第1章: 开篇");
  }
}
