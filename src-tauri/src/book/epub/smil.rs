//! EPUB Media Overlay (SMIL) 解析

use roxmltree::Document;

use super::error::EpubError;
use super::path::{parse_clock, resolve_url};

/// media overlay 的一段
#[derive(Debug, Clone)]
pub struct SmilParEntry {
  pub text: String,
  pub src: String,
  pub begin: Option<f64>,
  pub end: Option<f64>,
}

/// 解析 SMIL par 项（对应 epub.js MediaOverlay 的基础数据）
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
