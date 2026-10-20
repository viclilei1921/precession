//! MOBI / AZW：按 PDB record 解析正文，含 KF8 与 combo。

mod chapter;
mod error;
mod header;
mod index;
mod render;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use regex::Regex;

use crate::book::types::{BookError, BookMeta, Chapter, ChapterRef, Format};

pub use error::MobiError;

use chapter::looks_like_chapter_title;
use header::*;
use render::{clean_mobi_html, contains_renderable_media, extract_body, unescape_html};

/// MOBI 头里表示「该偏移不可用」的哨兵值（缺字段时也会回填成它）
const OFFSET_UNAVAILABLE: u32 = u32::MAX;

/// MOBI 轻量元数据
///
/// # 字段
/// - `title`: 书名。EXTH 503 和头内标题都失败时为「未知标题」
/// - `author`: 作者。没有 EXTH 100 时为空字符串
///
/// # 示例
/// 见 [`parse_metadata`]
pub struct MobiMeta {
  /// 书名。EXTH 503 和头内标题都失败时为「未知标题」
  pub title: String,
  /// 作者。没有 EXTH 100 时为空字符串
  pub author: String,
}

/// MOBI 单章
///
/// # 字段
/// - `title`: 章标题。拆不出章节时为「正文」
/// - `content`: 清洗后的 HTML 正文
///
/// # 示例
/// 见 [`parse_content`]
pub struct MobiChapter {
  /// 章标题。拆不出章节时为「正文」
  pub title: String,
  /// 清洗后的 HTML 正文
  pub content: String,
}

/// MOBI 文本流种类
///
/// # 变体
/// - `Mobi6`: 头版本小于 8
/// - `Kf8`: 头版本大于等于 8，或从 EXTH 121 切到 KF8
///
/// # 示例
/// ```ignore
/// let version = 8u32;
/// let variant = if version >= 8 { MobiVariant::Kf8 } else { MobiVariant::Mobi6 };
/// assert_eq!(variant, MobiVariant::Kf8);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MobiVariant {
  /// 头版本小于 8
  Mobi6,
  /// 头版本大于等于 8，或从 EXTH 121 切到 KF8
  Kf8,
}

/// PDB/MOBI 解析器：持有整文件字节与 record 偏移，支持 MOBI6 与 KF8/combo
///
/// # 字段
/// - `data`: 完整文件内容
/// - `record_offsets`: 各 PDB record 起始偏移（末尾追加文件长度哨兵）
/// - `start`: 当前文本流起始 record 索引（combo 切换到 KF8 boundary 后非 0）
/// - `resource_start`: 资源区起始 record。切换到 KF8 前记下，为 `u32::MAX` 时不可用
/// - `palmdoc`: PalmDOC 头
/// - `mobi`: MOBI 头
/// - `exth`: EXTH 记录表，没有则为空
/// - `kf8`: KF8 头偏移，版本小于 8 时为空
/// - `variant`: 当前文本流是 MOBI6 还是 KF8
///
/// # 示例
/// 见 [`from_bytes`]
struct MobiParser {
  /// 完整文件内容
  data: Vec<u8>,
  /// 各 PDB record 起始偏移（末尾追加文件长度哨兵）
  record_offsets: Vec<u32>,
  /// 当前文本流起始 record 索引（combo 切换到 KF8 boundary 后非 0）
  start: u32,
  /// 资源区起始 record
  resource_start: u32,
  /// PalmDOC 头
  palmdoc: PalmDocHeader,
  /// MOBI 头
  mobi: MobiHeader,
  /// EXTH 记录表，没有则为空
  exth: Option<ExthHeader>,
  /// KF8 头偏移，版本小于 8 时为空
  kf8: Option<Kf8Header>,
  /// 当前文本流是 MOBI6 还是 KF8
  variant: MobiVariant,
}

/// PalmDOC 头
///
/// # 字段
/// - `compression`: 压缩类型。`1` 不压缩，`2` 为 PalmDOC，`17480` 为不支持的 HUFF/CDIC
/// - `num_text_records`: 文本记录数
/// - `_record_size`: 记录大小，读出后未使用
/// - `_encryption`: 加密标志，读出后未使用
///
/// # 示例
/// ```ignore
/// let header = PalmDocHeader { compression: 2, num_text_records: 3, _record_size: 0, _encryption: 0 };
/// assert!(matches!(header.compression, 1 | 2 | 17480));
/// ```
#[derive(Clone)]
struct PalmDocHeader {
  /// 压缩类型。`1` 不压缩，`2` 为 PalmDOC，`17480` 为不支持的 HUFF/CDIC
  compression: u16,
  /// 文本记录数
  num_text_records: u16,
  /// 记录大小，读出后未使用
  _record_size: u16,
  /// 加密标志，读出后未使用
  _encryption: u16,
}

/// MOBI 头
///
/// 记录偏移 16 处必须是 `MOBI`。资源起点和 INDX 缺字段时回填为 `u32::MAX`，EXTH 标志和尾随标志回填为 0。
///
/// # 字段
/// - `length`: 头长度，EXTH 从 `16 + length` 开始
/// - `encoding`: 编码号，用来解码标题和 EXTH 字符串
/// - `version`: 版本。大于等于 8 时按 KF8 处理
/// - `title_offset`: 标题在文本起点记录中的偏移
/// - `title_length`: 标题字节长度
/// - `resource_start`: 资源区起始 record，缺省为 `u32::MAX`
/// - `exth_flag`: EXTH 标志。`0x40` 置位且后面还有数据时解析 EXTH
/// - `trailing_flags`: 文本记录尾随字节标志
/// - `indx`: NCX 索引 record，缺省为 `u32::MAX`
///
/// # 示例
/// 见 [`parse_headers`]
#[derive(Clone)]
struct MobiHeader {
  /// 头长度，EXTH 从 `16 + length` 开始
  length: u32,
  /// 编码号，用来解码标题和 EXTH 字符串
  encoding: u32,
  /// 版本。大于等于 8 时按 KF8 处理
  version: u32,
  /// 标题在文本起点记录中的偏移
  title_offset: u32,
  /// 标题字节长度
  title_length: u32,
  /// 资源区起始 record，缺省为 `u32::MAX`
  resource_start: u32,
  /// EXTH 标志。`0x40` 置位且后面还有数据时解析 EXTH
  exth_flag: u32,
  /// 文本记录尾随字节标志
  trailing_flags: u32,
  /// NCX 索引 record，缺省为 `u32::MAX`
  indx: u32,
}

/// KF8 头偏移
///
/// 只在 MOBI 版本大于等于 8 时读出。三个偏移缺省都是 `0xFFFFFFFF`。
///
/// # 字段
/// - `fdst`: FDST 记录的相对下标
/// - `frag`: fragment 索引的相对下标
/// - `skel`: skeleton 索引的相对下标
///
/// # 示例
/// 见 [`parse_headers`]
#[derive(Clone)]
struct Kf8Header {
  /// FDST 记录的相对下标
  fdst: u32,
  /// fragment 索引的相对下标
  frag: u32,
  /// skeleton 索引的相对下标
  skel: u32,
}

/// EXTH 记录表
///
/// 魔数必须是 `EXTH`，且至少 12 字节。
///
/// # 字段
/// - `records`: 类型和载荷。载荷不含 8 字节记录头；长度小于 8 或超出数据时停止
///
/// # 示例
/// 见 [`parse_exth`]
#[derive(Clone)]
struct ExthHeader {
  /// 类型和载荷。载荷不含 8 字节记录头；长度小于 8 或超出数据时停止
  records: Vec<(u32, Vec<u8>)>,
}

/// INDX 头
///
/// 记录必须以 `INDX` 开头且至少 56 字节。
///
/// # 字段
/// - `length`: 头长度，TAGX 从这里开始
/// - `num_records`: 后续 INDX 记录数
/// - `encoding`: 解码 CNCX 的编码号
/// - `num_cncx`: CNCX 记录数
///
/// # 示例
/// ```ignore
/// let header = IndxHeader { length: 56, num_records: 1, encoding: 0, num_cncx: 0 };
/// let tagx_offset = header.length as usize;
/// assert_eq!(tagx_offset, 56);
/// ```
struct IndxHeader {
  /// 头长度，TAGX 从这里开始
  length: u32,
  /// 后续 INDX 记录数
  num_records: u32,
  /// 解码 CNCX 的编码号
  encoding: u32,
  /// CNCX 记录数
  num_cncx: u32,
}

/// TAGX 头
///
/// 区段必须以 `TAGX` 开头且至少 12 字节。
///
/// # 字段
/// - `length`: 区段长度。标签个数为 `(length - 12) / 4`，不足 12 时为 0
/// - `num_control_bytes`: 控制字节数，名字之后先跳过这些字节
///
/// # 示例
/// ```ignore
/// let tagx = TagxHeader { length: 8, num_control_bytes: 0 };
/// assert_eq!(tagx.length.saturating_sub(12) / 4, 0);
/// ```
struct TagxHeader {
  /// 区段长度。标签个数为 `(length - 12) / 4`，不足 12 时为 0
  length: u32,
  /// 控制字节数，名字之后先跳过这些字节
  num_control_bytes: u32,
}

