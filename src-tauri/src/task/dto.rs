use serde::{Deserialize, Serialize};

use crate::ffmpeg::models::{TimeSegment, VideoEncode};
use crate::image::models::{AvifEncodeParams, JxlEncodeParams};

/// 失败或取消后用来还原界面
///
/// 密码不会出现在这里。线上用 `kind` 区分变体，字段名是 camelCase。
///
/// # 变体
/// - `EncryptFile`: 加密文件，载荷是输入和输出路径
/// - `DecryptFile`: 解密文件，载荷是输入和输出路径
/// - `ConvertVideo`: 转换视频
/// - `CutVideo`: 按片段裁剪视频
/// - `MergeVideo`: 合并多段视频
/// - `AppendVideo`: 把多段视频追加到基准片
/// - `ConvertAvif`: 转成 AVIF
/// - `ConvertJxl`: 转成 JPEG XL
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TaskDraft {
  /// 加密文件，载荷是输入和输出路径
  EncryptFile { input: String, output: String },
  /// 解密文件，载荷是输入和输出路径
  DecryptFile { input: String, output: String },
  /// 转换视频
  ConvertVideo { input: String, output: String, encode: VideoEncode },
  /// 按片段裁剪视频
  CutVideo { input: String, output: String, segments: Vec<TimeSegment>, encode: VideoEncode },
  /// 合并多段视频
  MergeVideo { inputs: Vec<String>, output: String, encode: VideoEncode },
  /// 把多段视频追加到基准片
  AppendVideo { base: String, inputs: Vec<String>, output: String, encode: VideoEncode },
  /// 转成 AVIF
  ConvertAvif { input: String, output: String, params: AvifEncodeParams },
  /// 转成 JPEG XL
  ConvertJxl { input: String, output: String, params: JxlEncodeParams },
}

/// 队列里的一条任务
///
/// 密码不会出现在这里。线上字段名是 camelCase。
///
/// # 字段
/// - `id`: 任务 ID
/// - `kind`: 任务类型，`encryptFile`、`decryptFile`、`importUpload`、`encryptUpload`、`decryptUpload`、`convertVideo`、`cutVideo`、`mergeVideo`、`appendVideo`、`convertAvif`、`convertJxl`
/// - `status`: 任务状态，`pending`、`processing`、`completed`、`failed`、`canceled`
/// - `progress`: 任务进度，0.0 到 1.0
/// - `message`: 任务消息。任务处理中时才有
/// - `upload_id`: 上传文件 ID。上传文件任务时才有
/// - `title`: 文件名。多文件时带上数量
/// - `output`: 输出文件。空则没有可打开的结果。任务完成时才有
/// - `draft`: 任务草稿。任务取消时才有
///
/// # 示例
/// 见 [`task_enqueue`](super::commands::task_enqueue)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
  /// 任务 ID
  pub id: String,
  /// 任务类型: encryptFile, decryptFile, importUpload, encryptUpload, decryptUpload, convertVideo, cutVideo, mergeVideo, appendVideo, convertAvif, convertJxl
  pub kind: String,
  /// 任务状态: pending, processing, completed, failed, canceled
  pub status: String,
  /// 任务进度: 0.0 - 1.0
  pub progress: f64,
  /// 任务消息。任务处理中时才有。
  pub message: String,
  /// 上传文件 ID。上传文件任务时才有。
  pub upload_id: Option<String>,
  /// 文件名。多文件时带上数量。
  pub title: String,
  /// 输出文件。空则没有可打开的结果。任务完成时才有。
  pub output: String,
  /// 任务草稿。任务取消时才有。
  pub draft: Option<TaskDraft>,
}

/// 入队参数
///
/// 密码只在这一次传入。线上用 `kind` 区分变体，变体名和字段名都是 camelCase。视频编码和图片参数可以省略，省略时用该类型的默认值。
///
/// # 变体
/// - `EncryptFile`: 加密文件，载荷含密码
/// - `DecryptFile`: 解密文件，载荷含密码
/// - `ImportUpload`: 导入上传文件，载荷含所属记录和是否加密
/// - `EncryptUpload`: 加密已上传文件
/// - `DecryptUpload`: 解密已上传文件
/// - `ConvertVideo`: 转换视频
/// - `CutVideo`: 按片段裁剪视频
/// - `MergeVideo`: 合并多段视频
/// - `AppendVideo`: 把多段视频追加到基准片
/// - `ConvertAvif`: 转成 AVIF
/// - `ConvertJxl`: 转成 JPEG XL
///
/// # 示例
/// ```
/// let input = TaskInput::EncryptFile {
///   input: "a.bin".into(),
///   output: "a.bin.enc".into(),
///   password: "secret".into(),
/// };
/// assert!(matches!(input, TaskInput::EncryptFile { .. }));
/// ```
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TaskInput {
  /// 加密文件
  EncryptFile { input: String, output: String, password: String },
  /// 解密文件
  DecryptFile { input: String, output: String, password: String },
  /// 导入上传文件
  ImportUpload { source: String, owner: String, owner_id: String, upload_kind: String, encrypt: bool, password: String },
  /// 加密上传文件
  EncryptUpload { upload_id: String, password: String },
  /// 解密上传文件
  DecryptUpload { upload_id: String, password: String },
  /// 转换视频
  ConvertVideo {
    input: String,
    output: String,
    #[serde(default)]
    encode: VideoEncode,
  },
  /// 剪辑视频
  CutVideo {
    input: String,
    output: String,
    segments: Vec<TimeSegment>,
    #[serde(default)]
    encode: VideoEncode,
  },
  /// 合并视频
  MergeVideo {
    inputs: Vec<String>,
    output: String,
    #[serde(default)]
    encode: VideoEncode,
  },
  /// 添加视频
  AppendVideo {
    base: String,
    inputs: Vec<String>,
    output: String,
    #[serde(default)]
    encode: VideoEncode,
  },
  /// 转换 AVIF
  ConvertAvif {
    input: String,
    output: String,
    #[serde(default)]
    params: AvifEncodeParams,
  },
  /// 转换 JXL
  ConvertJxl {
    input: String,
    output: String,
    #[serde(default)]
    params: JxlEncodeParams,
  },
}
