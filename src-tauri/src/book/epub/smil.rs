//! EPUB Media Overlay (SMIL) 解析

use roxmltree::Document;

use super::error::EpubError;
use super::path::{parse_clock, resolve_url};

/// media overlay 的一段
///
/// 同一个 `par` 里要同时有 `text` 和 `audio`，两边 `src` 解析后都非空才保留。
///
/// # 字段
/// - `text`: `text` 的 `src`，按 SMIL 路径解析
/// - `src`: `audio` 的 `src`，按 SMIL 路径解析
/// - `begin`: `clipBegin` 解析出的秒数，没有则为空
/// - `end`: `clipEnd` 解析出的秒数，没有则为空
///
/// # 示例
/// 见 [`parse_smil_par_entries`]
#[derive(Debug, Clone)]
pub struct SmilParEntry {
  /// `text` 的 `src`，按 SMIL 路径解析
  pub text: String,
  /// `audio` 的 `src`，按 SMIL 路径解析
  pub src: String,
  /// `clipBegin` 解析出的秒数，没有则为空
  pub begin: Option<f64>,
  /// `clipEnd` 解析出的秒数，没有则为空
  pub end: Option<f64>,
}

/// 解析 SMIL 里的 par 条目
///
/// 解析 XML 后遍历 `par`。每个 `par` 需要同时有 `text` 和 `audio`，且两边的 `src` 解析后都非空。音频的 `clipBegin`、`clipEnd` 按时长解析。
///
/// # 参数
/// - `xml`: SMIL XML 文本
/// - `smil_href`: 用来解析相对 `src` 的基准路径
///
/// # 返回值
/// 成功返回 par 列表，没有合格项时为空列表
///
/// # 错误
/// XML 解析失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let xml = r#"<smil><par><text src="c.xhtml#a"/><audio src="a.mp3" clipBegin="1s"/></par></smil>"#;
/// let items = parse_smil_par_entries(xml, "ov.smil").unwrap();
/// assert_eq!(items.len(), 1);
/// assert_eq!(items[0].begin, Some(1.0));
/// ```
pub fn parse_smil_par_entries(xml: &str, smil_href: &str) -> Result<Vec<SmilParEntry>, EpubError> {
  let doc = Document::parse(xml).map_err(|e| EpubError(format!("smil 解析失败: {}", e)))?;
  let mut out = Vec::new();

  for par in doc.descendants().filter(|n| n.is_element() && n.tag_name().name() == "par") {
    let text_el = par.children().find(|n| n.is_element() && n.tag_name().name() == "text");
    let audio_el = par.children().find(|n| n.is_element() && n.tag_name().name() == "audio");
    let (Some(text_el), Some(audio_el)) = (text_el, audio_el) else {
      continue;
    };

    let text = text_el.attribute("src").map(|x| resolve_url(x, smil_href)).unwrap_or_default();
    let src = audio_el.attribute("src").map(|x| resolve_url(x, smil_href)).unwrap_or_default();
    if text.is_empty() || src.is_empty() {
      continue;
    }

    out.push(SmilParEntry {
      text,
      src,
      begin: audio_el.attribute("clipBegin").and_then(parse_clock),
      end: audio_el.attribute("clipEnd").and_then(parse_clock),
    });
  }

  Ok(out)
}
