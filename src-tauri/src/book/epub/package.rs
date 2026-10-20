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
///
/// # 字段
/// - `href`: 相对 OPF 解析后的路径
/// - `media_type`: `media-type`，没有则为空字符串
/// - `properties`: `properties` 按空白拆开，空段丢掉
///
/// # 示例
/// ```ignore
/// let item = ManifestItem {
///   href: "nav.xhtml".to_string(),
///   media_type: "application/xhtml+xml".to_string(),
///   properties: vec!["nav".to_string()],
/// };
/// assert!(item.properties.iter().any(|p| p == "nav"));
/// ```
#[derive(Debug, Clone)]
pub(crate) struct ManifestItem {
  /// 相对 OPF 解析后的路径
  pub href: String,
  /// `media-type`，没有则为空字符串
  pub media_type: String,
  /// `properties` 按空白拆开，空段丢掉
  pub properties: Vec<String>,
}

/// Spine 条目（已解析章标题）
///
/// # 字段
/// - `href`: 章节在压缩包里的路径
/// - `title`: 导航标题。没有标题时写成 `第 {n} 章`，序号从 1 开始
///
/// # 示例
/// ```ignore
/// let entry = SpineEntry { href: "c.xhtml".to_string(), title: format!("第 {} 章", 1) };
/// assert_eq!(entry.title, "第 1 章");
/// ```
#[derive(Debug, Clone)]
pub(crate) struct SpineEntry {
  /// 章节在压缩包里的路径
  pub href: String,
  /// 导航标题。没有标题时写成 `第 {n} 章`，序号从 1 开始
  pub title: String,
}

/// 已解析的 OPF 包：元数据 + spine
///
/// # 字段
/// - `meta`: 书名和作者。空字符串分别换成「未知标题」「未知作者」
/// - `meta_full`: OPF 里的完整元数据
/// - `spine`: `itemref` 按 manifest 生成的章节条目
///
/// # 示例
/// 见 [`parse_package`]
#[derive(Debug, Clone)]
pub(crate) struct ParsedPackage {
  /// 书名和作者。空字符串分别换成「未知标题」「未知作者」
  pub meta: EpubMeta,
  /// OPF 里的完整元数据
  pub meta_full: EpubMetaFull,
  /// `itemref` 按 manifest 生成的章节条目
  pub spine: Vec<SpineEntry>,
}

/// 解析 EPUB 压缩包里的 OPF 包
///
/// 读取 `META-INF/container.xml` 的 `rootfile`，再解析 OPF。收集 manifest，优先用带 `nav` 属性的文档建立路径到标题的映射，失败时改用 spine 的 `toc` 或 NCX。spine 的 `itemref` 按 manifest 生成条目，没有标题时写成 `第 {n} 章`。标题或作者为空时分别填「未知标题」「未知作者」。
///
/// # 参数
/// - `path`: EPUB 文件路径
///
/// # 返回值
/// 成功返回元数据和 spine
///
/// # 错误
/// 打开压缩包、缺少 `rootfile` 或 XML 解析失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let package = parse_package(Path::new("book.epub")).unwrap();
/// assert!(!package.meta.title.is_empty());
/// ```
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

/// 把导航项的路径和标签写入映射
///
/// 链接路径非空且规范化后的路径尚未出现时，插入该标签。然后递归处理子项。
///
/// # 参数
/// - `items`: 导航树
/// - `out`: 路径到标签的映射，已有路径不会覆盖
///
/// # 返回值
/// 无
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let mut out = std::collections::HashMap::new();
/// let items = [NavItem {
///   label: "章".to_string(),
///   href: Some("a/../c.xhtml#id".to_string()),
///   subitems: Vec::new(),
///   type_props: Vec::new(),
/// }];
/// collect_nav_labels(&items, &mut out);
/// assert_eq!(out.get("c.xhtml").map(String::as_str), Some("章"));
/// ```
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

