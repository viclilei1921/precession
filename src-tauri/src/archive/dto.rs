use serde::Serialize;

/// 当前档案在磁盘上的位置和文件用量
///
/// 用量只计 `upload` 里的正式文件，不含导入临时目录。
///
/// # 字段
/// - `path`: 数据库文件的绝对路径
/// - `file_count`: 正式文件个数
/// - `file_bytes`: 正式文件的字节数
///
/// # 示例
/// ```
/// let info = ArchiveInfo { path: "data.sqlite".to_string(), file_count: 1, file_bytes: 4 };
/// assert_eq!(info.file_count, 1);
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveInfo {
  /// 数据库文件的绝对路径
  pub path: String,
  /// 正式文件个数
  pub file_count: u64,
  /// 正式文件的字节数
  pub file_bytes: u64,
}

/// 导出完成后的压缩包
///
/// # 字段
/// - `path`: 用户选择的保存路径
/// - `file_count`: 打进包里的正式文件个数
/// - `bytes`: 压缩包字节数
///
/// # 示例
/// ```
/// let packed = ArchiveExport { path: "archive.zip".to_string(), file_count: 2, bytes: 100 };
/// assert_eq!(packed.file_count, 2);
/// ```
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveExport {
  /// 用户选择的保存路径
  pub path: String,
  /// 打进包里的正式文件个数
  pub file_count: u64,
  /// 压缩包字节数
  pub bytes: u64,
}
