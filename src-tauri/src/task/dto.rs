use serde::{Deserialize, Serialize};

use crate::ffmpeg::models::{TimeSegment, VideoEncode};
use crate::image::params::{AvifEncodeParams, JxlEncodeParams};

/// 队列里的一条任务。密码不会出现在这里。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
  pub id: String,
  pub kind: String,
  pub status: String,
  pub progress: f64,
  pub message: String,
  pub media_id: Option<String>,
}

/// 入队参数。密码只在这一次传入。
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TaskInput {
  EncryptFile {
    input: String,
    output: String,
    password: String,
  },
  DecryptFile {
    input: String,
    output: String,
    password: String,
  },
  ImportMedia {
    source: String,
    owner: String,
    owner_id: String,
    media_kind: String,
    encrypt: bool,
    password: String,
  },
  EncryptMedia {
    media_id: String,
    password: String,
  },
  DecryptMedia {
    media_id: String,
    password: String,
  },
  ConvertVideo {
    input: String,
    output: String,
    #[serde(default)]
    encode: VideoEncode,
  },
  CutVideo {
    input: String,
    output: String,
    segments: Vec<TimeSegment>,
    #[serde(default)]
    encode: VideoEncode,
  },
  MergeVideo {
    inputs: Vec<String>,
    output: String,
    #[serde(default)]
    encode: VideoEncode,
  },
  AppendVideo {
    base: String,
    inputs: Vec<String>,
    output: String,
    #[serde(default)]
    encode: VideoEncode,
  },
  ConvertAvif {
    input: String,
    output: String,
    #[serde(default)]
    params: AvifEncodeParams,
  },
  ConvertJxl {
    input: String,
    output: String,
    #[serde(default)]
    params: JxlEncodeParams,
  },
}