/// 打开 EPUB 压缩包
///
/// # 参数
/// - `path`: EPUB 文件路径
///
/// # 返回值
/// 成功返回 ZIP 归档
///
/// # 错误
/// 打开文件或解析 ZIP 失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let zip = open_zip(Path::new("book.epub")).unwrap();
/// assert!(zip.len() > 0);
/// ```
pub(crate) fn open_zip(path: &Path) -> Result<ZipArchive<File>, EpubError> {
  let file = File::open(path).map_err(|e| EpubError(format!("打开 EPUB 失败: {}", e)))?;
  ZipArchive::new(file).map_err(|e| EpubError(format!("读取 ZIP 失败: {}", e)))
}

/// 从 EPUB 路径读取一个文本条目
///
/// 重新打开压缩包后按条目名读取，并用损失性 UTF-8 转成字符串。
///
/// # 参数
/// - `path`: EPUB 文件路径
/// - `entry`: 压缩包内的条目名
///
/// # 返回值
/// 成功返回条目文本
///
/// # 错误
/// 打开压缩包或读取条目失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let text = read_zip_text(Path::new("book.epub"), "META-INF/container.xml").unwrap();
/// assert!(text.contains("container"));
/// ```
pub(crate) fn read_zip_text(path: &Path, entry: &str) -> Result<String, EpubError> {
  let mut zip = open_zip(path)?;
  read_zip_text_from_archive(&mut zip, entry)
}

/// 从 EPUB 路径读取一个字节条目
///
/// # 参数
/// - `path`: EPUB 文件路径
/// - `entry`: 压缩包内的条目名
///
/// # 返回值
/// 成功返回条目字节
///
/// # 错误
/// 打开压缩包或读取条目失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let bytes = read_zip_bytes(Path::new("book.epub"), "META-INF/container.xml").unwrap();
/// assert!(!bytes.is_empty());
/// ```
pub(crate) fn read_zip_bytes(path: &Path, entry: &str) -> Result<Vec<u8>, EpubError> {
  let mut zip = open_zip(path)?;
  read_zip_bytes_from_archive(&mut zip, entry)
}

