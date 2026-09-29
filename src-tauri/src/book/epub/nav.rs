//! EPUB NAV / NCX 导航文档解析

use roxmltree::Document;

use super::error::EpubError;
use super::path::{resolve_url, split_toc_href};

/// 导航项（对应 epub.js parseNav / parseNCX 的基础结构）
#[derive(Debug, Clone)]
pub struct NavItem {
  pub label: String,
  pub href: Option<String>,
  pub subitems: Vec<NavItem>,
  pub type_props: Vec<String>,
}

/// 导航文档结构
#[derive(Debug, Clone, Default)]
pub struct NavDoc {
  pub toc: Vec<NavItem>,
  pub page_list: Vec<NavItem>,
  pub landmarks: Vec<NavItem>,
  pub others: Vec<NavSection>,
}

#[derive(Debug, Clone)]
pub struct NavSection {
  pub label: String,
  pub section_type: Vec<String>,
  pub list: Vec<NavItem>,
}

/// NCX 文档结构
#[derive(Debug, Clone, Default)]
pub struct NcxDoc {
  pub toc: Vec<NavItem>,
  pub page_list: Vec<NavItem>,
  pub others: Vec<NavSection>,
}

/// 与前端定位一致的 href 解析结果
#[derive(Debug, Clone)]
pub struct ResolvedHref {
  pub index: usize,
  pub path: String,
  pub fragment: Option<String>,
}

/// 解析 nav.xhtml（对应 epub.js parseNav）
pub fn parse_nav_document(xhtml: &str, nav_href: &str) -> Result<NavDoc, EpubError> {
  let doc = Document::parse(xhtml).map_err(|e| EpubError(format!("nav 解析失败: {}", e)))?;
  let mut out = NavDoc::default();

  for nav in doc.descendants().filter(|n| n.is_element() && n.tag_name().name() == "nav") {
    let nav_type = get_epub_type(nav);
    let list = parse_nav_ol(nav, nav_href, !nav_type.is_empty());

    if nav_type.iter().any(|x| x == "toc") {
      if out.toc.is_empty() {
        out.toc = list;
      }
    } else if nav_type.iter().any(|x| x == "page-list") {
      if out.page_list.is_empty() {
        out.page_list = list;
      }
    } else if nav_type.iter().any(|x| x == "landmarks") {
      if out.landmarks.is_empty() {
        out.landmarks = list;
      }
    } else {
      let label = nav.children().find(|c| c.is_element()).and_then(|n| n.text()).unwrap_or_default().trim().to_string();
      out.others.push(NavSection { label, section_type: nav_type, list });
    }
  }
  Ok(out)
}

/// 解析 NCX（对应 epub.js parseNCX）
pub fn parse_ncx_document(xml: &str, ncx_href: &str) -> Result<NcxDoc, EpubError> {
  let doc = Document::parse(xml).map_err(|e| EpubError(format!("ncx 解析失败: {}", e)))?;
  let mut out = NcxDoc::default();

  if let Some(nav_map) = doc.descendants().find(|n| n.is_element() && n.tag_name().name() == "navMap") {
    out.toc = nav_map
      .children()
      .filter(|n| n.is_element() && n.tag_name().name() == "navPoint")
      .map(|n| parse_ncx_item(n, ncx_href))
      .collect();
  }

  if let Some(page_list) = doc.descendants().find(|n| n.is_element() && n.tag_name().name() == "pageList") {
    out.page_list = page_list
      .children()
      .filter(|n| n.is_element() && n.tag_name().name() == "pageTarget")
      .map(|n| parse_ncx_item(n, ncx_href))
      .collect();
  }

  for nav_list in doc.descendants().filter(|n| n.is_element() && n.tag_name().name() == "navList") {
    let label = nav_list
      .descendants()
      .find(|n| n.is_element() && n.tag_name().name() == "navLabel")
      .and_then(|n| n.descendants().find(|x| x.is_element() && x.tag_name().name() == "text"))
      .and_then(|n| n.text())
      .unwrap_or_default()
      .trim()
      .to_string();
    let list = nav_list
      .children()
      .filter(|n| n.is_element() && n.tag_name().name() == "navTarget")
      .map(|n| parse_ncx_item(n, ncx_href))
      .collect();
    out.others.push(NavSection { label, section_type: Vec::new(), list });
  }

  Ok(out)
}

