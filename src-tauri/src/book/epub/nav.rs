//! EPUB NAV / NCX 导航文档解析

use roxmltree::Document;

use super::error::EpubError;
use super::path::{resolve_url, split_toc_href};

/// 导航项（对应 epub.js parseNav / parseNCX 的基础结构）
///
/// # 字段
/// - `label`: 标签。nav 优先用非空文本，否则用 `title`；NCX 取 `navLabel/text`
/// - `href`: 解析后的链接，没有则为空
/// - `subitems`: 子项。nav 来自嵌套 `ol`，NCX 只在 `navPoint` 下递归
/// - `type_props`: epub type。NCX 和未读取 type 时为空
///
/// # 示例
/// 见 [`parse_nav_li`]
#[derive(Debug, Clone)]
pub struct NavItem {
  /// 标签。nav 优先用非空文本，否则用 `title`；NCX 取 `navLabel/text`
  pub label: String,
  /// 解析后的链接，没有则为空
  pub href: Option<String>,
  /// 子项。nav 来自嵌套 `ol`，NCX 只在 `navPoint` 下递归
  pub subitems: Vec<NavItem>,
  /// epub type。NCX 和未读取 type 时为空
  pub type_props: Vec<String>,
}

/// 导航文档结构
///
/// 每种已知类型只保留第一份。
///
/// # 字段
/// - `toc`: `epub:type` 含 `toc` 的列表
/// - `page_list`: 含 `page-list` 的列表
/// - `landmarks`: 含 `landmarks` 的列表
/// - `others`: 其余 `nav`，标签取第一个子元素的文本
///
/// # 示例
/// 见 [`parse_nav_document`]
#[derive(Debug, Clone, Default)]
pub struct NavDoc {
  /// `epub:type` 含 `toc` 的列表
  pub toc: Vec<NavItem>,
  /// 含 `page-list` 的列表
  pub page_list: Vec<NavItem>,
  /// 含 `landmarks` 的列表
  pub landmarks: Vec<NavItem>,
  /// 其余 `nav`，标签取第一个子元素的文本
  pub others: Vec<NavSection>,
}

/// 不属于目录、页码或 landmarks 的导航节
///
/// # 字段
/// - `label`: 节标签
/// - `section_type`: epub type。NCX 的 `navList` 为空
/// - `list`: 该节的导航项
///
/// # 示例
/// ```
/// let section = NavSection {
///   label: "导读".to_string(),
///   section_type: vec!["loi".to_string()],
///   list: Vec::new(),
/// };
/// assert_eq!(section.section_type, ["loi"]);
/// ```
#[derive(Debug, Clone)]
pub struct NavSection {
  /// 节标签
  pub label: String,
  /// epub type。NCX 的 `navList` 为空
  pub section_type: Vec<String>,
  /// 该节的导航项
  pub list: Vec<NavItem>,
}

/// NCX 文档结构
///
/// # 字段
/// - `toc`: `navMap` 下的 `navPoint`
/// - `page_list`: `pageList` 下的 `pageTarget`
/// - `others`: 每个 `navList` 一节
///
/// # 示例
/// 见 [`parse_ncx_document`]
#[derive(Debug, Clone, Default)]
pub struct NcxDoc {
  /// `navMap` 下的 `navPoint`
  pub toc: Vec<NavItem>,
  /// `pageList` 下的 `pageTarget`
  pub page_list: Vec<NavItem>,
  /// 每个 `navList` 一节
  pub others: Vec<NavSection>,
}

/// 与前端定位一致的 href 解析结果
///
/// 路径相同且片段也相同则命中。目标没有片段时，返回第一个路径相同的项。
///
/// # 字段
/// - `index`: 导航树先序摊平后的下标
/// - `path`: 去掉片段后的路径
/// - `fragment`: `#` 后的片段，没有则为空
///
/// # 示例
/// 见 [`resolve_href_in_nav`]
#[derive(Debug, Clone)]
pub struct ResolvedHref {
  /// 导航树先序摊平后的下标
  pub index: usize,
  /// 去掉片段后的路径
  pub path: String,
  /// `#` 后的片段，没有则为空
  pub fragment: Option<String>,
}