/// 从已打开的压缩包读取文本条目
///
/// 读出字节后用损失性 UTF-8 转成字符串。
///
/// # 参数
/// - `zip`: 已打开的 EPUB 压缩包
/// - `entry`: 压缩包内的条目名
///
/// # 返回值
/// 成功返回条目文本
///
/// # 错误
/// 读取条目失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let mut zip = open_zip(Path::new("book.epub")).unwrap();
/// let text = read_zip_text_from_archive(&mut zip, "META-INF/container.xml").unwrap();
/// assert!(text.contains("container"));
/// ```
pub(crate) fn read_zip_text_from_archive(zip: &mut ZipArchive<File>, entry: &str) -> Result<String, EpubError> {
  let bytes = read_zip_bytes_from_archive(zip, entry)?;
  Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// 从已打开的压缩包读取字节条目
///
/// # 参数
/// - `zip`: 已打开的 EPUB 压缩包
/// - `entry`: 压缩包内的条目名
///
/// # 返回值
/// 成功返回条目的全部字节
///
/// # 错误
/// 找不到条目或读完内容失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let mut zip = open_zip(Path::new("book.epub")).unwrap();
/// let bytes = read_zip_bytes_from_archive(&mut zip, "META-INF/container.xml").unwrap();
/// assert!(!bytes.is_empty());
/// ```
pub(crate) fn read_zip_bytes_from_archive(zip: &mut ZipArchive<File>, entry: &str) -> Result<Vec<u8>, EpubError> {
  let mut file = zip.by_name(entry).map_err(|e| EpubError(format!("读取条目失败 [{}]: {}", entry, e)))?;
  let mut bytes = Vec::new();
  file.read_to_end(&mut bytes).map_err(|e| EpubError(format!("读取条目内容失败 [{}]: {}", entry, e)))?;
  Ok(bytes)
}

const PREFIX_RENDITION: &str = "http://www.idpf.org/vocab/rendition/#";
const PREFIX_MEDIA: &str = "http://www.idpf.org/epub/vocab/overlays/#";
const PREFIX_DCTERMS: &str = "http://purl.org/dc/terms/";

/// 从 OPF 文档提取完整元数据
///
/// 标识优先取 `package` 的 `unique-identifier` 指向的 `identifier`，否则取第一个 `identifier`。标题和作者分别取第一个 `title`、`creator`，缺失时用「未知标题」「未知作者」。同时收集语言、简介、出版者、主题、权利、来源和其余标识。`meta` 里识别修改时间、系列名、组内序号、rendition 与 media 属性，以及 Calibre 的 `calibre:series`。`date` 按 `publication`、`modification` 取值，出版日期空缺时用遇到的日期文本。系列名空缺时回退到 Calibre 字段。媒体时长取 media 里的 `duration`。最后从备用标识中去掉主标识。
///
/// # 参数
/// - `doc`: 已解析的 OPF 文档
///
/// # 返回值
/// 填好的完整元数据
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let doc = roxmltree::Document::parse(
///   r#"<package unique-identifier="id"><metadata><identifier id="id">urn:x</identifier><title>书</title><creator>人</creator></metadata></package>"#,
/// )
/// .unwrap();
/// let meta = parse_opf_metadata(&doc);
/// assert_eq!(meta.title, "书");
/// assert_eq!(meta.author, "人");
/// assert_eq!(meta.identifier.as_deref(), Some("urn:x"));
/// ```
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

/// 读取节点文本并去掉首尾空白
///
/// # 参数
/// - `node`: 要读取的节点
///
/// # 返回值
/// 文本去空白后非空时返回 `Some`，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let doc = roxmltree::Document::parse("<title> 书 </title>").unwrap();
/// let node = doc.descendants().find(|n| n.tag_name().name() == "title").unwrap();
/// assert_eq!(node_text_trimmed(node).as_deref(), Some("书"));
/// ```
fn node_text_trimmed(node: roxmltree::Node<'_, '_>) -> Option<String> {
  node.text().map(|x| x.trim().to_string()).filter(|x| !x.is_empty())
}

/// 去掉属性名上的词表前缀
///
/// 先按 `prefix_url` 去掉前缀。若 `prefix_url` 是 Dublin Core terms，并且属性名以该地址开头，则返回其后的部分。
///
/// # 参数
/// - `prop`: 属性名
/// - `prefix_url`: 要去掉的前缀
///
/// # 返回值
/// 去掉前缀后的名字；两种方式都不匹配时返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(
///   strip_prefix_property("http://www.idpf.org/vocab/rendition/#layout", "http://www.idpf.org/vocab/rendition/#").as_deref(),
///   Some("layout")
/// );
/// assert_eq!(strip_prefix_property("layout", "http://www.idpf.org/vocab/rendition/#"), None);
/// ```
fn strip_prefix_property(prop: &str, prefix_url: &str) -> Option<String> {
  if let Some(x) = prop.strip_prefix(prefix_url) {
    return Some(x.to_string());
  }
  if prop.starts_with(PREFIX_DCTERMS) && prefix_url == PREFIX_DCTERMS {
    return Some(prop[PREFIX_DCTERMS.len()..].to_string());
  }
  None
}

/// 把连字符或冒号分隔的名字转成驼峰
///
/// 遇到 `-` 或 `:` 时跳过该字符，并把下一个字符转成大写。
///
/// # 参数
/// - `s`: 原始名字
///
/// # 返回值
/// 转换后的字符串
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// assert_eq!(to_camel("layout-flow"), "layoutFlow");
/// assert_eq!(to_camel("media:duration"), "mediaDuration");
/// ```
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
