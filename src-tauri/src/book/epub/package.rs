//! EPUB package：container.xml / OPF / spine

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use roxmltree::Document;
use zip::ZipArchive;

use super::error::EpubError;
use super::meta::{EpubMeta, EpubMetaFull};
use super::nav::{NavItem, parse_nav_document, parse_ncx_document};
use super::path::{normalize_path, parse_clock, resolve_url, split_toc_href};

/// Manifest 单项
#[derive(Debug, Clone)]
pub(crate) struct ManifestItem {
  pub href: String,
  pub media_type: String,
  pub properties: Vec<String>,
}

/// Spine 条目（已解析章标题）
#[derive(Debug, Clone)]
pub(crate) struct SpineEntry {
  pub href: String,
  pub title: String,
}

/// 已解析的 OPF 包：元数据 + spine
#[derive(Debug, Clone)]
pub(crate) struct ParsedPackage {
  pub meta: EpubMeta,
  pub meta_full: EpubMetaFull,
  pub spine: Vec<SpineEntry>,
}

pub(crate) fn parse_package(path: &Path) -> Result<ParsedPackage, EpubError> {
  let mut zip = open_zip(path)?;
  let container_xml = read_zip_text_from_archive(&mut zip, "META-INF/container.xml")?;
  let container_doc =
    Document::parse(&container_xml).map_err(|e| EpubError(format!("container.xml 解析失败: {}", e)))?;
  let rootfile = container_doc
    .descendants()
    .find(|n| n.is_element() && n.tag_name().name() == "rootfile")
    .and_then(|n| n.attribute("full-path"))
    .ok_or_else(|| EpubError("container.xml 缺少 rootfile/full-path".to_string()))?
    .to_string();

  let opf_xml = read_zip_text_from_archive(&mut zip, &rootfile)?;
  let opf_doc = Document::parse(&opf_xml).map_err(|e| EpubError(format!("OPF 解析失败: {}", e)))?;
  let meta_full = parse_opf_metadata(&opf_doc);
  let title = if meta_full.title.is_empty() { "未知标题".to_string() } else { meta_full.title.clone() };
  let author = if meta_full.author.is_empty() { "未知作者".to_string() } else { meta_full.author.clone() };

  let mut manifest = HashMap::<String, ManifestItem>::new();
  for item in opf_doc.descendants().filter(|n| n.is_element() && n.tag_name().name() == "item") {
    let Some(id) = item.attribute("id").map(|x| x.to_string()) else {
      continue;
    };
    let href_raw = item.attribute("href").unwrap_or_default();
    let href = resolve_url(href_raw, &rootfile);
    let media_type = item.attribute("media-type").unwrap_or_default().to_string();
    let properties = item
      .attribute("properties")
      .unwrap_or_default()
      .split_whitespace()
      .filter(|x| !x.is_empty())
      .map(|x| x.to_string())
      .collect::<Vec<_>>();
    manifest.insert(id, ManifestItem { href, media_type, properties });
  }

  let spine_node = opf_doc.descendants().find(|n| n.is_element() && n.tag_name().name() == "spine");
  let spine_toc = spine_node.and_then(|n| n.attribute("toc")).map(|x| x.to_string());

  let nav_path = manifest.values().find(|x| x.properties.iter().any(|p| p == "nav")).map(|x| x.href.clone());
  let ncx_path = spine_toc
    .as_ref()
    .and_then(|id| manifest.get(id))
    .map(|x| x.href.clone())
    .or_else(|| manifest.values().find(|x| x.media_type == "application/x-dtbncx+xml").map(|x| x.href.clone()));

  let mut title_map = HashMap::<String, String>::new();
  if let Some(nav_path) = nav_path.as_ref()
    && let Ok(nav_xml) = read_zip_text_from_archive(&mut zip, nav_path)
    && let Ok(nav_doc) = parse_nav_document(&nav_xml, nav_path)
  {
    collect_nav_labels(&nav_doc.toc, &mut title_map);
  } else if let Some(ncx_path) = ncx_path.as_ref()
    && let Ok(ncx_xml) = read_zip_text_from_archive(&mut zip, ncx_path)
    && let Ok(ncx_doc) = parse_ncx_document(&ncx_xml, ncx_path)
  {
    collect_nav_labels(&ncx_doc.toc, &mut title_map);
  }

  let mut spine = Vec::new();
  for (i, itemref) in opf_doc.descendants().filter(|n| n.is_element() && n.tag_name().name() == "itemref").enumerate() {
    let Some(idref) = itemref.attribute("idref") else {
      continue;
    };
    let Some(mi) = manifest.get(idref) else {
      continue;
    };
    let title = title_map.get(&normalize_path(&mi.href)).cloned().unwrap_or_else(|| format!("第 {} 章", i + 1));
    spine.push(SpineEntry { href: mi.href.clone(), title });
  }

  Ok(ParsedPackage { meta: EpubMeta { title, author }, meta_full, spine })
}

