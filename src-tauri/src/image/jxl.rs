use tauri::AppHandle;

use crate::image::bridge::{BridgeInput, EncodeTarget, needs_bridge, path_ext_lower, resolve_encode_input};
use crate::image::models::JxlEncodeParams;
use crate::image::utils::{ensure_output_parent, run_sidecar_encode};
use crate::sidecar::{SidecarError, SidecarResult};
use crate::task::JobCtx;

/// 按 JXL 参数组装 cjxl 参数，最后接输入和输出路径
///
/// 无损时距离写 0，否则把质量收到 0–25。effort 收到 1–10，解码速度收到 0–4。不写 `--container=0`。`use_container` 为真或要注入 EXIF 时写 `--container=1`。超出各自允许范围的可选数值不写入。
///
/// # 参数
/// - `input_path`: 输入路径
/// - `output_path`: 输出路径
/// - `params`: JXL 参数
/// - `exif_inject`: 要注入的 EXIF 文件。有值时追加 `-x exif=`
///
/// # 返回值
/// cjxl 参数
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```
/// let args = build_cjxl_args("a.png", "a.jxl", &JxlEncodeParams::default(), None);
/// assert!(args.iter().any(|arg| arg == "-d"));
/// ```
fn build_cjxl_args(
  input_path: &str,
  output_path: &str,
  params: &JxlEncodeParams,
  exif_inject: Option<&str>,
) -> Vec<String> {
  let effort = params.effort.clamp(1, 10);
  let decoding_speed = params.decoding_speed.clamp(0, 4);
  let distance = if params.lossless { 0.0 } else { params.quality.clamp(0.0, 25.0) };

  let mut args = Vec::with_capacity(48);
  args.push("-d".to_string());
  args.push(format!("{distance}"));
  args.push("-e".to_string());
  args.push(effort.to_string());
  args.push(format!("--lossless_jpeg={}", if params.lossless_jpeg { 1 } else { 0 }));
  // 勿默认传 --container=0：cjxl 会据此清空 EXIF/XMP。
  // 注入 EXIF 时必须使用容器。
  if params.use_container || exif_inject.is_some() {
    args.push("--container=1".to_string());
  }
  args.push(format!("--faster_decoding={decoding_speed}"));

  if let Some(nits) = params.target_intensity {
    if nits.is_finite() && nits >= 0.0 {
      args.push(format!("--intensity_target={nits}"));
    }
  }

  if let Some(ad) = params.alpha_distance {
    if ad.is_finite() {
      args.push(format!("--alpha_distance={}", ad.clamp(0.0, 25.0)));
    }
  }

  if params.progressive {
    args.push("--progressive".to_string());
  }

  if let Some(g) = params.group_order {
    if g <= 1 {
      args.push(format!("--group_order={g}"));
    }
  }

  if let Some(c) = params.compress_boxes {
    if c <= 1 {
      args.push(format!("--compress_boxes={c}"));
    }
  }

  if let Some(b) = params.brotli_effort {
    if b <= 11 {
      args.push(format!("--brotli_effort={b}"));
    }
  }

  if let Some(m) = params.modular {
    if m <= 1 {
      args.push(format!("--modular={m}"));
    }
  }

  if let Some(n) = params.num_threads {
    if n >= 1 {
      args.push(format!("--num_threads={n}"));
    }
  }

  if let Some(iso) = params.photon_noise_iso {
    if iso.is_finite() && iso >= 0.0 {
      args.push(format!("--photon_noise_iso={iso}"));
    }
  }

  if let Some(a) = params.allow_jpeg_reconstruction {
    if a <= 1 {
      args.push(format!("--allow_jpeg_reconstruction={a}"));
    }
  }

  if let Some(level) = params.codestream_level {
    if matches!(level, -1 | 5 | 10) {
      args.push(format!("--codestream_level={level}"));
    }
  }

  if let Some(buf) = params.buffering {
    if (-1..=3).contains(&buf) {
      args.push(format!("--buffering={buf}"));
    }
  }

  if let Some(p) = params.premultiply {
    if (-1..=1).contains(&p) {
      args.push(format!("--premultiply={p}"));
    }
  }

  if let Some(k) = params.keep_invisible {
    if k <= 1 {
      args.push(format!("--keep_invisible={k}"));
    }
  }

  if params.progressive_ac {
    args.push("--progressive_ac".to_string());
  }
  if params.qprogressive_ac {
    args.push("--qprogressive_ac".to_string());
  }

  if let Some(dc) = params.progressive_dc {
    if (-1..=2).contains(&dc) {
      args.push(format!("--progressive_dc={dc}"));
    }
  }

  if let Some(r) = params.resampling {
    if matches!(r, -1 | 1 | 2 | 4 | 8) {
      args.push(format!("--resampling={r}"));
    }
  }

  if let Some(r) = params.ec_resampling {
    if matches!(r, -1 | 1 | 2 | 4 | 8) {
      args.push(format!("--ec_resampling={r}"));
    }
  }

  if params.already_downsampled {
    args.push("--already_downsampled".to_string());
  }

  if let Some(m) = params.upsampling_mode {
    if (-1..=1).contains(&m) {
      args.push(format!("--upsampling_mode={m}"));
    }
  }

  if let Some(e) = params.epf {
    if (-1..=3).contains(&e) {
      args.push(format!("--epf={e}"));
    }
  }

  if let Some(g) = params.gaborish {
    if g <= 1 {
      args.push(format!("--gaborish={g}"));
    }
  }

  if let Some(bits) = params.override_bitdepth {
    if bits > 0 {
      args.push(format!("--override_bitdepth={bits}"));
    }
  }

  if let Some(n) = params.noise {
    if n <= 1 {
      args.push(format!("--noise={n}"));
    }
  }
  if let Some(d) = params.dots {
    if d <= 1 {
      args.push(format!("--dots={d}"));
    }
  }
  if let Some(p) = params.patches {
    if p <= 1 {
      args.push(format!("--patches={p}"));
    }
  }

  if let Some(enc) = params.color_encoding {
    args.push("-x".to_string());
    args.push(format!("color_space={}", enc.hint()));
  }

  for hint in &params.extra_hints {
    let kv = hint.trim();
    if kv.is_empty() {
      continue;
    }
    args.push("-x".to_string());
    args.push(kv.to_string());
  }

  if let Some(exif) = exif_inject {
    args.push("-x".to_string());
    args.push(format!("exif={exif}"));
  }

  args.push(input_path.to_string());
  args.push(output_path.to_string());
  args
}