/// 在 TOC 列表中按 href 查找最匹配项（返回 flatten 后的索引）
pub fn resolve_href_in_nav(items: &[NavItem], href: &str) -> Option<ResolvedHref> {
  let (want_path, want_frag) = split_toc_href(href);
  let flat = flatten_nav(items);

  for (index, item) in flat.iter().enumerate() {
    let Some(item_href) = &item.href else { continue };
    let (p, f) = split_toc_href(item_href);
    if p == want_path {
      // 优先 exact fragment；其次 path 命中
      if f == want_frag {
        return Some(ResolvedHref { index, path: p, fragment: f });
      }
      if want_frag.is_none() {
        return Some(ResolvedHref { index, path: p, fragment: f });
      }
    }
  }
  None
}

fn parse_nav_ol(nav: roxmltree::Node<'_, '_>, nav_href: &str, with_type: bool) -> Vec<NavItem> {
  let Some(ol) = nav.children().find(|n| n.is_element() && n.tag_name().name() == "ol") else {
    return Vec::new();
  };

  ol.children()
    .filter(|n| n.is_element() && n.tag_name().name() == "li")
    .map(|li| parse_nav_li(li, nav_href, with_type))
    .collect()
}

fn parse_nav_li(li: roxmltree::Node<'_, '_>, nav_href: &str, with_type: bool) -> NavItem {
  let a_or_span =
    li.children().find(|n| n.is_element() && (n.tag_name().name() == "a" || n.tag_name().name() == "span"));
  let href = a_or_span.and_then(|n| n.attribute("href")).map(|h| resolve_url(h, nav_href));
  let label = a_or_span
    .and_then(|n| n.text().map(|x| x.trim().to_string()))
    .filter(|x| !x.is_empty())
    .or_else(|| a_or_span.and_then(|n| n.attribute("title").map(|x| x.to_string())))
    .unwrap_or_default();

  let subitems = li
    .children()
    .find(|n| n.is_element() && n.tag_name().name() == "ol")
    .map(|ol| {
      ol.children()
        .filter(|n| n.is_element() && n.tag_name().name() == "li")
        .map(|x| parse_nav_li(x, nav_href, with_type))
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();

  let type_props = if with_type { a_or_span.and_then(get_epub_type_attr).unwrap_or_default() } else { Vec::new() };

  NavItem { label, href, subitems, type_props }
}

fn parse_ncx_item(node: roxmltree::Node<'_, '_>, ncx_href: &str) -> NavItem {
  let label = node
    .children()
    .find(|n| n.is_element() && n.tag_name().name() == "navLabel")
    .and_then(|n| n.descendants().find(|x| x.is_element() && x.tag_name().name() == "text"))
    .and_then(|n| n.text())
    .unwrap_or_default()
    .trim()
    .to_string();
  let href = node
    .children()
    .find(|n| n.is_element() && n.tag_name().name() == "content")
    .and_then(|n| n.attribute("src"))
    .map(|x| resolve_url(x, ncx_href));

  let child_name = node.tag_name().name();
  let subitems = if child_name == "navPoint" {
    node
      .children()
      .filter(|n| n.is_element() && n.tag_name().name() == "navPoint")
      .map(|x| parse_ncx_item(x, ncx_href))
      .collect()
  } else {
    Vec::new()
  };

  NavItem { label, href, subitems, type_props: Vec::new() }
}

fn flatten_nav(items: &[NavItem]) -> Vec<NavItem> {
  let mut out = Vec::new();
  for item in items {
    out.push(item.clone());
    out.extend(flatten_nav(&item.subitems));
  }
  out
}

fn get_epub_type(node: roxmltree::Node<'_, '_>) -> Vec<String> {
  get_epub_type_attr(node).unwrap_or_default()
}

fn get_epub_type_attr(node: roxmltree::Node<'_, '_>) -> Option<Vec<String>> {
  for attr in node.attributes() {
    if attr.name().ends_with(":type") || attr.name() == "type" {
      let vals = attr.value().split_whitespace().filter(|x| !x.is_empty()).map(|x| x.to_string()).collect::<Vec<_>>();
      if !vals.is_empty() {
        return Some(vals);
      }
    }
  }
  None
}
