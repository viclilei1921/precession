use tauri::AppHandle;

use crate::image::bridge::{BridgeInput, EncodeTarget, needs_bridge, path_ext_lower, resolve_encode_input};
use crate::image::models::AvifEncodeParams;
use crate::image::utils::{ensure_output_parent, run_sidecar_encode};
use crate::sidecar::{SidecarError, SidecarResult};
use crate::task::JobCtx;

/// 按 AVIF 参数组装 avifenc 参数，最后接输入和输出路径
///
/// 质量和透明质量收到 0–100，速度收到 0–10。`Auto` 色度和范围不传对应参数。自动分块时不写行列 tile。要注入 EXIF 时不写 `--ignore-exif`。旋转只写 0 到 3，镜像只写 0 或 1。
///
/// # 参数
/// - `input_path`: 输入路径
/// - `output_path`: 输出路径
/// - `params`: AVIF 参数
/// - `exif_inject`: 要注入的 EXIF 文件。有值时追加 `--exif`
///
/// # 返回值
/// avifenc 参数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let args = build_avifenc_args("a.png", "a.avif", &AvifEncodeParams::default(), None);
/// assert_eq!(args[args.len() - 2], "a.png");
/// assert_eq!(args[args.len() - 1], "a.avif");
/// ```
fn build_avifenc_args(
  input_path: &str,
  output_path: &str,
  params: &AvifEncodeParams,
  exif_inject: Option<&str>,
) -> Vec<String> {
  let mut args = Vec::with_capacity(36);

  if params.lossless {
    args.push("-l".to_string());
  }

  args.push("-q".to_string());
  args.push(params.qcolor.min(100).to_string());
  args.push("--qalpha".to_string());
  args.push(params.qalpha.min(100).to_string());
  args.push("-s".to_string());
  args.push(params.speed.min(10).to_string());

  if let Some(jobs) = params.jobs {
    args.push("-j".to_string());
    args.push(jobs.max(1).to_string());
  }

  args.push("-d".to_string());
  args.push(params.depth.flag().to_string());

  if let Some(yuv) = params.yuv.flag() {
    args.push("-y".to_string());
    args.push(yuv.to_string());
  }

  if params.premultiply {
    args.push("-p".to_string());
  }
  if params.sharpyuv {
    args.push("--sharpyuv".to_string());
  }
  if params.ignore_exif && exif_inject.is_none() {
    args.push("--ignore-exif".to_string());
  }
  if params.ignore_xmp {
    args.push("--ignore-xmp".to_string());
  }
  if params.ignore_icc {
    args.push("--ignore-icc".to_string());
  }

  if let Some(range) = params.range.flag() {
    args.push("-r".to_string());
    args.push(range.to_string());
  }

  let cicp = params.cicp.trim();
  if !cicp.is_empty() {
    args.push("--cicp".to_string());
    args.push(cicp.to_string());
  }

  if params.autotiling {
    args.push("--autotiling".to_string());
  } else {
    if let Some(rows) = params.tilerowslog2 {
      args.push("--tilerowslog2".to_string());
      args.push(rows.min(6).to_string());
    }
    if let Some(cols) = params.tilecolslog2 {
      args.push("--tilecolslog2".to_string());
      args.push(cols.min(6).to_string());
    }
  }

  let codec = params.codec.trim();
  if !codec.is_empty() {
    args.push("-c".to_string());
    args.push(codec.to_string());
  }

  if let Some(size) = params.target_size {
    if size > 0 {
      args.push("--target-size".to_string());
      args.push(size.to_string());
    }
  }

  if params.progressive {
    args.push("--progressive".to_string());
  }

  let pasp = params.pasp.trim();
  if !pasp.is_empty() {
    args.push("--pasp".to_string());
    args.push(pasp.to_string());
  }

  let crop = params.crop.trim();
  if !crop.is_empty() {
    args.push("--crop".to_string());
    args.push(crop.to_string());
  }

  if let Some(angle) = params.irot {
    if angle <= 3 {
      args.push("--irot".to_string());
      args.push(angle.to_string());
    }
  }

  if let Some(axis) = params.imir {
    if axis <= 1 {
      args.push("--imir".to_string());
      args.push(axis.to_string());
    }
  }

  let clli = params.clli.trim();
  if !clli.is_empty() {
    args.push("--clli".to_string());
    args.push(clli.to_string());
  }

  for adv in &params.advanced {
    let kv = adv.trim();
    if kv.is_empty() {
      continue;
    }
    args.push("-a".to_string());
    args.push(kv.to_string());
  }

  if let Some(exif) = exif_inject {
    args.push("--exif".to_string());
    args.push(exif.to_string());
  }

  args.push(input_path.to_string());
  args.push(output_path.to_string());
  args
}

/// 把图片转成 AVIF
///
/// 非原生格式先报「桥接转 PNG…」再解码。忽略 EXIF 时不提取。质量、透明质量和速度先收到范围内，桥接句柄留到函数结束才清理。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `job`: 任务执行期句柄
/// - `input_path`: 源图片
/// - `output_path`: 输出路径
/// - `params`: AVIF 参数
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 已取消返回 `SidecarError::Canceled`。创建输出目录、桥接或 avifenc 失败时返回对应的 `SidecarError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// run_convert_avif(app, job, input_path, output_path, params).await?;
/// ```
pub async fn run_convert_avif(
  app: &AppHandle,
  job: &JobCtx,
  input_path: &str,
  output_path: &str,
  params: AvifEncodeParams,
) -> SidecarResult<()> {
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  job.report(5.0, "准备转码...").await;
  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  ensure_output_parent(output_path).await?;

  let ext = path_ext_lower(input_path).unwrap_or_default();
  let bridged = needs_bridge(&ext, EncodeTarget::Avif);
  if bridged {
    job.report(8.0, "桥接转 PNG…").await;
  }

  let inject_exif = !params.ignore_exif;
  let (encode_input, bridge, exif_path) = resolve_encode_input(app, input_path, EncodeTarget::Avif, inject_exif)?;
  let _bridge: Option<BridgeInput> = bridge;

  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  let params = AvifEncodeParams {
    qcolor: params.qcolor.min(100),
    qalpha: params.qalpha.min(100),
    speed: params.speed.min(10),
    ..params
  };

  let exif_inject = exif_path.as_deref();
  let args = build_avifenc_args(&encode_input, output_path, &params, exif_inject);
  let label = format!(
    "正在 AVIF 编码（-q {}, --qalpha {}, -s {}, lossless={}）...",
    params.qcolor, params.qalpha, params.speed, params.lossless
  );
  run_sidecar_encode(app, job, "avifenc", &args, &label).await
}