/// 解析 nav 文档里的导航列表
///
/// 遍历所有 `nav`。按 `epub:type` 或 `type` 把第一份 `toc`、`page-list`、`landmarks` 放进对应字段，其余放进 `others`，标签取该 `nav` 第一个子元素的文本。
///
/// # 参数
/// - `xhtml`: nav 文档文本
/// - `nav_href`: 解析条目相对链接用的基准路径
///
/// # 返回值
/// 成功返回导航文档
///
/// # 错误
/// XML 解析失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let xhtml = r#"<nav epub:type="toc"><ol><li><a href="c.xhtml">章</a></li></ol></nav>"#;
/// let doc = parse_nav_document(xhtml, "nav.xhtml").unwrap();
/// assert_eq!(doc.toc.len(), 1);
/// assert_eq!(doc.toc[0].label, "章");
/// assert_eq!(doc.toc[0].href.as_deref(), Some("c.xhtml"));
/// ```
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

/// 解析 NCX 文档里的导航列表
///
/// `navMap` 下的 `navPoint` 进入目录，`pageList` 下的 `pageTarget` 进入页码列表。每个 `navList` 用其中 `navLabel/text` 作标签，子级 `navTarget` 作列表，放进 `others`。
///
/// # 参数
/// - `xml`: NCX 文本
/// - `ncx_href`: 解析 `content src` 用的基准路径
///
/// # 返回值
/// 成功返回 NCX 文档
///
/// # 错误
/// XML 解析失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let xml = r#"<ncx><navMap><navPoint><navLabel><text>章</text></navLabel><content src="c.xhtml"/></navPoint></navMap></ncx>"#;
/// let doc = parse_ncx_document(xml, "toc.ncx").unwrap();
/// assert_eq!(doc.toc.len(), 1);
/// assert_eq!(doc.toc[0].label, "章");
/// assert_eq!(doc.toc[0].href.as_deref(), Some("c.xhtml"));
/// ```
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

/// 在导航项里按链接查找最匹配的一项
///
/// 先把树按先序摊平。路径相同且片段也相同则立即返回；目标没有片段时，返回第一个路径相同的项。
///
/// # 参数
/// - `items`: 导航树
/// - `href`: 要查找的链接，可带 `#` 片段
///
/// # 返回值
/// 找到时返回摊平后的下标、路径和片段，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let items = [NavItem {
///   label: "章".to_string(),
///   href: Some("c.xhtml#a".to_string()),
///   subitems: Vec::new(),
///   type_props: Vec::new(),
/// }];
/// let hit = resolve_href_in_nav(&items, "c.xhtml#a").unwrap();
/// assert_eq!(hit.index, 0);
/// assert_eq!(hit.fragment.as_deref(), Some("a"));
/// ```
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

