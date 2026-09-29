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
pub struct MobiMeta {
  pub title: String,
  pub author: String,
}

/// MOBI 单章
pub struct MobiChapter {
  pub title: String,
  pub content: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MobiVariant {
  Mobi6,
  Kf8,
}

/// PDB/MOBI 解析器：持有整文件字节与 record 偏移，支持 MOBI6 与 KF8/combo
struct MobiParser {
  /// 完整文件内容
  data: Vec<u8>,
  /// 各 PDB record 起始偏移（末尾追加文件长度哨兵）
  record_offsets: Vec<u32>,
  /// 当前文本流起始 record 索引（combo 切换到 KF8 boundary 后非 0）
  start: u32,
  /// 资源区起始 record
  resource_start: u32,
  palmdoc: PalmDocHeader,
  mobi: MobiHeader,
  exth: Option<ExthHeader>,
  kf8: Option<Kf8Header>,
  variant: MobiVariant,
}

#[derive(Clone)]
struct PalmDocHeader {
  compression: u16,
  num_text_records: u16,
  _record_size: u16,
  _encryption: u16,
}

#[derive(Clone)]
struct MobiHeader {
  length: u32,
  encoding: u32,
  version: u32,
  title_offset: u32,
  title_length: u32,
  resource_start: u32,
  exth_flag: u32,
  trailing_flags: u32,
  indx: u32,
}

#[derive(Clone)]
struct Kf8Header {
  fdst: u32,
  frag: u32,
  skel: u32,
}

#[derive(Clone)]
struct ExthHeader {
  records: Vec<(u32, Vec<u8>)>,
}

struct IndxHeader {
  length: u32,
  num_records: u32,
  encoding: u32,
  num_cncx: u32,
}

struct TagxHeader {
  length: u32,
  num_control_bytes: u32,
}

struct IndexData {
  table: Vec<IndexEntry>,
  cncx: HashMap<u32, String>,
}

struct NcxChapter {
  offset: u32,
  label: String,
}

struct IndexEntry {
  name: String,
  tag_map: HashMap<u8, Vec<u32>>,
}

#[derive(Clone)]
struct Kf8Skeleton {
  num_frag: usize,
  offset: usize,
  length: usize,
}

#[derive(Clone)]
struct Kf8Fragment {
  insert_offset: usize,
  offset: usize,
  length: usize,
}

#[derive(Clone)]
struct Kf8Section {
  skel: Kf8Skeleton,
  frags: Vec<Kf8Fragment>,
  length: usize,
}

struct Kf8Context {
  full_raw: Vec<u8>,
  fdst_table: Vec<(usize, usize)>,
  sections: Vec<Kf8Section>,
}

impl ExthHeader {
  fn get_first_u32(&self, record_type: u32) -> Option<u32> {
    self.records.iter().find_map(|(tid, payload)| {
      if *tid == record_type && payload.len() >= 4 { Some(read_u32_be(payload)) } else { None }
    })
  }

  fn get_first_string(&self, record_type: u32, encoding: u32) -> Option<String> {
    self.records.iter().find_map(
      |(tid, payload)| {
        if *tid == record_type { decode_exth_string(payload, encoding).ok() } else { None }
      },
    )
  }
}

impl MobiParser {
  fn from_path(path: &Path) -> Result<Self, MobiError> {
    let mut f = File::open(path).map_err(|e| MobiError(format!("打开文件失败: {}", e)))?;
    let mut data = Vec::new();
    f.read_to_end(&mut data).map_err(|e| MobiError(format!("读取失败: {}", e)))?;
    Self::from_bytes(&data)
  }

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

  fn load_record(&self, index: usize) -> Result<Vec<u8>, MobiError> {
    let absolute = self.start as usize + index;
    self.load_record_absolute(absolute)
  }

  fn load_record_absolute(&self, absolute: usize) -> Result<Vec<u8>, MobiError> {
    let slice = Self::load_record_data(&self.data, &self.record_offsets, absolute)?;
    Ok(slice.to_vec())
  }

  /// 按绝对 PDB record 索引加载资源（图片、字体等），resource_index 为 0-based 资源编号。
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

  fn load_all_text_bytes(&self) -> Result<Vec<u8>, MobiError> {
    let mut buf = Vec::new();
    for i in 0..self.palmdoc.num_text_records as usize {
      buf.extend_from_slice(&self.load_text(i)?);
    }
    Ok(buf)
  }

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

  /// 从 INDX/NCX 索引提取章节锚点（字节偏移 + 标题）
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

  /// 旧版 MOBI 无 pagebreak/NCX 时，按 `<br> 标题 <br>` 结构拆分
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

  fn load_flow<'a>(&self, ctx: &'a Kf8Context, index: usize) -> Option<&'a [u8]> {
    let (start, end) = *ctx.fdst_table.get(index)?;
    if start > end || end > ctx.full_raw.len() {
      return None;
    }
    Some(&ctx.full_raw[start..end])
  }

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

  fn decode_buffer(&self, data: &[u8]) -> String {
    let (text, _, _) = get_decoder(self.mobi.encoding).decode(data);
    text.into_owned()
  }

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

  fn replace_mobi6_resources(&self, html: &str) -> String {
    let with_images = self.replace_recindex_images(html);
    self.replace_mediarecindex_tags(&with_images)
  }

  fn raw_html_mobi6(&self) -> Result<String, MobiError> {
    Ok(self.decode_buffer(&self.load_all_text_bytes()?))
  }

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

  fn get_author(&self) -> String {
    self
      .exth
      .as_ref()
      .and_then(|x| x.get_first_string(100, self.mobi.encoding))
      .map(|s| unescape_html(s.trim()))
      .unwrap_or_default()
  }

  /// 从 EXTH coverOffset (201) 或 thumbnailOffset (202) 读取封面图片原始字节
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
pub struct MobiBook {
  meta: BookMeta,
  chapters: Vec<Chapter>,
}

impl MobiBook {
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

  pub fn metadata(&self) -> &BookMeta {
    &self.meta
  }

  pub fn outline(&self) -> Vec<ChapterRef> {
    self.chapters.iter().map(|c| ChapterRef { index: c.index, title: c.title.clone() }).collect()
  }

  pub fn chapter(&self, index: usize) -> Result<Chapter, BookError> {
    self.chapters.get(index).cloned().ok_or(BookError::ChapterOutOfRange)
  }

  pub fn content(self) -> Result<(BookMeta, Vec<Chapter>), BookError> {
    Ok((self.meta, self.chapters))
  }
}

pub fn parse_metadata(path: &Path) -> Result<MobiMeta, MobiError> {
  let p = MobiParser::from_path(path)?;
  Ok(MobiMeta { title: p.get_title(), author: p.get_author() })
}

pub fn parse_content(path: &Path) -> Result<(MobiMeta, Vec<MobiChapter>), MobiError> {
  let p = MobiParser::from_path(path)?;
  let meta = MobiMeta { title: p.get_title(), author: p.get_author() };
  let chapters = p.extract_chapters()?;
  Ok((meta, chapters))
}

/// 提取封面图片原始字节及 MIME 类型，用于保存为文件
pub fn extract_cover(path: &Path) -> Result<Option<(Vec<u8>, &'static str)>, MobiError> {
  let p = MobiParser::from_path(path)?;
  Ok(p.get_cover())
}