/// 一条 INDX 的条目表和 CNCX 文本
///
/// # 字段
/// - `table`: 解析出的条目
/// - `cncx`: 记录内偏移到文本。下一条 CNCX 记录的偏移加上 `0x10000`
///
/// # 示例
/// ```ignore
/// let mut cncx = std::collections::HashMap::new();
/// cncx.insert(0u32, "甲".to_string());
/// cncx.insert(0x10000, "乙".to_string());
/// let data = IndexData { table: Vec::new(), cncx };
/// assert!(data.cncx.contains_key(&0x10000));
/// ```
struct IndexData {
  /// 解析出的条目
  table: Vec<IndexEntry>,
  /// 记录内偏移到文本。下一条 CNCX 记录的偏移加上 `0x10000`
  cncx: HashMap<u32, String>,
}

/// NCX 章节锚点
///
/// # 字段
/// - `offset`: tag 1 的字节偏移
/// - `label`: tag 3 指向的 CNCX 文本。空白则用条目名，并还原 HTML 实体
///
/// # 示例
/// ```ignore
/// let mut chapters = [
///   NcxChapter { offset: 8, label: "乙".to_string() },
///   NcxChapter { offset: 0, label: "甲".to_string() },
/// ];
/// chapters.sort_by_key(|c| c.offset);
/// assert_eq!(chapters[0].label, "甲");
/// ```
struct NcxChapter {
  /// tag 1 的字节偏移
  offset: u32,
  /// tag 3 指向的 CNCX 文本。空白则用条目名，并还原 HTML 实体
  label: String,
}

/// INDX 里的一条
///
/// # 字段
/// - `name`: 条目名
/// - `tag_map`: 标签号到变长整数列表
///
/// # 示例
/// ```ignore
/// let mut tag_map = std::collections::HashMap::new();
/// tag_map.insert(1u8, vec![0]);
/// let entry = IndexEntry { name: "章".to_string(), tag_map };
/// assert_eq!(entry.tag_map.get(&1).and_then(|v| v.first()).copied(), Some(0));
/// ```
struct IndexEntry {
  /// 条目名
  name: String,
  /// 标签号到变长整数列表
  tag_map: HashMap<u8, Vec<u32>>,
}

/// KF8 skeleton 索引项
///
/// # 字段
/// - `num_frag`: tag 1，缺省为 0
/// - `offset`: tag 6 的第一个值，缺省为 0
/// - `length`: tag 6 的第二个值，缺省为 0
///
/// # 示例
/// ```ignore
/// let tag6 = [10u32];
/// let skel = Kf8Skeleton {
///   num_frag: 1,
///   offset: tag6.first().copied().unwrap_or(0) as usize,
///   length: tag6.get(1).copied().unwrap_or(0) as usize,
/// };
/// assert_eq!((skel.offset, skel.length), (10, 0));
/// ```
#[derive(Clone)]
struct Kf8Skeleton {
  /// tag 1，缺省为 0
  num_frag: usize,
  /// tag 6 的第一个值，缺省为 0
  offset: usize,
  /// tag 6 的第二个值，缺省为 0
  length: usize,
}

/// KF8 fragment 索引项
///
/// # 字段
/// - `insert_offset`: 条目名解析出的插入位置，解析失败为 0
/// - `offset`: tag 6 的第一个值，缺省为 0
/// - `length`: tag 6 的第二个值，缺省为 0
///
/// # 示例
/// ```ignore
/// let frag = Kf8Fragment { insert_offset: "12".parse().unwrap_or(0), offset: 0, length: 3 };
/// assert_eq!(frag.insert_offset, 12);
/// ```
#[derive(Clone)]
struct Kf8Fragment {
  /// 条目名解析出的插入位置，解析失败为 0
  insert_offset: usize,
  /// tag 6 的第一个值，缺省为 0
  offset: usize,
  /// tag 6 的第二个值，缺省为 0
  length: usize,
}

/// 一节 KF8 骨架及其片段
///
/// # 字段
/// - `skel`: 骨架
/// - `frags`: 这一骨架名下的片段
/// - `length`: 骨架长度加上各片段长度
///
/// # 示例
/// ```ignore
/// let skel = Kf8Skeleton { num_frag: 1, offset: 0, length: 10 };
/// let frag = Kf8Fragment { insert_offset: 0, offset: 0, length: 2 };
/// let section = Kf8Section { length: skel.length + frag.length, skel, frags: vec![frag] };
/// assert_eq!(section.length, 12);
/// ```
#[derive(Clone)]
struct Kf8Section {
  /// 骨架
  skel: Kf8Skeleton,
  /// 这一骨架名下的片段
  frags: Vec<Kf8Fragment>,
  /// 骨架长度加上各片段长度
  length: usize,
}

/// KF8 渲染上下文
///
/// # 字段
/// - `full_raw`: 拼接并解压后的全部文本
/// - `fdst_table`: flow 的起点和终点。FDST 无效时只有 `(0, 全文长度)`
/// - `sections`: 按骨架切出的节。索引为空时补一节覆盖全文
///
/// # 示例
/// 见 [`build_kf8_context`]
struct Kf8Context {
  /// 拼接并解压后的全部文本
  full_raw: Vec<u8>,
  /// flow 的起点和终点。FDST 无效时只有 `(0, 全文长度)`
  fdst_table: Vec<(usize, usize)>,
  /// 按骨架切出的节。索引为空时补一节覆盖全文
  sections: Vec<Kf8Section>,
}

impl ExthHeader {
  /// 读取第一条指定类型的整数记录
  ///
  /// 找到类型匹配且载荷至少 4 字节的记录，把前 4 字节按大端读成整数。
  ///
  /// # 参数
  /// - `record_type`: EXTH 记录类型
  ///
  /// # 返回值
  /// 找到时返回整数，否则返回 `None`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let raw = b"EXTH\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x64\x00\x00\x00\x0c\x00\x00\x00\x2a";
  /// let exth = MobiParser::parse_exth(raw).unwrap();
  /// assert_eq!(exth.get_first_u32(100), Some(42));
  /// ```
  fn get_first_u32(&self, record_type: u32) -> Option<u32> {
    self.records.iter().find_map(|(tid, payload)| {
      if *tid == record_type && payload.len() >= 4 { Some(read_u32_be(payload)) } else { None }
    })
  }

  /// 读取第一条指定类型的字符串记录
  ///
  /// 找到类型匹配的记录后按给定编码解码。解码失败则继续找下一条，全部失败时返回 `None`。
  ///
  /// # 参数
  /// - `record_type`: EXTH 记录类型
  /// - `encoding`: 传给解码器的编码号
  ///
  /// # 返回值
  /// 解码成功时返回字符串，否则返回 `None`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let raw = b"EXTH\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x64\x00\x00\x00\x0bAnn";
  /// let exth = MobiParser::parse_exth(raw).unwrap();
  /// assert_eq!(exth.get_first_string(100, 65001).as_deref(), Some("Ann"));
  /// ```
  fn get_first_string(&self, record_type: u32, encoding: u32) -> Option<String> {
    self.records.iter().find_map(
      |(tid, payload)| {
        if *tid == record_type { decode_exth_string(payload, encoding).ok() } else { None }
      },
    )
  }
}

impl MobiParser {
  /// 从文件路径构造解析器
  ///
  /// 打开文件并读入全部字节，再交给字节解析。
  ///
  /// # 参数
  /// - `path`: MOBI 文件路径
  ///
  /// # 返回值
  /// 成功返回解析器
  ///
  /// # 错误
  /// 打开、读取或字节解析失败时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let parser = MobiParser::from_path(Path::new("book.mobi")).unwrap();
  /// assert_eq!(parser.get_title(), parser.get_title());
  /// ```
  fn from_path(path: &Path) -> Result<Self, MobiError> {
    let mut f = File::open(path).map_err(|e| MobiError(format!("打开文件失败: {}", e)))?;
    let mut data = Vec::new();
    f.read_to_end(&mut data).map_err(|e| MobiError(format!("读取失败: {}", e)))?;
    Self::from_bytes(&data)
  }

  /// 从 PDB 字节构造解析器
  ///
  /// 短于 78 字节，或 record 表放不下时失败。读取 record 数量和各记录偏移，并在末尾加上文件长度。解析 0 号记录的头。若当前是 MOBI6，且 EXTH 121 指向的记录能解析成 KF8，则改用该记录的头并把文本起点移到那里。
  ///
  /// # 参数
  /// - `data`: 整个 PDB 文件的字节
  ///
  /// # 返回值
  /// 成功返回持有文件副本的解析器
  ///
  /// # 错误
  /// 文件过短、record 表不完整或 0 号记录头无效时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert!(MobiParser::from_bytes(b"short").is_err());
  /// ```
  fn from_bytes(data: &[u8]) -> Result<Self, MobiError> {
    if data.len() < 78 {
      return Err(MobiError("文件过短，非有效 PDB".into()));
    }
    let num_records = read_u16_be(&data[76..78]) as usize;
    if data.len() < 78 + num_records * 8 {
      return Err(MobiError("PDB record 表不完整".into()));
    }

    let mut offsets = Vec::with_capacity(num_records + 1);
    for i in 0..num_records {
      let off = 78 + i * 8;
      offsets.push(read_u32_be(&data[off..off + 4]));
    }
    offsets.push(data.len() as u32);

    let rec0 = Self::load_record_data(data, &offsets, 0)?;
    let (mut palmdoc, mut mobi, mut exth, mut kf8) = Self::parse_headers(rec0)?;
    let primary_resource_start = mobi.resource_start;
    let mut variant = if mobi.version >= 8 { MobiVariant::Kf8 } else { MobiVariant::Mobi6 };
    let mut start = 0u32;

    if variant == MobiVariant::Mobi6 {
      if let Some(boundary) = exth.as_ref().and_then(|x| x.get_first_u32(121)) {
        if boundary < 0xFFFF_FFFF && (boundary as usize) < num_records {
          if let Ok(boundary_rec) = Self::load_record_data(data, &offsets, boundary as usize) {
            if let Ok((new_palmdoc, new_mobi, new_exth, new_kf8)) = Self::parse_headers(boundary_rec) {
              if new_mobi.version >= 8 {
                tauri_plugin_log::log::info!("[mobi] 检测到 combo MOBI/KF8，切换到 boundary={}", boundary);
                palmdoc = new_palmdoc;
                mobi = new_mobi;
                exth = new_exth;
                kf8 = new_kf8;
                start = boundary;
                variant = MobiVariant::Kf8;
              }
            }
          }
        }
      }
    }

    let parser = Self {
      data: data.to_vec(),
      record_offsets: offsets,
      start,
      resource_start: primary_resource_start,
      palmdoc,
      mobi,
      exth,
      kf8,
      variant,
    };

    tauri_plugin_log::log::info!(
      "[mobi] init: variant={:?}, start={}, num_records={}, palmdoc_text_records={}, mobi_version={}, primary_resource_start={}, active_resource_start={}",
      parser.variant,
      parser.start,
      parser.record_offsets.len().saturating_sub(1),
      parser.palmdoc.num_text_records,
      parser.mobi.version,
      parser.resource_start,
      parser.mobi.resource_start
    );

    Ok(parser)
  }