/// 解析 nav 下第一层 ol 里的 li
///
/// 没有 `ol` 时返回空列表。
///
/// # 参数
/// - `nav`: `nav` 元素
/// - `nav_href`: 解析链接用的基准路径
/// - `with_type`: 为真时读取每个链接上的 epub type
///
/// # 返回值
/// 该层列表项
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let doc = roxmltree::Document::parse(r#"<nav><ol><li><a href="c.xhtml">章</a></li></ol></nav>"#).unwrap();
/// let nav = doc.descendants().find(|n| n.tag_name().name() == "nav").unwrap();
/// let items = parse_nav_ol(nav, "nav.xhtml", false);
/// assert_eq!(items.len(), 1);
/// assert_eq!(items[0].label, "章");
/// ```
fn parse_nav_ol(nav: roxmltree::Node<'_, '_>, nav_href: &str, with_type: bool) -> Vec<NavItem> {
  let Some(ol) = nav.children().find(|n| n.is_element() && n.tag_name().name() == "ol") else {
    return Vec::new();
  };

  ol.children()
    .filter(|n| n.is_element() && n.tag_name().name() == "li")
    .map(|li| parse_nav_li(li, nav_href, with_type))
    .collect()
}

/// 解析一个 li 导航项
///
/// 链接和标签取自第一个 `a` 或 `span`。标签优先用非空文本，否则用 `title`。嵌套 `ol` 递归解析。`with_type` 为真时读取该元素的 epub type。
///
/// # 参数
/// - `li`: `li` 元素
/// - `nav_href`: 解析 `href` 用的基准路径
/// - `with_type`: 为真时填充 `type_props`
///
/// # 返回值
/// 含标签、链接、子项和类型的导航项
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let doc = roxmltree::Document::parse(r#"<li><a href="c.xhtml">章</a></li>"#).unwrap();
/// let li = doc.descendants().find(|n| n.tag_name().name() == "li").unwrap();
/// let item = parse_nav_li(li, "nav.xhtml", false);
/// assert_eq!(item.label, "章");
/// assert_eq!(item.href.as_deref(), Some("c.xhtml"));
/// ```
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

/// 解析一个 NCX 导航节点
///
/// 标签取自子级 `navLabel` 里的 `text`。链接取自子级 `content` 的 `src`。节点本身是 `navPoint` 时，子级 `navPoint` 递归成为子项。
///
/// # 参数
/// - `node`: `navPoint`、`pageTarget` 或 `navTarget`
/// - `ncx_href`: 解析 `src` 用的基准路径
///
/// # 返回值
/// 导航项，`type_props` 为空
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let doc = roxmltree::Document::parse(
///   r#"<navPoint><navLabel><text>章</text></navLabel><content src="c.xhtml"/></navPoint>"#,
/// )
/// .unwrap();
/// let node = doc.descendants().find(|n| n.tag_name().name() == "navPoint").unwrap();
/// let item = parse_ncx_item(node, "toc.ncx");
/// assert_eq!(item.label, "章");
/// assert_eq!(item.href.as_deref(), Some("c.xhtml"));
/// ```
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

/// 按先序把导航树摊平
///
/// 先放入当前项，再递归放入其子项。
///
/// # 参数
/// - `items`: 导航树
///
/// # 返回值
/// 先序排列的导航项副本
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let items = [NavItem {
///   label: "父".to_string(),
///   href: None,
///   subitems: vec![NavItem {
///     label: "子".to_string(),
///     href: None,
///     subitems: Vec::new(),
///     type_props: Vec::new(),
///   }],
///   type_props: Vec::new(),
/// }];
/// let flat = flatten_nav(&items);
/// assert_eq!(flat.len(), 2);
/// assert_eq!(flat[1].label, "子");
/// ```
fn flatten_nav(items: &[NavItem]) -> Vec<NavItem> {
  let mut out = Vec::new();
  for item in items {
    out.push(item.clone());
    out.extend(flatten_nav(&item.subitems));
  }
  out
}

/// 读取元素上的 epub type 列表
///
/// 没有可用属性时返回空列表。
///
/// # 参数
/// - `node`: 要读取的元素
///
/// # 返回值
/// 按空白拆开的 type 值；没有时为空
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let doc = roxmltree::Document::parse(r#"<nav epub:type="toc landmarks"/>"#).unwrap();
/// let nav = doc.descendants().find(|n| n.tag_name().name() == "nav").unwrap();
/// assert_eq!(get_epub_type(nav), vec!["toc".to_string(), "landmarks".to_string()]);
/// ```
fn get_epub_type(node: roxmltree::Node<'_, '_>) -> Vec<String> {
  get_epub_type_attr(node).unwrap_or_default()
}

/// 读取元素上的 epub type 属性
///
/// 查找名字为 `type` 或以 `:type` 结尾的属性，按空白拆成非空值。找到非空列表就返回。
///
/// # 参数
/// - `node`: 要读取的元素
///
/// # 返回值
/// 有非空 type 时返回 `Some`，否则返回 `None`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let doc = roxmltree::Document::parse(r#"<nav epub:type="toc"/>"#).unwrap();
/// let nav = doc.descendants().find(|n| n.tag_name().name() == "nav").unwrap();
/// assert_eq!(get_epub_type_attr(nav), Some(vec!["toc".to_string()]));
/// ```
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
