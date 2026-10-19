use serde::{Deserialize, Serialize};

use crate::ffmpeg::models::{TimeSegment, VideoEncode};
use crate::image::params::{AvifEncodeParams, JxlEncodeParams};

/// 失败或取消后用来还原界面。密码不会出现在这里。
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TaskDraft {
  EncryptFile { input: String, output: String },
  DecryptFile { input: String, output: String },
  ConvertVideo { input: String, output: String, encode: VideoEncode },
  CutVideo { input: String, output: String, segments: Vec<TimeSegment>, encode: VideoEncode },
  MergeVideo { inputs: Vec<String>, output: String, encode: VideoEncode },
  AppendVideo { base: String, inputs: Vec<String>, output: String, encode: VideoEncode },
  ConvertAvif { input: String, output: String, params: AvifEncodeParams },
  ConvertJxl { input: String, output: String, params: JxlEncodeParams },
}

/// 队列里的一条任务。密码不会出现在这里。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
  pub id: String,
  /// 任务类型: encryptFile, decryptFile, importMedia, encryptMedia, decryptMedia, convertVideo, cutVideo, mergeVideo, appendVideo, convertAvif, convertJxl
  pub kind: String,
  /// 任务状态: pending, processing, completed, failed, canceled
  pub status: String,
  /// 任务进度: 0.0 - 1.0
  pub progress: f64,
  /// 任务消息。任务处理中时才有。
  pub message: String,
  /// 媒体 ID。媒体任务时才有。
  pub media_id: Option<String>,
  /// 文件名。多文件时带上数量。
  pub title: String,
  /// 输出文件。空则没有可打开的结果。任务完成时才有。
  pub output: String,
  /// 任务草稿。任务取消时才有。
  pub draft: Option<TaskDraft>,
}

/// 入队参数。密码只在这一次传入。
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TaskInput {
  /// 加密文件
  EncryptFile { input: String, output: String, password: String },
  /// 解密文件
  DecryptFile { input: String, output: String, password: String },
  /// 导入媒体
  ImportMedia { source: String, owner: String, owner_id: String, media_kind: String, encrypt: bool, password: String },
  /// 加密媒体
  EncryptMedia { media_id: String, password: String },
  /// 解密媒体
  DecryptMedia { media_id: String, password: String },
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