  /// 解析一条记录里的 PalmDOC、MOBI、EXTH 和 KF8 头
  ///
  /// 记录短于 132 字节，或偏移 16 处不是 `MOBI` 时失败。EXTH 标志的 `0x40` 置位且后面还有超过 12 字节时解析 EXTH。版本大于等于 8 时读取 FDST、fragment、skeleton 偏移，缺省为 `0xFFFFFFFF`。
  ///
  /// # 参数
  /// - `record`: 含文件头的 PDB 记录
  ///
  /// # 返回值
  /// 成功返回四类头，EXTH 和 KF8 可能为空
  ///
  /// # 错误
  /// 记录过短、缺少 MOBI 头或 EXTH 无效时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert!(MobiParser::parse_headers(&[0; 10]).is_err());
  /// ```
  fn parse_headers(
    record: &[u8],
  ) -> Result<(PalmDocHeader, MobiHeader, Option<ExthHeader>, Option<Kf8Header>), MobiError> {
    if record.len() < 132 {
      return Err(MobiError("MOBI record 头信息过短".into()));
    }

    let palmdoc = PalmDocHeader {
      compression: read_u16_be(&record[0..2]),
      num_text_records: read_u16_be(&record[8..10]),
      _record_size: read_u16_be(&record[10..12]),
      _encryption: read_u16_be(&record[12..14]),
    };

    let mobi_magic = std::str::from_utf8(&record[16..20]).unwrap_or("");
    if mobi_magic != "MOBI" {
      return Err(MobiError("缺少 MOBI 头".into()));
    }

    let mobi = MobiHeader {
      length: read_u32_be(&record[20..24]),
      encoding: read_u32_be(&record[28..32]),
      version: read_u32_be(&record[36..40]),
      title_offset: read_u32_be(&record[84..88]),
      title_length: read_u32_be(&record[88..92]),
      resource_start: read_u32_be_at(record, 108).unwrap_or(OFFSET_UNAVAILABLE),
      exth_flag: read_u32_be_at(record, 128).unwrap_or(0),
      trailing_flags: read_u32_be_at(record, 240).unwrap_or(0),
      indx: read_u32_be_at(record, 244).unwrap_or(OFFSET_UNAVAILABLE),
    };

    let exth = if mobi.exth_flag & 0x40 != 0 {
      let exth_start = 16 + mobi.length as usize;
      if record.len() > exth_start + 12 { Some(Self::parse_exth(&record[exth_start..])?) } else { None }
    } else {
      None
    };

    let kf8 = if mobi.version >= 8 {
      Some(Kf8Header {
        fdst: read_u32_be_at(record, 192).unwrap_or(0xFFFF_FFFF),
        frag: read_u32_be_at(record, 248).unwrap_or(0xFFFF_FFFF),
        skel: read_u32_be_at(record, 252).unwrap_or(0xFFFF_FFFF),
      })
    } else {
      None
    };

    Ok((palmdoc, mobi, exth, kf8))
  }