/// 把图片转成 JXL
///
/// 非原生格式先桥接。总是尝试提取 EXIF。质量收到 0–25，effort 收到 1–10，解码速度收到 0–4。无损时进度说明里的距离写成 0。
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `job`: 任务执行期句柄
/// - `input_path`: 源图片
/// - `output_path`: 输出路径
/// - `params`: JXL 参数
///
/// # 返回值
/// 成功时无额外值
///
/// # 错误
/// 已取消返回 `SidecarError::Canceled`。创建输出目录、桥接或 cjxl 失败时返回对应的 `SidecarError`
///
/// # Panics
/// 不会主动 panic
///
/// # 示例
/// ```ignore
/// run_convert_jxl(app, job, input_path, output_path, params).await?;
/// ```
pub async fn run_convert_jxl(
  app: &AppHandle,
  job: &JobCtx,
  input_path: &str,
  output_path: &str,
  params: JxlEncodeParams,
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
  let bridged = needs_bridge(&ext, EncodeTarget::Jxl);
  if bridged {
    job.report(8.0, "桥接转 PNG…").await;
  }

  let (encode_input, bridge, exif_path) = resolve_encode_input(app, input_path, EncodeTarget::Jxl, true)?;
  let _bridge: Option<BridgeInput> = bridge;

  if job.is_canceled().await {
    return Err(SidecarError::canceled());
  }

  let params = JxlEncodeParams {
    quality: params.quality.clamp(0.0, 25.0),
    effort: params.effort.clamp(1, 10),
    decoding_speed: params.decoding_speed.clamp(0, 4),
    ..params
  };

  let exif_inject = exif_path.as_deref();
  let args = build_cjxl_args(&encode_input, output_path, &params, exif_inject);
  let distance = if params.lossless { 0.0 } else { params.quality };
  let label = format!(
    "正在 JXL 编码（lossless={}, distance={distance:.2}, effort={}, lossless_jpeg={}）...",
    params.lossless, params.effort, params.lossless_jpeg
  );
  run_sidecar_encode(app, job, "cjxl", &args, &label).await
}