pub(crate) fn collect_nav_labels(items: &[NavItem], out: &mut HashMap<String, String>) {
  for item in items {
    if let Some(href) = &item.href {
      let (path, _) = split_toc_href(href);
      if !path.is_empty() && !out.contains_key(&normalize_path(&path)) {
        out.insert(normalize_path(&path), item.label.clone());
      }
    }
    collect_nav_labels(&item.subitems, out);
  }
}

pub(crate) fn open_zip(path: &Path) -> Result<ZipArchive<File>, EpubError> {
  let file = File::open(path).map_err(|e| EpubError(format!("打开 EPUB 失败: {}", e)))?;
  ZipArchive::new(file).map_err(|e| EpubError(format!("读取 ZIP 失败: {}", e)))
}

pub(crate) fn read_zip_text(path: &Path, entry: &str) -> Result<String, EpubError> {
  let mut zip = open_zip(path)?;
  read_zip_text_from_archive(&mut zip, entry)
}

pub(crate) fn read_zip_bytes(path: &Path, entry: &str) -> Result<Vec<u8>, EpubError> {
  let mut zip = open_zip(path)?;
  read_zip_bytes_from_archive(&mut zip, entry)
}

pub(crate) fn read_zip_text_from_archive(zip: &mut ZipArchive<File>, entry: &str) -> Result<String, EpubError> {
  let bytes = read_zip_bytes_from_archive(zip, entry)?;
  Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub(crate) fn read_zip_bytes_from_archive(zip: &mut ZipArchive<File>, entry: &str) -> Result<Vec<u8>, EpubError> {
  let mut file = zip.by_name(entry).map_err(|e| EpubError(format!("读取条目失败 [{}]: {}", entry, e)))?;
  let mut bytes = Vec::new();
  file.read_to_end(&mut bytes).map_err(|e| EpubError(format!("读取条目内容失败 [{}]: {}", entry, e)))?;
  Ok(bytes)
}

const PREFIX_RENDITION: &str = "http://www.idpf.org/vocab/rendition/#";
const PREFIX_MEDIA: &str = "http://www.idpf.org/epub/vocab/overlays/#";
const PREFIX_DCTERMS: &str = "http://purl.org/dc/terms/";

fn parse_opf_metadata(doc: &Document<'_>) -> EpubMetaFull {
  let mut out = EpubMetaFull::default();

  let unique_id_ref = doc
    .descendants()
    .find(|n| n.is_element() && n.tag_name().name() == "package")
    .and_then(|n| n.attribute("unique-identifier"));

  if let Some(uid_ref) = unique_id_ref {
    out.identifier = doc
      .descendants()
      .find(|n| n.is_element() && n.tag_name().name() == "identifier" && n.attribute("id") == Some(uid_ref))
      .and_then(node_text_trimmed);
  }
  if out.identifier.is_none() {
    out.identifier =
      doc.descendants().find(|n| n.is_element() && n.tag_name().name() == "identifier").and_then(node_text_trimmed);
  }

  out.title = doc
    .descendants()
    .find(|n| n.is_element() && n.tag_name().name() == "title")
    .and_then(node_text_trimmed)
    .unwrap_or_else(|| "未知标题".to_string());
  out.author = doc
    .descendants()
    .find(|n| n.is_element() && n.tag_name().name() == "creator")
    .and_then(node_text_trimmed)
    .unwrap_or_else(|| "未知作者".to_string());

  out.language = doc
    .descendants()
    .filter(|n| n.is_element() && n.tag_name().name() == "language")
    .filter_map(node_text_trimmed)
    .collect();
  out.description =
    doc.descendants().find(|n| n.is_element() && n.tag_name().name() == "description").and_then(node_text_trimmed);
  out.publisher = doc
    .descendants()
    .filter(|n| n.is_element() && n.tag_name().name() == "publisher")
    .filter_map(node_text_trimmed)
    .collect();
  out.subject = doc
    .descendants()
    .filter(|n| n.is_element() && n.tag_name().name() == "subject")
    .filter_map(node_text_trimmed)
    .collect();
  out.rights =
    doc.descendants().find(|n| n.is_element() && n.tag_name().name() == "rights").and_then(node_text_trimmed);
  out.alt_identifier = doc
    .descendants()
    .filter(|n| n.is_element() && n.tag_name().name() == "identifier")
    .filter_map(node_text_trimmed)
    .collect();
  out.source = doc
    .descendants()
    .filter(|n| n.is_element() && n.tag_name().name() == "source")
    .filter_map(node_text_trimmed)
    .collect();

  let mut pub_date = None;
  let mut mod_date = None;

  // legacy calibre series
  let mut legacy_series_name = None;
  let mut legacy_series_index = None;

  for meta in doc.descendants().filter(|n| n.is_element() && n.tag_name().name() == "meta") {
    let property = meta.attribute("property").unwrap_or_default();
    let name = meta.attribute("name").unwrap_or_default();
    let content = meta.attribute("content").map(|x| x.trim().to_string()).filter(|x| !x.is_empty());
    let value = node_text_trimmed(meta).or(content.clone());

    if !property.is_empty() {
      if property.ends_with("dcterms:modified")
        || property.ends_with("terms/modified")
        || property == "dcterms:modified"
      {
        if mod_date.is_none() {
          mod_date = value.clone();
        }
      }

      if property == "belongs-to-collection" && out.series_name.is_none() {
        out.series_name = value.clone();
      }
      if property == "group-position" && out.series_position.is_none() {
        out.series_position = value.as_deref().and_then(|x| x.parse::<f64>().ok());
      }

      if let Some(k) = strip_prefix_property(property, PREFIX_RENDITION)
        .or_else(|| property.strip_prefix("rendition:").map(|x| x.to_string()))
        && let Some(v) = value.clone()
      {
        out.rendition.insert(to_camel(&k), v);
      }

      if let Some(k) =
        strip_prefix_property(property, PREFIX_MEDIA).or_else(|| property.strip_prefix("media:").map(|x| x.to_string()))
        && let Some(v) = value.clone()
      {
        out.media.insert(to_camel(&k), v);
      }
    }

    if name == "calibre:series" {
      legacy_series_name = content.clone();
    } else if name == "calibre:series_index" {
      legacy_series_index = content.as_deref().and_then(|x| x.parse::<f64>().ok());
    }
  }

  // publication / modification date fallback from dc:date
  for date_node in doc.descendants().filter(|n| n.is_element() && n.tag_name().name() == "date") {
    let event = date_node.attribute("event").unwrap_or_default();
    let v = node_text_trimmed(date_node);
    if event == "publication" && pub_date.is_none() {
      pub_date = v.clone();
    }
    if event == "modification" && mod_date.is_none() {
      mod_date = v.clone();
    }
    if pub_date.is_none() {
      pub_date = v;
    }
  }
  out.published = pub_date;
  out.modified = mod_date;

  if out.series_name.is_none() && legacy_series_name.is_some() {
    out.series_name = legacy_series_name;
  }
  if out.series_position.is_none() && legacy_series_index.is_some() {
    out.series_position = legacy_series_index;
  }

  // media duration
  out.media_duration = out.media.get("duration").and_then(|x| parse_clock(x));

  // 清理重复 alt identifier
  if let Some(id) = &out.identifier {
    out.alt_identifier.retain(|x| x != id);
  }

  out
}

fn node_text_trimmed(node: roxmltree::Node<'_, '_>) -> Option<String> {
  node.text().map(|x| x.trim().to_string()).filter(|x| !x.is_empty())
}

fn strip_prefix_property(prop: &str, prefix_url: &str) -> Option<String> {
  if let Some(x) = prop.strip_prefix(prefix_url) {
    return Some(x.to_string());
  }
  if prop.starts_with(PREFIX_DCTERMS) && prefix_url == PREFIX_DCTERMS {
    return Some(prop[PREFIX_DCTERMS.len()..].to_string());
  }
  None
}

fn to_camel(s: &str) -> String {
  let mut out = String::new();
  let mut upper = false;
  for ch in s.chars() {
    if ch == '-' || ch == ':' {
      upper = true;
      continue;
    }
    if upper {
      out.push(ch.to_ascii_uppercase());
      upper = false;
    } else {
      out.push(ch);
    }
  }
  out
}