  /// 按 record 偏移表切出一条记录
  ///
  /// 起点取 `offsets[index]`，终点取下一条偏移；没有下一条时用数据长度。终点不超过数据长度。
  ///
  /// # 参数
  /// - `data`: 整个文件字节
  /// - `offsets`: record 起始偏移，通常末尾还有文件长度
  /// - `index`: 从 0 开始的 record 下标
  ///
  /// # 返回值
  /// 成功返回该记录的子切片
  ///
  /// # 错误
  /// 下标越界、起点超出文件或起点大于终点时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// assert_eq!(MobiParser::load_record_data(b"abcd", &[0, 4], 0).unwrap(), b"abcd");
  /// assert!(MobiParser::load_record_data(b"abcd", &[0, 4], 3).is_err());
  /// ```
  fn load_record_data<'a>(data: &'a [u8], offsets: &[u32], index: usize) -> Result<&'a [u8], MobiError> {
    let start = offsets
      .get(index)
      .copied()
      .ok_or_else(|| MobiError(format!("record {} 越界 (total={})", index, offsets.len())))? as usize;
    let end = offsets.get(index + 1).copied().unwrap_or(data.len() as u32) as usize;

    if start >= data.len() {
      return Err(MobiError(format!(
        "record {} 超出文件范围 (offset={:#x}, file_size={:#x})",
        index,
        start,
        data.len()
      )));
    }

    let end = end.min(data.len());
    if start > end {
      return Err(MobiError(format!(
        "record {} 偏移倒置 (start={:#x} > end={:#x})",
        index, start, end
      )));
    }

    Ok(&data[start..end])
  }

  /// 按文本流内的相对下标加载记录
  ///
  /// 绝对下标是文本起点 `start` 加上 `index`。
  ///
  /// # 参数
  /// - `index`: 相对文本起点的 record 下标
  ///
  /// # 返回值
  /// 成功返回该记录的字节副本
  ///
  /// # 错误
  /// 绝对下标对应的记录无法加载时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// assert!(parser.load_record(parser.record_offsets.len()).is_err());
  /// ```
  fn load_record(&self, index: usize) -> Result<Vec<u8>, MobiError> {
    let absolute = self.start as usize + index;
    self.load_record_absolute(absolute)
  }

  /// 按绝对 PDB record 下标加载记录
  ///
  /// # 参数
  /// - `absolute`: 从文件 record 表 0 开始的下标
  ///
  /// # 返回值
  /// 成功返回该记录的字节副本
  ///
  /// # 错误
  /// 下标越界或偏移无效时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// assert!(parser.load_record_absolute(parser.record_offsets.len()).is_err());
  /// ```
  fn load_record_absolute(&self, absolute: usize) -> Result<Vec<u8>, MobiError> {
    let slice = Self::load_record_data(&self.data, &self.record_offsets, absolute)?;
    Ok(slice.to_vec())
  }

  /// 按资源序号加载图片或字体等记录
  ///
  /// `resource_start` 为哨兵值时失败。绝对下标是资源区起点加上 `resource_index`。
  ///
  /// # 参数
  /// - `resource_index`: 从 0 开始的资源序号
  ///
  /// # 返回值
  /// 成功返回资源字节
  ///
  /// # 错误
  /// 资源区起点不可用，或绝对记录无法加载时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// if parser.resource_start == u32::MAX {
  ///   assert!(parser.load_resource(0).is_err());
  /// }
  /// ```
  fn load_resource(&self, resource_index: usize) -> Result<Vec<u8>, MobiError> {
    if self.resource_start == OFFSET_UNAVAILABLE {
      return Err(MobiError("resource_start 不可用".into()));
    }
    let absolute = self.resource_start as usize + resource_index;
    tauri_plugin_log::log::info!(
      "[mobi] load_resource: variant={:?}, start={}, resource_start={}, resource_index={}, absolute_record={}, total_records={}",
      self.variant,
      self.start,
      self.resource_start,
      resource_index,
      absolute,
      self.record_offsets.len().saturating_sub(1)
    );
    self.load_record_absolute(absolute)
  }

  /// 按 MOBI 尾随标志去掉记录末尾的附加字节
  ///
  /// 先按标志里除最低位外每个置位去掉一段变长尾。最低位表示多字节尾时，再按最后一字节低 2 位加 1 截掉，长度超出数据则不截。
  ///
  /// # 参数
  /// - `data`: 原始记录字节
  ///
  /// # 返回值
  /// 去掉尾随字节后的数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let raw = vec![1, 2, 3, 0x81];
  /// let trimmed = parser.remove_trailing(raw.clone());
  /// assert!(trimmed.len() <= raw.len());
  /// ```
  fn remove_trailing(&self, mut data: Vec<u8>) -> Vec<u8> {
    let flags = self.mobi.trailing_flags;
    let multibyte = flags & 1 != 0;
    let num_trailing = (flags >> 1).count_ones();

    for _ in 0..num_trailing {
      let len = get_var_len_from_end(&data) as usize;
      if len > 0 && len <= data.len() {
        data.truncate(data.len() - len);
      }
    }
    if multibyte && !data.is_empty() {
      let n = ((data[data.len() - 1] & 3) + 1) as usize;
      if n <= data.len() {
        data.truncate(data.len() - n);
      }
    }
    data
  }

  /// 加载并解压一条文本记录
  ///
  /// 读取 `index + 1` 号相对记录并去掉尾随字节。压缩类型 1 原样返回，2 做 PalmDOC 解压，17480 报不支持 HUFF/CDIC，其他类型报不支持。
  ///
  /// # 参数
  /// - `index`: 从 0 开始的文本记录序号
  ///
  /// # 返回值
  /// 成功返回解压后的字节
  ///
  /// # 错误
  /// 记录加载失败或不支持该压缩类型时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// if parser.palmdoc.compression == 17480 {
  ///   assert!(parser.load_text(0).is_err());
  /// }
  /// ```
  fn load_text(&self, index: usize) -> Result<Vec<u8>, MobiError> {
    let raw = self.load_record(index + 1)?;
    let trimmed = self.remove_trailing(raw);
    match self.palmdoc.compression {
      1 => Ok(trimmed),
      2 => Ok(decompress_palmdoc(&trimmed)),
      17480 => Err(MobiError("暂不支持 HUFF/CDIC 压缩的 MOBI".into())),
      _ => Err(MobiError(format!("不支持的压缩类型 {}", self.palmdoc.compression))),
    }
  }

  /// 按顺序拼接全部文本记录
  ///
  /// # 返回值
  /// 成功返回拼接后的正文字节
  ///
  /// # 错误
  /// 任一文本记录加载或解压失败时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let raw = parser.load_all_text_bytes().unwrap();
  /// assert_eq!(raw, parser.load_all_text_bytes().unwrap());
  /// ```
  fn load_all_text_bytes(&self) -> Result<Vec<u8>, MobiError> {
    let mut buf = Vec::new();
    for i in 0..self.palmdoc.num_text_records as usize {
      buf.extend_from_slice(&self.load_text(i)?);
    }
    Ok(buf)
  }

  /// 解析 EXTH 记录表
  ///
  /// 数据短于 12 字节，或前 4 字节不是 `EXTH` 时失败。按偏移 8 的数量逐条读取类型和长度；长度小于 8 或超出数据时停止。载荷是每条头部之后的字节。
  ///
  /// # 参数
  /// - `data`: 从 EXTH 魔数开始的字节
  ///
  /// # 返回值
  /// 成功返回记录列表，数量不足时列表可能短于声明的条数
  ///
  /// # 错误
  /// 数据过短或魔数不对时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```
  /// let exth = MobiParser::parse_exth(b"EXTH\x00\x00\x00\x00\x00\x00\x00\x00").unwrap();
  /// assert!(exth.get_first_u32(100).is_none());
  /// assert!(MobiParser::parse_exth(b"XXXX").is_err());
  /// ```
  fn parse_exth(data: &[u8]) -> Result<ExthHeader, MobiError> {
    if data.len() < 12 {
      return Err(MobiError("EXTH 过短".into()));
    }
    let magic = std::str::from_utf8(&data[0..4]).unwrap_or("");
    if magic != "EXTH" {
      return Err(MobiError("无效 EXTH".into()));
    }

    let count = read_u32_be(&data[8..12]) as usize;
    let mut records = Vec::new();
    let mut offset = 12usize;
    for _ in 0..count {
      if offset + 8 > data.len() {
        break;
      }
      let rec_type = read_u32_be(&data[offset..offset + 4]);
      let rec_len = read_u32_be(&data[offset + 4..offset + 8]) as usize;
      if rec_len < 8 || offset + rec_len > data.len() {
        break;
      }
      records.push((rec_type, data[offset + 8..offset + rec_len].to_vec()));
      offset += rec_len;
    }
    Ok(ExthHeader { records })
  }

  /// 解析一条 INDX 及其 CNCX、条目表
  ///
  /// 记录必须以 `INDX` 开头且至少 56 字节。读取 TAGX 和控制字节数，收集 CNCX 里的变长字符串，再逐条解析后续 INDX 记录中的名字和 tag 值。标签表、名字或变长值越界时跳过对应片段。
  ///
  /// # 参数
  /// - `indx_index`: 相对文本起点的 INDX 记录下标
  ///
  /// # 返回值
  /// 成功返回条目表和 CNCX 文本
  ///
  /// # 错误
  /// 记录无效、缺少 TAGX 或加载记录失败时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// assert!(parser.get_index_data(parser.record_offsets.len()).is_err());
  /// ```
  fn get_index_data(&self, indx_index: usize) -> Result<IndexData, MobiError> {
    let record = self.load_record(indx_index)?;
    if record.len() < 56 || &record[0..4] != b"INDX" {
      return Err(MobiError(format!("INDX record {} 无效", indx_index)));
    }

    let header = IndxHeader {
      length: read_u32_be(&record[4..8]),
      num_records: read_u32_be(&record[24..28]),
      encoding: read_u32_be(&record[28..32]),
      num_cncx: read_u32_be(&record[52..56]),
    };

    let tagx_offset = header.length as usize;
    let tagx_buf = record.get(tagx_offset..).ok_or_else(|| MobiError(format!("INDX {} 缺少 TAGX 区段", indx_index)))?;
    if tagx_buf.len() < 12 || &tagx_buf[0..4] != b"TAGX" {
      return Err(MobiError(format!("INDX {} TAGX 无效", indx_index)));
    }

    let tagx = TagxHeader { length: read_u32_be(&tagx_buf[4..8]), num_control_bytes: read_u32_be(&tagx_buf[8..12]) };
    let num_tags = tagx.length.saturating_sub(12) as usize / 4;
    let tag_table: Vec<[u8; 4]> = (0..num_tags)
      .filter_map(|i| {
        let start = 12 + i * 4;
        let chunk = tagx_buf.get(start..start + 4)?;
        Some([chunk[0], chunk[1], chunk[2], chunk[3]])
      })
      .collect();

    let decoder = get_decoder(header.encoding);
    let mut cncx = HashMap::new();
    let mut cncx_record_offset = 0u32;
    for i in 0..header.num_cncx as usize {
      let cncx_record = self.load_record(indx_index + header.num_records as usize + i + 1)?;
      let mut pos = 0usize;
      while pos < cncx_record.len() {
        let index = pos as u32;
        let (value, length) = match get_var_len(&cncx_record, pos) {
          Some(x) => x,
          None => break,
        };
        pos += length;
        let value = value as usize;
        if pos + value > cncx_record.len() {
          break;
        }
        let (text, _, _) = decoder.decode(&cncx_record[pos..pos + value]);
        cncx.insert(cncx_record_offset + index, text.into_owned());
        pos += value;
      }
      cncx_record_offset += 0x10000;
    }

    let mut table = Vec::new();
    for i in 0..header.num_records as usize {
      let record = self.load_record(indx_index + 1 + i)?;
      if record.len() < 28 || &record[0..4] != b"INDX" {
        continue;
      }

      let entry_count = read_u32_be(&record[24..28]) as usize;
      let idxt = read_u32_be(&record[20..24]) as usize;
      for j in 0..entry_count {
        let offset_offset = idxt + 4 + 2 * j;
        let Some(offset) = read_u16_be_at(&record, offset_offset).map(|x| x as usize) else {
          continue;
        };
        let Some(name_len) = record.get(offset).copied().map(usize::from) else {
          continue;
        };
        let Some(name_bytes) = record.get(offset + 1..offset + 1 + name_len) else {
          continue;
        };
        let name = String::from_utf8_lossy(name_bytes).into_owned();

        let start_pos = offset + 1 + name_len;
        let mut control_byte_index = 0usize;
        let mut pos = start_pos + tagx.num_control_bytes as usize;
        let mut tag_defs = Vec::<(u8, Option<u32>, Option<u32>, u8)>::new();

        for [tag, num_values, mask, end] in &tag_table {
          if end & 1 != 0 {
            control_byte_index += 1;
            continue;
          }
          let Some(control) = record.get(start_pos + control_byte_index).copied() else {
            continue;
          };
          let value = control & mask;
          if value == *mask {
            if count_bits_set(*mask) > 1 {
              if let Some((value, length)) = get_var_len(&record, pos) {
                tag_defs.push((*tag, None, Some(value), *num_values));
                pos += length;
              }
            } else {
              tag_defs.push((*tag, Some(1), None, *num_values));
            }
          } else {
            let shifted = (value >> count_unset_end(*mask)) as u32;
            tag_defs.push((*tag, Some(shifted), None, *num_values));
          }
        }

        let mut tag_map = HashMap::new();
        for (tag, value_count, value_bytes, num_values) in tag_defs {
          let mut values = Vec::new();
          if let Some(value_count) = value_count {
            for _ in 0..(value_count * u32::from(num_values)) {
              let Some((value, length)) = get_var_len(&record, pos) else {
                break;
              };
              values.push(value);
              pos += length;
            }
          } else if let Some(value_bytes) = value_bytes {
            let mut read = 0u32;
            while read < value_bytes {
              let Some((value, length)) = get_var_len(&record, pos) else {
                break;
              };
              values.push(value);
              pos += length;
              read += length as u32;
            }
          }
          tag_map.insert(tag, values);
        }

        table.push(IndexEntry { name, tag_map });
      }
    }

    Ok(IndexData { table, cncx })
  }

  /// 从 NCX 索引取出章节偏移和标题
  ///
  /// 索引不可用时返回空列表。条目需要 tag 1 的偏移和 tag 3 的标签号；标签优先取 CNCX 文本并还原 HTML 实体，否则用条目名。结果按偏移排序并去重。若存在多于一条 0 级标题，则改返回这些顶层项。
  ///
  /// # 返回值
  /// 成功返回按偏移排序的章节锚点
  ///
  /// # 错误
  /// 读取索引失败时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// if parser.mobi.indx == u32::MAX {
  ///   assert!(parser.get_ncx_chapters().unwrap().is_empty());
  /// }
  /// ```
  fn get_ncx_chapters(&self) -> Result<Vec<NcxChapter>, MobiError> {
    if self.mobi.indx == OFFSET_UNAVAILABLE {
      return Ok(Vec::new());
    }

    let index_data = self.get_index_data(self.mobi.indx as usize)?;
    let mut chapters: Vec<NcxChapter> = index_data
      .table
      .iter()
      .filter_map(|entry| {
        let offset = entry.tag_map.get(&1)?.first().copied()?;
        let label_idx = entry.tag_map.get(&3)?.first().copied()?;
        let label = index_data
          .cncx
          .get(&label_idx)
          .cloned()
          .filter(|s| !s.trim().is_empty())
          .map(|s| unescape_html(s.trim()))
          .unwrap_or_else(|| entry.name.clone());
        Some(NcxChapter { offset, label })
      })
      .collect();

    chapters.sort_by_key(|c| c.offset);
    chapters.dedup_by_key(|c| c.offset);

    // 优先使用顶层目录项（headingLevel == 0），避免子条目过多
    let top_level: Vec<NcxChapter> = index_data
      .table
      .iter()
      .filter_map(|entry| {
        let heading = entry.tag_map.get(&4)?.first().copied()?;
        if heading != 0 {
          return None;
        }
        let offset = entry.tag_map.get(&1)?.first().copied()?;
        let label_idx = entry.tag_map.get(&3)?.first().copied()?;
        let label = index_data
          .cncx
          .get(&label_idx)
          .cloned()
          .filter(|s| !s.trim().is_empty())
          .map(|s| unescape_html(s.trim()))
          .unwrap_or_else(|| entry.name.clone());
        Some(NcxChapter { offset, label })
      })
      .collect();

    if top_level.len() > 1 {
      let mut sorted = top_level;
      sorted.sort_by_key(|c| c.offset);
      sorted.dedup_by_key(|c| c.offset);
      Ok(sorted)
    } else {
      Ok(chapters)
    }
  }

  /// 按 pagebreak 标签把 HTML 拆成章节
  ///
  /// 清理每段后，跳过空白，以及去掉标签后没有文字且没有媒体的段。标题取该段第一个 `h1` 到 `h6` 的文本，没有则用 `第 {n} 章`。
  ///
  /// # 参数
  /// - `html`: 已渲染的 HTML
  ///
  /// # 返回值
  /// 按 pagebreak 切开的章节，可能为空
  ///
  /// # Panics
  /// 正则编译失败时 `unwrap` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// let chapters = parser.split_by_pagebreaks("<h1>甲</h1>a<mbp:pagebreak/>b");
  /// assert_eq!(chapters[0].title, "甲");
  /// assert_eq!(chapters[1].title, "第 2 章");
  /// ```
  fn split_by_pagebreaks(&self, html: &str) -> Vec<MobiChapter> {
    let pagebreak_re = Regex::new(r"(?i)<\s*(?:mbp:)?pagebreak[^>]*/?\s*>").unwrap();
    let heading_re = Regex::new(r"(?is)<h[1-6][^>]*>(.*?)</h[1-6]>").unwrap();
    let tag_strip_re = Regex::new(r"(?is)<[^>]+>").unwrap();

    let mut chapters = Vec::new();
    for section in pagebreak_re.split(html) {
      let content = clean_mobi_html(section);
      let trimmed = content.trim();
      if trimmed.is_empty() {
        continue;
      }

      let text_only = tag_strip_re.replace_all(trimmed, "");
      let has_media = contains_renderable_media(trimmed);
      if text_only.trim().is_empty() && !has_media {
        continue;
      }

      let title = heading_re
        .captures(section)
        .and_then(|c| c.get(1))
        .map(|m| tag_strip_re.replace_all(m.as_str(), "").into_owned())
        .map(|s| unescape_html(s.trim()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("第 {} 章", chapters.len() + 1));

      chapters.push(MobiChapter { title, content: trimmed.to_string() });
    }
    chapters
  }

  /// 按 NCX 字节偏移把原文拆成章节
  ///
  /// 起点超出原文，或起点不小于终点的条目被跳过。每段解码后取 body 并清理，正文为空则跳过。标题为空时用 `第 {n} 章`。
  ///
  /// # 参数
  /// - `raw`: 解压后的正文字节
  /// - `ncx`: 带字节偏移和标题的锚点
  ///
  /// # 返回值
  /// 成功返回切出的章节
  ///
  /// # 错误
  /// 当前实现不会返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let chapters = parser.split_by_ncx(b"", &[]).unwrap();
  /// assert!(chapters.is_empty());
  /// ```
  fn split_by_ncx(&self, raw: &[u8], ncx: &[NcxChapter]) -> Result<Vec<MobiChapter>, MobiError> {
    let mut chapters = Vec::new();
    for (i, entry) in ncx.iter().enumerate() {
      let start = entry.offset as usize;
      if start >= raw.len() {
        continue;
      }
      let end = ncx.get(i + 1).map(|next| next.offset as usize).unwrap_or(raw.len()).min(raw.len());
      if start >= end {
        continue;
      }

      let html = self.decode_buffer(&raw[start..end]);
      let body = extract_body(&html);
      let content = clean_mobi_html(&body);
      if content.trim().is_empty() {
        continue;
      }

      let title = if entry.label.is_empty() { format!("第 {} 章", chapters.len() + 1) } else { entry.label.clone() };

      chapters.push(MobiChapter { title, content });
    }
    Ok(chapters)
  }

  /// 按连续换行之间的短标题拆成章节
  ///
  /// 用正则寻找 `<br>` 夹着的 1 到 20 个字符，并且该文本要像章节标题。命中不超过 1 处时返回空列表。各段清理后，没有文字也没有媒体的段被跳过。
  ///
  /// # 参数
  /// - `html`: 已渲染的 HTML
  ///
  /// # 返回值
  /// 拆出的章节；结构不匹配时为空
  ///
  /// # Panics
  /// 正则编译失败时 `unwrap` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// assert!(parser.split_by_br_titles("<p>没有标题</p>").is_empty());
  /// ```
  fn split_by_br_titles(&self, html: &str) -> Vec<MobiChapter> {
    let boundary_re = Regex::new(
      r"(?i)(?:^|<body>|(?:<br\s*/?\s*>\s*(?:&nbsp;|\s)*){2})<br\s*/?\s*>\s*([^<&\n\r]{1,20}?)\s*<br\s*/?\s*>",
    )
    .unwrap();
    let tag_strip_re = Regex::new(r"(?is)<[^>]+>").unwrap();

    let boundaries: Vec<(usize, String)> = boundary_re
      .captures_iter(html)
      .filter_map(|caps| {
        let m = caps.get(0)?;
        let title = caps.get(1)?.as_str().trim();
        if title.is_empty() || !looks_like_chapter_title(title) {
          return None;
        }
        Some((m.start(), unescape_html(title)))
      })
      .collect();

    if boundaries.len() <= 1 {
      return Vec::new();
    }

    let mut chapters = Vec::new();
    for (i, (start, title)) in boundaries.iter().enumerate() {
      let end = boundaries.get(i + 1).map(|(pos, _)| *pos).unwrap_or(html.len());
      let section = &html[*start..end];
      let content = clean_mobi_html(section);
      if content.trim().is_empty() {
        continue;
      }
      let text_only = tag_strip_re.replace_all(content.trim(), "");
      if text_only.trim().is_empty() && !contains_renderable_media(&content) {
        continue;
      }
      chapters.push(MobiChapter { title: title.clone(), content });
    }
    chapters
  }

  /// 组装 KF8 的全文、FDST 表和分节
  ///
  /// 没有 KF8 头时失败。FDST 有效且记录以 `FDST` 开头时读取起止偏移，否则整段正文算一条。skeleton 和 fragment 索引合成各节；一节都没有时用整段正文补一节。
  ///
  /// # 返回值
  /// 成功返回 KF8 渲染上下文
  ///
  /// # 错误
  /// 当前不是 KF8，或加载正文、索引失败时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// if parser.kf8.is_none() {
  ///   assert!(parser.build_kf8_context().is_err());
  /// }
  /// ```
  fn build_kf8_context(&self) -> Result<Kf8Context, MobiError> {
    let kf8 = self.kf8.as_ref().ok_or_else(|| MobiError("当前书籍不是 KF8".into()))?;
    let full_raw = self.load_all_text_bytes()?;

    let fdst_table = if kf8.fdst < 0xFFFF_FFFF {
      let fdst_record = self.load_record(kf8.fdst as usize)?;
      if fdst_record.len() >= 12 && &fdst_record[0..4] == b"FDST" {
        let num_entries = read_u32_be(&fdst_record[8..12]) as usize;
        (0..num_entries)
          .filter_map(|i| {
            let offset = 12 + i * 8;
            let start = read_u32_be_at(&fdst_record, offset)? as usize;
            let end = read_u32_be_at(&fdst_record, offset + 4)? as usize;
            Some((start.min(full_raw.len()), end.min(full_raw.len())))
          })
          .collect()
      } else {
        vec![(0, full_raw.len())]
      }
    } else {
      vec![(0, full_raw.len())]
    };

    let skel_index = kf8.skel as usize;
    let frag_index = kf8.frag as usize;
    let skel_data = self.get_index_data(skel_index)?;
    let frag_data = self.get_index_data(frag_index)?;

    let skel_table: Vec<Kf8Skeleton> = skel_data
      .table
      .into_iter()
      .map(|entry| Kf8Skeleton {
        num_frag: entry.tag_map.get(&1).and_then(|v| v.first()).copied().unwrap_or(0) as usize,
        offset: entry.tag_map.get(&6).and_then(|v| v.first()).copied().unwrap_or(0) as usize,
        length: entry.tag_map.get(&6).and_then(|v| v.get(1)).copied().unwrap_or(0) as usize,
      })
      .collect();

    let frag_table: Vec<Kf8Fragment> = frag_data
      .table
      .into_iter()
      .map(|entry| Kf8Fragment {
        insert_offset: entry.name.parse::<usize>().unwrap_or(0),
        offset: entry.tag_map.get(&6).and_then(|v| v.first()).copied().unwrap_or(0) as usize,
        length: entry.tag_map.get(&6).and_then(|v| v.get(1)).copied().unwrap_or(0) as usize,
      })
      .collect();

    let mut sections = Vec::new();
    let mut frag_cursor = 0usize;
    for skel in skel_table {
      let frag_end = (frag_cursor + skel.num_frag).min(frag_table.len());
      let frags = frag_table[frag_cursor..frag_end].to_vec();
      let length = skel.length + frags.iter().map(|frag| frag.length).sum::<usize>();
      sections.push(Kf8Section { skel, frags, length });
      frag_cursor = frag_end;
    }

    if sections.is_empty() {
      sections.push(Kf8Section {
        skel: Kf8Skeleton { num_frag: 0, offset: 0, length: full_raw.len() },
        frags: Vec::new(),
        length: full_raw.len(),
      });
    }

    Ok(Kf8Context { full_raw, fdst_table, sections })
  }

  /// 按 FDST 下标取出一段 flow 字节
  ///
  /// 下标不存在、起点大于终点，或终点超出全文字节时返回 `None`。
  ///
  /// # 参数
  /// - `ctx`: 已建好的 KF8 上下文
  /// - `index`: FDST 表下标
  ///
  /// # 返回值
  /// 找到且范围有效时返回子切片
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// if let Ok(ctx) = parser.build_kf8_context() {
  ///   assert!(parser.load_flow(&ctx, usize::MAX).is_none());
  /// }
  /// ```
  fn load_flow<'a>(&self, ctx: &'a Kf8Context, index: usize) -> Option<&'a [u8]> {
    let (start, end) = *ctx.fdst_table.get(index)?;
    if start > end || end > ctx.full_raw.len() {
      return None;
    }
    Some(&ctx.full_raw[start..end])
  }

  /// 把一节 skeleton 和 fragment 拼成文本
  ///
  /// 按 skeleton 偏移和本节长度从全文截取，先放 skeleton，再把各 fragment 插到相对偏移处，最后按 MOBI 编码解码。
  ///
  /// # 参数
  /// - `ctx`: KF8 上下文
  /// - `section`: 要渲染的一节
  ///
  /// # 返回值
  /// 解码后的本节文本
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// if let Ok(ctx) = parser.build_kf8_context() {
  ///   let _text = parser.render_kf8_section(&ctx, &ctx.sections[0]);
  /// }
  /// ```
  fn render_kf8_section(&self, ctx: &Kf8Context, section: &Kf8Section) -> String {
    let start = section.skel.offset.min(ctx.full_raw.len());
    let end = (section.skel.offset + section.length).min(ctx.full_raw.len());
    let raw = &ctx.full_raw[start..end];

    let skel_len = section.skel.length.min(raw.len());
    let mut skeleton = raw[..skel_len].to_vec();
    for frag in &section.frags {
      let insert_offset = frag.insert_offset.saturating_sub(section.skel.offset).min(skeleton.len());
      let offset = skel_len + frag.offset;
      let end = (offset + frag.length).min(raw.len());
      if offset >= end {
        continue;
      }
      let frag_raw = &raw[offset..end];
      let mut combined = Vec::with_capacity(skeleton.len() + frag_raw.len());
      combined.extend_from_slice(&skeleton[..insert_offset]);
      combined.extend_from_slice(frag_raw);
      combined.extend_from_slice(&skeleton[insert_offset..]);
      skeleton = combined;
    }
    self.decode_buffer(&skeleton)
  }

  /// 按文件头里的编码把字节解码成字符串
  ///
  /// # 参数
  /// - `data`: 待解码字节
  ///
  /// # 返回值
  /// 解码后的字符串
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// assert_eq!(parser.decode_buffer(b""), "");
  /// ```
  fn decode_buffer(&self, data: &[u8]) -> String {
    let (text, _, _) = get_decoder(self.mobi.encoding).decode(data);
    text.into_owned()
  }

  /// 把 Kindle 资源读成 data URI
  ///
  /// `depth` 大于 4 时返回 `None`。资源号按 32 进制解析。`flow` 从 FDST 取字节，其他类型在序号为 0 时返回 `None`，否则加载 `序号 - 1` 的资源。MIME 用参数、图片探测或 `application/octet-stream`。文本类资源会再替换其中的 kindle URI。
  ///
  /// # 参数
  /// - `resource_type`: `flow` 或其他资源类型
  /// - `encoded_id`: 32 进制资源号
  /// - `mime`: 调用方给出的 MIME，可空
  /// - `ctx`: KF8 上下文
  /// - `depth`: 当前递归深度
  ///
  /// # 返回值
  /// 成功返回 data URI，资源不存在或深度超限时返回 `None`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// if let Ok(ctx) = parser.build_kf8_context() {
  ///   assert!(parser.load_kindle_resource_data_uri("flow", "1", None, &ctx, 5).is_none());
  /// }
  /// ```
  fn load_kindle_resource_data_uri(
    &self,
    resource_type: &str,
    encoded_id: &str,
    mime: Option<&str>,
    ctx: &Kf8Context,
    depth: usize,
  ) -> Option<String> {
    if depth > 4 {
      return None;
    }

    let id = usize::from_str_radix(encoded_id, 32).ok()?;
    let raw = if resource_type.eq_ignore_ascii_case("flow") {
      self.load_flow(ctx, id)?.to_vec()
    } else {
      if id == 0 {
        return None;
      }
      self.load_resource(id - 1).ok()?
    };

    let mime = mime.filter(|x| !x.is_empty()).or_else(|| detect_image_mime(&raw)).unwrap_or("application/octet-stream");

    if is_textual_resource_mime(mime) {
      let nested = self.replace_kindle_uris(&self.decode_buffer(&raw), ctx, depth + 1);
      return Some(make_data_uri(mime, nested.as_bytes()));
    }

    Some(make_data_uri(mime, &raw))
  }

  /// 把 `kindle:flow` 和 `kindle:embed` 换成 data URI
  ///
  /// 能加载资源时写入 URI，否则保留原来的匹配文本。
  ///
  /// # 参数
  /// - `html`: 含 kindle URI 的 HTML
  /// - `ctx`: KF8 上下文
  /// - `depth`: 传给资源加载的递归深度
  ///
  /// # 返回值
  /// 替换后的 HTML
  ///
  /// # Panics
  /// 正则编译失败时 `unwrap` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// if let Ok(ctx) = parser.build_kf8_context() {
  ///   assert_eq!(parser.replace_kindle_uris("hello", &ctx, 0), "hello");
  /// }
  /// ```
  fn replace_kindle_uris(&self, html: &str, ctx: &Kf8Context, depth: usize) -> String {
    let re = Regex::new(r#"kindle:(flow|embed):(\w+)(?:\?mime=([\w/+.-]+))?"#).unwrap();
    let mut result = String::with_capacity(html.len());
    let mut last_end = 0usize;
    for caps in re.captures_iter(html) {
      let Some(full) = caps.get(0) else {
        continue;
      };
      result.push_str(&html[last_end..full.start()]);
      let resource_type = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
      let encoded_id = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
      let mime = caps.get(3).map(|m| m.as_str());
      if let Some(uri) = self.load_kindle_resource_data_uri(resource_type, encoded_id, mime, ctx, depth) {
        result.push_str(&uri);
      } else {
        result.push_str(full.as_str());
      }
      last_end = full.end();
    }
    result.push_str(&html[last_end..]);
    result
  }

  /// 把带 `recindex` 的 img 换成图片 data URI
  ///
  /// 序号要大于 0。加载对应资源并识别出图片后，去掉原来的 `recindex` 和 `src`，写入新的 `src`。失败时丢掉整个标签并写警告日志。
  ///
  /// # 参数
  /// - `html`: 含 recindex 图片的 HTML
  ///
  /// # 返回值
  /// 替换后的 HTML
  ///
  /// # Panics
  /// 正则编译失败时 `unwrap` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// assert_eq!(parser.replace_recindex_images("hello"), "hello");
  /// ```
  fn replace_recindex_images(&self, html: &str) -> String {
    let re = Regex::new(r#"(?is)<img\b[^>]*?\brecindex\s*=\s*["']?(\d+)["']?[^>]*?/?>"#).unwrap();
    let recindex_attr_re = Regex::new(r#"(?i)\s+recindex\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)"#).unwrap();
    let src_attr_re = Regex::new(r#"(?i)\s+src\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)"#).unwrap();

    let mut result = String::with_capacity(html.len());
    let mut last_end = 0usize;
    for caps in re.captures_iter(html) {
      let Some(full) = caps.get(0) else {
        continue;
      };
      result.push_str(&html[last_end..full.start()]);
      let idx = caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok()).filter(|idx| *idx > 0);

      let replacement = idx.and_then(|idx| {
        let raw = match self.load_resource(idx - 1) {
          Ok(raw) => raw,
          Err(err) => {
            tauri_plugin_log::log::warn!("[mobi] recindex={} 加载资源失败: {}", idx, err);
            return None;
          }
        };
        let mime = match detect_image_mime(&raw) {
          Some(mime) => mime,
          None => {
            tauri_plugin_log::log::warn!(
              "[mobi] recindex={} 资源不是可识别图片，magic={:02X?}",
              idx,
              &raw[..raw.len().min(8)]
            );
            return None;
          }
        };
        let data_uri = make_data_uri(mime, &raw);
        let mut tag = full.as_str().to_string();
        tag = recindex_attr_re.replace_all(&tag, "").into_owned();
        tag = src_attr_re.replace_all(&tag, "").into_owned();
        let is_self_closing = tag.trim_end().ends_with("/>");
        if let Some(insert_at) = tag.rfind(if is_self_closing { "/>" } else { ">" }) {
          let closing = if is_self_closing { " />" } else { ">" };
          tag.replace_range(insert_at.., &format!(r#" src="{}"{}"#, data_uri, closing));
          Some(tag)
        } else {
          None
        }
      });

      if let Some(replacement) = replacement {
        result.push_str(&replacement);
      } else {
        tauri_plugin_log::log::warn!("[mobi] recindex 图片替换失败，移除原始标签: {}", full.as_str());
      }
      last_end = full.end();
    }
    result.push_str(&html[last_end..]);
    result
  }

  /// 把带 `mediarecindex` 的音视频标签换成 data URI
  ///
  /// 处理 `audio`、`video`、`source`。媒体序号加载为 `src`，标签上的 `recindex` 若能识别成图片则写成 `poster`。失败时保留原标签。
  ///
  /// # 参数
  /// - `html`: 含 mediarecindex 的 HTML
  ///
  /// # 返回值
  /// 替换后的 HTML
  ///
  /// # Panics
  /// 正则编译失败时 `unwrap` 会 panic
  ///
  /// # 示例
  /// ```ignore
  /// assert_eq!(parser.replace_mediarecindex_tags("hello"), "hello");
  /// ```
  fn replace_mediarecindex_tags(&self, html: &str) -> String {
    let re = Regex::new(r#"(?is)<(audio|video|source)\b[^>]*?\bmediarecindex\s*=\s*["']?(\d+)["']?[^>]*?/?>"#).unwrap();
    let media_attr_re = Regex::new(r#"(?i)\s+mediarecindex\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)"#).unwrap();
    let recindex_attr_re = Regex::new(r#"(?i)\s+recindex\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)"#).unwrap();
    let src_attr_re = Regex::new(r#"(?i)\s+src\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)"#).unwrap();
    let poster_attr_re = Regex::new(r#"(?i)\s+poster\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)"#).unwrap();
    let recindex_capture_re = Regex::new(r#"(?i)\brecindex\s*=\s*["']?(\d+)["']?"#).unwrap();

    let mut result = String::with_capacity(html.len());
    let mut last_end = 0usize;
    for caps in re.captures_iter(html) {
      let Some(full) = caps.get(0) else {
        continue;
      };
      result.push_str(&html[last_end..full.start()]);

      let media_idx = caps.get(2).and_then(|m| m.as_str().parse::<usize>().ok()).filter(|idx| *idx > 0);
      let poster_idx = recindex_capture_re
        .captures(full.as_str())
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<usize>().ok())
        .filter(|idx| *idx > 0);

      let replacement = media_idx.and_then(|media_idx| {
        let media_raw = self.load_resource(media_idx - 1).ok()?;
        let media_mime = detect_image_mime(&media_raw).unwrap_or("application/octet-stream");
        let media_uri = make_data_uri(media_mime, &media_raw);

        let poster_uri = poster_idx.and_then(|idx| {
          let raw = self.load_resource(idx - 1).ok()?;
          let mime = detect_image_mime(&raw)?;
          Some(make_data_uri(mime, &raw))
        });

        let mut tag = full.as_str().to_string();
        tag = media_attr_re.replace_all(&tag, "").into_owned();
        tag = recindex_attr_re.replace_all(&tag, "").into_owned();
        tag = src_attr_re.replace_all(&tag, "").into_owned();
        tag = poster_attr_re.replace_all(&tag, "").into_owned();
        let is_self_closing = tag.trim_end().ends_with("/>");
        let closing = if is_self_closing { " />" } else { ">" };
        let mut attrs = format!(r#" src="{}""#, media_uri);
        if let Some(uri) = poster_uri {
          attrs.push_str(&format!(r#" poster="{}""#, uri));
        }
        if let Some(insert_at) = tag.rfind(if is_self_closing { "/>" } else { ">" }) {
          tag.replace_range(insert_at.., &format!("{}{}", attrs, closing));
          Some(tag)
        } else {
          None
        }
      });

      result.push_str(replacement.as_deref().unwrap_or(full.as_str()));
      last_end = full.end();
    }
    result.push_str(&html[last_end..]);
    result
  }

  /// 替换 MOBI6 正文里的图片和音视频资源
  ///
  /// 先处理 `recindex` 图片，再处理 `mediarecindex` 标签。
  ///
  /// # 参数
  /// - `html`: 正文 HTML
  ///
  /// # 返回值
  /// 两步替换后的 HTML
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// assert_eq!(parser.replace_mobi6_resources("hello"), "hello");
  /// ```
  fn replace_mobi6_resources(&self, html: &str) -> String {
    let with_images = self.replace_recindex_images(html);
    self.replace_mediarecindex_tags(&with_images)
  }

  /// 解码全部 MOBI6 文本记录
  ///
  /// # 返回值
  /// 成功返回按头里编码解码后的字符串
  ///
  /// # 错误
  /// 加载文本记录失败时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let html = parser.raw_html_mobi6().unwrap();
  /// assert_eq!(html, parser.decode_buffer(&parser.load_all_text_bytes().unwrap()));
  /// ```
  fn raw_html_mobi6(&self) -> Result<String, MobiError> {
    Ok(self.decode_buffer(&self.load_all_text_bytes()?))
  }

  /// 按变体渲染整本 HTML
  ///
  /// MOBI6 解码全文、取 body，再替换 recindex 资源。KF8 先建上下文，失败时退回全文解码并同样替换资源。成功时逐节渲染，body 为空则保留原文本，各节都空时使用全文；最后再替换资源并处理 kindle URI。
  ///
  /// # 返回值
  /// 成功返回渲染后的 HTML
  ///
  /// # 错误
  /// MOBI6 加载文本失败，或 KF8 回退时加载文本失败，返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let html = parser.render_html().unwrap();
  /// assert_eq!(html, parser.render_html().unwrap());
  /// ```
  fn render_html(&self) -> Result<String, MobiError> {
    match self.variant {
      MobiVariant::Mobi6 => {
        let html = self.raw_html_mobi6()?;
        let body = extract_body(&html);
        Ok(self.replace_mobi6_resources(&body))
      }
      MobiVariant::Kf8 => {
        let ctx = match self.build_kf8_context() {
          Ok(ctx) => ctx,
          Err(err) => {
            tauri_plugin_log::log::warn!("[mobi] KF8 上下文构建失败，回退到纯文本拼接: {}", err);
            let html = self.decode_buffer(&self.load_all_text_bytes()?);
            return Ok(self.replace_mobi6_resources(&extract_body(&html)));
          }
        };

        let mut parts = Vec::new();
        for section in &ctx.sections {
          let raw = self.render_kf8_section(&ctx, section);
          let body = extract_body(&raw);
          if !body.trim().is_empty() {
            parts.push(body);
          } else if !raw.trim().is_empty() {
            parts.push(raw);
          }
        }
        if parts.is_empty() {
          parts.push(self.decode_buffer(&ctx.full_raw));
        }

        let joined = parts.join("\n");
        let joined = self.replace_mobi6_resources(&joined);
        Ok(self.replace_kindle_uris(&joined, &ctx, 0))
      }
    }
  }

  /// 读取书名
  ///
  /// 优先用 EXTH 503，去掉空白后非空则还原 HTML 实体。否则按 MOBI 头里的标题偏移和长度，从文本起点那条记录解码。都失败时返回「未知标题」。
  ///
  /// # 返回值
  /// 书名；记录里的空标题会原样返回
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// assert_eq!(parser.get_title(), parser.get_title());
  /// ```
  fn get_title(&self) -> String {
    if let Some(title) = self
      .exth
      .as_ref()
      .and_then(|x| x.get_first_string(503, self.mobi.encoding))
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
    {
      return unescape_html(&title);
    }

    if let Ok(record) = Self::load_record_data(&self.data, &self.record_offsets, self.start as usize) {
      let start = self.mobi.title_offset as usize;
      let end = start + self.mobi.title_length as usize;
      if let Some(raw) = record.get(start..end) {
        if let Ok(title) = decode_bytes(raw, self.mobi.encoding) {
          return unescape_html(&title);
        }
      }
    }

    "未知标题".to_string()
  }

  /// 读取作者
  ///
  /// 取 EXTH 100，去掉空白并还原 HTML 实体。没有该记录时返回空字符串。
  ///
  /// # 返回值
  /// 作者名，没有时为空字符串
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// assert_eq!(parser.get_author(), parser.get_author());
  /// ```
  fn get_author(&self) -> String {
    self
      .exth
      .as_ref()
      .and_then(|x| x.get_first_string(100, self.mobi.encoding))
      .map(|s| unescape_html(s.trim()))
      .unwrap_or_default()
  }

  /// 读取封面图片字节和 MIME
  ///
  /// 资源区起点不可用时返回 `None`。优先用 EXTH 201，没有或是哨兵值时用 202。加载该资源序号并识别图片类型。
  ///
  /// # 返回值
  /// 识别到封面时返回字节和 MIME，否则返回 `None`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// if parser.resource_start == u32::MAX {
  ///   assert!(parser.get_cover().is_none());
  /// }
  /// ```
  fn get_cover(&self) -> Option<(Vec<u8>, &'static str)> {
    if self.resource_start == OFFSET_UNAVAILABLE {
      return None;
    }
    let exth = self.exth.as_ref()?;
    let offset = exth
      .get_first_u32(201)
      .filter(|x| *x != OFFSET_UNAVAILABLE)
      .or_else(|| exth.get_first_u32(202).filter(|x| *x != OFFSET_UNAVAILABLE))? as usize;
    let raw = self.load_resource(offset).ok()?;
    let mime = detect_image_mime(&raw)?;
    Some((raw, mime))
  }

  /// 把渲染后的 HTML 拆成章节
  ///
  /// 先按 pagebreak 拆，多于一章就采用。否则在索引可用时按 NCX 字节偏移拆。仍不多于一章时，再按换行标题拆。最后若一章都没有，则把清理后的全文作为「正文」。
  ///
  /// # 返回值
  /// 成功返回至少一章
  ///
  /// # 错误
  /// 渲染 HTML 失败时返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let chapters = parser.extract_chapters().unwrap();
  /// assert!(!chapters.is_empty());
  /// ```
  fn extract_chapters(&self) -> Result<Vec<MobiChapter>, MobiError> {
    let html = self.render_html()?;
    tauri_plugin_log::log::info!("[mobi] variant={:?}, rendered_html 长度={}", self.variant, html.len());

    // Tier 1: pagebreak 拆分
    let mut chapters = self.split_by_pagebreaks(&html);
    if chapters.len() > 1 {
      tauri_plugin_log::log::info!("[mobi] 按 pagebreak 拆分为 {} 章", chapters.len());
      return Ok(chapters);
    }

    // Tier 2: NCX 索引拆分（字节偏移）
    if self.mobi.indx < 0xFFFF_FFFF {
      match (self.get_ncx_chapters(), self.load_all_text_bytes()) {
        (Ok(ncx), Ok(raw)) if ncx.len() > 1 => {
          chapters = self.split_by_ncx(&raw, &ncx)?;
          if chapters.len() > 1 {
            tauri_plugin_log::log::info!("[mobi] 按 NCX 拆分为 {} 章", chapters.len());
            return Ok(chapters);
          }
        }
        (Err(err), _) | (_, Err(err)) => {
          tauri_plugin_log::log::warn!("[mobi] NCX 章节拆分失败: {}", err);
        }
        _ => {}
      }
    }

    // Tier 3: 旧版 MOBI 的 br 标题结构
    chapters = self.split_by_br_titles(&html);
    if chapters.len() > 1 {
      tauri_plugin_log::log::info!("[mobi] 按 br 标题拆分为 {} 章", chapters.len());
      return Ok(chapters);
    }

    // 最终回退：整本书作为单章
    if chapters.is_empty() {
      chapters.push(MobiChapter { title: "正文".to_string(), content: clean_mobi_html(&html) });
    }

    Ok(chapters)
  }
}

/// 已打开的 MOBI：打开时整本解析并缓存各章正文（无按章流式能力）
///
/// # 字段
/// - `meta`: 书名和作者，格式为 [`Format::Mobi`]
/// - `chapters`: 打开时整本解析好的章节，下标从 0 编号
///
/// # 示例
/// 见 [`open`]
pub struct MobiBook {
  /// 书名和作者，格式为 [`Format::Mobi`]
  meta: BookMeta,
  /// 打开时整本解析好的章节，下标从 0 编号
  chapters: Vec<Chapter>,
}

impl MobiBook {
  /// 打开 MOBI 并缓存全部分章结果
  ///
  /// 解析元数据和章节后，按顺序从 0 给每章编号。
  ///
  /// # 参数
  /// - `path`: MOBI 文件路径
  ///
  /// # 返回值
  /// 成功返回已缓存章节的 `MobiBook`
  ///
  /// # 错误
  /// 解析失败时返回 `BookError::Invalid`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = MobiBook::open(Path::new("book.mobi")).unwrap();
  /// assert_eq!(book.metadata().format, Format::Mobi);
  /// ```
  pub fn open(path: &Path) -> Result<Self, BookError> {
    let (meta, chapters) = parse_content(path).map_err(BookError::invalid)?;
    Ok(Self {
      meta: BookMeta { title: meta.title, author: meta.author, format: Format::Mobi },
      chapters: chapters
        .into_iter()
        .enumerate()
        .map(|(i, c)| Chapter { index: i, title: c.title, content: c.content })
        .collect(),
    })
  }

  /// 返回已缓存的书籍元数据
  ///
  /// # 返回值
  /// 打开时解析出的元数据
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = MobiBook::open(Path::new("book.mobi")).unwrap();
  /// let _meta = book.metadata();
  /// ```
  pub fn metadata(&self) -> &BookMeta {
    &self.meta
  }

  /// 返回不含正文的章节目录
  ///
  /// # 返回值
  /// 每章的索引和标题
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = MobiBook::open(Path::new("book.mobi")).unwrap();
  /// let outline = book.outline();
  /// assert_eq!(outline[0].index, 0);
  /// ```
  pub fn outline(&self) -> Vec<ChapterRef> {
    self.chapters.iter().map(|c| ChapterRef { index: c.index, title: c.title.clone() }).collect()
  }

  /// 按索引克隆一章
  ///
  /// # 参数
  /// - `index`: 从 0 开始的章节下标
  ///
  /// # 返回值
  /// 成功返回该章
  ///
  /// # 错误
  /// 下标超出章节列表时返回 `BookError::ChapterOutOfRange`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = MobiBook::open(Path::new("book.mobi")).unwrap();
  /// let chapter = book.chapter(0).unwrap();
  /// assert_eq!(chapter.index, 0);
  /// ```
  pub fn chapter(&self, index: usize) -> Result<Chapter, BookError> {
    self.chapters.get(index).cloned().ok_or(BookError::ChapterOutOfRange)
  }

  /// 交出缓存的元数据和全部章节
  ///
  /// # 返回值
  /// 始终返回 `Ok`，内容是打开时解析好的元数据和章节
  ///
  /// # 错误
  /// 当前实现不会返回 `Err`
  ///
  /// # Panics
  /// 不会主动 panic
  ///
  /// # 示例
  /// ```ignore
  /// let book = MobiBook::open(Path::new("book.mobi")).unwrap();
  /// let (_meta, chapters) = book.content().unwrap();
  /// assert!(!chapters.is_empty());
  /// ```
  pub fn content(self) -> Result<(BookMeta, Vec<Chapter>), BookError> {
    Ok((self.meta, self.chapters))
  }
}

/// 解析 MOBI 的书名和作者
///
/// # 参数
/// - `path`: MOBI 文件路径
///
/// # 返回值
/// 成功返回书名和作者
///
/// # 错误
/// 打开或解析文件失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let meta = parse_metadata(Path::new("book.mobi")).unwrap();
/// assert_eq!(meta.title, MobiParser::from_path(Path::new("book.mobi")).unwrap().get_title());
/// ```
pub fn parse_metadata(path: &Path) -> Result<MobiMeta, MobiError> {
  let p = MobiParser::from_path(path)?;
  Ok(MobiMeta { title: p.get_title(), author: p.get_author() })
}

/// 解析 MOBI 的元数据和全部章节
///
/// # 参数
/// - `path`: MOBI 文件路径
///
/// # 返回值
/// 成功返回书名、作者和章节
///
/// # 错误
/// 打开文件或拆章失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// let (_meta, chapters) = parse_content(Path::new("book.mobi")).unwrap();
/// assert!(!chapters.is_empty());
/// ```
pub fn parse_content(path: &Path) -> Result<(MobiMeta, Vec<MobiChapter>), MobiError> {
  let p = MobiParser::from_path(path)?;
  let meta = MobiMeta { title: p.get_title(), author: p.get_author() };
  let chapters = p.extract_chapters()?;
  Ok((meta, chapters))
}

/// 提取封面图片的原始字节和 MIME
///
/// # 参数
/// - `path`: MOBI 文件路径
///
/// # 返回值
/// 成功时，有封面返回字节和 MIME，没有封面返回 `None`
///
/// # 错误
/// 打开或解析文件失败时返回 `Err`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// match extract_cover(Path::new("book.mobi")).unwrap() {
///   Some((bytes, _)) => assert!(bytes.len() >= 4),
///   None => {}
/// }
/// ```
pub fn extract_cover(path: &Path) -> Result<Option<(Vec<u8>, &'static str)>, MobiError> {
  let p = MobiParser::from_path(path)?;
  Ok(p.get_cover())
}
